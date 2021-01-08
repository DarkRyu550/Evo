use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{Read, Write};
use shaderc::{Compiler, ShaderKind, CompileOptions, OptimizationLevel, TargetEnv, EnvVersion, IncludeType, ResolvedInclude};
use std::ffi::OsStr;

/** Directory in which all shaders will be compiled automatically. */
const SHADER_DIR: &'static str = "shaders/";

/** Entry point name. Is always `main` for GLSL. */
const ENTRY_POINT_GLSL: &'static str = "main";

/** Perform the recursive directory scan and source compile. */
fn scan<A: AsRef<Path>>(path_ref: A, compiler: &mut Compiler, options: &CompileOptions) {
	let path = path_ref.as_ref();
	let name = path.to_str()
		.expect("path does not name valid utf8");
	println!("cargo:rerun-if-changed={}", name);

	let target_path = std::env::var_os("OUT_DIR")
		.expect("cargo should have set the OUT_DIR variable, but it is missing");

	let mut target_path = PathBuf::from(target_path).join(path);
	if path.is_dir() {
		/* Create the corresponding directory in the output path, then rerun
		 * the scan from within the directory. */
		std::fs::create_dir_all(target_path)
			.expect("could not create target directory for shader compilation");
		let read = match path.read_dir() {
			Ok(read) => read,
			Err(what) => {
				println!("cargo:warning=shader compilation will skip directory \
					{} due to an error: {}", name, what);
				return
			}
		};

		for item in read {
			let item = match item {
				Ok(item) => item,
				Err(what) => {
					println!("cargo:warning=shader compilation will skip directory \
						entry of {} due to an error: {}", name, what);
					continue
				}
			};

			scan(item.path(), compiler, options);
		}

		/* Nothing else to do. */
		return
	}

	/* Check for file extension. */
	let has_glsl = path.extension()
		.map(|ext| ext == AsRef::<OsStr>::as_ref("glsl"))
		.unwrap_or(false);
	if !has_glsl { return; }

	/* Change the extension of the target. */
	target_path.set_extension("spv");

	/* Open and compile the file. */
	println!("compiling: {:?} => {:?}", path, target_path);

	let source = {
		let mut file = File::open(path)
			.expect("could not open shader source file");
		let mut buf = String::new();
		file.read_to_string(&mut buf)
			.expect("could not read shader source file");

		buf
	};

	/* Theres this terrible bug in `shaderc-rs` that fucks up includes really
	 * badly such that no program with includes will ever compile correctly.
	 *
	 * To circumvent this we use our own include function. */
	let source = resolve_includes(source, name);

	let compiled = match compiler.compile_into_spirv(
		&source,
		ShaderKind::InferFromSource,
		name,
		ENTRY_POINT_GLSL,
		Some(&options)){
		Ok(x) => x,
		Err(e) => {
			use shaderc::Error::*;
			match e {
				CompilationError(_, s) => panic!("Could not compile source file:\n{}", s),
				_ => panic!("Could not compile source file: {:?}", e)
			}
		}
	};

	let mut target = File::create(&target_path)
		.expect("could not create spirv target file");
	target.write_all(compiled.as_binary_u8())
		.expect("could not write to spirv target file");
}

/** Manually resolve include directives in a string.
 * This is needed because of #86 in `shaderc` (See
 * https://github.com/google/shaderc-rs/issues/86). */
fn resolve_includes<A: AsRef<str>>(src_ref: A, name: &str) -> String {
	let source = src_ref.as_ref();

	let mut buf = String::with_capacity(source.len());
	for line in source.lines() {
		const INCLUDE_LOCAL:  &'static str = r#"#include ""#;
		const INCLUDE_GLOBAL: &'static str = r#"#include <"#;

		let (include_file, kind) = if line.starts_with(INCLUDE_LOCAL) {
			let (_, file) = line.split_at(INCLUDE_LOCAL.len());
			let file = match file.split('"').next() {
				Some(file) => file,
				None => panic!("invalid include directive \"{}\"", line)
			};
			(file, IncludeType::Relative)
		} else if line.starts_with(INCLUDE_GLOBAL) {
			let (_, file) = line.split_at(INCLUDE_GLOBAL.len());
			let file = match file.split('>').next() {
				Some(file) => file,
				None => panic!("invalid include directive \"{}\"", line)
			};
			(file, IncludeType::Standard)
		} else {
			/* No include directives, just copy the line. */
			buf.extend(std::iter::once(line));
			buf.extend(std::iter::once('\n'));
			continue
		};

		/* We have an include directive. */
		match include(include_file, kind, name, 0) {
			Ok(ResolvedInclude { content, .. }) => {
				buf.extend(std::iter::once(content));
				buf.extend(std::iter::once('\n'));
			}
			Err(what) => {
				panic!("could not resolve {}: {}", line, what)
			}
		}
	}

	buf
}

/** Find an include file given a path. */
fn include(request: &str, kind: IncludeType, source: &str, _: usize)
	-> Result<ResolvedInclude, String> {

	let path = match kind {
		IncludeType::Relative =>
			/* Sources at this point should always have a parent. */
			Path::new(source).parent().unwrap(),
		IncludeType::Standard => Path::new(SHADER_DIR)
	};

	let request_path = Path::new(request);
	if !request_path.is_relative() {
		let what = format!("expected relative path, got: {}", request);
		return Err(what)
	}

	let candidate = path.join(request_path);
	if !candidate.is_file() {
		let what = format!("requested include does not exist at path {:?}",
			candidate);
		return Err(what)
	}

	let mut file = match File::open(&candidate) {
		Ok(file) => file,
		Err(what) => {
			let what = format!("could not open requested include {:?}: {}",
				candidate,
				what);
			return Err(what)
		}
	};

	let mut data = "\n".to_owned();
	if let Err(what) = file.read_to_string(&mut data) {
		let what = format!("could not read the requested include {:?}: {}",
			candidate,
			what);
		return Err(what)
	}
	println!("    -> resolved #include {}{}{} to file {:?}",
		match kind { IncludeType::Standard => "<", IncludeType::Relative => "\"" },
		request,
		match kind { IncludeType::Standard => ">", IncludeType::Relative => "\"" },
		candidate);

	Ok(ResolvedInclude {
		resolved_name: format!("{:?}", candidate),
		content: data
	})
}

/** We resolve includes manually. So the compiler must never actually see any
 * of the include directives. */
fn fail_include(_: &str, _: IncludeType, _: &str, _: usize)
	-> Result<ResolvedInclude, String> {
	panic!("includes are resolved manually, the compiler shouldn't have gotten \
		here")
}

/** Build the shaders. */
pub fn build() {
	let shader_dir = Path::new(SHADER_DIR);
	if !shader_dir.exists() {
		println!("cargo:warning=no shaders will be compiled as the shader \
			directory {:?} does not exist", shader_dir);
		return;
	} else if !shader_dir.is_dir() {
		println!("cargo:warning=no shaders will be compiled as the shader \
			directory {:?} does exist, but is not a directory", shader_dir);
		return;
	}

	let mut compiler = Compiler::new().expect("could not initialize compiler");
	let mut options = CompileOptions::new()
		.expect("could not initialize compiler options");
	options.set_optimization_level(OptimizationLevel::Performance);
	options.set_target_env(
		TargetEnv::Vulkan,
		EnvVersion::Vulkan1_0 as u32);
	options.set_include_callback(fail_include);

	scan(shader_dir, &mut compiler, &options);
	println!("cargo:rerun-if-changed={}", SHADER_DIR);
}

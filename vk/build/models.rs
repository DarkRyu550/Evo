use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{Read, Write, BufReader};
use std::ffi::OsStr;
use obj::Obj;

/** Directory in which all models will be compiled automatically. */
const MODEL_DIR: &'static str = "models/";

/** Perform the recursive directory scan and model compile. */
fn scan<A: AsRef<Path>>(path_ref: A) {
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
			.expect("could not create target directory for model compilation");
		let read = match path.read_dir() {
			Ok(read) => read,
			Err(what) => {
				println!("cargo:warning=model compilation will skip directory \
					{} due to an error: {}", name, what);
				return
			}
		};

		for item in read {
			let item = match item {
				Ok(item) => item,
				Err(what) => {
					println!("cargo:warning=model compilation will skip directory \
						entry of {} due to an error: {}", name, what);
					continue
				}
			};

			scan(item.path());
		}

		/* Nothing else to do. */
		return
	}

	/* Check for file extension. */
	let has_obj = path.extension()
		.map(|ext| ext == AsRef::<OsStr>::as_ref("obj"))
		.unwrap_or(false);
	if !has_obj { return; }

	/* Create the target directory and specify the index and vertex data files. */
	std::fs::create_dir_all(&target_path)
		.expect("could not create directory for model data");

	let target_indices = target_path.join("indices");
	let target_vertices = target_path.join("vertices");

	/* Open and compile the file. */
	println!("compiling: {:?} => {:?}", path, target_path);

	let source = File::open(path)
		.expect("could not open shader source file");
	let source = BufReader::new(source);
	let source: Obj<obj::Vertex, u32> = obj::load_obj(source)
		.expect("could not read model data");
	let Obj { vertices, indices, .. } = source;

	let mut target = File::create(&target_vertices)
		.expect("could not create target vertices file");
	for vertex in vertices {
		target.write_all(&vertex.position[0].to_ne_bytes()).expect("vertex write failed");
		target.write_all(&vertex.position[1].to_ne_bytes()).expect("vertex write failed");
		target.write_all(&vertex.position[2].to_ne_bytes()).expect("vertex write failed");

		target.write_all(&vertex.normal[0].to_ne_bytes()).expect("normal write failed");
		target.write_all(&vertex.normal[1].to_ne_bytes()).expect("normal write failed");
		target.write_all(&vertex.normal[2].to_ne_bytes()).expect("normal write failed");
	}

	let mut target = File::create(&target_indices)
		.expect("could not create target indices file");
	for index in indices {
		target.write_all(&index.to_ne_bytes()).expect("index write failed");
	}
}

/** Build the models. */
pub fn build() {
	let model_dir = Path::new(MODEL_DIR);
	if !model_dir.exists() {
		println!("cargo:warning=no model will be compiled as the model \
			directory {:?} does not exist", model_dir);
		return;
	} else if !model_dir.is_dir() {
		println!("cargo:warning=no model will be compiled as the model \
			directory {:?} does exist, but is not a directory", model_dir);
		return;
	}

	scan(model_dir);
	println!("cargo:rerun-if-changed={}", MODEL_DIR);
}

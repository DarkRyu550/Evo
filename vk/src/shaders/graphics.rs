
/** Shader modules for drawing the herbivore population. */
pub mod draw_herbivore {
	use wgpu::ShaderModuleSource;

	pub const VERTEX_SHADER:   ShaderModuleSource = wgpu::include_spirv!("");
	pub const FRAGMENT_SHADER: ShaderModuleSource = wgpu::include_spirv!("");
}

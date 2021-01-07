
/** Shader modules for drawing the herbivore population. */
pub mod draw_herbivore {
	use wgpu::ShaderModuleSource;

	pub fn vertex_shader() -> ShaderModuleSource<'static> {
		wgpu::include_spirv!(
			concat!(env!("OUT_DIR"), "/shaders/DrawHerbivore/vert.spv"))
	}
	pub fn fragment_shader() -> ShaderModuleSource<'static> {
		wgpu::include_spirv!(
			concat!(env!("OUT_DIR"), "/shaders/DrawHerbivore/frag.spv"))
	}
}

/** Shader modules for drawing the predator population. */
pub mod draw_predator {
	use wgpu::ShaderModuleSource;

	pub fn vertex_shader() -> ShaderModuleSource<'static> {
		wgpu::include_spirv!(
			concat!(env!("OUT_DIR"), "/shaders/DrawPredator/vert.spv"))
	}
	pub fn fragment_shader() -> ShaderModuleSource<'static> {
		wgpu::include_spirv!(
			concat!(env!("OUT_DIR"), "/shaders/DrawPredator/frag.spv"))
	}
}
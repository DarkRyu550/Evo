
/** Shader modules for the geometry pass of the herbivore population. */
pub mod herbivore_geometry_pass {
	use wgpu::ShaderModuleSource;

	pub fn vertex_shader() -> ShaderModuleSource<'static> {
		wgpu::include_spirv!(
			concat!(env!("OUT_DIR"), "/shaders/Graphics/HerbivoreGeometryPass/vert.spv"))
	}
	pub fn fragment_shader() -> ShaderModuleSource<'static> {
		wgpu::include_spirv!(
			concat!(env!("OUT_DIR"), "/shaders/Graphics/HerbivoreGeometryPass/frag.spv"))
	}
}

/** Shader modules for geometry pass of the predator population. */
pub mod predator_geometry_pass {
	use wgpu::ShaderModuleSource;

	pub fn vertex_shader() -> ShaderModuleSource<'static> {
		wgpu::include_spirv!(
			concat!(env!("OUT_DIR"), "/shaders/Graphics/PredatorGeometryPass/vert.spv"))
	}
	pub fn fragment_shader() -> ShaderModuleSource<'static> {
		wgpu::include_spirv!(
			concat!(env!("OUT_DIR"), "/shaders/Graphics/PredatorGeometryPass/frag.spv"))
	}
}

/** Shader modules for the lighting pass. */
pub mod lighting_pass {
	use wgpu::ShaderModuleSource;

	pub fn vertex_shader() -> ShaderModuleSource<'static> {
		wgpu::include_spirv!(
			concat!(env!("OUT_DIR"), "/shaders/Graphics/LightingPass/vert.spv"))
	}
	pub fn fragment_shader() -> ShaderModuleSource<'static> {
		wgpu::include_spirv!(
			concat!(env!("OUT_DIR"), "/shaders/Graphics/LightingPass/frag.spv"))
	}
}

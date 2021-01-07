use wgpu::ShaderModuleSource;

/** The shader performing one step of the simulation. */
pub fn simulate() -> ShaderModuleSource<'static> {
	wgpu::include_spirv!(concat!(env!("OUT_DIR"), "/shaders/simulate.spv"))
}




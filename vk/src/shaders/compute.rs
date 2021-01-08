/** Compute shaders for the herbivore group. */
pub mod herbivores {
	use wgpu::ShaderModuleSource;

	/** The shader performing one step of the simulation. */
	pub fn simulate() -> ShaderModuleSource<'static> {
		wgpu::include_spirv!(concat!(env!("OUT_DIR"), "/shaders/SimulateHerbivore.spv"))
	}

	/** The shader performing the shuffling and evolution step. */
	pub fn shuffle() -> ShaderModuleSource<'static> {
		wgpu::include_spirv!(concat!(env!("OUT_DIR"), "/shaders/ShuffleHerbivore.spv"))
	}
}




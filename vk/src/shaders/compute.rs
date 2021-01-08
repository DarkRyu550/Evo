use wgpu::ShaderModuleSource;

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

/** Compute shaders for the predator group. */
pub mod predators {
	use wgpu::ShaderModuleSource;

	/** The shader performing one step of the simulation. */
	pub fn simulate() -> ShaderModuleSource<'static> {
		wgpu::include_spirv!(concat!(env!("OUT_DIR"), "/shaders/SimulatePredator.spv"))
	}

	/** The shader performing the shuffling and evolution step. */
	pub fn shuffle() -> ShaderModuleSource<'static> {
		wgpu::include_spirv!(concat!(env!("OUT_DIR"), "/shaders/ShufflePredator.spv"))
	}
}

/** Pre run shader for setting everything up. */
pub fn pre_run() -> ShaderModuleSource<'static> {
	wgpu::include_spirv!(concat!(env!("OUT_DIR"), "/shaders/PreRun.spv"))
}

/** Shader for running updates on the simulation field. */
pub fn field_update() -> ShaderModuleSource<'static> {
	wgpu::include_spirv!(concat!(env!("OUT_DIR"), "/shaders/FieldUpdate.spv"))
}

use wgpu::ShaderModuleSource;

/** Compute shaders for the herbivore group. */
pub mod herbivores {
	use wgpu::ShaderModuleSource;

	/** Pre run shader for setting everything up. */
	pub fn pre_run() -> ShaderModuleSource<'static> {
		wgpu::include_spirv!(
			concat!(env!("OUT_DIR"),
			"/shaders/Compute/Herbivore/PreRun.spv"))
	}

	/** The shader performing one step of the simulation. */
	pub fn simulate() -> ShaderModuleSource<'static> {
		wgpu::include_spirv!(
			concat!(env!("OUT_DIR"),
			"/shaders/Compute/Herbivore/Simulate.spv"))
	}

	/** The shader performing the shuffling and evolution step. */
	pub fn shuffle() -> ShaderModuleSource<'static> {
		wgpu::include_spirv!(
			concat!(env!("OUT_DIR"),
			"/shaders/Compute/Herbivore/Shuffle.spv"))
	}
}

/** Compute shaders for the predator group. */
pub mod predators {
	use wgpu::ShaderModuleSource;

	/** Pre run shader for setting everything up. */
	pub fn pre_run() -> ShaderModuleSource<'static> {
		wgpu::include_spirv!(
			concat!(env!("OUT_DIR"),
			"/shaders/Compute/Predator/PreRun.spv"))
	}

	/** The shader performing one step of the simulation. */
	pub fn simulate() -> ShaderModuleSource<'static> {
		wgpu::include_spirv!(
			concat!(env!("OUT_DIR"),
			"/shaders/Compute/Predator/Simulate.spv"))
	}

	/** The shader performing the shuffling and evolution step. */
	pub fn shuffle() -> ShaderModuleSource<'static> {
		wgpu::include_spirv!(
			concat!(env!("OUT_DIR"),
			"/shaders/Compute/Predator/Shuffle.spv"))
	}
}

/** The shader responsible for weaving in data and updating the simulation
 * plane. */
pub fn update_plane() -> ShaderModuleSource<'static> {
	wgpu::include_spirv!(
		concat!(env!("OUT_DIR"),
		"/shaders/Compute/UpdatePlane.spv"))
}

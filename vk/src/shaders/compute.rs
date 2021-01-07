use wgpu::ShaderModuleSource;

/** The shader performing one step of the simulation. */
const SIMULATE: ShaderModuleSource = wgpu::include_spirv!("");



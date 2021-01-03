
/** The shader performing one step of the simulation. */
mod simulate {
	vulkano_shaders::shader! {
		ty: "compute",
		path: "shaders/simulate.glsl"
	}
}


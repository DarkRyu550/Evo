
/** Model of the unit cube. */
pub mod cube {
	/** Vertex array data, in F32Vertex3NE_F32Normal3NE format. */
	pub const VERTICES: &'static [u8] =
		include_bytes!(concat!(env!("OUT_DIR"), "/models/Cube.obj/vertices"));
	/** Index array data, in U32_NE format. */
	pub const INDICES: &'static [u8] =
		include_bytes!(concat!(env!("OUT_DIR"), "/models/Cube.obj/indices"));
}

/** Model of the unit square. */
pub mod square {
	/** Vertex array data, in F32Vertex3NE_F32Normal3NE format. */
	pub const VERTICES: &'static [u8] =
		include_bytes!(concat!(env!("OUT_DIR"), "/models/Square.obj/vertices"));
	/** Index array data, in U32_NE format. */
	pub const INDICES: &'static [u8] =
		include_bytes!(concat!(env!("OUT_DIR"), "/models/Square.obj/indices"));
}


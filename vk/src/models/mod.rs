/** Basic model set. */
pub mod basic;

/** Model of the board tile. */
pub mod tile {
	/** Vertex array data, in F32Vertex3NE_F32Normal3NE format. */
	pub const VERTICES: &'static [u8] =
		include_bytes!(concat!(env!("OUT_DIR"), "/models/Tile.obj/vertices"));
	/** Index array data, in U32_NE format. */
	pub const INDICES: &'static [u8] =
		include_bytes!(concat!(env!("OUT_DIR"), "/models/Tile.obj/indices"));
}

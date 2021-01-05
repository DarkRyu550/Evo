use wgpu::BindGroupLayout;

/** A population dataset synchronization mechanism that allows for a producer
 * to create data points as fast as it desires and for a consumer guaranteed
 * access to the most recent complete snapshot of the data from the producer. */
pub struct Flipbook {
	bundles: [Bundle; 3],
}
impl Flipbook {
	/* Borrow a handle to the most current snapshot. */
	pub fn snapshot(&self) -> Snapshot {

	}

	/**  */
	pub fn blank(&self) -> Frame {

	}
}

struct Bundle {
	pub
}

pub struct Snapshot<'a> {

}


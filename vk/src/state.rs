use std::error::Error;
use wgpu::{Instance, Adapter, Device, Queue, Surface, BackendBit, RequestAdapterOptions, PowerPreference, DeviceDescriptor, Features};

/** This structure contains the group of Wgpu primitives and devices being
 * used throughout the program, as well as facilities for caching common
 * objects, if that ever turns out to be a requirement. */
pub struct State {
	instance: Instance,
	physical: Adapter,
	device: Device,
	queue: Queue
}
impl State {
	/** Creates a new instance, automatically selecting a suitable device for
	 * our operations, as well as all of the needed queues.
	 *
	 * # Error
	 * The error type here is demoted to a boxed `dyn Error`, in general, this
	 * is not the best practice. But here it shouldn't matter as this is a very
	 * top level error to start with. */
	pub async fn new(
		mut surface: impl FnMut(&Instance) -> Surface) -> Result<(Self, Surface), Box<dyn Error>> {

		let instance = Instance::new(BackendBit::DX12);
		let surface = surface(&instance);

		let adapter = instance.request_adapter(
			&RequestAdapterOptions {
				power_preference: PowerPreference::HighPerformance,
				compatible_surface: Some(&surface)
			})
			.await
			.ok_or("could not find a suitable adapter")?;

		let (device, queue) = adapter.request_device(
			&DeviceDescriptor {
				features: Features::MAPPABLE_PRIMARY_BUFFERS,
				limits: Default::default(),
				..Default::default()
			},
			None).await?;
			
		Ok((Self {
			instance,
			physical: adapter,
			device,
			queue
		}, surface))
	}

	/** Handle to the WebGpu instance being used. */
	pub fn instance(&self) -> &Instance {
		&self.instance
	}

	/** Handle to the logical device being used. */
	pub fn device(&self) -> &Device {
		&self.device
	}

	/** Handle to the physical device being used. */
	pub fn adapter(&self) -> &Adapter {
		&self.physical
	}

	/** Queue object of the device being used. */
	pub fn queue(&self) -> &Queue {
		&self.queue
	}
}

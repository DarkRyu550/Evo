use std::sync::Arc;
use vulkano::instance::{Instance as VulkanoInstance, PhysicalDevice, QueueFamily};
use vulkano::device::{Device, RawDeviceExtensions, Features, Queue};
use std::error::Error;

/** This structure contains the group of Vulkan primitives and devices being
 * used throughout the program, as well as facilities for caching common
 * objects, if that ever turns out to be a requirement. */
pub struct Instance {
	instance: Arc<VulkanoInstance>,
	physical: usize,
	device: Arc<Device>,

	graphics: Arc<Queue>,
	compute: Arc<Queue>,
	graphics_family: u32,
	compute_family: u32
}
impl Instance {
	/** Creates a new instance, automatically selecting a suitable device for
	 * our operations, as well as all of the needed queues.
	 *
	 * # Extensions and layers
	 * The instance will be created with the given instance extensions and
	 * validation layers. The device will be created will all of its supported
	 * extensions enabled, returning an error of any of the given required
	 * extensions could not be enabled.
	 *
	 * # Error
	 * The error type here is demoted to a boxed `dyn Error`, in general, this
	 * is not the best practice. But here it shouldn't matter as this is a very
	 * top level error to start with. */
	pub fn new(
		vulkano_instance:     Arc<VulkanoInstance>,
		physical_device_find: impl FnMut(&PhysicalDevice) -> bool,
		device_features:      &Features,
		device_extensions:    impl Into<RawDeviceExtensions>
	) -> Result<Self, Box<dyn Error>> {
		let instance = vulkano_instance;
		let physical = PhysicalDevice::enumerate(&instance)
			.filter(|dev| {
				let mut graphics = false;
				let mut compute = false;

				for family in  dev.queue_families() {
					if family.supports_compute()  { compute  = true; }
					if family.supports_graphics() { graphics = true; }
				}

				let keep = graphics && compute;
				if !keep {
					warn!("Ignoring device for not supporting {} operations: {}",
						if !graphics && !compute { "graphics or compute" }
						else if !graphics { "graphics" }
						else if !compute  { "compute" }
						else { "WAIT WHAT" },
						dev.name())
				}
				keep
			})
			.filter(|dev| {
				let keep = dev.supported_features()
					.superset_of(device_features);
				if !keep {
					warn!("Ignoring device for missing features: {}",
						dev.name());
				}
				keep
			})
			.find(physical_device_find)
			.ok_or("could not pick any physical device")?;
		info!("Initializing device: {}", physical.name());

		let mut families = physical.queue_families();
		let graphics = families
			.find(|f| f.supports_graphics())
			.unwrap();

		let compute = families.find(|f| f.supports_compute() && *f != graphics);
		let compute = match compute {
			Some(dedicated) => dedicated,
			None => if !graphics.supports_compute() {
				panic!("at least one family has to support compute at this point")
			} else {
				graphics
			}
		};

		let queues = {
			info!("Using a graphics queue{}",
				if graphics == compute {
					", which will combo as a compute queue".to_owned()
				} else {
					"and a dedicated compute queue"
				});

			if graphics == compute {
				let count = graphics.queues_count().clamp(0, 2);
				let mut creator = Vec::with_capacity(count);

				let mut priority = 1.0;
				for _ in 0..count {
					creator.push((graphics, priority));
					priority = 0.5;
				}

				creator
			} else {
				let mut creator = Vec::with_capacity(2);
				creator.push((graphics, 1.0));
				creator.push((compute, 0.5));

				creator
			}
		};

		let device_extensions = device_extensions.into();
		debug!("Enabling extensions {:#?}", device_extensions);
		debug!("Enabling features {:#?}", physical.supported_features());

		let (device, mut queues) = Device::new(
			physical,
			physical.supported_features(),
			device_extensions,
			queues)?;

		let graphics_queue = queues.next().unwrap();
		let compute_queue = match queues.next() {
			Some(queue) => queue,
			None if graphics.supports_compute() =>
				/* Reuse the graphics queue for the compute operation. */
				graphics_queue.clone(),
			_ =>
				/* Should be unreachable at this point, but better safe than
				 * sorry, this isn't in the hot path anyway. */
				panic!("no available compute queue for usage")
		};

		Ok(Self {
			physical: physical.index(),
			instance,
			device,
			graphics: graphics_queue,
			compute: compute_queue,
			graphics_family: graphics.id(),
			compute_family: compute.id()
		})
	}

	/** Handle to the raw Vulkano instance being used. */
	pub fn instance(&self) -> &Arc<VulkanoInstance> {
		&self.instance
	}

	/** Handle to the logical device being used. */
	pub fn device(&self) -> &Arc<Device> {
		&self.device
	}

	/** Handle to the physical device being used. */
	pub fn physical(&self) -> PhysicalDevice {
		PhysicalDevice::from_index(&self.instance, self.physical)
			.expect("saved invalid physical device index in instance")
	}

	/** Queue family to which the graphics queue belongs. */
	pub fn graphics_queue_family(&self) -> QueueFamily {
		self.physical()
			.queue_family_by_id(self.graphics_family)
			.expect("saved invalid graphics queue family id")
	}

	/** Queue family to which the compute queue belongs. */
	pub fn compute_queue_family(&self) -> QueueFamily {
		self.physical()
			.queue_family_by_id(self.compute_family)
			.expect("saved invalid compute queue family id")
	}

	/** Handle to the compute queue. */
	pub fn compute(&self) -> &Arc<Queue> {
		&self.compute
	}

	/** Handle to the graphics queue. */
	pub fn graphics(&self) -> &Arc<Queue> {
		&self.graphics
	}
}

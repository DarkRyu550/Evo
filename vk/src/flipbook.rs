use wgpu::{BindGroupLayout, Buffer, Texture, BufferUsage, BindGroup, BindGroupLayoutDescriptor, BindGroupLayoutEntry, ShaderStage, BindingType, TextureViewDimension, TextureFormat, Device, TextureDescriptor, Extent3d, TextureDimension, TextureUsage, BindGroupDescriptor, BindGroupEntry, BindingResource, TextureViewDescriptor, TextureAspect, MapMode};
use crate::state::State;
use std::borrow::Borrow;
use crate::settings::Preferences;
use std::sync::{Mutex, Arc};
use std::time::Instant;
use wgpu::util::{DeviceExt, BufferInitDescriptor};
use crate::dataset::BackChannel;
use std::ops::{Range, RangeBounds, Bound};

/** Creates a new flipbook dataset channel, creating all the required backing
 * storage and binding descriptors, modeled and initialized after the parameters
 * specified in the given preferences structure.
 *
 * # Layout
 * The layout of a flipbook is made up of three frames - or pages - of data,
 * with each frame containing a complete copy of the dataset, each at one given
 * point in time.
 *
 * # Synchronization
 * Flipbooks implement a synchronization mechanism that allows for a producer to
 * create data points as fast as it desires and for a consumer to have
 * guaranteed access to the most recent complete snapshot of the data from the
 * producer.
 *
 * At any given point in time there are at most one producer and one consumer of
 * the data in a flipbook. This limitation is put in place to guarantee the
 * previously established property that snapshots will always contain the most
 * recently produced copy of the dataset.
 */
pub fn channel<A>(state: A, prefs: &Preferences) -> (Producer, Consumer)
	where A: Borrow<State> {

	let device = state.borrow().device();
	let layout = device.create_bind_group_layout(
		&BindGroupLayoutDescriptor {
			label: Some("Flipbook/Dataset/BindGroupLayout"),
			entries: &[
				/* Simulation plane. */
				BindGroupLayoutEntry {
					binding: 0,
					visibility: ShaderStage::COMPUTE | ShaderStage::VERTEX,
					ty: BindingType::StorageTexture {
						dimension: TextureViewDimension::D2,
						format: TextureFormat::Rgba32Float,
						readonly: false
					},
					count: None
				},
				/* Herbivore group. */
				BindGroupLayoutEntry {
					binding: 1,
					visibility: ShaderStage::COMPUTE | ShaderStage::VERTEX,
					ty: BindingType::StorageBuffer {
						dynamic: false,
						min_binding_size: None,
						readonly: false
					},
					count: None
				},
				/* Predator group. */
				BindGroupLayoutEntry {
					binding: 2,
					visibility: ShaderStage::COMPUTE | ShaderStage::VERTEX,
					ty: BindingType::StorageBuffer {
						dynamic: false,
						min_binding_size: None,
						readonly: false
					},
					count: None
				},
				/* Back channel. */
				BindGroupLayoutEntry {
					binding: 3,
					visibility: ShaderStage::COMPUTE,
					ty: BindingType::StorageBuffer {
						dynamic: false,
						min_binding_size: None,
						readonly: false
					},
					count: None
				}
			]
		});

	let bundles = {
		let mut iter = BundleFactory::new(device, &layout, prefs);
		[
			iter.next().unwrap(),
			iter.next().unwrap(),
			iter.next().unwrap(),
		]
	};

	let index = Mutex::new({
		/* Sharing the same time for all of them avoids needless swaps. */
		let now = Instant::now();
		Index {
			producer: (0, now),
			storage:  (1, now),
			consumer: (2, now)
		}
	});

	let flipbook = Arc::new(Flipbook {
		bundles,
		index,
		layout
	});

	(
		Producer { book: flipbook.clone() },
		Consumer { book: flipbook.clone() }
	)
}

/** Flipbook bundle index. */
#[derive(Debug)]
struct Index {
	producer: (u8, Instant),
	storage:  (u8, Instant),
	consumer: (u8, Instant),
}

/** Flipbook storage bundle. */
#[derive(Debug)]
struct Bundle {
	/** Handle to the underlying herbivore storage buffer. Along with a count. */
	herbivores: (Buffer, u32),
	/** Handle to the underlying predator storage buffer. Along with a count. */
	predators: (Buffer, u32),
	/** Handle to the host back-channeling buffer. */
	back_channel: Buffer,
	/** Handle to the simulation plane storage. */
	plane: (Texture, u32, u32),
	/** Bind group for the resources in this bundle. */
	bind: BindGroup
}
impl Bundle {
	/** Create a new bundle on the given device with the given preferences and
	 * the given initial buffer data. */
	fn new_with_populations<A, B>(
		device: &Device,
		layout: &BindGroupLayout,
		prefs: &Preferences,
		herbivores: A,
		predators: B) -> Self
		where A: AsRef<[u8]>,
			  B: AsRef<[u8]> {


		let herbivores = device.create_buffer_init(
			&BufferInitDescriptor {
				label: Some("Flipbook/Dataset/HerbivoreBuffer"),
				contents: &herbivores.as_ref()[..],
				usage: BufferUsage::STORAGE
			});

		let predators = device.create_buffer_init(
			&BufferInitDescriptor {
				label: Some("Flipbook/Dataset/PredatorBuffer"),
				contents: &predators.as_ref()[..],
				usage: BufferUsage::STORAGE
			});

		let mut back_channel_buf = Vec::with_capacity(16);
		let back_channel = BackChannel {
			herbivores: 0..prefs.simulation.herbivores.individuals,
			predators: 0..prefs.simulation.predators.individuals
		};
		back_channel.bytes(&mut back_channel_buf);

		let back_channel = device.create_buffer_init(
			&BufferInitDescriptor {
				label: Some("Flipbook/Dataset/BackChannelBuffer"),
				contents: &back_channel_buf[..],
				usage: BufferUsage::STORAGE | BufferUsage::MAP_READ | BufferUsage::MAP_WRITE
			});

		let plane = device.create_texture(
			&TextureDescriptor {
				label: Some("Flipbook/Dataset/PlaneTexture"),
				size: Extent3d {
					width: prefs.simulation.horizontal_granularity,
					height: prefs.simulation.vertical_granularity,
					depth: 1
				},
				mip_level_count: 1,
				sample_count: 1,
				dimension: TextureDimension::D2,
				format: TextureFormat::Rgba32Float,
				usage: TextureUsage::STORAGE
			});

		let plane_view = plane.create_view(
			&TextureViewDescriptor {
				label: Some("Flipbook/Dataset/PlaneTextureView"),
				format: Some(TextureFormat::Rgba32Float),
				dimension: Some(TextureViewDimension::D2),
				aspect: TextureAspect::All,
				base_mip_level: 0,
				level_count: None,
				base_array_layer: 0,
				array_layer_count: None
			});

		let bind = device.create_bind_group(
			&BindGroupDescriptor {
				label: Some("Flipbook/Dataset/BindGroup"),
				layout,
				entries: &[
					BindGroupEntry {
						binding: 0,
						resource: BindingResource::TextureView(&plane_view)
					},
					BindGroupEntry {
						binding: 1,
						resource: BindingResource::Buffer(herbivores.slice(..))
					},
					BindGroupEntry {
						binding: 2,
						resource: BindingResource::Buffer(predators.slice(..))
					},
					BindGroupEntry {
						binding: 3,
						resource: BindingResource::Buffer(back_channel.slice(..))
					}
				]
			});

		Self {
			herbivores: (herbivores, prefs.simulation.herbivores.budget),
			predators:  (predators,  prefs.simulation.predators.budget),
			back_channel,
			plane: (
				plane,
				prefs.simulation.horizontal_granularity,
				prefs.simulation.vertical_granularity
			),
			bind
		}
	}

	/** Read from the back channel buffer and copies the results. */
	pub async fn read_back_channel(&self) -> BackChannel {
		let channel = {
			let slice = self.back_channel.slice(..);
			slice.map_async(MapMode::Read)
				.await
				.expect("could not map back channel for reading");
			let mapped = slice.get_mapped_range();

			BackChannel::from_bytes(&*mapped)
		};

		self.back_channel.unmap();
		channel
	}

	/** Write to the given data to the back channel buffer. */
	pub async fn write_back_channel(&self, data: BackChannel) -> usize {
		let written = {
			let slice = self.back_channel.slice(..);

			slice.map_async(MapMode::Write)
				.await
				.expect("could not map back channel for writing");
			let mut mapped = slice.get_mapped_range_mut();

			let mut buf = Vec::with_capacity(16);
			data.bytes(&mut buf);

			let target = &mut *mapped;
			target.copy_from_slice(&buf[..]);

			buf.len()
		};

		self.back_channel.unmap();
		written
	}
}

/** An iterator that yields any number of identical bundles. */
struct BundleFactory<'a> {
	device: &'a Device,
	layout: &'a BindGroupLayout,
	prefs:  &'a Preferences,

	herbivores: Vec<u8>,
	predators:  Vec<u8>,
}
impl<'a> BundleFactory<'a> {
	/** Create a new iterator with the given parameters. */
	pub fn new(
		device: &'a Device,
		layout: &'a BindGroupLayout,
		prefs: &'a Preferences) -> Self {

		use crate::dataset;
		Self {
			device,
			layout,
			prefs,
			herbivores: dataset::population_bytes(&prefs.simulation.herbivores),
			predators:  dataset::population_bytes(&prefs.simulation.predators),
		}
	}
}
impl<'a> Iterator for BundleFactory<'a> {
	type Item = Bundle;
	fn next(&mut self) -> Option<Self::Item> {
		Some(Bundle::new_with_populations(
				self.device,
				self.layout,
				self.prefs,
				&self.herbivores,
				&self.predators))
	}
}

/** Shared state for the flipbook style channel. */
#[derive(Debug)]
struct Flipbook {
	/** Data bundles, in no specific order. */
	bundles: [Bundle; 3],
	/** Index setting specific roles to bundles in the storage array. */
	index: Mutex<Index>,
	/** Layout for the binding groups of the bundles. */
	layout: BindGroupLayout
}

#[derive(Debug)]
pub struct Consumer {
	book: Arc<Flipbook>,
}
impl Consumer {
	pub fn snapshot(&mut self) -> Snapshot {
		let (index, timestamp) = {
			let mut index = self.book.index.lock().unwrap();
			let now = Instant::now();
			if now.duration_since(index.storage.1) < now.duration_since(index.consumer.1) {
				let tmp = index.storage;
				index.storage  = index.consumer;
				index.consumer = tmp;
			}

			index.consumer
		};

		/* A note on the soundness of this bit of code:
		 *
		 * Technically speaking, passing a reference with the index we evaluate
		 * right now is not a sound strategy, that is a consequence of the way
		 * wgpu implements shared resource ownership. If we're not careful, it
		 * can lead to the same buffer being used by both the producer and the
		 * consumer at the same time.
		 *
		 * In order to avoid that from happening, we wil have to delve into a
		 * bit of semantics as an implementation detail: this function takes a
		 * mutable reference to the consumer, and so does the snapshot. Doing
		 * this guarantees that this function will not be called while a
		 * `Snapshot` is alive. Since this is the only function that is allowed
		 * to change the value of the index, we can guarantee at compile time
		 * that the user-facing API will behave in a sound manner.
		 */
		Snapshot {
			root: self,
			time: timestamp,
			data: index
		}
	}

	/** A reference to the bind group layout used by the buffer data contained
	 * in the snapshots obtained from this consumer. */
	pub fn layout(&self) -> &BindGroupLayout {
		&self.book.layout
	}

	/** The budget for individuals in the herbivore group in each of the
	 * snapshots obtained from this consumer. */
	pub fn herbivore_budget(&self) -> u32 {
		self.book.bundles[0].herbivores.1
	}

	/** The budget for individuals in the predator group in each of the
	 * snapshots obtained from this consumer. */
	pub fn predator_budget(&self) -> u32 {
		self.book.bundles[0].predators.1
	}
}

#[derive(Debug)]
pub struct Snapshot<'a> {
	/* Must be a mutable reference (See soundness note for the Consumer). */
	root: &'a mut Consumer,
	time: Instant,
	data: u8,
}
impl<'a> Snapshot<'a> {
	/** Data handle given the internal index. */
	fn data(&self) -> &Bundle {
		assert!(self.data < 3);
		&self.root.book.bundles[usize::from(self.data)]
	}

	/** A reference to the bind group for the data bundle contained in this
	 * snapshot. The layout of this bind group can be obtained by means of the
	 * [`Consumer::layout()`] function. */
	pub fn bind_group(&self) -> &BindGroup {
		&self.data().bind
	}

	/** Width of the current simulation plane, in storage units. */
	pub fn plane_width(&self) -> u32 {
		self.data().plane.1
	}

	/** Height of the current simulation plane, in storage units. */
	pub fn plane_height(&self) -> u32 {
		self.data().plane.2
	}

	/** Range of individuals currently alive in the herbivore group. */
	pub async fn herbivores(&self) -> Range<u32> {
		self.data()
			.read_back_channel()
			.await
			.herbivores
	}

	/** Range of individuals currently alive in the predator group. */
	pub async fn predators(&self) -> Range<u32> {
		self.data()
			.read_back_channel()
			.await
			.predators
	}
}


#[derive(Debug)]
pub struct Producer {
	book: Arc<Flipbook>,
}
impl Producer {
	pub fn frame(&mut self) -> Frame {
		let (index, _) = {
			let index = self.book.index.lock().unwrap();
			index.producer
		};

		/* A note on the soundness of this code:
		 *
		 * While the general gist of why this is not sound by default and how
		 * that is dealt with is the same as with the Consumer, there is a key
		 * difference in behavior that is worth noting for the Producer,
		 * specifically.
		 *
		 * Unlike with the Consumer, the change to `index.producer` only happens
		 * when a Frame gets dropped. So, in the Producer, the mutable reference
		 * is used as a means to guarantee no two Frames can exist at the same
		 * time, which guarantees no ill behavior from two frames getting
		 * dropped while having the same index.
		 */
		Frame {
			root: self,
			data: index
		}
	}

	/** A reference to the bind group layout used by the buffer data contained
	 * in the frames obtained from this producer. */
	pub fn layout(&self) -> &BindGroupLayout {
		&self.book.layout
	}

	/** The budget for individuals in the herbivore group in each of the frames
	 * obtained from this producer. */
	pub fn herbivore_budget(&self) -> u32 {
		self.book.bundles[0].herbivores.1
	}

	/** The budget for individuals in the predator group in each of the frames
	 * obtained from this producer. */
	pub fn predator_budget(&self) -> u32 {
		self.book.bundles[0].predators.1
	}
}

pub struct Frame<'a> {
	/* Must be a mutable reference (See soundness note for the Producer). */
	root: &'a mut Producer,
	data: u8
}
impl<'a> Frame<'a> {
	/** Data handle given the internal index. */
	fn data(&self) -> &Bundle {
		assert!(self.data < 3);
		&self.root.book.bundles[usize::from(self.data)]
	}

	/** A reference to the bind group for the data bundle contained in this
	 * frame. The layout of this bind group can be obtained by means of the
	 * [`Producer::layout()`] function. */
	pub fn bind_group(&self) -> &BindGroup {
		&self.data().bind
	}

	/** Width of the current simulation plane, in storage units. */
	pub fn plane_width(&self) -> u32 {
		self.data().plane.1
	}

	/** Height of the current simulation plane, in storage units. */
	pub fn plane_height(&self) -> u32 {
		self.data().plane.2
	}

	/** Read the back current back channel. */
	async fn back_channel(&self) -> BackChannel {
		self.data()
			.read_back_channel()
			.await
	}

	/** Range of individuals currently alive in the herbivore group. */
	pub async fn herbivores(&self) -> Range<u32> {
		self.back_channel()
			.await
			.herbivores
	}

	/** Range of individuals currently alive in the predator group. */
	pub async fn predators(&self) -> Range<u32> {
		self.back_channel()
			.await
			.predators
	}

	/** Set the range of individuals currently alive in the herbivore group.
	 *
	 * # Panics
	 * This function will panic if the given bounds fall outside the budget
	 * range of the herbivores or if the lower bound is greater than the upper
	 * bound. */
	pub async fn set_herbivores<R>(&mut self, range: R)
		where R: RangeBounds<u32> {

		let budget = self.data().herbivores.1;
		let lower = match range.start_bound() {
			Bound::Unbounded => 0,
			Bound::Excluded(val) => val.saturating_add(1),
			Bound::Included(val) => *val
		};
		let upper = match range.end_bound() {
			Bound::Unbounded => budget,
			Bound::Excluded(val) => *val,
			Bound::Included(val) => val.saturating_add(1)
		};

		if lower > upper {
			panic!("lower bound > upper bound: {} > {}", lower, upper);
		}
		if upper > budget {
			panic!("upper bound > budget: {} > {}", upper, budget);
		}
		if lower > budget {
			panic!("lower bound > budget: {} > {}", lower, budget);
		}

		let mut back_channel = self.back_channel().await;
		back_channel.herbivores = lower..upper;

		self.data()
			.write_back_channel(back_channel)
			.await;
	}

	/** Set the range of individuals currently alive in the predator group.
	 *
	 * # Panics
	 * This function will panic if the given bounds fall outside the budget
	 * range of the predators or if the lower bound is greater than the upper
	 * bound. */
	pub async fn set_predators<R>(&mut self, range: R)
		where R: RangeBounds<u32> {

		let budget = self.data().predators.1;
		let lower = match range.start_bound() {
			Bound::Unbounded => 0,
			Bound::Excluded(val) => val.saturating_add(1),
			Bound::Included(val) => *val
		};
		let upper = match range.end_bound() {
			Bound::Unbounded => budget,
			Bound::Excluded(val) => *val,
			Bound::Included(val) => val.saturating_add(1)
		};

		if lower > upper {
			panic!("lower bound > upper bound: {} > {}", lower, upper);
		}
		if upper > budget {
			panic!("upper bound > budget: {} > {}", upper, budget);
		}
		if lower > budget {
			panic!("lower bound > budget: {} > {}", lower, budget);
		}

		let mut back_channel = self.back_channel().await;
		back_channel.predators = lower..upper;

		self.data()
			.write_back_channel(back_channel)
			.await;
	}
}
impl<'a> Drop for Frame<'a> {
	fn drop(&mut self) {
		let now = Instant::now();
		let book = &*self.root.book;

		let mut index = book.index.lock().unwrap();
		index.producer.1 = now;

		let tmp = index.producer;
		index.producer = index.storage;
		index.storage  = tmp;
	}
}

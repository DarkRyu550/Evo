use wgpu::{BindGroupLayout, Buffer, Texture, BufferUsage, BindGroup, BindGroupLayoutDescriptor, BindGroupLayoutEntry, ShaderStage, BindingType, TextureViewDimension, TextureFormat, Device, TextureDescriptor, Extent3d, TextureDimension, TextureUsage, BindGroupDescriptor, BindGroupEntry, BindingResource, TextureViewDescriptor, TextureAspect, MapMode, Queue, TextureCopyView, Origin3d, TextureDataLayout, CommandBuffer, CommandEncoderDescriptor};
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
pub fn channel(state: Arc<State>, prefs: &Preferences) -> (Producer, Consumer) {

	let device = state.device();
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
				/* Lock plane. */
				BindGroupLayoutEntry {
					binding: 1,
					visibility: ShaderStage::COMPUTE | ShaderStage::VERTEX,
					ty: BindingType::StorageTexture {
						dimension: TextureViewDimension::D2,
						format: TextureFormat::R32Uint,
						readonly: false
					},
					count: None
				},
				/* Herbivore group. */
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
				/* Predator group. */
				BindGroupLayoutEntry {
					binding: 3,
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
					binding: 4,
					visibility: ShaderStage::COMPUTE | ShaderStage::VERTEX,
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
		let mut iter = BundleFactory::new(
			device,
			state.queue(),
			&layout,
			prefs);
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
		state,
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
	/** Handle to the underlying herbivore storage buffer. Along with the number
	 * of individuals allocated and the size of the data, in bytes. */
	herbivores: (Buffer, u32, u64),
	/** Handle to the underlying predator storage buffer. Along with the number
	 * of individuals allocated and the size of the data, in bytes. */
	predators: (Buffer, u32, u64),
	/** Handle to the host back-channeling buffer. Along with the size of the
	 * data, in bytes. */
	back_channel: (Buffer, u64),
	/** Handle to the simulation plane storage. */
	plane: (Texture, u32, u32),
	/** Handle to the simulation plane lock storage. */
	lock: (Texture, u32, u32),
	/** Bind group for the resources in this bundle. */
	bind: BindGroup
}
impl Bundle {
	/** Create a new bundle on the given device with the given preferences and
	 * the given initial buffer data. */
	fn new_with_populations<A, B>(
		device: &Device,
		queue:  &Queue,
		layout: &BindGroupLayout,
		prefs: &Preferences,
		herbivores: A,
		predators: B) -> Self
		where A: AsRef<[u8]>,
			  B: AsRef<[u8]> {


		let herbivores_len = herbivores.as_ref().len();
		let predators_len  = herbivores.as_ref().len();

		let herbivores = device.create_buffer_init(
			&BufferInitDescriptor {
				label: Some("Flipbook/Dataset/HerbivoreBuffer"),
				contents: &herbivores.as_ref()[..],
				usage: BufferUsage::STORAGE | BufferUsage::COPY_SRC
					| BufferUsage::COPY_DST
			});

		let predators = device.create_buffer_init(
			&BufferInitDescriptor {
				label: Some("Flipbook/Dataset/PredatorBuffer"),
				contents: &predators.as_ref()[..],
				usage: BufferUsage::STORAGE | BufferUsage::COPY_SRC
					| BufferUsage::COPY_DST
			});

		let mut back_channel_buf = Vec::with_capacity(16);
		let back_channel = BackChannel {
			herbivores: 0..prefs.simulation.herbivores.individuals,
			predators: 0..prefs.simulation.predators.individuals
		};
		let back_channel_len = back_channel.bytes(&mut back_channel_buf);

		let back_channel = device.create_buffer_init(
			&BufferInitDescriptor {
				label: Some("Flipbook/Dataset/BackChannelBuffer"),
				contents: &back_channel_buf[..],
				usage: BufferUsage::STORAGE | BufferUsage::MAP_READ
					| BufferUsage::MAP_WRITE | BufferUsage::COPY_DST
					| BufferUsage::COPY_SRC
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
				usage: TextureUsage::STORAGE | TextureUsage::COPY_SRC
					| TextureUsage::COPY_DST
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

		let mut clear = Vec::new();
		clear.resize_with(
			(prefs.simulation.horizontal_granularity
				* prefs.simulation.vertical_granularity * 16) as usize, || 0_u8);
		clear.chunks_exact_mut(4)
			.for_each(|source| {
				let [x, y, z, w] = f32::to_ne_bytes(0.0);

				source[0] = x;
				source[1] = y;
				source[2] = z;
				source[3] = w;
			});

		queue.write_texture(
			TextureCopyView {
				texture: &plane,
				mip_level: 0,
				origin: Origin3d { x: 0, y: 0, z: 0 }
			},
			&clear[..],
			TextureDataLayout {
				offset: 0,
				bytes_per_row: 16 * prefs.simulation.horizontal_granularity,
				rows_per_image: prefs.simulation.vertical_granularity
			},
			Extent3d {
				width: prefs.simulation.horizontal_granularity,
				height: prefs.simulation.vertical_granularity,
				depth: 1
			});

		let lock = device.create_texture(
			&TextureDescriptor {
				label: Some("Flipbook/Dataset/LockTexture"),
				size: Extent3d {
					width: prefs.simulation.horizontal_granularity,
					height: prefs.simulation.vertical_granularity,
					depth: 1
				},
				mip_level_count: 1,
				sample_count: 1,
				dimension: TextureDimension::D2,
				format: TextureFormat::R32Uint,
				usage: TextureUsage::STORAGE | TextureUsage::COPY_DST
					| TextureUsage::COPY_SRC
			});
		let lock_view = lock.create_view(
			&TextureViewDescriptor {
				label: Some("Flipbook/Dataset/PlaneTextureView"),
				format: Some(TextureFormat::R32Uint),
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
						resource: BindingResource::TextureView(&lock_view)
					},
					BindGroupEntry {
						binding: 2,
						resource: BindingResource::Buffer(herbivores.slice(..))
					},
					BindGroupEntry {
						binding: 3,
						resource: BindingResource::Buffer(predators.slice(..))
					},
					BindGroupEntry {
						binding: 4,
						resource: BindingResource::Buffer(back_channel.slice(..))
					}
				]
			});

		Self {
			herbivores: (
				herbivores,
				prefs.simulation.herbivores.budget,
				herbivores_len as u64
			),
			predators: (
				predators,
				prefs.simulation.predators.budget,
				predators_len as u64
			),
			back_channel: (
				back_channel,
				back_channel_len as u64
			),
			plane: (
				plane,
				prefs.simulation.horizontal_granularity,
				prefs.simulation.vertical_granularity
			),
			lock: (
				lock,
				prefs.simulation.horizontal_granularity,
				prefs.simulation.vertical_granularity
			),
			bind
		}
	}

	/** Read from the back channel buffer and copies the results. */
	pub async fn read_back_channel(&self) -> BackChannel {
		let channel = {
			let slice = self.back_channel.0.slice(..);
			slice.map_async(MapMode::Read)
				.await
				.expect("could not map back channel for reading");
			let mapped = slice.get_mapped_range();

			BackChannel::from_bytes(&*mapped)
		};

		self.back_channel.0.unmap();
		channel
	}

	/** Write to the given data to the back channel buffer. */
	pub async fn write_back_channel(&self, data: BackChannel) -> usize {
		let written = {
			let slice = self.back_channel.0.slice(..);

			slice.map_async(MapMode::Write)
				.await
				.expect("could not map back channel for writing");
			let mut mapped = slice.get_mapped_range_mut();

			let mut buf = Vec::with_capacity(16);
			data.bytes(&mut buf);

			let target = &mut *mapped;
			(&mut target[..buf.len()]).copy_from_slice(&buf[..]);

			buf.len()
		};

		self.back_channel.0.unmap();
		written
	}
}

/** An iterator that yields any number of identical bundles. */
struct BundleFactory<'a> {
	device: &'a Device,
	queue:  &'a Queue,
	layout: &'a BindGroupLayout,
	prefs:  &'a Preferences,

	herbivores: Vec<u8>,
	predators:  Vec<u8>,
}
impl<'a> BundleFactory<'a> {
	/** Create a new iterator with the given parameters. */
	pub fn new(
		device: &'a Device,
		queue:  &'a Queue,
		layout: &'a BindGroupLayout,
		prefs: &'a Preferences) -> Self {

		use crate::dataset;
		Self {
			device,
			queue,
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
				self.queue,
				self.layout,
				self.prefs,
				&self.herbivores,
				&self.predators))
	}
}

/** Shared state for the flipbook style channel. */
struct Flipbook {
	/** Underlying state. */
	state: Arc<State>,
	/** Data bundles, in no specific order. */
	bundles: [Bundle; 3],
	/** Index setting specific roles to bundles in the storage array. */
	index: Mutex<Index>,
	/** Layout for the binding groups of the bundles. */
	layout: BindGroupLayout,
}
impl Flipbook {
	/** Copies the data from the bundle at the first index to the bundle at the
	 * second index.
	 * # Panic
	 * This function panics if either of the given indices don't exist.
	 */
	pub fn bundle_copy(&self, source: u8, target: u8) {
		if source >= 3 { panic!("illegal copy bundle params: source ({}) > 3", source) }
		if target >= 3 { panic!("illegal copy bundle params: target ({}) > 3", target) }

		let source = usize::from(source);
		let target = usize::from(target);

		let device = self.state.device();

		let label = format!("Flipbook/Transfer[{} -> {}]", source, target);
		let mut encoder = device.create_command_encoder(
			&CommandEncoderDescriptor {
				label: Some(&label)
			});

		let bundles = &self.bundles;
		let i = source;
		let j = target;

		encoder.copy_buffer_to_buffer(
			&bundles[i].herbivores.0,
			0,
			&bundles[j].herbivores.0,
			0,
			if bundles[i].herbivores.2 != bundles[j].herbivores.2 {
				panic!("both herbivore bundles must have been the same \
						size, but, instead, we got: bundles[{}].herbivores.2 \
						({}) != bundles[{}].herbivores.2 ({})",
					i, bundles[i].herbivores.2,
					j, bundles[j].herbivores.2)
			} else {
				bundles[i].herbivores.2
			});

		encoder.copy_buffer_to_buffer(
			&bundles[i].predators.0,
			0,
			&bundles[j].predators.0,
			0,
			if bundles[i].predators.2 != bundles[j].predators.2 {
				panic!("both predator bundles must have been the same \
						size, but, instead, we got: bundles[{}].predators.2 \
						({}) != bundles[{}].predators.2 ({})",
					i, bundles[i].predators.2,
					j, bundles[j].predators.2)
			} else {
				bundles[i].predators.2
			});

		encoder.copy_buffer_to_buffer(
			&bundles[i].back_channel.0,
			0,
			&bundles[j].back_channel.0,
			0,
			if bundles[i].back_channel.1 != bundles[j].back_channel.1 {
				panic!("both back channel bundles must have been the same \
						size, but, instead, we got: bundles[{}].back_channel.1 \
						({}) != bundles[{}].back_channel.1 ({})",
					i, bundles[i].back_channel.1,
					j, bundles[j].back_channel.1)
			} else {
				bundles[i].back_channel.1
			});

		encoder.copy_texture_to_texture(
			TextureCopyView {
				texture: &bundles[i].plane.0,
				mip_level: 0,
				origin: Origin3d::ZERO
			},
			TextureCopyView {
				texture: &bundles[j].plane.0,
				mip_level: 0,
				origin: Origin3d::ZERO
			},
			Extent3d {
				width: if bundles[i].plane.1 != bundles[j].plane.1 {
					panic!("both simulation plane bundles must have been \
							the same width, but, instead, we got: \
							bundles[{}].plane.1 ({}) != \
							bundles[{}].plane.1 ({})",
						i, bundles[i].plane.1,
						j, bundles[j].plane.1)
				} else {
					bundles[i].plane.1
				},
				height: if bundles[i].plane.2 != bundles[j].plane.2 {
					panic!("both simulation plane bundles must have been \
							the same height, but, instead, we got: \
							bundles[{}].plane.2 ({}) != \
							bundles[{}].plane.2 ({})",
						i, bundles[i].plane.2,
						j, bundles[j].plane.2)
				} else {
					bundles[i].plane.2
				},
				depth: 1
			});

		encoder.copy_texture_to_texture(
			TextureCopyView {
				texture: &bundles[i].lock.0,
				mip_level: 0,
				origin: Origin3d::ZERO
			},
			TextureCopyView {
				texture: &bundles[j].lock.0,
				mip_level: 0,
				origin: Origin3d::ZERO
			},
			Extent3d {
				width: if bundles[i].lock.1 != bundles[j].lock.1 {
					panic!("both lock plane bundles must have been \
							the same width, but, instead, we got: \
							bundles[{}].lock.1 ({}) != \
							bundles[{}].lock.1 ({})",
						i, bundles[i].lock.1,
						j, bundles[j].lock.1)
				} else {
					bundles[i].lock.1
				},
				height: if bundles[i].lock.2 != bundles[j].lock.2 {
					panic!("both lock plane bundles must have been \
							the same height, but, instead, we got: \
							bundles[{}].lock.2 ({}) != \
							bundles[{}].lock.2 ({})",
						i, bundles[i].lock.2,
						j, bundles[j].lock.2)
				} else {
					bundles[i].lock.2
				},
				depth: 1
			});

		self.state.queue()
			.submit(std::iter::once(encoder.finish()))
	}
}

pub struct Consumer {
	book: Arc<Flipbook>,
}
impl Consumer {
	pub fn snapshot(&mut self) -> Snapshot {
		let (index, timestamp) = {
			let mut index = self.book.index.lock().unwrap();
			let now = Instant::now();

			if now.duration_since(index.storage.1) < now.duration_since(index.consumer.1) {
				self.book.bundle_copy(
					index.storage.0,
					index.consumer.0);
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
		index.storage.1 = now;

		self.root.book.bundle_copy(
			index.producer.0,
			index.storage.0);
	}
}

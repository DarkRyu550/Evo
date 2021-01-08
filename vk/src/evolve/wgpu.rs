use std::borrow::Borrow;
use crate::state::State;
use wgpu::{ShaderModule, PipelineLayout, ComputePipeline, ShaderModuleSource, Device, PipelineLayoutDescriptor, ComputePipelineDescriptor, ProgrammableStageDescriptor, CommandEncoderDescriptor, Buffer, BindGroupLayout, BindGroup, BufferUsage, BindGroupLayoutDescriptor, BindGroupLayoutEntry, ShaderStage, BindingType, BindGroupDescriptor, BindGroupEntry, BindingResource, Queue};
use crate::flipbook::Producer;
use std::time::Duration;
use wgpu::util::{DeviceExt, BufferInitDescriptor};
use crate::settings::Preferences;

/** An instance of the compute pipeline. */
struct Pipeline {
	/** Compute shader running the pipeline. */
	pub shader: ShaderModule,
	/** Handle to the layout of the compute pipeline. */
	pub layout: PipelineLayout,
	/** Handle to the compute pipeline. */
	pub pipeline: ComputePipeline
}
impl Pipeline {
	/** Creates a new compute pipeline with the given shader. */
	pub fn new(
		device:   &Device,
		flipbook: &Producer,
		shader:   ShaderModuleSource<'static>) -> Self {

		let shader = device.create_shader_module(shader);
		let layout = device.create_pipeline_layout(
			&PipelineLayoutDescriptor {
				label: Some("Evo/Pipeline/Layout"),
				bind_group_layouts: &[
					flipbook.layout()
				],
				push_constant_ranges: &[]
			});
		let pipeline = device.create_compute_pipeline(
			&ComputePipelineDescriptor {
				label: Some("Evo/Pipeline"),
				layout: Some(&layout),
				compute_stage: ProgrammableStageDescriptor {
					module: &shader,
					entry_point: "main"
				}
			});

		Self {
			shader,
			layout,
			pipeline
		}
	}
}

/** Parameters passed to the compute functions. */
struct ComputeParameters {
	/** Storage buffer backing the parameter data. */
	pub buffer: Buffer,
	/** Layout of the binding group for these parameters. */
	pub layout: BindGroupLayout,
	/** Binding group for these parameters. */
	pub bind: BindGroup
}
impl ComputeParameters {
	/** Creates a new compute parameter set. */
	pub fn new(
		device: &Device,
		params: crate::dataset::ComputeParameters) -> Self {

		let mut buffer = Vec::with_capacity(4);
		params.bytes(&mut buffer);
		let buffer = device.create_buffer_init(
			&BufferInitDescriptor {
				label: Some("Evo/ComputeParameters/Buffer"),
				contents: &buffer[..],
				usage: BufferUsage::UNIFORM
			});
		let layout = device.create_bind_group_layout(
			&BindGroupLayoutDescriptor {
				label: Some("Evo/ComputeParameters/Layout"),
				entries: &[
					BindGroupLayoutEntry {
						binding: 0,
						visibility: ShaderStage::COMPUTE,
						ty: BindingType::UniformBuffer {
							dynamic: false,
							min_binding_size: None
						},
						count: None
					}
				]
			});
		let bind = device.create_bind_group(
			&BindGroupDescriptor {
				label: Some("Evo/ComputeParameters/BindGroup"),
				layout: &layout,
				entries: &[
					BindGroupEntry {
						binding: 0,
						resource: BindingResource::Buffer(buffer.slice(..))
					}
				]
			});

		Self {
			buffer,
			layout,
			bind
		}
	}

	/** Update the data in the buffer with the given data. */
	pub fn update(&mut self, queue: &Queue, params: crate::dataset::ComputeParameters) {
		let mut buffer = Vec::with_capacity(4);
		params.bytes(&mut buffer);
		queue.write_buffer(&self.buffer, 0, &buffer[..]);
	}
}

pub struct Evo<A> {
	state: A,
	base_params: crate::dataset::ComputeParameters,
	params: ComputeParameters,
	flipbook: Producer,
	simulate: Pipeline,
	shuffle: Pipeline,
}
impl<A> Evo<A>
	where A: Borrow<State> {

	/** Creates a new instance of the evolution driver. */
	pub fn new(state: A, flipbook: Producer, prefs: &Preferences) -> Self {
		let device = state.borrow().device();

		let simulate = crate::shaders::compute::herbivores::simulate();
		let simulate = Pipeline::new(device, &flipbook, simulate);

		let shuffle = crate::shaders::compute::herbivores::shuffle();
		let shuffle = Pipeline::new(device, &flipbook, shuffle);

		let base_params = crate::dataset::ComputeParameters {
			delta: 0.0,
			herbivore_view_radius: prefs.simulation.herbivores.view_radius,
			predator_view_radius: prefs.simulation.predators.view_radius,
			herbivore_max_speed: prefs.simulation.herbivores.max_speed,
			predator_max_speed: prefs.simulation.predators.max_speed,
			herbivore_penalty: [
				prefs.simulation.herbivores.metabolism_min,
				prefs.simulation.herbivores.metabolism_max,
			],
			predator_penalty: [
				prefs.simulation.predators.metabolism_min,
				prefs.simulation.predators.metabolism_max,
			],
			simulation: [
				prefs.simulation.plane_width,
				prefs.simulation.plane_height
			]
		};
		let params = ComputeParameters::new(
			device,
			base_params);

		Self {
			state,
			base_params,
			params,
			flipbook,
			simulate,
			shuffle
		}
	}

	/** Run an iteration of the evolution algorithm. */
	pub async fn iterate(&mut self, delta: Duration) {
		let device = self.state.borrow().device();
		let queue = self.state.borrow().queue();
		let frame = self.flipbook.frame();

		/* Update the compute parameters. */
		self.params.update(
			queue,
			crate::dataset::ComputeParameters {
				delta: delta.as_secs_f32(),
				..self.base_params
			});

		let mut encoder = device.create_command_encoder(
			&CommandEncoderDescriptor {
				label: Some("Evo/CommandEncoder")
			});
		let mut pass = encoder.begin_compute_pass();

		pass.set_pipeline(&self.simulate.pipeline);
		pass.set_bind_group(0, frame.bind_group(), &[]);
		pass.dispatch(
			frame.herbivores().await.end,
			1,
			1);
		std::mem::drop(pass);

		queue.submit(std::iter::once(encoder.finish()));
	}
}

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
		params:   &ComputeParameters,
		flipbook: &Producer,
		shader:   ShaderModuleSource<'static>) -> Self {

		let shader = device.create_shader_module(shader);
		let layout = device.create_pipeline_layout(
			&PipelineLayoutDescriptor {
				label: Some("Evo/Pipeline/Layout"),
				bind_group_layouts: &[
					flipbook.layout(),
					&params.layout
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
				usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST
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

	pre_run_herbivores: Pipeline,
	simulate_herbivores: Pipeline,
	shuffle_herbivores: Pipeline,

	pre_run_predators: Pipeline,
	simulate_predators: Pipeline,
	shuffle_predators: Pipeline,

	update_plane: Pipeline,
}
impl<A> Evo<A>
	where A: Borrow<State> {

	/** Creates a new instance of the evolution driver. */
	pub fn new(state: A, flipbook: Producer, prefs: &Preferences) -> Self {
		let device = state.borrow().device();

		let base_params = crate::dataset::ComputeParameters {
			delta: 0.0,
			growth_rate: prefs.simulation.growth_rate,
			decomposition_rate: prefs.simulation.decomposition_rate,
			herbivore_view_radius: prefs.simulation.herbivores.view_radius,
			predator_view_radius: prefs.simulation.predators.view_radius,
			herbivore_max_speed: prefs.simulation.herbivores.max_speed,
			predator_max_speed: prefs.simulation.predators.max_speed,
			herbivore_reproduction_cost: prefs.simulation.herbivores.reproduction_cost,
			predator_reproduction_cost: prefs.simulation.predators.reproduction_cost,
			herbivore_reproduction_min: prefs.simulation.herbivores.reproduction_min,
			predator_reproduction_min: prefs.simulation.predators.reproduction_min,
			herbivore_offspring_energy: prefs.simulation.herbivores.offspring_energy,
			predator_offspring_energy: prefs.simulation.predators.offspring_energy,
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

		let simulate_herbivores = crate::shaders::compute::herbivores::simulate();
		let simulate_herbivores = Pipeline::new(device, &params, &flipbook, simulate_herbivores);

		let shuffle_herbivores = crate::shaders::compute::herbivores::shuffle();
		let shuffle_herbivores = Pipeline::new(device, &params, &flipbook, shuffle_herbivores);

		let simulate_predators = crate::shaders::compute::predators::simulate();
		let simulate_predators = Pipeline::new(device, &params, &flipbook, simulate_predators);

		let shuffle_predators = crate::shaders::compute::predators::shuffle();
		let shuffle_predators = Pipeline::new(device, &params, &flipbook, shuffle_predators);

		let pre_run_herbivores = crate::shaders::compute::herbivores::pre_run();
		let pre_run_herbivores = Pipeline::new(device, &params, &flipbook, pre_run_herbivores);

		let pre_run_predators = crate::shaders::compute::predators::pre_run();
		let pre_run_predators = Pipeline::new(device, &params, &flipbook, pre_run_predators);

		let update_plane = crate::shaders::compute::update_plane();
		let update_plane = Pipeline::new(device, &params, &flipbook, update_plane);

		Self {
			state,
			base_params,
			params,
			flipbook,
			simulate_herbivores,
			shuffle_herbivores,
			pre_run_predators,
			simulate_predators,
			shuffle_predators,
			pre_run_herbivores,
			update_plane
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

		let herbivores = frame.herbivores().await.end;
		let predators = frame.predators().await.end;


		let mut encoder = device.create_command_encoder(
			&CommandEncoderDescriptor {
				label: Some("Evo/CommandEncoder")
			});
		let mut pass = encoder.begin_compute_pass();

		/* Do the herbivore run. */
		pass.set_pipeline(&self.pre_run_herbivores.pipeline);
		pass.set_bind_group(0, frame.bind_group(), &[]);
		pass.set_bind_group(1, &self.params.bind, &[]);
		pass.dispatch(
			frame.plane_width(),
			frame.plane_height(),
			herbivores);

		pass.set_pipeline(&self.simulate_herbivores.pipeline);
		pass.set_bind_group(0, frame.bind_group(), &[]);
		pass.set_bind_group(1, &self.params.bind, &[]);
		pass.dispatch(
			herbivores,
			1,
			1);

		/* Do the predator run. */
		pass.set_pipeline(&self.pre_run_predators.pipeline);
		pass.set_bind_group(0, frame.bind_group(), &[]);
		pass.set_bind_group(1, &self.params.bind, &[]);
		pass.dispatch(
			frame.plane_width(),
			frame.plane_height(),
			predators);

		pass.set_pipeline(&self.simulate_predators.pipeline);
		pass.set_bind_group(0, frame.bind_group(), &[]);
		pass.set_bind_group(1, &self.params.bind, &[]);
		pass.dispatch(
			predators,
			1,
			1);

		/* Perform all of the required population shuffles. */
		pass.set_pipeline(&self.shuffle_herbivores.pipeline);
		pass.set_bind_group(0, frame.bind_group(), &[]);
		pass.set_bind_group(1, &self.params.bind, &[]);
		pass.dispatch(
			herbivores,
			1,
			1);

		pass.set_pipeline(&self.shuffle_predators.pipeline);
		pass.set_bind_group(0, frame.bind_group(), &[]);
		pass.set_bind_group(1, &self.params.bind, &[]);
		pass.dispatch(
			predators,
			1,
			1);

		/* Weave the results and update the plane. */
		pass.set_pipeline(&self.update_plane.pipeline);
		pass.set_bind_group(0, frame.bind_group(), &[]);
		pass.set_bind_group(1, &self.params.bind, &[]);
		pass.dispatch(
			frame.plane_width(),
			frame.plane_height(),
			1);

		std::mem::drop(pass);
		queue.submit(std::iter::once(encoder.finish()));
	}
}

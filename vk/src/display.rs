use crate::state::State;
use std::time::Duration;
use std::borrow::Borrow;
use winit::window::Window;
use std::sync::Arc;
use wgpu::{Surface, SwapChain, SwapChainDescriptor, TextureUsage, TextureFormat, PresentMode, SwapChainError, CommandEncoder, CommandEncoderDescriptor, RenderPassDescriptor, RenderPassColorAttachmentDescriptor, RenderPipeline, RenderPipelineDescriptor, PipelineLayout, PipelineLayoutDescriptor, BindGroupLayoutDescriptor, BindGroupEntry};
use crate::settings::{Preferences, PresentationMode};

pub struct Display<A> {
	state:     A,
	surface:   Surface,
	swapchain: SwapChain,

	individual_render_layout: PipelineLayout,
	individual_render_pipeline: RenderPipeline
}
impl<A> Display<A>
	where A: Borrow<State> {

	/** Creates a new display with the given instance. */
	pub fn new(state: A, surface: Surface, prefs: &Preferences) -> Self {
		let device = state.borrow().device();

		let swapchain = device.create_swap_chain(
			&surface,
			&SwapChainDescriptor {
				usage: TextureUsage::OUTPUT_ATTACHMENT,
				format: TextureFormat::Bgra8Unorm,
				width: prefs.window.width,
				height: prefs.window.height,
				present_mode: match prefs.window.swapchain_mode {
					PresentationMode::Mailbox => PresentMode::Mailbox,
					PresentationMode::Fifo => PresentMode::Fifo
				}
			});

		let individual_render_layout = device.create_pipeline_layout(
			&PipelineLayoutDescriptor {
				label: Some("DrawIndividuals/PipelineLayout"),
				bind_group_layouts: &[
					&device.create_bind_group_layout(
						&BindGroupLayoutDescriptor {
							label: None,
							entries: &[
								BindGroupEntry {
									binding: 0,
									resource: ()
								}
							]
						})
				],
				push_constant_ranges: &[]
			});

		let individual_render_pipeline = device.create_render_pipeline(
			&RenderPipelineDescriptor {
				label: Some("DrawIndividuals/Pipeline"),
				layout: &,
				vertex_stage: ProgrammableStageDescriptor {},
				fragment_stage: None,
				rasterization_state: None,
				primitive_topology: PrimitiveTopology::PointList,
				color_states: &[],
				depth_stencil_state: None,
				vertex_state: VertexStateDescriptor {},
				sample_count: 0,
				sample_mask: 0,
				alpha_to_coverage_enabled: false
			}
		)
		
		Self {
			state,
			surface,
			swapchain,
		}
	}

	pub fn iterate(&mut self, delta: Duration) {
		let state = self.state.borrow();

		/* Acquire the next image in the swapchain, blocking the current thread
		 * to wait for it if it is not immediately available. */
		let image = match self.swapchain.get_current_frame() {
			Ok(image) => image,
			Err(what) => match what {
				SwapChainError::Lost | SwapChainError::Outdated => {
					/* Recreate the swapchain. */
					panic!("TODO: Recreate the swapchain here")
				},
				SwapChainError::Timeout =>
					panic!("timed out while trying to get next frame")
				SwapChainError::OutOfMemory =>
					panic!("out of memory while trying to get the next frame")
			}
		};

		let mut buffer = state.device().create_command_encoder(
			&CommandEncoderDescriptor {
				label: Some("DrawIndividuals")
			})
			.begin_render_pass(
				&RenderPassDescriptor {
					color_attachments: &[
						RenderPassColorAttachmentDescriptor {
							attachment: &image.output.view,
							resolve_target: None,
							ops: Default::default()
						}
					],
					depth_stencil_attachment: None
				})
			.set_pipeline(&self.)



		buffer.
	}
}

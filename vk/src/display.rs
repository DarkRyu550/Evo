use crate::instance::Instance;
use std::time::Duration;
use std::borrow::Borrow;
use vulkano::swapchain::{Surface, Swapchain, ColorSpace, CompositeAlpha, PresentMode, FullscreenExclusive};
use winit::window::Window;
use std::sync::Arc;
use vulkano::format::Format;
use vulkano::image::{ImageUsage, SwapchainImage};
use vulkano::sync::{SharingMode, GpuFuture};
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::framebuffer::FramebufferAbstract;

pub struct Display<A, W> {
	swapchain: Arc<Swapchain<W>>,
	images: Vec<Arc<SwapchainImage<W>>>,

	framebuffer: Arc<dyn FramebufferAbstract + Send + Sync>,
	pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,

	instance: A,
}
impl<A, B> Display<A, B>
	where A: Borrow<Instance>,
		  B: Borrow<Surface<&Window>>{

	/** Creates a new display with the given instance. */
	pub fn new(inst: A, surface: Arc<Surface<W>>) -> Self {
		let instance = inst.borrow();
		let capabilities = surface.capabilities(instance.physical())
			.expect("could not query capabilities of the presentation surface");
		let (format, color) = capabilities.supported_formats.get(0)
			.expect("presentation surface supports no formats");
		debug!("Presenting in the {:?} format, {:?} color space", format, color);

		/* Create the swapchain. */
		let (swapchain, images) = Swapchain::new(
			instance.device().clone(),
			surface,
			3,
			*format,
			{
				let extent = capabilities.current_extent
					.expect("presentation surface should have an extent by now");
				debug!("Presenting with an extent of {}x{} pixels", extent[0], extent[1]);
				extent
			},
			1,
			ImageUsage::color_attachment(),
			SharingMode::Exclusive,
			capabilities.current_transform,
			CompositeAlpha::Inherit,
			{
				let mailbox = capabilities.present_modes.mailbox;

				if mailbox {
					debug!("Presenting in mailbox mode");
					PresentMode::Mailbox
				} else {
					debug!("Presenting in the fallback FIFO mode");
					PresentMode::Fifo
				}
			},
			FullscreenExclusive::Allowed,
			true,
			*color)
			.expect("could not create a presentation swapchain");

		/* Create the graphics pipelines that will be used for the drawing. */
		let pipeline = GraphicsPipeline::start()
			.triangle_strip()
			.build(instance.device().clone())
			.expect("could not build main pipeline");

		Self {
			instance: inst,
			swapchain,
			images
		}
	}

	pub fn iterate(&mut self, delta: Duration) {
		let instance = self.instance.borrow();

		/* Acquire the next image in the swapchain, blocking the current thread
		 * to wait for it if it is not immediately available. */
		let (index, _, future) = vulkano::swapchain
			::acquire_next_image(self.swapchain.clone(), None)
			.expect("could not acquire next image");
		future.then_signal_fence_and_flush()
			.expect("could not flush swapchain acquire")
			.wait(None);


		let mut buffer = AutoCommandBufferBuilder::primary_one_time_submit(
			instance.device().clone(),
			instance.graphics_queue_family()
		).expect("could not create a command buffer");


		buffer.
	}
}

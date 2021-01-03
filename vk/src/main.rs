#[macro_use]
extern crate log;

use winit::window::WindowBuilder;
use winit::event_loop::{EventLoop, ControlFlow};
use vulkano::instance::{Instance, ApplicationInfo, Version};
use vulkano::device::{Features, DeviceExtensions};
use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};
use std::time::Instant;
use std::sync::Arc;

mod display;
mod instance;
mod evolve;
mod carriage;
mod shaders;

fn main() {
	env_logger::init();

	let lp = EventLoop::new();
	let window = WindowBuilder::new()
		.with_resizable(false)
		.with_title("Evo")
		.with_inner_size(PhysicalSize{ width: 800, height: 600 })
		.build(&lp)
		.expect("could not initialize window");

	let app_info = ApplicationInfo {
		application_name: Some("Evo".into()),
		engine_name:      Some(env!("CARGO_PKG_NAME").into()),
		engine_version:   Some(Version { major: 0, minor: 1, patch: 0 }),
		.. Default::default()
	};
	let instance = Instance::new(
		Some(&app_info),
		&vulkano_win::required_extensions(),
		std::iter::empty()
	).expect("could not create vulkan instance");

	let surface = vulkano_win::create_vk_surface(&window, instance.clone())
		.expect("could not create window surface for vulkan");

	let instance = Arc::new(instance::Instance::new(
		instance,
		|dev| {
			let mut presentable = false;
			for family in dev.queue_families() {
				if surface.is_supported(family).unwrap() {
					presentable = true;
				}
			}

			presentable
		},
		&Features {
			shader_buffer_int64_atomics: true,
			.. Features::none()
		},
		&DeviceExtensions {
			khr_swapchain: true,
			.. DeviceExtensions::none()
		}).expect("could not create program instance"));
	let mut display = display::Display::new(instance);

	let mut time = Instant::now();
	lp.run(move |event, target, flow| {
		*flow = ControlFlow::Poll;
		if let Event::WindowEvent { event, .. } = event {
			match event {
				WindowEvent::Destroyed => {
					*flow = ControlFlow::Exit;
				},
				_ => {}
			}
		}

		let now = Instant::now();
		let delta = now.duration_since(time);
		time = now;

		display.iterate(delta);
	});
}

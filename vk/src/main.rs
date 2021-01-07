#[macro_use]
extern crate log;

use winit::window::WindowBuilder;
use winit::event_loop::{EventLoop, ControlFlow};
use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};
use std::time::Instant;
use crate::settings::Preferences;
use crate::state::State;
use crate::display::Display;
use std::sync::Arc;
use tokio::runtime::Runtime;
use wgpu::Maintain;
use log::LevelFilter;
use std::time::Duration;

mod display;
/*mod evolve;*/
mod shaders;
mod state;
mod settings;
mod flipbook;
mod dataset;
mod models;

fn main() {
	env_logger::builder()
		/* Disgusting. */
		.filter(Some("gfx_backend_vulkan"), LevelFilter::Off)
		.init();

	let prefs = Preferences::try_load()
		.unwrap_or_else(|what| {
			warn!("could not load settings file, falling back to defaults: {}", what);
			Default::default()
		});

	let window_size = PhysicalSize {
		width:  prefs.window.width,
		height: prefs.window.height
	};
	let event_loop = EventLoop::new();
	let window = WindowBuilder::new()
		.with_resizable(false)
		.with_title("Evo | Loading")
		.with_inner_size(window_size)
		.build(&event_loop)
		.expect("could not initialize window");

	let (state, surface) = futures::executor
		::block_on(State::new(|instance| {
			unsafe { instance.create_surface(&window) }
		}, &prefs)).expect("could not initialize state");
	let state = Arc::new(state);

	let (producer, consumer) = flipbook::channel(state.clone(), &prefs);
	let mut display = Display::new(
		state.clone(),
		surface,
		consumer,
		&prefs);

	/** Keep a thread taking care of polling the device. */
	std::thread::spawn(move || loop {
		state.device().poll(Maintain::Wait);
	});

	let runtime = tokio::runtime::Builder::new_multi_thread()
		.enable_all()
		.build()
		.unwrap();

	let mut time = Instant::now();

	let mut sec = Instant::now();
	let mut frames = 0_u64;

	window.set_title("Evo | FPS: 0");
	event_loop.run(move |event, target, flow| {
		let _ = (&window);
		*flow = ControlFlow::Poll;

		let mut pass = false;
		match event {
			Event::WindowEvent { event, .. } => match event {
				WindowEvent::CloseRequested => *flow = ControlFlow::Exit,
				_ => {}
			},
			Event::MainEventsCleared => pass = true,
			_ => {}
		}
		if !pass {
			/* We can start rendering only once all the events have been polled,
			 * which guarantees that we will never have more than a full frame
			 * of latency. */
			return;
		}

		let now = Instant::now();
		let delta = now.duration_since(time);
		time = now;

		runtime.block_on(display.iterate(delta));

		frames += 1;
		if sec.elapsed() >= Duration::from_secs(1) {
			sec = Instant::now();

			window.set_title(&format!("Evo | FPS: {}", frames));
			frames = 0;
		}
	});
}

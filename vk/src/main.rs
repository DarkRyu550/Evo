#![feature(min_const_generics)]
#[macro_use]
extern crate log;

use winit::window::WindowBuilder;
use winit::event_loop::{EventLoop, ControlFlow};
use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};
use std::time::Instant;
use crate::settings::Preferences;
use crate::state::State;

/*mod display;*/
mod state;
mod settings;
mod flipbook;
mod dataset;

fn main() {
	env_logger::init();
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
		.with_title("Evo")
		.with_inner_size(window_size)
		.build(&event_loop)
		.expect("could not initialize window");

	let (state, surface) = futures::executor
		::block_on(State::new(move |instance| {
			unsafe { instance.create_surface(&window) }
		})).expect("could not initialize state");

	let mut time = Instant::now();
	event_loop.run(move |event, target, flow| {
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

		/*display.iterate(delta);*/
	});
}

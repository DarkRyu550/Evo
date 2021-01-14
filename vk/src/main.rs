#![feature(exclusive_range_pattern)]
#![cfg_attr(debug_assertions, allow(dead_code, unused_variables))]

#[macro_use]
extern crate log;

use winit::window::WindowBuilder;
use winit::event_loop::{EventLoop, ControlFlow};
use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};
use std::time::Instant;
use crate::settings::{Preferences, SimulationMode};
use crate::state::State;
use crate::display::Display;
use std::sync::Arc;
use wgpu::Maintain;
use log::LevelFilter;
use std::time::Duration;
use crate::evolve::wgpu::Evo;
use crate::evolve::cpu::World;

mod display;
mod shaders;
mod state;
mod settings;
mod flipbook;
mod dataset;
mod models;
mod evolve;

/** Backend driver to be used for evolution. */
enum Backend {
	/** Use the GPU code. */
	Gpu(Evo<Arc<State>>),
	/** Use the CPU code. */
	Cpu(World)
}

fn main() {
	env_logger::builder()
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

	let runtime = tokio::runtime::Builder::new_multi_thread()
		.enable_all()
		.build()
		.unwrap();

	let mut backend = if prefs.simulation.mode == SimulationMode::Gpu {
		let evo = Evo::new(
			state.clone(),
			producer,
			&prefs);
		Backend::Gpu(evo)
	} else {
		let world = World::new(&prefs.simulation);
		Backend::Cpu(world)
	};

	/* Keep a thread taking care of polling the device. */
	std::thread::spawn(move || loop {
		state.device().poll(Maintain::Wait);
	});

	/* Spawn the evolution loop. It still should run in real time, except that
	 * now it may be able to run more or steps than it would be able to run if
	 * it were tied to the frame rate. */
	runtime.spawn(async move {
		if let Ok("1") = std::env::var("EVO_DROP_SIMULATION")
			.as_ref()
			.map(|s| s.as_str()) {

			/* We were instructed to drop the simulation. */
			warn!("Environment variable EVO_DROP_SIMULATION is set to \"1\"");
			warn!("Dropping the simulation task immediately!");
			return;
		}

		let mut time = Instant::now();
		loop {
			let now = Instant::now();
			let delta = now.duration_since(time);
			time = now;

			/* Dilate and clamp. */
			let delta_dil = delta.as_nanos() as f64;
			let delta_dil = delta_dil * f64::from(prefs.simulation.time_dilation);
			let delta_dil = delta_dil.round() as u128;

			let max = Duration::from_secs_f64(
				f64::from(prefs.simulation.max_discrete_time));

			let delta = if delta_dil > max.as_nanos() {
				warn!("time step of the simulation had to be clamped from {:?} \
					to the maximum specified value of {:?}",
					delta,
					max);
				warn!("considering lowering the dilation factor or increasing \
					the time window tolerance of the simulation");

				max
			} else {
				Duration::new(
					(delta_dil / 1_000_000_000) as u64,
					(delta_dil % 1_000_000_000) as u32)
			};


			/* Simulate. */
			match backend {
				Backend::Gpu(ref mut driver) => driver.iterate(delta).await,
				Backend::Cpu(ref mut driver) => driver.step(delta)
			}
		}
	});

	let mut time = Instant::now();

	let mut sec = Instant::now();
	let mut frames = 0_u64;

	window.set_title("Evo | FPS: 0");
	event_loop.run(move |event, target, flow| {
		let _ = &window;
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

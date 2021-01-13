use std::error::Error;
use std::fs::File;
use std::io::Read;
use serde::{Serialize, Deserialize};

/** Description of a pheromone. */
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Pheromone {
	/** Amount of the red chemical. Clamped between 0.0 and 1.0. */
	pub red: f32,
	/** Amount of the green chemical. Clamped between 0.0 and 1.0. */
	pub green: f32,
	/** Amount of the blue chemical. Clamped between 0.0 and 1.0.*/
	pub blue: f32
}

/** Settings controlling specific groups of individuals in the simulation. */
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Group {
	/** Number of individuals this group will start off with. */
	pub individuals: u32,
	/** Maximum number of individuals this groups will allow. */
	pub budget: u32,
	/** Spawn area for this group. */
	pub spawn_range: [f32; 4],
	/** Radius of vision in simulation board units. */
	pub view_radius: f32,
	/** Amount of energy to consume while standing still. */
	pub metabolism_min: f32,
	/** Amount of energy to consume while running at max speed. */
	pub metabolism_max: f32,
	/** Maximum speed individuals in this group can reach. */
	pub max_speed: f32,
	/** Energy needed to be put in by both parents to reproduce. */
	pub reproduction_cost: f32,
	/** Minimum energy needed for two parents to decide to reproduce. */
	pub reproduction_min: f32,
	/** Energy newborns of this group group start off with. */
	pub offspring_energy: f32,
	/** The signature pheromone composition for this group. This will be used as
	 * the initial value for the chemical composition in the genes of all
	 * individuals of the group. */
	pub signature: Pheromone,
	/** Whether to initialize the other parameters to random values. */
	pub init_to_random: bool,
}

/** Settings controlling all the parameters for the simulation. */
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Simulation {
	/** Whether the cpu or gpu should be used. */
	pub mode: SimulationMode,
	/** Time dilation factor of the simulation. */
	pub time_dilation: f32,
	/** Minimum length a discrete time step is allowed to cover. Any simulation
	 * steps that tries to cover more time than this will be clamped to this
	 * value. */
	pub max_discrete_time: f32,

	/** Width of the simulation plane. */
	pub plane_width: f32,
	/** Height of the simulation plane. */
	pub plane_height: f32,

	/** Granularity of the pheromone cells on the horizontal axis of the plane.
	 *
	 * This parameter controls the number of cells this will be lined up
	 * horizontally on every row of the simulation plane. Consequently, the
	 * width of every cell will be equal to
	 * `plane_width / horizontal_granularity`.
	 */
	pub horizontal_granularity: u32,

	/** Granularity of the pheromone cells on the vertical axis of the plane.
	 *
	 * This parameter controls the number of cells this will be stacked
	 * vertically on every column of the simulation plane. Consequently, the
	 * height of every cell will be equal to
	 * `plane_height / vertical_granularity`.
	 */
	pub vertical_granularity: u32,

	/** Growth rate of the grass on the field, in units per second. */
	pub growth_rate: f32,
	/** Decomposition rate of the chemicals on the field, in units per second. */
	pub decomposition_rate: f32,

	/** Parameters for the herbivore group. */
	pub herbivores: Group,
	/** Parameter for the predator group. */
	pub predators: Group,
}

/** Modes of presentation for the swapchain.
 *
 * See the Vulkan documentation for more information on what these actually
 * do and how these behave like. */
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum PresentationMode {
	Mailbox,
	Fifo,
}

/** Backend to use for wgpu. */
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum Backend {
	Vulkan,
	GL,
	Metal,
	DX12,
	DX11,
	BrowserWebGpu
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum SimulationMode {
	Cpu,
	Gpu
}

/** Settings for the window and display of the simulation. */
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Window {
	/** Width of the display window. */
	pub width: u32,
	/** Height of the display window. */
	pub height: u32,
	/** Allowed presentation modes in order of priority. */
	pub swapchain_mode: PresentationMode,
	/** Backends to use */
	pub backends: Vec<Backend>
}

/** Application settings. */
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Preferences {
	/** Section for the window and display settings. */
	pub window: Window,
	/** Section for the simulation settings. */
	pub simulation: Simulation
}
impl Preferences {
	/** Tries to load the application preferences from the default location. */
	pub fn try_load() -> Result<Self, Box<dyn Error>> {
		let mut file = File::open("Settings.toml")?;
		let mut data = Vec::new();
		file.read_to_end(&mut data)?;

		toml::from_slice(&data[..])
			.map_err(Into::into)
	}
}
impl Default for Preferences {
	fn default() -> Self {
		Self {
			window: Window {
				width: 800,
				height: 600,
				swapchain_mode: PresentationMode::Mailbox,
				backends: vec![Backend::Vulkan, Backend::Metal,
							   Backend::DX12, Backend::BrowserWebGpu]
			},
			simulation: Simulation {
				mode: SimulationMode::Gpu,
				time_dilation: 1.0,
				max_discrete_time: 0.5,
				plane_width: 100.0,
				plane_height: 100.0,
				horizontal_granularity: 100,
				vertical_granularity: 100,
				growth_rate: 0.1,
				decomposition_rate: 0.1,
				herbivores: Group {
					individuals: 100,
					budget: 1024,
					spawn_range: [0.0, 100.0, 0.0, 100.0],
					view_radius: 1.0,
					metabolism_min: 0.01,
					metabolism_max: 0.05,
					max_speed: 10.0,
					reproduction_cost: 0.1,
					reproduction_min: 0.5,
					offspring_energy: 1.0,
					signature: Pheromone {
						red:   0.0,
						green: 1.0,
						blue:  1.0
					},
					init_to_random: true
				},
				predators: Group {
					individuals: 10,
					budget: 1024,
					spawn_range: [0.0, 100.0, 0.0, 100.0],
					view_radius: 1.0,
					metabolism_min: 0.02,
					metabolism_max: 0.10,
					max_speed: 10.0,
					reproduction_cost: 0.4,
					reproduction_min: 0.8,
					offspring_energy: 1.0,
					signature: Pheromone {
						red:   1.0,
						green: 0.0,
						blue:  0.0
					},
					init_to_random: true
				}
			}
		}
	}
}

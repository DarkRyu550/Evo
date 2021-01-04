use std::error::Error;
use std::fs::File;
use std::io::Read;

/** Description of a pheromone. */
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Pheromone {
	/** Amount of the red chemical. Clamped between 0.0 and 1.0. */
	pub red: f64,
	/** Amount of the green chemical. Clamped between 0.0 and 1.0. */
	pub green: f64,
	/** Amount of the blue chemical. Clamped between 0.0 and 1.0.*/
	pub blue: f64
}

/** Settings controlling specific groups of individuals in the simulation. */
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Group {
	/** Number of individuals this group will start off with. */
	pub individuals: u32,
	/** Radius of vision in simulation board units. */
	pub view_radius: f64,
	/** The signature pheromone composition for this group. This will be used as
	 * the initial value for the chemical composition in the genes of all
	 * individuals of the group. */
	pub signature: Pheromone
}

/** Settings controlling all the parameters for the simulation. */
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Simulation {
	/** Width of the simulation plane. */
	pub plane_width: f64,
	/** Height of the simulation plane. */
	pub plane_height: f64,

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

/** Settings for the window and display of the simulation. */
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Window {
	/** Width of the display window. */
	pub width: u32,
	/** Height of the display window. */
	pub height: u32,
	/** Allowed presentation modes in order of priority. */
	pub swapchain_modes: Vec<PresentationMode>,
}

/** Application settings. */
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
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

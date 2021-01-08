use crate::dataset::Individual;
use crate::settings::{Preferences, Simulation};

#[derive(Copy, Clone, Debug)]
pub struct Cell {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub grass: f32
}

#[derive(Clone, Debug)]
pub struct Map {
    cells: Vec<Cell>,
    width: u32
}

impl Map {
    fn new(params: &Simulation) -> Self {
        let mut cells = Vec::with_capacity(
            (params.horizontal_granularity * params.vertical_granularity) as usize
        );

        for _ in 0..cells.capacity() {
            cells.push(Cell {
                red: 0.0,
                green: 0.0,
                blue: 0.0,
                grass: 0.0
            })
        }

        Self {
            cells,
            width: params.horizontal_granularity
        }
    }

    pub fn cells_around(&self, x: u32, y: u32, radius: f32) -> impl Iterator<Item=&Cell> {
        let rsquared = radius.powf(2f32);
        self.cells.iter().enumerate().filter(move |(pos, cell)| {
            let (cx, cy) = self.pos_of(*pos);
            let dx = cx as i32 - x as i32;
            let dy = cy as i32 - y as i32;
            (dx as f32).powf(2.0) + (dy as f32).powf(2.0) < rsquared
        }).map(|(_, cell)| cell)
    }

    #[inline(always)]
    pub fn cell_at(&self, x: u32, y: u32) -> &Cell {
        &self.cells[self.cell_index(x, y)]
    }

    #[inline(always)]
    pub fn cell_at_mut(&mut self, x: u32, y: u32) -> &mut Cell {
        //make borrow checker happy
        let i = self.cell_index(x, y);
        &mut self.cells[i]
    }

    #[inline(always)]
    fn cell_index(&self, x: u32, y: u32) -> usize {
        (y * self.width + x) as usize
    }

    #[inline(always)]
    fn pos_of(&self, idx: usize) -> (u32, u32) {
        (idx as u32 % self.width, idx as u32 / self.width)
    }
}

#[derive(Clone, Debug)]
pub struct State {
    pub herbivores: Vec<Individual>,
    pub carnivores: Vec<Individual>,
    pub map: Map,
    params: Simulation,
}

impl State {
    fn new(params: &Simulation) -> Self {
        Self {
            herbivores: crate::dataset::population(&params.herbivores),
            carnivores: crate::dataset::population(&params.predators),
            map: Map::new(params),
            params: params.clone()
        }
    }

    fn step(&self, output: &mut State) {
        let bounds_check = {
            let max_x = self.params.plane_width;
            let max_y = self.params.plane_height;
            move |pos: [f32; 2], vel: [f32; 2]| {
                [
                    (pos[0] + vel[0]).clamp(0.0, max_x),
                    (pos[1] + vel[1]).clamp(0.0, max_y)
                ]
            }
        };

        let herb_step = {
            move |i: &mut Individual| {
                i.velocity = [
                    1.0 - i.velocity[0],
                    1.0 - i.velocity[1]
                ];
                i.position = bounds_check(i.position, i.velocity)
            }
        };
        group_step(&self.herbivores, &mut output.herbivores, herb_step);

        let pred_step = {
            let herb = &mut output.herbivores;
            move |i: &mut Individual| {

            }
        };
        group_step(&self.carnivores, &mut output.carnivores, pred_step);
    }

    pub fn herbivores_around(&self, x: f32, y: f32, radius: f32) -> impl Iterator<Item=&Individual> {
        let rsquared = radius.powf(2f32);
        self.herbivores.iter().filter(move |h| {
            (h.position[0] - x).powf(2f32) + (h.position[1] - y).powf(2f32) < rsquared
        })
    }
}

#[derive(Clone, Debug)]
pub struct World {
    state: [State; 2],
    current_state: usize
}

impl World {
    pub fn new(params: &Simulation) -> Self {
        let state = State::new(params);
        World {
            state: [state.clone(), state.clone()],
            current_state: 0
        }
    }

    pub fn step(&mut self) {
        let (a, b) = self.state.split_at_mut(1);
        let (orig, dest) = if self.current_state == 0 {
            (a, b)
        } else {
            (b, a)
        };
        orig[0].step(&mut dest[0]);
        self.current_state = 1 - self.current_state;
    }

    pub fn current_state(&self) -> &State {
        &self.state[self.current_state]
    }

    pub fn current_state_mut(&mut self) -> &mut State {
        &mut self.state[self.current_state]
    }
}


fn group_step<F: FnMut(&mut Individual) -> ()>(src: &Vec<Individual>, dest: &mut Vec<Individual>, mut f: F) {
    dest.clear();
    src.iter()
        .filter_map(|i| {
            if i.energy <= 0.0 {
                return None;
            }
            let mut i = *i;
            f(&mut i);
            Some(i)
        })
        .for_each(|i| dest.push(i));
}
use crate::dataset::Individual;
use crate::settings::{Preferences, Simulation};

#[derive(Copy, Clone, Debug)]
struct Cell {
    red: f32,
    green: f32,
    blue: f32,
    grass: f32
}

#[derive(Clone, Debug)]
struct State {
    herbivores: Vec<Individual>,
    carnivores: Vec<Individual>,
    map: Vec<Cell>,
    params: Simulation,
}

impl State {
    fn new(params: &Simulation) -> Self {
        let mut map = Vec::with_capacity(
            (params.horizontal_granularity * params.vertical_granularity) as usize
        );

        for _ in 0..map.capacity() {
            map.push(Cell {
                red: 0.0,
                green: 0.0,
                blue: 0.0,
                grass: 0.0
            })
        }

        Self {
            herbivores: crate::dataset::population(&params.herbivores),
            carnivores: crate::dataset::population(&params.predators),
            map,
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
            };
        };
        let herb_step = {
            move |i: &mut Individual| {

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

    pub fn herbivores_at(&self, x: f32, y: f32, radius: f32) -> impl Iterator<Item=&Individual> {
        let rsquared = radius.powf(2f32);
        self.herbivores.iter().filter(move |h| {
            (h.position[0] - x).powf(2f32) + (h.position[1] - y).powf(2f32) < rsquared
        })
    }

    #[inline(always)]
    pub fn cell_at(&self, x: u32, y: u32) -> &Cell {
        &self.map[self.cell_index(x, y)]
    }

    #[inline(always)]
    pub fn cell_at_mut(&mut self, x: u32, y: u32) -> &mut Cell {
        //make borrow checker happy
        let i = self.cell_index(x, y);
        &mut self.map[i]
    }

    #[inline(always)]
    fn cell_index(&self, x: u32, y: u32) -> usize {
        (y * self.params.horizontal_granularity + x) as usize
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
}


fn group_step<F: FnMut(&mut Individual) -> ()>(src: &Vec<Individual>, dest: &mut Vec<Individual>, mut f: F) {
    //let mut items = std::mem::replace(v, vec!());
    //items = items.into_iter()
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
    //*v = items;
}

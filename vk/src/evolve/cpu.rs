use std::time::Duration;

use crate::dataset::Individual;
use crate::settings::{Simulation, Group};

#[derive(Copy, Clone, Debug)]
pub struct Cell {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub grass: f32,
}

#[derive(Clone, Debug)]
pub struct Map {
    cells: Vec<Cell>,
    width: u32,
    #[cfg(debug_assertions)]
    height: u32,
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
                grass: 0.0,
            })
        }

        Self {
            cells,
            width: params.horizontal_granularity,
            #[cfg(debug_assertions)]
            height: params.vertical_granularity,
        }
    }

    pub fn decay(&mut self, decay_red: f32, decay_green: f32, decay_blue: f32, grass_growth: f32) {
        for mut c in self.cells.iter_mut() {
            c.red -= decay_red;
            c.green -= decay_green;
            c.blue -= decay_blue;
            c.grass += grass_growth;
        }
    }

    pub fn cells_around(&self, x: u32, y: u32, radius: f32) -> impl Iterator<Item=&Cell> {
        let rsquared = radius.powf(2f32);
        self.cells.iter().enumerate().filter(move |(pos, _cell)| {
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
        #[cfg(debug_assertions)]
            {
                assert!(x < self.width, "Invalid X coordinate");
                assert!(y < self.height, "Invalid Y coordinate");
            }
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
        let population = {
            let max_x = params.plane_width - 0.01;
            let max_y = params.plane_height - 0.01;
            move |params: &Group| {
                let mut p = crate::dataset::population(params);
                for mut i in p.iter_mut() {
                    i.position = [
                        i.position[0].clamp(0.0, max_x),
                        i.position[1].clamp(0.0, max_y)
                    ];
                }
                p
            }
        };
        Self {
            herbivores: population(&params.herbivores),
            carnivores: population(&params.predators),
            map: Map::new(params),
            params: params.clone(),
        }
    }

    fn gradient<F: Fn(&Cell) -> f32>(&self, group: &Group, individual: &Individual, selector: F) -> (f32, f32, f32) {
        let radius = f32::min(
            group.view_radius / self.params.plane_width,
            group.view_radius / self.params.plane_height,
        );
        let (top, bottom, left, right) = {
            let [x, y] = individual.position;
            (
                (y - radius).clamp(0.0, self.params.plane_height).round() as u32,
                (y + radius).clamp(0.0, self.params.plane_height).round() as u32,
                (x - radius).clamp(0.0, self.params.plane_width).round() as u32,
                (x + radius).clamp(0.0, self.params.plane_width).round() as u32,
            )
        };
        let center = self.individual_pos(individual);
        let center_val = selector(self.map.cell_at(center.0, center.1));

        let mut gradient = (0.0f32, 0.0);

        for i in top..=bottom {
            for j in left..=right {
                let (direction, dist) = {
                    let (x, y) = ((i - center.0) as f32, (j - center.1) as f32);
                    let mag = (x.powf(2.0) + y.powf(2.0)).sqrt();
                    ((x / mag, y / mag), mag)
                };

                if dist > radius {
                    continue;
                }

                let val = selector(self.map.cell_at(j, i));
                gradient = (
                    gradient.0 + direction.0 * (val - center_val),
                    gradient.1 + direction.1 * (val - center_val),
                );
            }
        }
        {
            let mag = (gradient.0.powf(2.0) + gradient.1.powf(2.0)).sqrt();
            (gradient.0 / mag, gradient.1 / mag, mag)
        }
    }

    fn gradients(&self, group: &Group, individual: &Individual) -> [(f32, f32, f32); 4] {
        [
            self.gradient(group, individual, |c| c.red),
            self.gradient(group, individual, |c| c.green),
            self.gradient(group, individual, |c| c.blue),
            self.gradient(group, individual, |c| c.grass),
        ]
    }

    fn step(&self, output: &mut State, delta: Duration) {
        // This function *must* copy all (needed) state to output, which means all mutable fields,
        // otherwise state will get lost. The map is blindly copied at the beginning because it's
        // only read from the output (and updated there, obviously).
        // `Individual`s use the group_step function to write to the output, which also handles removal
        // of dead individuals (and avoids copying at all for those). Herbivores killed by predators
        // must pay the price of a Vec remove call, but there's not much that can be done, since herbivores
        // move first (so just setting a "dead" flag on them won't work)

        let delta = delta.as_secs_f32();

        (&mut output.map.cells[..]).copy_from_slice(&self.map.cells[..]);

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
            let map = &mut output.map;
            move |i: &mut Individual| {
                let (x, y) = self.individual_pos(i);
                /* energy */
                {
                    let cell = map.cell_at_mut(x, y);
                    let eat = cell.grass.min(1f32 - i.energy);
                    i.energy += eat;
                    cell.grass -= eat;
                }

                /* math go brrrr */
                let nn_result = {
                    let weights = ndarray::arr2(&i.weights);
                    let inputs = ndarray::arr1({
                        let [grad_r, grad_g, grad_b, grad_a] = self.gradients(&self.params.herbivores, i);
                        &[
                            i.velocity[0],
                            i.velocity[1],
                            grad_r.0, grad_r.1, grad_r.2,
                            grad_g.0, grad_g.1, grad_g.2,
                            grad_b.0, grad_b.1, grad_b.2,
                            grad_a.0, grad_a.1, grad_a.2
                        ]
                    }).into_shape((14, 1)).expect("Unable to reshape inputs to (14, 1)");
                    let biases = ndarray::arr1(&i.biases)
                        .into_shape((5, 1)).expect("Unable to reshape biases to (5, 1)");
                    let mut result = weights.dot(&inputs) + biases;
                    debug_assert!(result.len() == 5, "Wrong result length");
                    result.map_inplace(|f| {
                        let exp = f.exp();
                        *f = exp / (exp + 1.0);
                    });
                    result.into_shape((5, )).expect("Unable to reshape result to (5,)")
                };

                /* movement and energy */
                {
                    let theta = nn_result[0];
                    let magnitude = nn_result[1];
                    let mul = self.params.herbivores.max_speed * delta;
                    let movement = [
                        magnitude * f32::cos(theta * 2.0 * std::f32::consts::PI) * mul,
                        magnitude * f32::sin(theta * 2.0 * std::f32::consts::PI) * mul
                    ];

                    i.position = bounds_check(i.position, movement);
                    i.velocity = movement;

                    let penalty = {
                        let v = delta * magnitude;
                        self.params.herbivores.metabolism_min * (1.0 - v) + self.params.herbivores.metabolism_max * v
                    };

                    i.energy -= penalty;
                }

                /* drop pheromones */
                {
                    let mut cell = map.cell_at_mut(x, y);
                    cell.red = f32::clamp(nn_result[2], 0.0, 1.0);
                    cell.green = f32::clamp(nn_result[3], 0.0, 1.0);
                    cell.blue = f32::clamp(nn_result[4], 0.0, 1.0);
                }
            }
        };
        group_step(&self.herbivores, &mut output.herbivores, herb_step);

        let pred_step = {
            // Killing is implemented as setting energy to 0, such that the herbivore gets removed on the next
            // iteration. Code that renders the state should skip any individual with negative energy.
            let herb = &mut output.herbivores;
            fn herbivores_around(vec: &mut Vec<Individual>, x: f32, y: f32, radius: f32) -> impl Iterator<Item=&mut Individual> {
                let dist = radius.powf(2f32);
                vec.iter_mut().filter(move |h| {
                    h.energy > 0.0 && (h.position[0] - x).powf(2f32) + (h.position[1] - y).powf(2f32) < dist
                })
            }
            move |i: &mut Individual| {
                //TODO
            }
        };
        group_step(&self.carnivores, &mut output.carnivores, pred_step);

        output.map.decay(
            self.params.decomposition_rate,
            self.params.decomposition_rate,
            self.params.decomposition_rate,
            self.params.growth_rate,
        );
    }

    fn individual_pos(&self, i: &Individual) -> (u32, u32) {
        (
            (i.position[0] / self.params.plane_width).floor() as u32,
            (i.position[1] / self.params.plane_height).floor() as u32,
        )
    }
}

#[derive(Clone, Debug)]
pub struct World {
    state: [State; 2],
    current_state: usize,
}

impl World {
    pub fn new(params: &Simulation) -> Self {
        let state = State::new(params);
        World {
            state: [state.clone(), state.clone()],
            current_state: 0,
        }
    }

    pub fn step(&mut self, delta: Duration) {
        let (a, b) = self.state.split_at_mut(1);
        let (orig, dest) = if self.current_state == 0 {
            (a, b)
        } else {
            (b, a)
        };
        orig[0].step(&mut dest[0], delta);
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

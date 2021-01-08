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
pub struct World {
    herbivores: Vec<Individual>,
    carnivores: Vec<Individual>,
    map: Vec<Cell>,
    width: u32,
    height: u32
}

impl World {
    pub fn new(settings: &Simulation) -> World {
        let mut map = Vec::with_capacity(
            (settings.horizontal_granularity * settings.vertical_granularity) as usize
        );

        for _ in 0..map.capacity() {
            map.push(Cell {
                red: 0f32,
                green: 0f32,
                blue: 0f32,
                grass: 0f32
            })
        }

        World {
            herbivores: crate::dataset::population(&settings.herbivores),
            carnivores: crate::dataset::population(&settings.predators),
            map,
            width: settings.horizontal_granularity,
            height: settings.vertical_granularity
        }
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
        (y * self.width + x) as usize
    }
}

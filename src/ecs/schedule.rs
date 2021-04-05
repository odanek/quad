use super::world::World;

pub struct Schedule;

impl Schedule {
    pub fn run(&mut self, world: &mut World) {}
}

impl Default for Schedule {
    fn default() -> Self {
        Schedule {}
    }
}
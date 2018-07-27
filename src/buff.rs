use event::BuffRemoval;
use event::Buff;

pub trait Stack {
    fn push(&mut self, i32);
    fn remove(&mut self, i32);
    fn clear(&mut self);
    fn stacks(&self) -> u32;
}

pub struct Simulator {

}

impl Simulator {
    pub fn add_event<T: Buff>(&mut self, e: T) {
        match e.removal() {
            BuffRemoval::None   => unimplemented!("Add stack"),
            BuffRemoval::All    => unimplemented!("Clear stacks"),
            BuffRemoval::Single => unimplemented!("Clear single stack"),
            BuffRemoval::Manual => unimplemented!("Manual stack"),
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct Id(u64);
pub struct IdGenerator(u64);

impl IdGenerator {
    pub fn new() -> Self {
        IdGenerator(0)
    }

    pub fn generate(&mut self) -> Id {
        self.0 += 1;
        Id(self.0)
    }
}

use bcrypt::{hash, verify, DEFAULT_COST};

pub struct Password {
    cost: u32,
}

impl Password {
    pub fn new() -> Self {
        Password { cost: DEFAULT_COST }
    }

    pub fn build(mut self, cost: u32) -> Self {
        self.cost = cost;
        self
    }

    pub fn hash(&self, password: &str) -> String {
        hash(password, self.cost).unwrap()
    }

    pub fn verify(&self, password: &str, hash: &str) -> bool {
        verify(password, hash).unwrap()
    }
}

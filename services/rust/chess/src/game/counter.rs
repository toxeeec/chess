#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Counter {
    pub half: u32,
    pub full: u32,
}

impl Counter {
    pub fn update(&mut self, irreversible: bool, is_white: bool) {
        if irreversible {
            self.half = 0;
        } else {
            self.half += 1;
        }
        if !is_white {
            self.full += 1;
        }
    }
}

impl Default for Counter {
    fn default() -> Self {
        Self { half: 0, full: 1 }
    }
}

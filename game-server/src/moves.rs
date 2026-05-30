use crate::square::Square;
use std::fmt::{self, Write};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Move {
    pub(super) from: Square,
    pub(super) to: Square,
}

impl Move {
    pub(super) const fn new(from: Square, to: Square) -> Self {
        Self { from, to }
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char((b'a' + (self.from.0 % 8) as u8) as char)?;
        f.write_char((b'1' + (self.from.0 / 8) as u8) as char)?;

        f.write_char((b'a' + (self.to.0 % 8) as u8) as char)?;
        f.write_char((b'1' + (self.to.0 / 8) as u8) as char)
    }
}

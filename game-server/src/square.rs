use crate::game::Color;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Square(pub(super) u32);

impl Square {
    pub(super) const fn new(square: u32) -> Self {
        debug_assert!(square < 64);
        Self(square)
    }

    pub(super) const fn backward<const COLOR: Color, const N: u32>(self) -> Self {
        match COLOR {
            Color::White => Square::new(self.0 - N * 8),
            Color::Black => Square::new(self.0 + N * 8),
        }
    }

    pub(super) const fn backward_west<const COLOR: Color>(self) -> Self {
        match COLOR {
            Color::White => Square::new(self.0 - 9),
            Color::Black => Square::new(self.0 + 7),
        }
    }

    pub(super) const fn backward_east<const COLOR: Color>(self) -> Self {
        match COLOR {
            Color::White => Square::new(self.0 - 7),
            Color::Black => Square::new(self.0 + 9),
        }
    }
}

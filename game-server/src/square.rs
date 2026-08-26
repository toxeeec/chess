use std::fmt;

use crate::state::Color;

#[macro_export]
macro_rules! square {
    ($square:ident) => {{
        const SQUARE: $crate::square::Square =
            $crate::square::Square::from_name(stringify!($square));
        SQUARE
    }};
}

#[macro_export]
macro_rules! squares {
    ($($square:ident),* $(,)?) => {
        [$($crate::square!($square)),*]
    };
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Square(u32);

impl Square {
    pub(super) const fn new(square: u32) -> Self {
        debug_assert!(square < 64);
        Self(square)
    }

    pub(super) fn file(self) -> u8 {
        (self.0 % 8) as u8
    }

    pub(super) fn rank(self) -> u8 {
        (self.0 / 8) as u8
    }

    pub(super) const fn from_name(square: &str) -> Self {
        let square = square.as_bytes();
        assert!(
            square.len() == 2
                && square[0] >= b'a'
                && square[0] <= b'h'
                && square[1] >= b'1'
                && square[1] <= b'8',
            "invalid square"
        );

        Self::new((square[1] - b'1') as u32 * 8 + (square[0] - b'a') as u32)
    }

    pub(super) fn backward<const COLOR: Color, const N: u32>(self) -> Self {
        match COLOR {
            Color::White => Square::new(self.0 - N * 8),
            Color::Black => Square::new(self.0 + N * 8),
        }
    }

    pub(super) fn backward_west<const COLOR: Color>(self) -> Self {
        match COLOR {
            Color::White => Square::new(self.0 - 9),
            Color::Black => Square::new(self.0 + 7),
        }
    }

    pub(super) fn backward_east<const COLOR: Color>(self) -> Self {
        match COLOR {
            Color::White => Square::new(self.0 - 7),
            Color::Black => Square::new(self.0 + 9),
        }
    }
}

impl From<Square> for u32 {
    fn from(square: Square) -> Self {
        square.0 as Self
    }
}

const impl From<Square> for usize {
    fn from(square: Square) -> Self {
        square.0 as Self
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let file = char::from(b'a' + self.file());
        let rank = char::from(b'1' + self.rank());
        write!(f, "{file}{rank}")
    }
}

use crate::square::Square;
use std::{
    fmt,
    ops::{BitAnd, BitOrAssign},
};

#[derive(Clone, Copy, PartialEq)]
pub(super) struct Bitboard(u64);

#[macro_export]
macro_rules! bitboard {
    ($square: expr) => {
        $crate::bitboard::Bitboard::from($crate::square::Square::new($square))
    };

    ($($square: expr),* $(,)?) => {
        $crate::bitboard::Bitboard::from([$($crate::square::Square::new($square),)*])
    };
}

impl Bitboard {
    const EMPTY: Self = Self(0);

    pub(super) fn contains(self, square: Square) -> bool {
        self & Self::from(square) != Bitboard::EMPTY
    }
}

impl From<Square> for Bitboard {
    fn from(square: Square) -> Self {
        Self(1 << square.0)
    }
}

impl<const N: usize> From<[Square; N]> for Bitboard {
    fn from(squares: [Square; N]) -> Self {
        let mut bitboard = Self::EMPTY;

        for square in squares {
            bitboard |= square;
        }

        bitboard
    }
}

impl fmt::Debug for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rank in (0..8).rev() {
            for file in 0..8 {
                let sq = Square::new(rank * 8 + file);
                let bit = if self.contains(sq) { '1' } else { '0' };

                write!(f, "{bit}")?;

                if file < 7 {
                    write!(f, " ")?;
                }
            }

            writeln!(f)?;
        }

        Ok(())
    }
}

impl BitAnd for Bitboard {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitOrAssign<Square> for Bitboard {
    fn bitor_assign(&mut self, rhs: Square) {
        self.0 |= Self::from(rhs).0;
    }
}

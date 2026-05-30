use crate::square::Square;
use std::{
    fmt,
    ops::{BitAnd, BitOr, BitOrAssign, Not, Shl},
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
    pub(super) const RANK_3: Self = Self(0xff << 16);
    pub(super) const RANK_7: Self = Self(0xff << 48);

    pub(super) fn contains(self, square: Square) -> bool {
        self & Self::from(square) != Bitboard::EMPTY
    }

    pub(super) fn empty(self) -> bool {
        self == Self::EMPTY
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

impl BitOr for Bitboard {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl Not for Bitboard {
    type Output = Self;
    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl Shl<u32> for Bitboard {
    type Output = Self;
    fn shl(self, rhs: u32) -> Self::Output {
        Self(self.0 << rhs)
    }
}

impl BitOrAssign<Square> for Bitboard {
    fn bitor_assign(&mut self, rhs: Square) {
        self.0 |= Self::from(rhs).0;
    }
}

impl Iterator for Bitboard {
    type Item = Square;
    fn next(&mut self) -> Option<Self::Item> {
        if self.empty() {
            return None;
        };

        let square = Square(self.0.trailing_zeros());
        self.0 &= self.0 - 1;
        Some(square)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.0.count_ones() as usize;
        (len, Some(len))
    }
}

impl ExactSizeIterator for Bitboard {}

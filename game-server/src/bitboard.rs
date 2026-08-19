use std::{
    fmt,
    marker::ConstParamTy,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Index, Not, Shl, Shr},
};

use crate::{game::Color, square::Square};

#[derive(Clone, Copy, PartialEq)]
pub(super) struct Bitboard(pub(super) u64);

#[derive(Clone, Copy, ConstParamTy, Eq, PartialEq)]
pub(super) enum Direction {
    North,
    South,
    East,
    West,
    Northeast,
    Northwest,
    Southeast,
    Southwest,
    Nne,
    Nnw,
    Nee,
    Nww,
    Sse,
    Ssw,
    See,
    Sww,
}

#[macro_export]
macro_rules! bitboard {
    ($square: expr) => {{
        let square = $square;
        debug_assert!((0..64).contains(&square));
        $crate::bitboard::Bitboard::from($crate::square::Square::new(square as u32))
    }};

    ($($square: expr),* $(,)?) => {
        $crate::bitboard::Bitboard::from([$({
            debug_assert!((0..64).contains(&$square));
            $crate::square::Square::new($square as u32)
        },)*])
    };
}

impl Bitboard {
    pub(super) const EMPTY: Self = Self(0);
    pub(super) const FILE_A: Self = Self(0x0101010101010101);
    pub(super) const FILE_B: Self = Self::FILE_A << 1;
    pub(super) const FILE_G: Self = Self::FILE_A << 6;
    pub(super) const FILE_H: Self = Self::FILE_A << 7;

    pub(super) fn empty(self) -> bool {
        self == Self::EMPTY
    }

    pub(super) const fn new(value: u64) -> Self {
        Self(value)
    }

    pub(super) fn forward<const COLOR: Color, const N: u32>(self) -> Self {
        match COLOR {
            Color::White => self.shift_n::<{ Direction::North }, N>(),
            Color::Black => self.shift_n::<{ Direction::South }, N>(),
        }
    }

    pub(super) const fn forward_west<const COLOR: Color>(self) -> Self {
        match COLOR {
            Color::White => self.shift::<{ Direction::Northwest }>(),
            Color::Black => self.shift::<{ Direction::Southwest }>(),
        }
    }

    pub(super) const fn forward_east<const COLOR: Color>(self) -> Self {
        match COLOR {
            Color::White => self.shift::<{ Direction::Northeast }>(),
            Color::Black => self.shift::<{ Direction::Southeast }>(),
        }
    }

    pub(super) const fn shift<const DIRECTION: Direction>(self) -> Self {
        match DIRECTION {
            Direction::North => self << 8,
            Direction::South => self >> 8,
            Direction::East => (self & !Self::FILE_H) << 1,
            Direction::West => (self & !Self::FILE_A) >> 1,
            Direction::Northeast => (self & !Self::FILE_H) << 9,
            Direction::Northwest => (self & !Self::FILE_A) << 7,
            Direction::Southeast => (self & !Self::FILE_H) >> 7,
            Direction::Southwest => (self & !Self::FILE_A) >> 9,
            Direction::Nne => (self & !Self::FILE_H) << 17,
            Direction::Nnw => (self & !Self::FILE_A) << 15,
            Direction::Nee => (self & !(Self::FILE_G | Self::FILE_H)) << 10,
            Direction::Nww => (self & !(Self::FILE_A | Self::FILE_B)) << 6,
            Direction::Sse => (self & !Self::FILE_H) >> 15,
            Direction::Ssw => (self & !Self::FILE_A) >> 17,
            Direction::See => (self & !(Self::FILE_G | Self::FILE_H)) >> 6,
            Direction::Sww => (self & !(Self::FILE_A | Self::FILE_B)) >> 10,
        }
    }

    const fn shift_n<const DIRECTION: Direction, const N: u32>(self) -> Self {
        let mut bitboard = self;
        let mut i = 0;

        while i < N {
            bitboard = bitboard.shift::<DIRECTION>();
            i += 1;
        }

        bitboard
    }

    pub(super) const fn relative_rank<const COLOR: Color>(n: u32) -> Self {
        debug_assert!(n >= 1 && n <= 8);
        let rank = match COLOR {
            Color::White => n,
            Color::Black => 9 - n,
        };
        Self(0xff << ((rank - 1) * 8))
    }

    pub(super) fn contains(self, square: Square) -> bool {
        self & square != Bitboard::EMPTY
    }

    pub(super) fn apply_move(&mut self, from: Square, to: Square) {
        let from_mask = Self::from(from).0;
        let to_mask = Self::from(to).0;
        let add_mask = ((self.0 & from_mask) >> usize::from(from)).wrapping_neg() & to_mask;

        self.0 &= !(from_mask | to_mask);
        self.0 |= add_mask;
    }
}

const impl From<Square> for Bitboard {
    fn from(square: Square) -> Self {
        Self(1 << usize::from(square))
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
                let bit = b'0' + self.contains(sq) as u8;

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

const impl BitAnd for Bitboard {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAnd<Square> for Bitboard {
    type Output = Bitboard;
    fn bitand(self, rhs: Square) -> Self::Output {
        Self(self.0 & Self::from(rhs).0)
    }
}

const impl BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

const impl BitOr for Bitboard {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

const impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitOrAssign<Square> for Bitboard {
    fn bitor_assign(&mut self, rhs: Square) {
        self.0 |= Self::from(rhs).0;
    }
}

const impl Not for Bitboard {
    type Output = Self;
    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

const impl Shl<u32> for Bitboard {
    type Output = Self;
    fn shl(self, rhs: u32) -> Self::Output {
        Self(self.0 << rhs)
    }
}

const impl Shr<u32> for Bitboard {
    type Output = Self;
    fn shr(self, rhs: u32) -> Self::Output {
        Self(self.0 >> rhs)
    }
}

impl Iterator for Bitboard {
    type Item = Square;
    fn next(&mut self) -> Option<Self::Item> {
        if self.empty() {
            return None;
        };

        let square = Square::new(self.0.trailing_zeros());
        self.0 &= self.0 - 1;
        Some(square)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.0.count_ones() as usize;
        (len, Some(len))
    }
}

impl ExactSizeIterator for Bitboard {}
unsafe impl std::iter::TrustedLen for Bitboard {}

impl Index<Square> for [Bitboard; 64] {
    type Output = Bitboard;

    fn index(&self, square: Square) -> &Self::Output {
        let square = usize::from(square);
        unsafe { self.get_unchecked(square) }
    }
}

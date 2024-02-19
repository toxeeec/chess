use crate::square::Square;
use std::marker::ConstParamTy;

#[derive(Copy, Clone, Default, Debug)]
#[derive_const(PartialEq)]
pub(super) struct Bitboard(pub(super) u64);

#[derive(Clone, Copy, ConstParamTy, PartialEq, Eq)]
pub(super) enum Direction {
    North = 8,
    East = 1,
    South = -8,
    West = -1,
    NorthEast = 9,
    NorthWest = 7,
    SouthEast = -7,
    SouthWest = -9,
    Nne = 17,
    Nee = 10,
    See = -6,
    Sse = -15,
    Ssw = -17,
    Sww = -10,
    Nww = 6,
    Nnw = 15,
}

#[macro_export]
macro_rules! bb {
    ($square: expr) => {
        $crate::bitboard::Bitboard::from_square($crate::square::Square::new($square))
    };

    ($($square: expr),* $(,)?) => {
        $crate::bitboard::Bitboard::from_squares([$($crate::square::Square::new($square),)*])
    };
}

const KING_LOOKUP: [Bitboard; 64] = {
    let mut bbs = [Bitboard::EMPTY; 64];
    let mut i = 0;
    while i < 64 {
        let bb = bb![i];

        bbs[i as usize] = bb
            .shifted::<{ Direction::North }>()
            .or(bb.shifted::<{ Direction::NorthEast }>())
            .or(bb.shifted::<{ Direction::East }>())
            .or(bb.shifted::<{ Direction::SouthEast }>())
            .or(bb.shifted::<{ Direction::South }>())
            .or(bb.shifted::<{ Direction::SouthWest }>())
            .or(bb.shifted::<{ Direction::West }>())
            .or(bb.shifted::<{ Direction::NorthWest }>());
        i += 1;
    }
    bbs
};

const KNIGHT_LOOKUP: [Bitboard; 64] = {
    let mut bbs = [Bitboard::EMPTY; 64];

    let mut i = 0;
    while i < 64 {
        let bb = bb![i];
        bbs[i as usize] = bb
            .shifted::<{ Direction::Nne }>()
            .or(bb.shifted::<{ Direction::Nee }>())
            .or(bb.shifted::<{ Direction::See }>())
            .or(bb.shifted::<{ Direction::Sse }>())
            .or(bb.shifted::<{ Direction::Ssw }>())
            .or(bb.shifted::<{ Direction::Sww }>())
            .or(bb.shifted::<{ Direction::Nww }>())
            .or(bb.shifted::<{ Direction::Nnw }>());
        i += 1;
    }
    bbs
};

impl Bitboard {
    pub(super) const EMPTY: Self = Self(0);
    pub(super) const FULL: Self = Self(!0);
    pub(super) const FILE_A: Self = bb![0, 8, 16, 24, 32, 40, 48, 56];
    const FILE_B: Self = Self::FILE_A.shl(1);
    const FILE_G: Self = Self::FILE_A.shl(6);
    pub(super) const FILE_H: Self = Self::FILE_A.shl(7);

    pub(super) const RANK_1: Self = bb![0, 1, 2, 3, 4, 5, 6, 7];
    pub(super) const RANK_8: Self = Self::RANK_1.shl(8 * 7);

    #[inline(always)]
    pub(super) const fn contains(self, square: Square) -> bool {
        self.and(Self::from_square(square)) != Bitboard::EMPTY
    }

    #[inline(always)]
    pub(super) const fn from_square(square: Square) -> Self {
        Self(1 << square.0)
    }

    #[inline(always)]
    pub(super) const fn from_squares<const N: usize>(squares: [Square; N]) -> Self {
        let mut bb = Self::EMPTY;
        let mut i = 0;
        while i < N {
            bb = bb.with_square(squares[i]);
            i += 1;
        }

        bb
    }

    #[inline(always)]
    pub(super) const fn with_square(self, square: Square) -> Self {
        Self(self.0 | Self::from_square(square).0)
    }

    #[inline(always)]
    pub(super) const fn and(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }

    #[inline(always)]
    pub(super) const fn or(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }

    #[inline(always)]
    pub(super) const fn xor(self, rhs: Self) -> Self {
        Self(self.0 & !rhs.0)
    }

    #[inline(always)]
    const fn shl(self, rhs: u32) -> Self {
        Self(self.0 << rhs)
    }

    #[inline(always)]
    const fn shr(self, rhs: u32) -> Self {
        Self(self.0 >> rhs)
    }

    #[inline(always)]
    const fn shifted<const DIR: Direction>(self) -> Self {
        match DIR {
            Direction::North => self.shl(8),
            Direction::South => self.shr(8),
            Direction::East => (self.xor(Self::FILE_H)).shl(1),
            Direction::West => (self.xor(Self::FILE_A)).shr(1),
            Direction::NorthEast => (self.xor(Self::FILE_H)).shl(9),
            Direction::SouthEast => (self.xor(Self::FILE_H)).shr(7),
            Direction::NorthWest => (self.xor(Self::FILE_A)).shl(7),
            Direction::SouthWest => (self.xor(Self::FILE_A)).shr(9),
            Direction::Nne => (self.xor(Self::FILE_H)).shl(17),
            Direction::Nee => (self.xor(Self::FILE_G.or(Self::FILE_H))).shl(10),
            Direction::See => (self.xor(Self::FILE_G.or(Self::FILE_H))).shr(6),
            Direction::Sse => (self.xor(Self::FILE_H)).shr(15),
            Direction::Ssw => (self.xor(Self::FILE_A)).shr(17),
            Direction::Sww => (self.xor(Self::FILE_A.or(Self::FILE_B))).shr(10),
            Direction::Nww => (self.xor(Self::FILE_A.or(Self::FILE_B))).shl(6),
            Direction::Nnw => (self.xor(Self::FILE_A)).shl(15),
        }
    }

    #[inline(always)]
    pub(super) const fn rank(sq: Square) -> Self {
        Self::RANK_1.shl(8 * (sq.rank()))
    }

    #[inline(always)]
    pub(super) const fn file(sq: Square) -> Self {
        Self::FILE_A.shl(sq.file())
    }
}

impl Iterator for Bitboard {
    type Item = Square;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        match self.is_empty() {
            true => None,
            false => {
                let sq = Square::new(self.0.trailing_zeros());
                self.0 &= self.0 - 1;
                Some(sq)
            }
        }
    }
}

impl ExactSizeIterator for Bitboard {
    #[inline(always)]
    fn len(&self) -> usize {
        self.0.count_ones() as usize
    }
}

impl DoubleEndedIterator for Bitboard {
    #[inline(always)]
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.is_empty() {
            true => None,
            false => {
                let sq = Square::new(64 - self.0.leading_zeros() - 1);
                self.0 &= (1 << (64 - self.0.leading_zeros() - 1)) - 1;
                Some(sq)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(bb![0] => bb![8])]
    #[test_case(bb![56] => Bitboard::EMPTY)]
    fn shifted_north_tests(bb: Bitboard) -> Bitboard {
        bb.shifted::<{ Direction::North }>()
    }

    #[test_case(bb![0] => bb![1])]
    #[test_case(bb![7] => Bitboard::EMPTY)]
    fn shifted_east_tests(bb: Bitboard) -> Bitboard {
        bb.shifted::<{ Direction::East }>()
    }

    #[test_case(bb![56] => bb![48])]
    #[test_case(bb![0] => Bitboard::EMPTY)]
    fn shifted_south_tests(bb: Bitboard) -> Bitboard {
        bb.shifted::<{ Direction::South }>()
    }

    #[test_case(bb![7] => bb![6])]
    #[test_case(bb![0] => Bitboard::EMPTY)]
    fn shifted_west_tests(bb: Bitboard) -> Bitboard {
        bb.shifted::<{ Direction::West }>()
    }

    #[test_case(bb![0] => bb![9])]
    #[test_case(bb![55] => Bitboard::EMPTY)]
    #[test_case(bb![62] => Bitboard::EMPTY)]
    #[test_case(bb![63] => Bitboard::EMPTY)]
    fn shifted_north_east_tests(bb: Bitboard) -> Bitboard {
        bb.shifted::<{ Direction::NorthEast }>()
    }

    #[test_case(bb![56] => bb![49])]
    #[test_case(bb![15] => Bitboard::EMPTY)]
    #[test_case(bb![7] => Bitboard::EMPTY)]
    #[test_case(bb![6] => Bitboard::EMPTY)]
    fn shifted_south_east_tests(bb: Bitboard) -> Bitboard {
        bb.shifted::<{ Direction::SouthEast }>()
    }

    #[test_case(bb![63] => bb![54])]
    #[test_case(bb![8] => Bitboard::EMPTY)]
    #[test_case(bb![1] => Bitboard::EMPTY)]
    #[test_case(bb![0] => Bitboard::EMPTY)]
    fn shifted_south_west_tests(bb: Bitboard) -> Bitboard {
        bb.shifted::<{ Direction::SouthWest }>()
    }

    #[test_case(bb![7] => bb![14])]
    #[test_case(bb![48] => Bitboard::EMPTY)]
    #[test_case(bb![56] => Bitboard::EMPTY)]
    #[test_case(bb![57] => Bitboard::EMPTY)]
    fn shifted_north_west_tests(bb: Bitboard) -> Bitboard {
        bb.shifted::<{ Direction::NorthWest }>()
    }

    #[test_case(bb![0] => bb![17])]
    #[test_case(bb![48] => Bitboard::EMPTY)]
    #[test_case(bb![7] => Bitboard::EMPTY)]
    fn shifted_nne_tests(bb: Bitboard) -> Bitboard {
        bb.shifted::<{ Direction::Nne }>()
    }

    #[test_case(bb![0] => bb![10])]
    #[test_case(bb![6] => Bitboard::EMPTY)]
    #[test_case(bb![56] => Bitboard::EMPTY)]
    fn shifted_nee_tests(bb: Bitboard) -> Bitboard {
        bb.shifted::<{ Direction::Nee }>()
    }

    #[test_case(bb![56] => bb![50])]
    #[test_case(bb![62] => Bitboard::EMPTY)]
    #[test_case(bb![0] => Bitboard::EMPTY)]
    fn shifted_see_tests(bb: Bitboard) -> Bitboard {
        bb.shifted::<{ Direction::See }>()
    }

    #[test_case(bb![56] => bb![41])]
    #[test_case(bb![8] => Bitboard::EMPTY)]
    #[test_case(bb![63] => Bitboard::EMPTY)]
    fn shifted_sse_tests(bb: Bitboard) -> Bitboard {
        bb.shifted::<{ Direction::Sse }>()
    }

    #[test_case(bb![63] => bb![46])]
    #[test_case(bb![15] => Bitboard::EMPTY)]
    #[test_case(bb![56] => Bitboard::EMPTY)]
    fn shifted_ssw_tests(bb: Bitboard) -> Bitboard {
        bb.shifted::<{ Direction::Ssw }>()
    }

    #[test_case(bb![63] => bb![53])]
    #[test_case(bb![57] => Bitboard::EMPTY)]
    #[test_case(bb![7] => Bitboard::EMPTY)]
    fn shifted_sww_tests(bb: Bitboard) -> Bitboard {
        bb.shifted::<{ Direction::Sww }>()
    }

    #[test_case(bb![7] => bb![13])]
    #[test_case(bb![1] => Bitboard::EMPTY)]
    #[test_case(bb![63] => Bitboard::EMPTY)]
    fn shifted_nww_tests(bb: Bitboard) -> Bitboard {
        bb.shifted::<{ Direction::Nww }>()
    }

    #[test_case(bb![7] => bb![22])]
    #[test_case(bb![55] => Bitboard::EMPTY)]
    #[test_case(bb![0] => Bitboard::EMPTY)]
    fn shifted_nnw_tests(bb: Bitboard) -> Bitboard {
        bb.shifted::<{ Direction::Nnw }>()
    }

    #[test_case(0 => bb![1, 8, 9])]
    #[test_case(9 => bb![0, 1, 2, 8, 10, 16, 17, 18])]
    fn king_lookup_tests(sq: usize) -> Bitboard {
        KING_LOOKUP[sq]
    }

    #[test_case(0 => bb![10, 17])]
    #[test_case(18 => bb![1, 3, 8, 12, 24, 28, 33, 35])]
    fn knight_lookup_tests(sq: usize) -> Bitboard {
        KNIGHT_LOOKUP[sq]
    }
}

use std::cmp::{max, min};

use super::{Bitboard, FILE_A, FILE_B, FILE_G, FILE_H};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Direction {
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

impl Direction {
    pub const fn toward(sq1: u32, sq2: u32) -> Option<Self> {
        let diff = max(sq1, sq2) - min(sq1, sq2);
        if diff == 0 {
            None
        } else if file_of(sq1) == file_of(sq2) {
            Some(Direction::North)
        } else if rank_of(sq1) == rank_of(sq2) {
            Some(Direction::East)
        } else if diff % 7 == 0 {
            Some(Direction::NorthWest)
        } else if diff % 9 == 0 {
            Some(Direction::NorthEast)
        } else {
            None
        }
    }

    pub const fn opposite(self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
            Direction::NorthEast => Direction::SouthWest,
            Direction::SouthEast => Direction::NorthWest,
            Direction::NorthWest => Direction::SouthEast,
            Direction::SouthWest => Direction::NorthEast,
            Direction::Nne => Direction::Ssw,
            Direction::Nee => Direction::Sww,
            Direction::See => Direction::Nww,
            Direction::Sse => Direction::Nnw,
            Direction::Ssw => Direction::Nne,
            Direction::Sww => Direction::Nee,
            Direction::Nww => Direction::See,
            Direction::Nnw => Direction::Sse,
        }
    }
}

impl Bitboard {
    pub const fn shifted<const DIR: Direction>(self) -> Self {
        match DIR {
            Direction::North => self << 8,
            Direction::South => self >> 8,
            Direction::East => (self & !FILE_H) << 1,
            Direction::West => (self & !FILE_A) >> 1,
            Direction::NorthEast => (self & !FILE_H) << 9,
            Direction::SouthEast => (self & !FILE_H) >> 7,
            Direction::NorthWest => (self & !FILE_A) << 7,
            Direction::SouthWest => (self & !FILE_A) >> 9,
            Direction::Nne => (self & !FILE_H) << 17,
            Direction::Nee => (self & !(FILE_G | FILE_H)) << 10,
            Direction::See => (self & !(FILE_G | FILE_H)) >> 6,
            Direction::Sse => (self & !FILE_H) >> 15,
            Direction::Ssw => (self & !FILE_A) >> 17,
            Direction::Sww => (self & !(FILE_A | FILE_B)) >> 10,
            Direction::Nww => (self & !(FILE_A | FILE_B)) << 6,
            Direction::Nnw => (self & !FILE_A) << 15,
        }
    }

    pub const fn shifted_by(self, dir: Direction) -> Self {
        match dir {
            Direction::North => self.shifted::<{ Direction::North }>(),
            Direction::South => self.shifted::<{ Direction::South }>(),
            Direction::East => self.shifted::<{ Direction::East }>(),
            Direction::West => self.shifted::<{ Direction::West }>(),
            Direction::NorthEast => self.shifted::<{ Direction::NorthEast }>(),
            Direction::SouthEast => self.shifted::<{ Direction::SouthEast }>(),
            Direction::NorthWest => self.shifted::<{ Direction::NorthWest }>(),
            Direction::SouthWest => self.shifted::<{ Direction::SouthWest }>(),
            Direction::Nne => self.shifted::<{ Direction::Nne }>(),
            Direction::Nee => self.shifted::<{ Direction::Nee }>(),
            Direction::See => self.shifted::<{ Direction::See }>(),
            Direction::Sse => self.shifted::<{ Direction::Sse }>(),
            Direction::Ssw => self.shifted::<{ Direction::Ssw }>(),
            Direction::Sww => self.shifted::<{ Direction::Sww }>(),
            Direction::Nww => self.shifted::<{ Direction::Nww }>(),
            Direction::Nnw => self.shifted::<{ Direction::Nnw }>(),
        }
    }

    pub const fn shifted_forward_left(self, is_white: bool) -> Self {
        if is_white {
            self.shifted::<{ Direction::NorthWest }>()
        } else {
            self.shifted::<{ Direction::SouthEast }>()
        }
    }

    pub const fn shifted_forward_right(self, is_white: bool) -> Self {
        if is_white {
            self.shifted::<{ Direction::NorthEast }>()
        } else {
            self.shifted::<{ Direction::SouthWest }>()
        }
    }

    pub const fn shifted_backward_left(self, is_white: bool) -> Self {
        if is_white {
            self.shifted::<{ Direction::SouthEast }>()
        } else {
            self.shifted::<{ Direction::NorthWest }>()
        }
    }

    pub const fn shifted_backward_right(self, is_white: bool) -> Self {
        if is_white {
            self.shifted::<{ Direction::SouthWest }>()
        } else {
            self.shifted::<{ Direction::NorthEast }>()
        }
    }
}

const fn rank_of(sq: u32) -> u32 {
    sq / 8
}

const fn file_of(sq: u32) -> u32 {
    sq % 8
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bb;
    use test_case::test_case;

    #[test_case(bb![0] => bb![8])]
    #[test_case(bb![56] => Bitboard::default())]
    fn shifted_north_tests(bb: Bitboard) -> Bitboard {
        bb.shifted::<{ Direction::North }>()
    }

    #[test_case(bb![0] => bb![1])]
    #[test_case(bb![7] => Bitboard::default())]
    fn shifted_east_tests(bb: Bitboard) -> Bitboard {
        bb.shifted::<{ Direction::East }>()
    }

    #[test_case(bb![56] => bb![48])]
    #[test_case(bb![0] => Bitboard::default())]
    fn shifted_south_tests(bb: Bitboard) -> Bitboard {
        bb.shifted::<{ Direction::South }>()
    }

    #[test_case(bb![7] => bb![6])]
    #[test_case(bb![0] => Bitboard::default())]
    fn shifted_west_tests(bb: Bitboard) -> Bitboard {
        bb.shifted::<{ Direction::West }>()
    }

    #[test_case(bb![0] => bb![9])]
    #[test_case(bb![55] => Bitboard::default())]
    #[test_case(bb![62] => Bitboard::default())]
    #[test_case(bb![63] => Bitboard::default())]
    fn shifted_north_east_tests(bb: Bitboard) -> Bitboard {
        bb.shifted::<{ Direction::NorthEast }>()
    }

    #[test_case(bb![56] => bb![49])]
    #[test_case(bb![15] => Bitboard::default())]
    #[test_case(bb![7] => Bitboard::default())]
    #[test_case(bb![6] => Bitboard::default())]
    fn shifted_south_east_tests(bb: Bitboard) -> Bitboard {
        bb.shifted::<{ Direction::SouthEast }>()
    }

    #[test_case(bb![63] => bb![54])]
    #[test_case(bb![8] => Bitboard::default())]
    #[test_case(bb![1] => Bitboard::default())]
    #[test_case(bb![0] => Bitboard::default())]
    fn shifted_south_west_tests(bb: Bitboard) -> Bitboard {
        bb.shifted::<{ Direction::SouthWest }>()
    }

    #[test_case(bb![7] => bb![14])]
    #[test_case(bb![48] => Bitboard::default())]
    #[test_case(bb![56] => Bitboard::default())]
    #[test_case(bb![57] => Bitboard::default())]
    fn shifted_north_west_tests(bb: Bitboard) -> Bitboard {
        bb.shifted::<{ Direction::NorthWest }>()
    }

    #[test_case(bb![0] => bb![17])]
    #[test_case(bb![48] => Bitboard::default())]
    #[test_case(bb![7] => Bitboard::default())]
    fn shifted_nne_tests(bb: Bitboard) -> Bitboard {
        bb.shifted::<{ Direction::Nne }>()
    }

    #[test_case(bb![0] => bb![10])]
    #[test_case(bb![6] => Bitboard::default())]
    #[test_case(bb![56] => Bitboard::default())]
    fn shifted_nee_tests(bb: Bitboard) -> Bitboard {
        bb.shifted::<{ Direction::Nee }>()
    }

    #[test_case(bb![56] => bb![50])]
    #[test_case(bb![62] => Bitboard::default())]
    #[test_case(bb![0] => Bitboard::default())]
    fn shifted_see_tests(bb: Bitboard) -> Bitboard {
        bb.shifted::<{ Direction::See }>()
    }

    #[test_case(bb![56] => bb![41])]
    #[test_case(bb![8] => Bitboard::default())]
    #[test_case(bb![63] => Bitboard::default())]
    fn shifted_sse_tests(bb: Bitboard) -> Bitboard {
        bb.shifted::<{ Direction::Sse }>()
    }

    #[test_case(bb![63] => bb![46])]
    #[test_case(bb![15] => Bitboard::default())]
    #[test_case(bb![56] => Bitboard::default())]
    fn shifted_ssw_tests(bb: Bitboard) -> Bitboard {
        bb.shifted::<{ Direction::Ssw }>()
    }

    #[test_case(bb![63] => bb![53])]
    #[test_case(bb![57] => Bitboard::default())]
    #[test_case(bb![7] => Bitboard::default())]
    fn shifted_sww_tests(bb: Bitboard) -> Bitboard {
        bb.shifted::<{ Direction::Sww }>()
    }

    #[test_case(bb![7] => bb![13])]
    #[test_case(bb![1] => Bitboard::default())]
    #[test_case(bb![63] => Bitboard::default())]
    fn shifted_nww_tests(bb: Bitboard) -> Bitboard {
        bb.shifted::<{ Direction::Nww }>()
    }

    #[test_case(bb![7] => bb![22])]
    #[test_case(bb![55] => Bitboard::default())]
    #[test_case(bb![0] => Bitboard::default())]
    fn shifted_nnw_tests(bb: Bitboard) -> Bitboard {
        bb.shifted::<{ Direction::Nnw }>()
    }
}

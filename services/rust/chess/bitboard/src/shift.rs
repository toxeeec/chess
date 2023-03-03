use super::{Bitboard, FILE_A, FILE_B, FILE_G, FILE_H};

#[derive(PartialEq, Eq)]
pub enum Direction {
    North,
    East,
    South,
    West,
    NorthEast,
    SouthEast,
    SouthWest,
    NorthWest,
    Nne,
    Nee,
    See,
    Sse,
    Ssw,
    Sww,
    Nww,
    Nnw,
}

impl Bitboard {
    pub const fn shifted<const DIR: Direction>(self) -> Bitboard {
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

mod bits;
pub mod shift;

#[derive_const(Default, PartialEq)]
#[derive(Debug, Eq, Clone, Copy)]
pub struct Bitboard(pub(crate) u64);

impl Bitboard {
    pub const fn is_empty(self) -> bool {
        self == Self::default()
    }

    pub const fn contains(self, sq: u32) -> bool {
        debug_assert!(sq < 64);
        !((self & Self::from_square(sq)).is_empty())
    }

    pub const fn from_square(sq: u32) -> Self {
        debug_assert!(sq < 64);
        Self(1 << sq)
    }

    pub const fn from_squares<const N: usize>(sqs: [u32; N]) -> Self {
        let mut bb = Self::default();
        let mut i = 0;
        while i < N {
            debug_assert!(sqs[i] < 64);
            bb |= Self::from_square(sqs[i]);
            i += 1;
        }
        bb
    }
}

#[macro_export]
macro_rules! bb {
    ($sq: expr) => {
        $crate::game::bitboard::Bitboard::from_square($sq)
    };

    ($($sq: expr),* $(,)?) => {
        $crate::game::bitboard::Bitboard::from_squares([$($sq,)*])
    };
}

#[macro_export]
macro_rules! for_each {
    ($bb: expr, $sq: expr, $block: block) => {
        while !$bb.is_empty() {
            $sq = $bb.0.trailing_zeros();
            $block;
            $bb.0 &= $bb.0.wrapping_sub(1);
        }
    };
}

pub const FILE_A: Bitboard = bb![0, 8, 16, 24, 32, 40, 48, 56];
pub const FILE_B: Bitboard = FILE_A << 1;
pub const FILE_G: Bitboard = FILE_A << 6;
pub const FILE_H: Bitboard = FILE_A << 7;

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(Bitboard::default(), 0 => false)]
    #[test_case(bb![1], 1 => true)]
    #[test_case(bb![0, 1, 2], 1 => true)]
    fn contains_tests(bb: Bitboard, sq: u32) -> bool {
        bb.contains(sq)
    }

    #[test_case(0 => Bitboard(1))]
    #[test_case(7 => Bitboard(0b10000000))]
    fn from_square_tests(sq: u32) -> Bitboard {
        Bitboard::from_square(sq)
    }

    #[test_case([] => Bitboard::default())]
    #[test_case([0, 1, 2, 3, 4, 5, 6, 7] => Bitboard(0b11111111))]
    fn from_squares_tests<const N: usize>(sqs: [u32; N]) -> Bitboard {
        Bitboard::from_squares(sqs)
    }
}

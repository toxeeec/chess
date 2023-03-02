mod bits;

#[derive_const(Default, PartialEq)]
#[derive(Eq, Clone, Copy, Debug)]
pub struct Bitboard(u64);

impl Bitboard {
    pub const fn is_empty(self) -> bool {
        self == Self::default()
    }

    pub const fn contains(self, sq: usize) -> bool {
        debug_assert!(sq < 64);
        !((self & Self::from_square(sq)).is_empty())
    }

    pub const fn from_square(sq: usize) -> Self {
        debug_assert!(sq < 64);
        Self(1 << sq)
    }

    pub const fn from_squares<const N: usize>(sqs: [usize; N]) -> Self {
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

macro_rules! bb {
    ($sq: expr) => {
        $crate::bitboard::Bitboard::from_square($sq)
    };

    ($($sq: expr),* $(,)?) => {
        $crate::bitboard::Bitboard::from_squares([$($sq,)*])
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(Bitboard::default(), 0 => false)]
    #[test_case(bb![1], 1 => true)]
    #[test_case(bb![0, 1, 2], 1 => true)]
    fn contains_tests(bb: Bitboard, sq: usize) -> bool {
        bb.contains(sq)
    }

    #[test_case(0 => Bitboard(1))]
    #[test_case(7 => Bitboard(0b10000000))]
    fn from_square_tests(sq: usize) -> Bitboard {
        Bitboard::from_square(sq)
    }

    #[test_case([] => Bitboard::default())]
    #[test_case([0, 1, 2, 3, 4, 5, 6, 7] => Bitboard(0b11111111))]
    fn from_squares_tests<const N: usize>(sqs: [usize; N]) -> Bitboard {
        Bitboard::from_squares(sqs)
    }
}

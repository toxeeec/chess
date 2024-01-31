#[derive(Copy, Clone, Default, Debug)]
#[derive_const(PartialEq)]
pub(super) struct Bitboard(u64);

#[macro_export]
macro_rules! bb {
    ($square: expr) => {
        $crate::bitboard::Bitboard::from_square($square)
    };

    ($($square: expr),* $(,)?) => {
        $crate::bitboard::Bitboard::from_squares([$($square,)*])
    };
}

impl Bitboard {
    const EMPTY: Self = Self(0);

    #[inline(always)]
    pub(super) const fn contains(self, square: u32) -> bool {
        debug_assert!(square < 64);
        self.0 & Self::from_square(square).0 != 0
    }

    #[inline(always)]
    pub(super) const fn from_square(square: u32) -> Self {
        debug_assert!(square < 64);
        Self(1 << square)
    }

    #[inline(always)]
    pub(super) const fn from_squares<const N: usize>(squares: [u32; N]) -> Self {
        let mut bb = Self::EMPTY;
        let mut i = 0;
        while i < N {
            debug_assert!(squares[i] < 64);
            bb = bb.with_square(squares[i]);
            i += 1;
        }

        bb
    }

    #[inline(always)]
    const fn with_square(self, square: u32) -> Self {
        debug_assert!(square < 64);
        Self(self.0 | Self::from_square(square).0)
    }
}

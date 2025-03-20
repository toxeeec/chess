use itertools::{Itertools, intersperse};
use std::{
    fmt,
    ops::{BitAnd, BitOr, BitOrAssign},
};

#[derive(Default, Clone, Copy)]
pub(super) struct Bitboard(u64);

impl Bitboard {
    pub(super) fn new<const N: usize>(squares: [u32; N]) -> Self {
        let mut bb = Self::default();
        for sq in squares {
            debug_assert!(sq < 64);
            bb |= 1 << sq
        }
        bb
    }

    pub(super) fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub(super) fn contains(self, square: u32) -> bool {
        debug_assert!(square < 64);
        !(self & 1 << square).is_empty()
    }
}

impl fmt::Debug for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            format!("{:064b}", self.0.reverse_bits())
                .chars()
                .collect_array::<64>()
                .unwrap()
                .chunks(8)
                .rev()
                .format_with("\n", |row, f| {
                    f(&String::from_iter(intersperse(row, &' ')))
                })
        )
    }
}

impl BitAnd<u64> for Bitboard {
    type Output = Self;
    fn bitand(self, rhs: u64) -> Self::Output {
        Self(self.0 & rhs)
    }
}

impl BitOr<Self> for Bitboard {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign<u64> for Bitboard {
    fn bitor_assign(&mut self, rhs: u64) {
        self.0 |= rhs;
    }
}

#![allow(incomplete_features)]
#![feature(
    const_trait_impl,
    const_mut_refs,
    derive_const,
    const_default_impls,
    adt_const_params,
    const_cmp,
    iter_intersperse,
    const_convert
)]

pub mod shift;
pub mod square;
pub mod squares;

use quote::{quote, ToTokens};
use square::Square;
use std::fmt::Debug;
mod bits;

#[derive_const(Default, PartialEq)]
#[derive(Eq, Clone, Copy)]
pub struct Bitboard(pub u64);

impl Bitboard {
    pub const fn is_empty(self) -> bool {
        self == Self::default()
    }

    pub const fn contains(self, sq: Square) -> bool {
        !((self & sq.into()).is_empty())
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

    pub const fn rank(sq: Square) -> Self {
        RANK_1 << (8 * (sq.rank()))
    }

    pub const fn file(sq: Square) -> Self {
        FILE_A << (sq.file())
    }

    pub const fn lsb(self) -> Square {
        debug_assert!(!self.is_empty());
        self.0.trailing_zeros().into()
    }

    pub const fn clear(&mut self, sq: Square) {
        *self &= !Bitboard::from(sq)
    }

    pub const fn set(&mut self, sq: Square) {
        *self |= sq.into()
    }
}

#[macro_export]
macro_rules! bb {
    ($sq: expr) => {
        $crate::Bitboard::from_square($sq)
    };

    ($($sq: expr),* $(,)?) => {
        $crate::Bitboard::from_squares([$($sq,)*])
    };
}

#[macro_export]
macro_rules! for_each {
    ($bb: expr, $sq: expr, $block: block) => {
        while !$bb.is_empty() {
            $sq = $crate::square::Square($bb.0.trailing_zeros());
            $block;
            $bb.0 &= $bb.0.wrapping_sub(1);
        }
    };
}
pub const RANK_1: Bitboard = bb![0, 1, 2, 3, 4, 5, 6, 7];
pub const RANK_2: Bitboard = RANK_1 << 8;
pub const RANK_3: Bitboard = RANK_1 << (8 * 2);
pub const RANK_6: Bitboard = RANK_1 << (8 * 5);
pub const RANK_7: Bitboard = RANK_1 << (8 * 6);
pub const RANK_8: Bitboard = RANK_1 << (8 * 7);

pub const FILE_A: Bitboard = bb![0, 8, 16, 24, 32, 40, 48, 56];
pub const FILE_B: Bitboard = FILE_A << 1;
pub const FILE_G: Bitboard = FILE_A << 6;
pub const FILE_H: Bitboard = FILE_A << 7;

impl Debug for Bitboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bb = self.0.reverse_bits();
        writeln!(f)?;
        for row in format!("{:064b}", bb)
            .chars()
            .collect::<Vec<_>>()
            .chunks(8)
            .rev()
        {
            writeln!(f, "{}", String::from_iter(row.iter().intersperse(&' ')))?;
        }
        Ok(())
    }
}

impl ToTokens for Bitboard {
    fn to_tokens(&self, tokens: &mut quote::__private::TokenStream) {
        let n = self.0;
        tokens.extend(quote! {
            Bitboard(#n)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(Bitboard::default(), 0 => false)]
    #[test_case(bb![1], 1 => true)]
    #[test_case(bb![0, 1, 2], 1 => true)]
    fn contains_tests(bb: Bitboard, sq: u32) -> bool {
        bb.contains(sq.into())
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

    #[test_case(0 => bb![0, 1, 2, 3, 4, 5, 6, 7])]
    #[test_case(63 => bb![56, 57, 58, 59, 60, 61, 62, 63])]
    fn rank_tests(sq: u32) -> Bitboard {
        Bitboard::rank(sq.into())
    }

    #[test_case(0 => bb![0, 8, 16, 24, 32, 40, 48, 56])]
    #[test_case(63 => bb![7, 15, 23, 31, 39, 47, 55, 63])]
    fn file_tests(sq: u32) -> Bitboard {
        Bitboard::file(sq.into())
    }

    #[test_case(bb![0] => 0)]
    #[test_case(bb![63] => 63)]
    #[test_case(bb![5, 10, 15] => 5)]
    fn lsb_tests(bb: Bitboard) -> u32 {
        bb.lsb().0
    }
}

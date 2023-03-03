use bitboard::Bitboard;
use magics::Magic;
include!(concat!(env!("OUT_DIR"), "/magics.rs"));

pub fn bishop_moves(sq: u32, blockers: Bitboard) -> Bitboard {
    debug_assert!(sq < 64);
    let sq = sq as usize;
    let index = (blockers.0 & BISHOP_MAGICS[sq].mask).wrapping_mul(BISHOP_MAGICS[sq].magic)
        >> (64 - BISHOP_MAGICS[sq].shift);
    BISHOP_ATTACKS[BISHOP_MAGICS[sq].attacks_index + index as usize]
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(9, Bitboard(0) => Bitboard::from_squares([0, 2, 16, 18, 27, 36, 45, 54, 63]))]
    #[test_case(9, Bitboard::from_square(18) => Bitboard::from_squares([0, 2, 16, 18]))]
    #[test_case(0, Bitboard::from_squares([2, 16, 18]) => Bitboard::from_squares([9, 18]))]
    fn bishop_moves_test(sq: u32, blockers: Bitboard) -> Bitboard {
        bishop_moves(sq, blockers)
    }
}

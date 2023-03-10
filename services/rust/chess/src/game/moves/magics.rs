use bitboard::{square::Square, Bitboard};
use magics::Magic;

include!(concat!(env!("OUT_DIR"), "/magics.rs"));

pub fn rook_moves(sq: Square, blockers: Bitboard) -> Bitboard {
    let sq = sq.0 as usize;
    let index = (blockers.0 & ROOK_MAGICS[sq].mask).wrapping_mul(ROOK_MAGICS[sq].magic)
        >> (64 - ROOK_MAGICS[sq].shift);
    ROOK_ATTACKS[ROOK_MAGICS[sq].attacks_index + index as usize]
}

pub fn bishop_moves(sq: Square, blockers: Bitboard) -> Bitboard {
    let sq = sq.0 as usize;
    let index = (blockers.0 & BISHOP_MAGICS[sq].mask).wrapping_mul(BISHOP_MAGICS[sq].magic)
        >> (64 - BISHOP_MAGICS[sq].shift);
    BISHOP_ATTACKS[BISHOP_MAGICS[sq].attacks_index + index as usize]
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitboard::bb;
    use test_case::test_case;

    #[test_case(0, Bitboard::default() => bb![1, 2, 3, 4, 5, 6, 7, 8, 16, 24, 32, 40, 48, 56])]
    #[test_case(0, bb![8] => bb![8, 1, 2, 3, 4, 5, 6, 7])]
    #[test_case(0, bb![2, 16, 18] => bb![1, 2, 8, 16])]
    fn rook_moves_tests(sq: u32, blockers: Bitboard) -> Bitboard {
        rook_moves(sq.into(), blockers)
    }

    #[test_case(9, Bitboard::default() => bb![0, 2, 16, 18, 27, 36, 45, 54, 63])]
    #[test_case(9, bb![18] => bb![0, 2, 16, 18])]
    #[test_case(0, bb![2, 16, 18] => bb![9, 18])]
    fn bishop_moves_tests(sq: u32, blockers: Bitboard) -> Bitboard {
        bishop_moves(sq.into(), blockers)
    }
}

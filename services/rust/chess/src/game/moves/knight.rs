use bitboard::{for_each, shift::Direction, Bitboard};

use super::{pins::Pins, Move};
use crate::game::{board::Board, moves, piece::Piece, state::State};

pub const KNIGHT_LOOKUP: [Bitboard; 64] = {
    let mut bbs = [Bitboard::default(); 64];
    let mut i = 0;
    while i < 64 {
        let bb = Bitboard::from_square(i);
        bbs[i as usize] = bb.shifted::<{ Direction::Nne }>()
            | bb.shifted::<{ Direction::Nee }>()
            | bb.shifted::<{ Direction::See }>()
            | bb.shifted::<{ Direction::Sse }>()
            | bb.shifted::<{ Direction::Ssw }>()
            | bb.shifted::<{ Direction::Sww }>()
            | bb.shifted::<{ Direction::Nww }>()
            | bb.shifted::<{ Direction::Nnw }>();
        i += 1;
    }

    bbs
};

fn knight(board: &Board, state: State, list: &mut Vec<Move>, pins: &Pins, checkmask: Bitboard) {
    let mut bb = board.get::<{ Piece::Knight }>(state.white) & !(pins.hv | pins.diag);
    let (mut from, mut to);
    for_each!(bb, from, {
        let mut moves = KNIGHT_LOOKUP[from as usize] & board.empty() & checkmask;
        for_each!(moves, to, {
            let m = Move::new(from, to, moves::Type::Quiet);
            list.push(m);
        });

        let mut moves = KNIGHT_LOOKUP[from as usize] & board.enemy(state.white) & checkmask;
        for_each!(moves, to, {
            let m = Move::new(from, to, moves::Type::Capture);
            list.push(m);
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitboard::bb;
    use test_case::test_case;

    #[test_case(0 => bb![10, 17])]
    #[test_case(18 => bb![1, 3, 8, 12, 24, 28, 33, 35])]
    fn knight_lookup_tests(sq: usize) -> Bitboard {
        KNIGHT_LOOKUP[sq]
    }
}

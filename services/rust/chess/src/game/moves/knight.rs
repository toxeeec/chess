use super::{pins::Pins, Move};
use crate::game::{board::Board, moves::Type, piece::Piece, state::State};
use bitboard::{bb, for_each, shift::Direction, Bitboard};

pub fn knight(board: &Board, state: State, list: &mut Vec<Move>, pins: &Pins, checkmask: Bitboard) {
    let mut bb = board.get::<{ Piece::Knight }>(state.white) & !(pins.hv | pins.diag);
    let enemy = board.enemy(state.white);
    let enemy_or_empty = board.enemy_or_empty(state.white);
    let (mut from, mut to);
    for_each!(bb, from, {
        let mut moves = KNIGHT_LOOKUP[from as usize] & enemy_or_empty & checkmask;
        for_each!(moves, to, {
            let typ = if enemy.contains(to) {
                Type::Capture
            } else {
                Type::Quiet
            };
            let m = Move::new(from, to, typ);
            list.push(m);
        });
    });
}

pub(crate) const KNIGHT_LOOKUP: [Bitboard; 64] = {
    let mut bbs = [Bitboard::default(); 64];
    let mut i = 0;
    while i < 64 {
        let bb = bb![i];
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

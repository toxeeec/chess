use crate::{
    bitboard,
    bitboard::{Bitboard, Direction},
    board::Board,
    game::Color,
    moves::{Move, MoveList},
};

pub(super) fn add_knight_moves<const COLOR: Color>(board: &Board, list: &mut MoveList) {
    let knights = board.knights::<COLOR>();
    let blockers = board.occupancy::<COLOR>();

    for from in knights {
        let moves = KNIGHT_ATTACKS[from.0 as usize] & !blockers;
        list.extend(moves.map(|to| Move::new(from, to)));
    }
}

const KNIGHT_ATTACKS: [Bitboard; 64] = {
    let mut attacks = [Bitboard::EMPTY; 64];
    let mut square = 0;

    while square < 64 {
        let bb = bitboard!(square);
        attacks[square] = bb.shift::<{ Direction::Nne }>()
            | bb.shift::<{ Direction::Nnw }>()
            | bb.shift::<{ Direction::Nee }>()
            | bb.shift::<{ Direction::Nww }>()
            | bb.shift::<{ Direction::Sse }>()
            | bb.shift::<{ Direction::Ssw }>()
            | bb.shift::<{ Direction::See }>()
            | bb.shift::<{ Direction::Sww }>();
        square += 1;
    }

    attacks
};

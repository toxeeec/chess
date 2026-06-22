use crate::{
    bitboard,
    bitboard::{Bitboard, Direction},
    board::Board,
    game::Color,
    moves::{Move, MoveList},
};

pub(super) fn add_king_moves<const COLOR: Color>(board: &Board, list: &mut MoveList) {
    let mut king = board.king::<COLOR>();
    let blockers = board.occupancy::<COLOR>();

    debug_assert_eq!(king.len(), 1);

    let from = unsafe { king.next().unwrap_unchecked() };
    let moves = KING_ATTACKS[from.0 as usize] & !blockers;
    list.extend(moves.map(|to| Move::new(from, to)));
}

const KING_ATTACKS: [Bitboard; 64] = {
    let mut attacks = [Bitboard::EMPTY; 64];
    let mut square = 0;

    while square < 64 {
        let bb = bitboard!(square);
        attacks[square] = bb.shift::<{ Direction::North }>()
            | bb.shift::<{ Direction::South }>()
            | bb.shift::<{ Direction::East }>()
            | bb.shift::<{ Direction::West }>()
            | bb.shift::<{ Direction::Northeast }>()
            | bb.shift::<{ Direction::Northwest }>()
            | bb.shift::<{ Direction::Southeast }>()
            | bb.shift::<{ Direction::Southwest }>();
        square += 1;
    }

    attacks
};

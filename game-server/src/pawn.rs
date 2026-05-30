use crate::{bitboard::Bitboard, board::Board, moves::Move, square::Square};

pub(super) fn add_pawn_moves(board: &Board, list: &mut Vec<Move>) {
    let empty = !board.occupied();

    let single_pushes = ((board.white_pawns() & !Bitboard::RANK_7) << 8) & empty;
    let double_pushes = ((single_pushes & Bitboard::RANK_3) << 8) & empty;

    list.reserve(single_pushes.len() + double_pushes.len());
    list.extend(single_pushes.map(|to| Move::new(Square::new(to.0 - 8), to)));
    list.extend(double_pushes.map(|to| Move::new(Square::new(to.0 - 16), to)));
}

use crate::{
    bitboard::Bitboard,
    board::Board,
    moves::{Move, MoveList},
    square::Square,
};

pub(super) fn add_pawn_moves<const IS_WHITE: bool>(board: &Board, list: &mut MoveList) {
    let empty = board.empty();
    let pawns = board.pawns::<IS_WHITE>();

    let single_pushes =
        ((pawns & !Bitboard::relative_rank::<IS_WHITE>(7)).forward::<IS_WHITE>(1)) & empty;
    let double_pushes =
        ((single_pushes & Bitboard::relative_rank::<IS_WHITE>(3)).forward::<IS_WHITE>(1)) & empty;

    list.reserve(single_pushes.len() + double_pushes.len());
    list.extend(single_pushes.map(|to| Move::new(Square::backward::<IS_WHITE>(to, 1), to)));
    list.extend(double_pushes.map(|to| Move::new(Square::backward::<IS_WHITE>(to, 2), to)));
}

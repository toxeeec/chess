use crate::{
    bitboard::Bitboard,
    board::Board,
    game::Color,
    moves::{Move, MoveList},
};

pub(super) fn add_pawn_moves<const COLOR: Color>(board: &Board, list: &mut MoveList) {
    let empty = board.empty();
    let pawns = board.pawns::<COLOR>();

    let single_pushes =
        ((pawns & !Bitboard::relative_rank::<COLOR>(7)).forward::<COLOR, 1>()) & empty;
    let double_pushes =
        ((single_pushes & Bitboard::relative_rank::<COLOR>(3)).forward::<COLOR, 1>()) & empty;

    list.reserve(single_pushes.len() + double_pushes.len());
    list.extend(single_pushes.map(|to| Move::new(to.backward::<COLOR, 1>(), to)));
    list.extend(double_pushes.map(|to| Move::new(to.backward::<COLOR, 2>(), to)));
}

use super::{list::List, pins::Pins, Move};
use crate::game::{
    board::Board,
    moves::{magics::rook_moves, Type},
    piece::Piece,
    state::State,
};
use bitboard::{for_each, Bitboard};

pub fn rook(board: &Board, state: State, list: &mut List, pins: &Pins, checkmask: Bitboard) {
    let bb = board.get::<{ Piece::Rook }>(state.white) & !pins.diag;
    inner(bb, board, state, list, pins, checkmask);
}

pub(crate) fn inner(
    mut bb: Bitboard,
    board: &Board,
    state: State,
    list: &mut List,
    pins: &Pins,
    checkmask: Bitboard,
) {
    let enemy = board.enemy(state.white);
    let enemy_or_empty = board.enemy_or_empty(state.white);
    let (mut from, mut to);
    for_each!(bb, from, {
        let mut moves = rook_moves(from, board.occ) & enemy_or_empty & checkmask;
        if pins.hv.contains(from) {
            moves &= pins.hv;
        }
        for_each!(moves, to, {
            let typ = if enemy.contains(to) {
                Type::Capture
            } else {
                Type::Quiet
            };
            let m = Move::new(from, to, typ);
            list.0.push(m);
        });
    });
}

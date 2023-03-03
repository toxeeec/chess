use super::{pins::Pins, Move};
use crate::game::{
    board::Board,
    moves::{magics::bishop_moves, Type},
    piece::Piece,
    state::State,
};
use bitboard::{for_each, Bitboard};

pub fn bishop(board: &Board, state: State, list: &mut Vec<Move>, pins: &Pins, checkmask: Bitboard) {
    let bb = board.get::<{ Piece::Bishop }>(state.white) & !pins.hv;
    inner(bb, board, state, list, pins, checkmask);
}

pub(crate) fn inner(
    mut bb: Bitboard,
    board: &Board,
    state: State,
    list: &mut Vec<Move>,
    pins: &Pins,
    checkmask: Bitboard,
) {
    let empty = board.empty();
    let enemy = board.enemy(state.white);
    let (mut from, mut to);
    for_each!(bb, from, {
        let mut moves = bishop_moves(from, board.occ) & empty & checkmask;
        if pins.diag.contains(from) {
            moves &= pins.diag;
        }
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

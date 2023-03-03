use super::{bishop, pins::Pins, rook, Move};
use crate::game::{board::Board, piece::Piece, state::State};
use bitboard::Bitboard;

pub fn queen(board: &Board, state: State, list: &mut Vec<Move>, pins: &Pins, checkmask: Bitboard) {
    let bb = board.get::<{ Piece::Queen }>(state.white);
    let not_diag_pinned = bb & !pins.diag;
    rook::inner(not_diag_pinned, board, state, list, pins, checkmask);
    let not_hv_pinned = bb & !pins.hv;
    bishop::inner(not_hv_pinned, board, state, list, pins, checkmask);
}

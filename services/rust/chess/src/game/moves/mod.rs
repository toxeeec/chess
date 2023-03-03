use self::{king::KING_LOOKUP, pins::Pins};

use super::{board::Board, piece::Piece, state::State};
use bishop::bishop;
use bitboard::Bitboard;
use king::king;
use knight::knight;
use queen::queen;
use rook::rook;

mod bishop;
mod king;
mod knight;
mod magics;
mod pins;
mod queen;
mod rook;

#[repr(u32)]
pub enum Type {
    Quiet,
    KingCastle,
    QueenCastle,
    Capture,
}

// From   | To     | Type
// xxxxxx | xxxxxx | xxxx
// 15-10  | 9-4    | 3-0
pub struct Move(u16);

impl Move {
    fn new(from: u32, to: u32, typ: Type) -> Self {
        Self(((from as u16) << 6) | ((to as u16) << 4) | (typ as u16))
    }
}

pub fn generate(list: &mut Vec<Move>, board: &Board, state: State, ep_square: u32) {
    let enemy_king_sq = board.get::<{ Piece::King }>(!state.white).lsb();
    let banned = KING_LOOKUP[enemy_king_sq as usize];
    // TODO: checkmask
    let pins = &Pins::new(state.white, board);
    king(board, state, list, pins, banned);
    let checkmask = Bitboard::default();
    if checkmask.is_empty() {
        return;
    }

    // TODO: pawn moves
    rook(board, state, list, pins, checkmask);
    knight(board, state, list, pins, checkmask);
    bishop(board, state, list, pins, checkmask);
    queen(board, state, list, pins, checkmask);
}

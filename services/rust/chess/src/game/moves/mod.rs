use self::{checkmask::checkmask, king::KING_LOOKUP, pins::Pins};
use super::{board::Board, piece::Piece, state::State};
use bishop::bishop;
use king::king;
use knight::knight;
use num_derive::FromPrimitive;
use queen::queen;
use rook::rook;
use std::fmt::Debug;

mod bishop;
mod checkmask;
mod king;
mod knight;
mod magics;
mod pins;
mod queen;
mod rook;

#[derive(Debug, FromPrimitive)]
pub enum Type {
    Quiet = 0,
    KingCastle = 1,
    QueenCastle = 2,
    Capture = 3,
}

// From   | To     | Type
// xxxxxx | xxxxxx | xxxx
// 15-10  | 9-4    | 3-0
#[derive(Clone, Copy)]
pub struct Move(u16);

impl Move {
    fn new(from: u32, to: u32, typ: Type) -> Self {
        Self(((from as u16) << 6) | ((to as u16) << 4) | (typ as u16))
    }

    fn from(self) -> u16 {
        self.0 >> 10
    }

    fn to(self) -> u16 {
        (self.0 >> 4) & 0b111111
    }

    fn typ(self) -> Type {
        num::FromPrimitive::from_u16(self.0 & 0b1111).unwrap()
    }
}

impl Debug for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<From: {}, To: {}, Type: {:?}>",
            self.from(),
            self.to(),
            self.typ()
        )
    }
}

pub fn generate(list: &mut Vec<Move>, board: &Board, state: State) {
    let enemy_king_sq = board.get::<{ Piece::King }>(!state.white).lsb();
    let mut banned = KING_LOOKUP[enemy_king_sq as usize];
    let pins = &Pins::new(state.white, board);
    king(board, state, list, pins, banned);
    let checkmask = checkmask(state.white, board, &mut banned);
    if checkmask.is_empty() {
        return;
    }

    // TODO: pawn moves
    rook(board, state, list, pins, checkmask);
    knight(board, state, list, pins, checkmask);
    bishop(board, state, list, pins, checkmask);
    queen(board, state, list, pins, checkmask);
}

use self::{checkmask::checkmask, king::KING_LOOKUP, pins::Pins};
use super::{board::Board, piece::Piece, state::State};
use bishop::bishop;
use king::king;
use knight::knight;
use num_derive::FromPrimitive;
use pawn::pawn;
use queen::queen;
use rook::rook;
use std::fmt::Debug;

mod bishop;
mod checkmask;
mod king;
mod knight;
mod magics;
mod pawn;
mod pins;
mod queen;
mod rook;

const CAPTURE: isize = 0b100;
const PROMOTION: isize = 0b1000;

#[derive(Debug, FromPrimitive, PartialEq, Eq, Clone, Copy)]
pub enum Type {
    Quiet,
    DoublePush,
    KingCastle,
    QueenCastle,
    Capture = CAPTURE,
    EnPassant,
    KnightPromotion = PROMOTION,
    BishopPromotion,
    RookPromotion,
    QueenPromotion,
    KnightPromotionCapture,
    BishopPromotionCapture,
    RookPromotionCapture,
    QueenPromotionCapture,
}

impl Type {
    pub const fn is_capture(self) -> bool {
        self as isize & CAPTURE != 0
    }

    pub const fn promotion_piece(self) -> Option<Piece> {
        match self {
            Type::KnightPromotion | Type::KnightPromotionCapture => Some(Piece::Knight),
            Type::BishopPromotion | Type::BishopPromotionCapture => Some(Piece::Bishop),
            Type::RookPromotion | Type::RookPromotionCapture => Some(Piece::Rook),
            Type::QueenPromotion | Type::QueenPromotionCapture => Some(Piece::Queen),
            _ => None,
        }
    }
}

// From   | To     | Type
// xxxxxx | xxxxxx | xxxx
// 15-10  | 9-4    | 3-0
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Move(u16);

impl Move {
    pub fn new(from: u32, to: u32, typ: Type) -> Self {
        debug_assert!(from < 64);
        debug_assert!(to < 64);
        Self(((from as u16) << 10) | ((to as u16) << 4) | (typ as u16))
    }

    pub fn from(self) -> u32 {
        (self.0 >> 10) as u32
    }

    pub fn to(self) -> u32 {
        ((self.0 >> 4) & 0b111111) as u32
    }

    pub fn typ(self) -> Type {
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
    king(board, state, list, banned);
    let checkmask = checkmask(state.white, board, &mut banned);
    if checkmask.is_empty() {
        return;
    }

    pawn(board, state, list, pins, checkmask);
    rook(board, state, list, pins, checkmask);
    knight(board, state, list, pins, checkmask);
    bishop(board, state, list, pins, checkmask);
    queen(board, state, list, pins, checkmask);
}

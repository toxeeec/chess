use self::{checkmask::checkmask_and_banned, pins::Pins};
use super::{board::Board, piece::Piece, state::State};
use bishop::bishop;
use bitboard::{square::Square, Bitboard};
use king::king;
use knight::knight;
use num_derive::FromPrimitive;
use pawn::pawn;
use queen::queen;
use rook::rook;
use std::fmt::{Debug, Display};

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
    pub fn new(from: Square, to: Square, typ: Type) -> Self {
        Self(((from.0 as u16) << 10) | ((to.0 as u16) << 4) | (typ as u16))
    }

    pub fn from(self) -> Square {
        (self.0 as u32 >> 10).into()
    }

    pub fn to(self) -> Square {
        ((self.0 as u32 >> 4) & 0b111111).into()
    }

    pub fn typ(self) -> Type {
        num::FromPrimitive::from_u16(self.0 & 0b1111).unwrap()
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.from(),
            self.to(),
            self.typ()
                .promotion_piece()
                .map_or(String::new(), |p| p.to_string())
        )
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

// returns if king is currently in check
pub fn generate(list: &mut Vec<Move>, board: &Board, state: State) -> bool {
    let (checkmask, banned) = checkmask_and_banned(state.white, board);
    king(board, state, list, banned);
    if checkmask.is_empty() {
        return true;
    }

    let pins = &Pins::new(state.white, board);
    pawn(board, state, list, pins, checkmask);
    rook(board, state, list, pins, checkmask);
    knight(board, state, list, pins, checkmask);
    bishop(board, state, list, pins, checkmask);
    queen(board, state, list, pins, checkmask);
    checkmask != !Bitboard::default()
}

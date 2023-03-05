use enum_iterator::Sequence;
use num_derive::FromPrimitive;

#[derive(Debug, PartialEq, Eq, FromPrimitive, Clone, Copy, Sequence)]
pub enum Piece {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

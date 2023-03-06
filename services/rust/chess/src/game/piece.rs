use std::fmt::Display;

use enum_iterator::Sequence;
use num_derive::FromPrimitive;
use thiserror::Error;

#[derive(Debug, PartialEq, Eq, FromPrimitive, Clone, Copy, Sequence)]
pub enum Piece {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ParsePieceError {
    #[error("unknown piece: {0}")]
    Unknown(char),
}

impl TryFrom<char> for Piece {
    type Error = ParsePieceError;
    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'P' | 'p' => Ok(Piece::Pawn),
            'R' | 'r' => Ok(Piece::Rook),
            'N' | 'n' => Ok(Piece::Knight),
            'B' | 'b' => Ok(Piece::Bishop),
            'Q' | 'q' => Ok(Piece::Queen),
            'K' | 'k' => Ok(Piece::King),
            _ => Err(ParsePieceError::Unknown(value)),
        }
    }
}

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Piece::Pawn => 'p',
                Piece::Rook => 'r',
                Piece::Knight => 'n',
                Piece::Bishop => 'b',
                Piece::Queen => 'q',
                Piece::King => 'k',
            }
        )
    }
}

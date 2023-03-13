use super::Move;
use crate::game::{piece::Piece, MoveError};

#[derive(Clone)]
pub struct List(pub Vec<Move>);

impl List {
    pub fn find(&self, move_str: &str) -> Result<Move, MoveError> {
        let len = move_str.len();
        if !(4..=5).contains(&len) {
            return Err(MoveError::Format);
        }
        let from = move_str[..2].parse()?;
        let to = move_str[2..4].parse()?;
        let promotion_piece = match move_str.chars().nth(4) {
            Some(c) => Some(Piece::try_from(c)?),
            None => None,
        };
        for mov in &self.0 {
            if mov.from() == from
                && mov.to() == to
                && mov.typ().promotion_piece() == promotion_piece
            {
                return Ok(*mov);
            }
        }
        Err(MoveError::Illegal(move_str.to_string()))
    }
}

impl Default for List {
    fn default() -> Self {
        List(Vec::with_capacity(32))
    }
}

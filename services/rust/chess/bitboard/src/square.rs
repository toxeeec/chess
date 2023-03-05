use std::str::FromStr;
use thiserror::Error;

use crate::{bb, shift::Direction, Bitboard};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Square(pub u32);

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ParseSquareError {
    #[error("square can only contain a letter from a to h and a number from 1 to 8")]
    Format,
    #[error("file must be a letter from a to h but was {0}")]
    File(char),
    #[error("rank must be a number from 1 to 8 but was {0}")]
    Rank(char),
}

impl Square {
    pub const fn new(sq: u32) -> Self {
        debug_assert!(sq < 64);
        Self(sq)
    }

    pub const fn rank_of(self) -> u32 {
        self.0 / 8
    }

    pub const fn shifted_by(self, dir: Direction) -> Self {
        let sq = Self(self.0.wrapping_add(dir as u32));
        debug_assert!(sq.0 < 64);
        sq
    }
}

impl FromStr for Square {
    type Err = ParseSquareError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = s.as_bytes();
        if bytes.len() != 2 {
            return Err(ParseSquareError::Format);
        }
        let file = bytes[0].wrapping_sub(b'a');
        if file > 7 {
            return Err(ParseSquareError::File(bytes[0] as char));
        }
        let rank = bytes[1].wrapping_sub(b'1');
        if rank > 7 {
            return Err(ParseSquareError::Rank(bytes[1] as char));
        }
        Ok(Self((file * 8 + rank) as u32))
    }
}

impl From<Square> for Bitboard {
    fn from(value: Square) -> Self {
        bb![value.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("a1" => Ok(Square(0)))]
    #[test_case("h8" => Ok(Square(63)))]
    #[test_case("abc" => Err(ParseSquareError::Format))]
    #[test_case("a9" => Err(ParseSquareError::Rank('9')))]
    #[test_case("i1" => Err(ParseSquareError::File('i')))]
    fn from_str_tests(s: &str) -> Result<Square, ParseSquareError> {
        s.parse()
    }
}

use std::{fmt::Display, ops::Sub, str::FromStr};
use thiserror::Error;

use crate::{bb, shift::Direction, Bitboard};

#[derive_const(PartialOrd, Ord, PartialEq)]
#[derive(Debug, Eq, Clone, Copy)]
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
    pub const fn file(self) -> u32 {
        self.0 % 8
    }

    pub const fn rank(self) -> u32 {
        self.0 / 8
    }

    pub const fn shifted_by(self, dir: Direction) -> Option<Self> {
        let sq = Self(self.0.wrapping_add(dir as u32));
        if sq.0 < 64 {
            Some(sq)
        } else {
            None
        }
    }
}

pub const WHITE_KING_SQ: Square = 4.into();
pub const BLACK_KING_SQ: Square = 60.into();
pub const WHITE_LEFT_ROOK_SQ: Square = 0.into();
pub const BLACK_LEFT_ROOK_SQ: Square = 56.into();
pub const WHITE_RIGHT_ROOK_SQ: Square = 7.into();
pub const BLACK_RIGHT_ROOK_SQ: Square = 63.into();
pub const WHITE_KING_KING_CASTLE_SQ: Square = 6.into();
pub const BLACK_KING_KING_CASTLE_SQ: Square = 62.into();
pub const WHITE_KING_QUEEN_CASTLE_SQ: Square = 2.into();
pub const BLACK_KING_QUEEN_CASTLE_SQ: Square = 58.into();

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
        Ok(Self((rank * 8 + file) as u32))
    }
}

impl const Sub for Square {
    type Output = u32;
    fn sub(self, rhs: Self) -> Self::Output {
        self.0 - rhs.0
    }
}

impl const From<u32> for Square {
    fn from(value: u32) -> Self {
        debug_assert!(value < 64);
        Self(value)
    }
}

impl const From<Square> for Bitboard {
    fn from(value: Square) -> Self {
        bb![value.0]
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            (b'a' + self.file() as u8) as char,
            (b'1' + self.rank() as u8) as char
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("a1" => Ok(Square(0)))]
    #[test_case("e4" => Ok(Square(28)))]
    #[test_case("h8" => Ok(Square(63)))]
    #[test_case("abc" => Err(ParseSquareError::Format))]
    #[test_case("a9" => Err(ParseSquareError::Rank('9')))]
    #[test_case("i1" => Err(ParseSquareError::File('i')))]
    fn from_str_tests(s: &str) -> Result<Square, ParseSquareError> {
        s.parse()
    }
}

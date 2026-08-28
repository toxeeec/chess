use std::{
    fmt::{self, Write},
    str::FromStr,
};

use anyhow::{Result, bail};

use crate::square::Square;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PromotionPiece {
    Queen,
    Rook,
    Bishop,
    Knight,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Move {
    pub(super) from: Square,
    pub(super) to: Square,
    pub(super) promotion: Option<PromotionPiece>,
}

pub(crate) struct MoveList(Vec<Move>);

const MAX_LEGAL_MOVES: usize = 218;

impl PromotionPiece {
    pub(super) const ALL: [Self; 4] = [Self::Queen, Self::Rook, Self::Bishop, Self::Knight];
}

impl Move {
    pub(super) fn new(from: Square, to: Square, promotion: Option<PromotionPiece>) -> Self {
        Self {
            from,
            to,
            promotion,
        }
    }
}

impl MoveList {
    pub(super) const EMPTY: &'static Self = &Self(Vec::new());

    #[cfg(test)]
    pub(super) fn from_ascii(ascii: &str) -> Self {
        let squares = ascii
            .chars()
            .filter(|square| !square.is_whitespace())
            .collect::<Vec<_>>();

        assert_eq!(squares.len(), 64, "moves! must contain 64 squares");

        let mut from = None;
        let mut targets = Vec::new();

        for (index, square) in squares.into_iter().enumerate() {
            let rank = 7 - index as u32 / 8;
            let file = index as u32 % 8;
            let square_index = Square::new(rank * 8 + file);

            match square {
                '.' => {}
                'o' => {
                    assert!(
                        from.replace(square_index).is_none(),
                        "moves! must contain one o source"
                    );
                }
                'x' => targets.push(square_index),
                _ => panic!("invalid moves! square `{square}`; expected . o or x"),
            }
        }

        let from = from.expect("moves! must contain one o source");

        Self(
            targets
                .into_iter()
                .map(|to| Move::new(from, to, None))
                .collect(),
        )
    }

    pub(crate) fn clear(&mut self) {
        self.0.clear();
    }

    pub(crate) fn contains(&self, mve: Move) -> bool {
        self.0.contains(&mve)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub(crate) fn extend<T: IntoIterator<Item = Move>>(&mut self, iter: T) {
        self.0.extend(iter);
    }

    pub(crate) fn push(&mut self, mve: Move) {
        self.0.push(mve);
    }

    #[cfg(any(test, feature = "benchmark"))]
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    #[cfg(any(test, feature = "benchmark"))]
    pub(crate) fn iter(&self) -> impl Iterator<Item = Move> + '_ {
        self.0.iter().copied()
    }
}

impl TryFrom<&u8> for PromotionPiece {
    type Error = anyhow::Error;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match value {
            b'q' => Ok(Self::Queen),
            b'r' => Ok(Self::Rook),
            b'b' => Ok(Self::Bishop),
            b'n' => Ok(Self::Knight),
            _ => bail!("invalid promotion piece"),
        }
    }
}

impl fmt::Display for PromotionPiece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let piece = match self {
            Self::Queen => 'q',
            Self::Rook => 'r',
            Self::Bishop => 'b',
            Self::Knight => 'n',
        };

        f.write_char(piece)
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char((b'a' + self.from.file()) as char)?;
        f.write_char((b'1' + self.from.rank()) as char)?;
        f.write_char((b'a' + self.to.file()) as char)?;
        f.write_char((b'1' + self.to.rank()) as char)?;
        if let Some(promotion) = self.promotion {
            write!(f, "{promotion}")?;
        }
        Ok(())
    }
}

impl FromStr for Move {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.as_bytes();
        if value.len() != 4 && value.len() != 5 {
            bail!("move must contain 4 or 5 characters");
        }

        let from = parse_square(value[0], value[1])?;
        let to = parse_square(value[2], value[3])?;
        let promotion = value.get(4).map(PromotionPiece::try_from).transpose()?;

        Ok(Self::new(from, to, promotion))
    }
}

impl fmt::Display for MoveList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut moves = self.0.iter();
        if let Some(first) = moves.next() {
            write!(f, "{}", first)?;

            for mve in moves {
                write!(f, " {}", mve)?;
            }
        }
        Ok(())
    }
}

impl Default for MoveList {
    fn default() -> Self {
        Self(Vec::with_capacity(MAX_LEGAL_MOVES))
    }
}

fn parse_square(file: u8, rank: u8) -> Result<Square> {
    if !(b'a'..=b'h').contains(&file) || !(b'1'..=b'8').contains(&rank) {
        bail!("invalid square");
    }

    Ok(Square::new((rank - b'1') as u32 * 8 + (file - b'a') as u32))
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::square;

    use super::{Move, PromotionPiece};

    #[test]
    fn parses_valid_moves() {
        assert_eq!(
            Move::from_str("a2a4").unwrap(),
            Move::new(square!(a2), square!(a4), None)
        );
        assert_eq!(
            Move::from_str("h7h5").unwrap(),
            Move::new(square!(h7), square!(h5), None)
        );
        assert_eq!(
            Move::from_str("a1h8").unwrap(),
            Move::new(square!(a1), square!(h8), None)
        );
        assert_eq!(
            Move::from_str("a7a8n").unwrap(),
            Move::new(square!(a7), square!(a8), Some(PromotionPiece::Knight))
        );
    }

    #[test]
    fn rejects_invalid_moves() {
        for mve in [
            "", "a2a", "a2a44q", "a7a8k", "a7a8Q", "i2a4", "a0a4", "a2i4", "a2a9", "A2A4",
        ] {
            assert!(Move::from_str(mve).is_err(), "{mve} should be invalid");
        }
    }

    #[test]
    fn roundtrips_through_string() {
        for mve in ["a2a4", "h7h5", "a1h8", "a7a8q", "b2a1r", "c7c8b", "h2h1n"] {
            assert_eq!(Move::from_str(mve).unwrap().to_string(), mve);
        }
    }
}

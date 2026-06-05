use crate::square::Square;
use anyhow::{Result, bail};
use std::{
    fmt::{self, Write},
    str::FromStr,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Move {
    pub(super) from: Square,
    pub(super) to: Square,
}

impl Move {
    pub(super) const fn new(from: Square, to: Square) -> Self {
        Self { from, to }
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char((b'a' + (self.from.0 % 8) as u8) as char)?;
        f.write_char((b'1' + (self.from.0 / 8) as u8) as char)?;

        f.write_char((b'a' + (self.to.0 % 8) as u8) as char)?;
        f.write_char((b'1' + (self.to.0 / 8) as u8) as char)
    }
}

impl FromStr for Move {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.as_bytes();
        if value.len() != 4 {
            bail!("move must contain exactly 4 characters");
        }

        Ok(Self::new(
            parse_square(value[0], value[1])?,
            parse_square(value[2], value[3])?,
        ))
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
    use super::Move;
    use crate::square::Square;
    use std::str::FromStr;

    #[test]
    fn parses_valid_moves() {
        assert_eq!(
            Move::from_str("a2a4").unwrap(),
            Move::new(Square::new(8), Square::new(24))
        );
        assert_eq!(
            Move::from_str("h7h5").unwrap(),
            Move::new(Square::new(55), Square::new(39))
        );
        assert_eq!(
            Move::from_str("a1h8").unwrap(),
            Move::new(Square::new(0), Square::new(63))
        );
    }

    #[test]
    fn rejects_invalid_moves() {
        for mve in ["", "a2a", "a2a44", "i2a4", "a0a4", "a2i4", "a2a9", "A2A4"] {
            assert!(Move::from_str(mve).is_err(), "{mve} should be invalid");
        }
    }

    #[test]
    fn roundtrips_through_string() {
        for mve in ["a2a4", "h7h5", "a1h8"] {
            assert_eq!(Move::from_str(mve).unwrap().to_string(), mve);
        }
    }
}

use std::{
    fmt::{self, Write},
    str::FromStr,
};

use anyhow::{Result, bail};

use crate::square::Square;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub(super) enum PromotionPiece {
    Knight = 0,
    Bishop = 1,
    Rook = 2,
    Queen = 3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub(super) enum MoveKind {
    Quiet = 0,
    DoublePush = 1,
    CastleKing = 2,
    CastleQueen = 3,
    Capture = 4,
    EnPassant = 5,
    PromoteKnight = 8,
    PromoteBishop = 9,
    PromoteRook = 10,
    PromoteQueen = 11,
    CapturePromoteKnight = 12,
    CapturePromoteBishop = 13,
    CapturePromoteRook = 14,
    CapturePromoteQueen = 15,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub(super) struct Move(u16);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub(super) struct UciMove(u16);

pub(crate) struct MoveList(Vec<Move>);

const MAX_LEGAL_MOVES: usize = 218;

impl MoveKind {
    pub(super) const QUIET_PROMOTIONS: [Self; 4] = [
        Self::PromoteQueen,
        Self::PromoteRook,
        Self::PromoteBishop,
        Self::PromoteKnight,
    ];
    pub(super) const CAPTURE_PROMOTIONS: [Self; 4] = [
        Self::CapturePromoteQueen,
        Self::CapturePromoteRook,
        Self::CapturePromoteBishop,
        Self::CapturePromoteKnight,
    ];

    pub(super) fn promotion(self) -> Option<PromotionPiece> {
        let code = self as u8;
        if code & 0b1000 == 0 {
            None
        } else {
            // SAFETY: Masking to the low two bits produces a code in `0..=3`.
            Some(unsafe { PromotionPiece::from_code_unchecked(code & 0b11) })
        }
    }

    pub(super) fn is_capture(self) -> bool {
        self as u8 & 0b100 != 0
    }

    /// # Safety
    ///
    /// `code` must be a valid `MoveKind` discriminant in `0..=5` or `8..=15`.
    unsafe fn from_code_unchecked(code: u8) -> Self {
        debug_assert!(code < 16 && code != 6 && code != 7);

        // SAFETY: `MoveKind` has `repr(u8)` discriminants for 0 through 5 and 8 through 15.
        unsafe { std::mem::transmute(code) }
    }
}

impl PromotionPiece {
    /// # Safety
    ///
    /// `code` must be a valid `PromotionPiece` discriminant in `0..=3`.
    unsafe fn from_code_unchecked(code: u8) -> Self {
        debug_assert!(code < 4);

        // SAFETY: PromotionPiece has contiguous repr(u8) discriminants from 0 through 3.
        unsafe { std::mem::transmute(code) }
    }
}

impl Move {
    const SQUARE_MASK: u16 = 0b111111;
    const KIND_MASK: u16 = 0b1111;
    const TO_SHIFT: u32 = 4;
    const FROM_SHIFT: u32 = 10;

    pub(super) fn new(from: Square, to: Square, kind: MoveKind) -> Self {
        Self(
            (u16::from(from) << Self::FROM_SHIFT) | (u16::from(to) << Self::TO_SHIFT) | kind as u16,
        )
    }

    pub(super) fn from(self) -> Square {
        Square::new(u32::from(self.0 >> Self::FROM_SHIFT))
    }

    pub(super) fn to(self) -> Square {
        Square::new(u32::from((self.0 >> Self::TO_SHIFT) & Self::SQUARE_MASK))
    }

    pub(super) fn kind(self) -> MoveKind {
        let code = (self.0 & Self::KIND_MASK) as u8;

        // SAFETY: `Move` values are only built from valid `MoveKind` discriminants.
        unsafe { MoveKind::from_code_unchecked(code) }
    }
}

impl MoveList {
    pub(super) const EMPTY: &'static Self = &Self(Vec::new());

    pub(crate) fn clear(&mut self) {
        self.0.clear();
    }

    pub(crate) fn resolve(&self, input: UciMove) -> Option<Move> {
        self.0
            .iter()
            .copied()
            .find(|mve| Into::<UciMove>::into(*mve) == input)
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

impl UciMove {
    pub(super) fn new(from: Square, to: Square, promotion: Option<PromotionPiece>) -> Self {
        let promotion = promotion.map_or(0, |piece| 0b1000 | u16::from(piece as u8));
        Self((u16::from(from) << Move::FROM_SHIFT) | (u16::from(to) << Move::TO_SHIFT) | promotion)
    }

    fn from(self) -> Square {
        Square::new(u32::from(self.0 >> Move::FROM_SHIFT))
    }

    fn to(self) -> Square {
        Square::new(u32::from((self.0 >> Move::TO_SHIFT) & Move::SQUARE_MASK))
    }

    fn promotion(self) -> Option<PromotionPiece> {
        let code = (self.0 & Move::KIND_MASK) as u8;
        if code == 0 {
            None
        } else {
            // SAFETY: All two-bit values are valid `PromotionPiece` discriminants.
            Some(unsafe { PromotionPiece::from_code_unchecked(code & 0b11) })
        }
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
        write!(f, "{}", Into::<UciMove>::into(*self))
    }
}

impl fmt::Display for UciMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let from = self.from();
        let to = self.to();

        f.write_char((b'a' + from.file()) as char)?;
        f.write_char((b'1' + from.rank()) as char)?;
        f.write_char((b'a' + to.file()) as char)?;
        f.write_char((b'1' + to.rank()) as char)?;
        if let Some(promotion) = self.promotion() {
            write!(f, "{promotion}")?;
        }
        Ok(())
    }
}

impl FromStr for UciMove {
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

impl From<Move> for UciMove {
    fn from(mve: Move) -> Self {
        let kind = mve.0 & Move::KIND_MASK;
        let promotion = (kind >> 3) * (0b1000 | (kind & 0b11));
        Self((mve.0 & !Move::KIND_MASK) | promotion)
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

const _: () = {
    assert!(MoveKind::PromoteKnight as u8 & 0b11 == PromotionPiece::Knight as u8);
    assert!(MoveKind::PromoteBishop as u8 & 0b11 == PromotionPiece::Bishop as u8);
    assert!(MoveKind::PromoteRook as u8 & 0b11 == PromotionPiece::Rook as u8);
    assert!(MoveKind::PromoteQueen as u8 & 0b11 == PromotionPiece::Queen as u8);

    assert!(MoveKind::CapturePromoteKnight as u8 & 0b11 == PromotionPiece::Knight as u8);
    assert!(MoveKind::CapturePromoteBishop as u8 & 0b11 == PromotionPiece::Bishop as u8);
    assert!(MoveKind::CapturePromoteRook as u8 & 0b11 == PromotionPiece::Rook as u8);
    assert!(MoveKind::CapturePromoteQueen as u8 & 0b11 == PromotionPiece::Queen as u8);
};

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::square;

    use super::{Move, MoveKind, UciMove};

    #[test]
    fn parses_and_roundtrips_moves() {
        for mve in ["a2a4", "h7h5", "a1h8", "a7a8q", "b2a1r", "c7c8b", "h2h1n"] {
            assert_eq!(UciMove::from_str(mve).unwrap().to_string(), mve);
        }
    }

    #[test]
    fn rejects_invalid_moves() {
        for mve in [
            "", "a2a", "a2a44q", "a7a8k", "a7a8Q", "i2a4", "a0a4", "a2i4", "a2a9", "A2A4",
        ] {
            assert!(UciMove::from_str(mve).is_err(), "{mve} should be invalid");
        }
    }

    #[test]
    fn every_internal_move_kind_roundtrips_through_uci() {
        for (kind, expected) in [
            (MoveKind::Quiet, "a2a3"),
            (MoveKind::DoublePush, "a2a3"),
            (MoveKind::CastleKing, "a2a3"),
            (MoveKind::CastleQueen, "a2a3"),
            (MoveKind::Capture, "a2a3"),
            (MoveKind::EnPassant, "a2a3"),
            (MoveKind::PromoteKnight, "a2a3n"),
            (MoveKind::PromoteBishop, "a2a3b"),
            (MoveKind::PromoteRook, "a2a3r"),
            (MoveKind::PromoteQueen, "a2a3q"),
            (MoveKind::CapturePromoteKnight, "a2a3n"),
            (MoveKind::CapturePromoteBishop, "a2a3b"),
            (MoveKind::CapturePromoteRook, "a2a3r"),
            (MoveKind::CapturePromoteQueen, "a2a3q"),
        ] {
            let uci = <UciMove as From<Move>>::from(Move::new(square!(a2), square!(a3), kind));
            assert_eq!(uci.to_string(), expected);
            assert_eq!(UciMove::from_str(expected).unwrap(), uci);
        }
    }
}

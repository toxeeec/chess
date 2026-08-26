use std::{marker::ConstParamTy, ops::Not};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::{board::Board, castling::CastlingRights, square::Square};

#[derive(Clone, Copy, ConstParamTy, Debug, Deserialize, Eq, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub(super) enum Color {
    White,
    Black,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) struct EnPassant(u32);

pub(super) struct State {
    pub(super) turn: Color,
    pub(super) castling_rights: CastlingRights,
    pub(super) en_passant: EnPassant,
}

impl Color {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::White => "white",
            Self::Black => "black",
        }
    }

    pub(super) const fn opponent(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }

    fn from_fen(value: &str) -> Result<Self> {
        match value {
            "w" => Ok(Self::White),
            "b" => Ok(Self::Black),
            _ => bail!("invalid FEN active color: {value}"),
        }
    }

    pub(super) const fn fen(self) -> &'static str {
        match self {
            Self::White => "w",
            Self::Black => "b",
        }
    }
}

impl EnPassant {
    pub(super) const NONE: Self = Self(64);

    pub(super) fn new(target: Square) -> Self {
        Self(target.into())
    }

    fn from_fen(value: &str, turn: Color, board: &Board) -> Result<Self> {
        if value == "-" {
            return Ok(Self::NONE);
        }

        let bytes = value.as_bytes();
        let expected_rank = match turn {
            Color::White => b'6',
            Color::Black => b'3',
        };
        if bytes.len() != 2 || !(b'a'..=b'h').contains(&bytes[0]) || bytes[1] != expected_rank {
            bail!("invalid FEN en passant target: {value}");
        }

        let target = Square::from_name(value);
        let captured = match turn {
            Color::White => target.backward::<{ Color::White }, 1>(),
            Color::Black => target.backward::<{ Color::Black }, 1>(),
        };
        let origin = match turn {
            Color::White => target.backward::<{ Color::Black }, 1>(),
            Color::Black => target.backward::<{ Color::White }, 1>(),
        };
        let enemy_pawns = match turn {
            Color::White => board.pawns::<{ Color::Black }>(),
            Color::Black => board.pawns::<{ Color::White }>(),
        };
        if board.occupied().contains(target)
            || board.occupied().contains(origin)
            || !enemy_pawns.contains(captured)
        {
            bail!("invalid FEN en passant target: {value}");
        }

        Ok(Self::new(target))
    }

    pub(super) fn target(self) -> Option<Square> {
        if self == Self::NONE {
            None
        } else {
            Some(Square::new(self.0))
        }
    }

    fn fen(self) -> String {
        self.target()
            .map_or_else(|| "-".to_owned(), |target| target.to_string())
    }
}

impl State {
    pub(super) const fn new(turn: Color, castling_rights: CastlingRights) -> Self {
        Self {
            turn,
            castling_rights,
            en_passant: EnPassant::NONE,
        }
    }

    pub(super) fn from_fen<'a>(
        fields: &mut impl Iterator<Item = &'a str>,
        board: &Board,
    ) -> Result<Self> {
        let active_color = fields.next().context("FEN must contain active color")?;
        let castling = fields.next().context("FEN must contain castling rights")?;
        let en_passant = fields
            .next()
            .context("FEN must contain en passant target")?;

        let turn = Color::from_fen(active_color)?;
        let castling_rights = CastlingRights::from_fen(castling, board)?;
        let en_passant = EnPassant::from_fen(en_passant, turn, board)?;

        Ok(Self {
            turn,
            castling_rights,
            en_passant,
        })
    }

    pub(super) fn fen(&self) -> String {
        format!(
            "{} {} {} 0 1",
            self.turn.fen(),
            self.castling_rights.fen(),
            self.en_passant.fen()
        )
    }
}

const impl Not for Color {
    type Output = Self;

    fn not(self) -> Self::Output {
        self.opponent()
    }
}

use std::fmt;

use serde::{Deserialize, Serialize, Serializer};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    game::{Game, Player},
    game_state::{GameLifecycle, GameState, MakeMoveError},
    moves::Move,
};

#[derive(Clone, Copy)]
#[wasm_bindgen]
pub enum GameStatus {
    Waiting = "waiting",
    Active = "active",
    Ended = "ended",
    Expired = "expired",
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen(getter_with_clone)]
pub struct SnapshotMessage {
    pub revision: u32,
    pub fen: String,
    pub status: GameStatus,
    #[wasm_bindgen(js_name = "legalMoves")]
    pub legal_moves: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct StatusMessage {
	status: GameStatus,
	legal_moves: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct MoveMessage {
    revision: u32,
    #[serde(rename = "move")]
    mve: String,
    turn: Player,
    legal_moves: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum ErrorMessage {
    GameNotActive,
    InvalidMessage,
    InvalidMoveFormat,
    InvalidPlayer,
    IllegalMove,
    NotYourTurn,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "data", rename_all = "lowercase")]
pub(super) enum ServerMessage {
    Snapshot(SnapshotMessage),
    Status(StatusMessage),
    Move(MoveMessage),
    Error(ErrorMessage),
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "lowercase")]
pub(super) enum ClientMessage {
    Move(String),
}

impl From<&GameState> for SnapshotMessage {
    fn from(state: &GameState) -> Self {
		Self {
			revision: state.revision,
			fen: state.game.fen(),
			status: state.lifecycle.into(),
			legal_moves: state.legal_moves().to_string(),
		}
	}
}

impl From<&GameState> for StatusMessage {
	fn from(state: &GameState) -> Self {
		Self {
			status: state.lifecycle.into(),
			legal_moves: state.legal_moves().to_string(),
		}
	}
}

impl From<GameLifecycle> for GameStatus {
    fn from(lifecycle: GameLifecycle) -> Self {
        match lifecycle {
            GameLifecycle::Waiting { .. } => Self::Waiting,
            GameLifecycle::Active { .. } => Self::Active,
            GameLifecycle::Ended => Self::Ended,
            GameLifecycle::Expired => Self::Expired,
        }
    }
}

impl Serialize for GameStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(match self {
            Self::Waiting => "waiting",
            Self::Active => "active",
            Self::Ended => "ended",
            Self::Expired => "expired",
            Self::__Invalid => unreachable!(),
        })
    }
}

impl fmt::Debug for GameStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Waiting => "Waiting",
            Self::Active => "Active",
            Self::Ended => "Ended",
            Self::Expired => "Expired",
            Self::__Invalid => "__Invalid",
        })
    }
}

impl From<MakeMoveError> for ErrorMessage {
    fn from(error: MakeMoveError) -> Self {
        match error {
            MakeMoveError::GameNotActive => Self::GameNotActive,
            MakeMoveError::IllegalMove => Self::IllegalMove,
            MakeMoveError::NotYourTurn => Self::NotYourTurn,
        }
    }
}

impl MoveMessage {
    pub(super) fn new(mve: Move, revision: u32, game: &Game) -> Self {
        Self {
            revision,
            mve: mve.to_string(),
            turn: game.turn,
            legal_moves: game.moves.to_string(),
        }
    }
}

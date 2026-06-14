use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    game::{Game, Player},
    game_state::{GameState, MakeMoveError},
    moves::Move,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen(getter_with_clone)]
pub struct SnapshotMessage {
    pub revision: u32,
    pub fen: String,
    pub status: String,
    #[wasm_bindgen(js_name = "legalMoves")]
    pub legal_moves: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct StatusMessage {
    status: String,
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
            status: state.lifecycle.to_string(),
            legal_moves: state.legal_moves().to_string(),
        }
    }
}

impl From<&GameState> for StatusMessage {
    fn from(state: &GameState) -> Self {
        Self {
            status: state.lifecycle.to_string(),
            legal_moves: state.legal_moves().to_string(),
        }
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

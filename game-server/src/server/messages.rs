use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::moves::Move;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen(getter_with_clone)]
pub struct SnapshotMessage {
    pub revision: u32,
    pub fen: String,
    #[wasm_bindgen(js_name = "legalMoves")]
    pub legal_moves: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct MoveMessage {
    revision: u32,
    #[serde(rename = "move")]
    mve: String,
    legal_moves: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum ErrorMessage {
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
    Move(MoveMessage),
    Error(ErrorMessage),
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "lowercase")]
pub(super) enum ClientMessage {
    Move(String),
}

impl MoveMessage {
    pub(super) fn new(mve: Move, revision: u32, legal_moves: String) -> Self {
        Self {
            revision,
            mve: mve.to_string(),
            legal_moves,
        }
    }
}

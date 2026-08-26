use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{moves::Move, state::Color};

use super::state::{GameEndReason, GameLifecycle, GameState, MakeMoveError};

#[wasm_bindgen(js_name = Color)]
pub enum WasmColor {
    White = "white",
    Black = "black",
}

#[wasm_bindgen(js_name = GameEndReason)]
pub enum WasmGameEndReason {
    Checkmate = "checkmate",
    Timeout = "timeout",
    Disconnect = "disconnect",
}

#[derive(Clone, Copy, Debug, Serialize)]
#[wasm_bindgen]
pub struct EndedStatus {
    winner: Color,
    reason: GameEndReason,
}

#[wasm_bindgen]
impl EndedStatus {
    #[wasm_bindgen(getter)]
    pub fn winner(&self) -> WasmColor {
        match self.winner {
            Color::White => WasmColor::White,
            Color::Black => WasmColor::Black,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn reason(&self) -> WasmGameEndReason {
        match self.reason {
            GameEndReason::Checkmate => WasmGameEndReason::Checkmate,
            GameEndReason::Timeout => WasmGameEndReason::Timeout,
            GameEndReason::Disconnect => WasmGameEndReason::Disconnect,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
#[wasm_bindgen]
pub enum GameStatus {
    Waiting = "waiting",
    Active = "active",
    Ended(EndedStatus),
    Expired = "expired",
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen(getter_with_clone)]
pub struct Clock {
    #[wasm_bindgen(js_name = "whiteRemainingMs")]
    pub white_remaining_ms: i32,
    #[wasm_bindgen(js_name = "blackRemainingMs")]
    pub black_remaining_ms: i32,
    pub running: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen(getter_with_clone)]
pub struct SnapshotMessage {
    pub revision: u32,
    pub fen: String,
    pub status: GameStatus,
    pub clock: Clock,
    #[wasm_bindgen(js_name = "legalMoves")]
    pub legal_moves: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct StatusMessage {
    status: GameStatus,
    clock: Clock,
    legal_moves: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct MoveMessage {
    revision: u32,
    #[serde(rename = "move")]
    mve: String,
    turn: Color,
    legal_moves: String,
    clock: Clock,
    status: GameStatus,
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

impl SnapshotMessage {
    pub(super) fn new(state: &GameState, now: i64) -> Self {
        Self {
            revision: state.revision,
            fen: state.game.fen(),
            status: state.lifecycle.into(),
            clock: Clock::new(state, now),
            legal_moves: state.legal_moves().to_string(),
        }
    }
}

impl StatusMessage {
    pub(super) fn new(state: &GameState, now: i64) -> Self {
        Self {
            status: state.lifecycle.into(),
            clock: Clock::new(state, now),
            legal_moves: state.legal_moves().to_string(),
        }
    }
}

impl Clock {
    pub(super) fn new(state: &GameState, now: i64) -> Self {
        if state.revision == 0 {
            return Self {
                white_remaining_ms: state.clock.white_remaining_ms,
                black_remaining_ms: state.clock.black_remaining_ms,
                running: false,
            };
        }

        let GameLifecycle::Active {
            turn_started_at, ..
        } = state.lifecycle
        else {
            return Self {
                white_remaining_ms: state.clock.white_remaining_ms,
                black_remaining_ms: state.clock.black_remaining_ms,
                running: false,
            };
        };

        debug_assert!(now >= turn_started_at);
        let elapsed_ms = (now - turn_started_at) as i32;
        match state.game.state.turn {
            Color::White => Self {
                white_remaining_ms: state
                    .clock
                    .white_remaining_ms
                    .saturating_sub(elapsed_ms)
                    .max(0),
                black_remaining_ms: state.clock.black_remaining_ms,
                running: true,
            },
            Color::Black => Self {
                white_remaining_ms: state.clock.white_remaining_ms,
                black_remaining_ms: state
                    .clock
                    .black_remaining_ms
                    .saturating_sub(elapsed_ms)
                    .max(0),
                running: true,
            },
        }
    }
}

impl From<GameLifecycle> for GameStatus {
    fn from(lifecycle: GameLifecycle) -> Self {
        match lifecycle {
            GameLifecycle::Waiting { .. } => Self::Waiting,
            GameLifecycle::Active { .. } => Self::Active,
            GameLifecycle::Ended { winner, reason } => Self::Ended(EndedStatus { winner, reason }),
            GameLifecycle::Expired => Self::Expired,
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
    pub(super) fn new(mve: Move, state: &GameState, now: i64) -> Self {
        Self {
            revision: state.revision,
            mve: mve.to_string(),
            turn: state.game.state.turn,
            legal_moves: state.legal_moves().to_string(),
            clock: Clock::new(state, now),
            status: state.lifecycle.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        board::Board,
        castling::CastlingRights,
        game::Game,
        server::state::{GameClock, GameLifecycle, GameTimeouts},
        state::State,
    };

    const NOW: i64 = 1_000;
    const TIME_CONTROL_MS: i32 = 1_000;
    const TEST_TIMEOUTS: GameTimeouts = GameTimeouts {
        join_timeout_ms: 100,
        first_move_timeout_ms: 200,
        disconnect_timeout_ms: 300,
    };
    const TEST_CLOCKS: GameClock = GameClock {
        white_remaining_ms: TIME_CONTROL_MS,
        black_remaining_ms: TIME_CONTROL_MS,
    };

    #[test]
    fn clock_does_not_count_down_before_first_move() {
        let state = GameState {
            game: Game::default(),
            revision: 0,
            timeouts: TEST_TIMEOUTS,
            clock: TEST_CLOCKS,
            lifecycle: GameLifecycle::Active {
                turn_started_at: NOW,
                white_disconnected_at: None,
                black_disconnected_at: None,
            },
        };

        assert_eq!(
            Clock::new(&state, NOW + 250),
            Clock {
                white_remaining_ms: TIME_CONTROL_MS,
                black_remaining_ms: TIME_CONTROL_MS,
                running: false,
            }
        );
    }

    #[test]
    fn black_clock_counts_down_after_white_moves() {
        let state = GameState {
            game: Game::new(
                Board::from_fen("rnbqkbnr/pppppppp/8/8/8/4P3/PPPP1PPP/RNBQKBNR").unwrap(),
                State::new(Color::Black, CastlingRights::NONE),
            ),
            revision: 1,
            timeouts: TEST_TIMEOUTS,
            clock: TEST_CLOCKS,
            lifecycle: GameLifecycle::Active {
                turn_started_at: NOW,
                white_disconnected_at: None,
                black_disconnected_at: None,
            },
        };

        assert_eq!(
            Clock::new(&state, NOW + 250),
            Clock {
                white_remaining_ms: TIME_CONTROL_MS,
                black_remaining_ms: TIME_CONTROL_MS - 250,
                running: true,
            }
        );
    }

    #[test]
    fn clock_clamps_active_player_to_zero() {
        let state = GameState {
            game: Game::new(
                Board::from_fen("rnbqkbnr/pppppppp/8/8/8/4P3/PPPP1PPP/RNBQKBNR").unwrap(),
                State::new(Color::Black, CastlingRights::NONE),
            ),
            revision: 1,
            timeouts: TEST_TIMEOUTS,
            clock: TEST_CLOCKS,
            lifecycle: GameLifecycle::Active {
                turn_started_at: NOW,
                white_disconnected_at: None,
                black_disconnected_at: None,
            },
        };

        assert_eq!(
            Clock::new(&state, NOW + i64::from(TIME_CONTROL_MS) + 1),
            Clock {
                white_remaining_ms: TIME_CONTROL_MS,
                black_remaining_ms: 0,
                running: true,
            }
        );
    }
}

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
    turn: Player,
    legal_moves: String,
    clock: Clock,
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
        match state.game.turn {
            Player::White => Self {
                white_remaining_ms: state
                    .clock
                    .white_remaining_ms
                    .saturating_sub(elapsed_ms)
                    .max(0),
                black_remaining_ms: state.clock.black_remaining_ms,
                running: true,
            },
            Player::Black => Self {
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
    pub(super) fn new(mve: Move, revision: u32, game: &Game, clock: Clock) -> Self {
        Self {
            revision,
            mve: mve.to_string(),
            turn: game.turn,
            legal_moves: game.moves.to_string(),
            clock,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        board::Board,
        game::Game,
        game_state::{GameClock, GameLifecycle, GameTimeouts},
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
                Player::Black,
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
                Player::Black,
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

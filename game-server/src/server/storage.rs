use anyhow::Context;
use serde::Deserialize;
use worker::{Date, Result, Storage};

use crate::{game::Game, state::Color};

use super::state::{
    DrawReason, GameClock, GameLifecycle, GameOutcome, GameState, GameTimeouts, WinReason,
};

pub(super) struct GameStorage {
    inner: Storage,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum GameStatus {
    Waiting,
    Active,
    Ended,
    Expired,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(untagged)]
enum EndReason {
    Won(WinReason),
    Drawn(DrawReason),
}

#[derive(Debug, Deserialize)]
struct SqlGameRow {
    revision: u32,
    status: GameStatus,
    winner: Option<Color>,
    end_reason: Option<EndReason>,
    fen: String,
    created_at: i64,
    white_disconnected_at: Option<i64>,
    black_disconnected_at: Option<i64>,
    turn_started_at: Option<i64>,
    join_timeout_ms: i32,
    first_move_timeout_ms: i32,
    disconnect_timeout_ms: i32,
    white_remaining_ms: i32,
    black_remaining_ms: i32,
}

impl GameStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Waiting => "waiting",
            Self::Active => "active",
            Self::Ended => "ended",
            Self::Expired => "expired",
        }
    }
}

impl GameStorage {
    pub(super) fn new(inner: Storage) -> Self {
        Self { inner }
    }

    pub(super) fn init(&self) -> Result<()> {
        self.inner.sql().exec(
            "CREATE TABLE IF NOT EXISTS game (\
                id INTEGER PRIMARY KEY CHECK (id = 1), \
                revision INTEGER NOT NULL, \
                status TEXT NOT NULL CHECK (status IN ('waiting', 'active', 'ended', 'expired')), \
                winner TEXT CHECK (winner IN ('white', 'black')), \
                end_reason TEXT CHECK (end_reason IN ('checkmate', 'stalemate', 'timeout', 'disconnect')), \
                fen TEXT NOT NULL, \
                created_at INTEGER NOT NULL, \
                white_disconnected_at INTEGER, \
                black_disconnected_at INTEGER, \
                turn_started_at INTEGER, \
                join_timeout_ms INTEGER NOT NULL, \
                first_move_timeout_ms INTEGER NOT NULL, \
                disconnect_timeout_ms INTEGER NOT NULL, \
                white_remaining_ms INTEGER NOT NULL, \
                black_remaining_ms INTEGER NOT NULL, \
                CHECK (\
                    (status = 'waiting' \
                        AND turn_started_at IS NULL \
                        AND white_disconnected_at IS NULL \
                        AND black_disconnected_at IS NULL \
                        AND winner IS NULL \
                        AND end_reason IS NULL) \
                    OR (status = 'active' \
                        AND turn_started_at IS NOT NULL \
                        AND winner IS NULL \
                        AND end_reason IS NULL) \
                    OR (status = 'ended' \
                        AND turn_started_at IS NULL \
                        AND white_disconnected_at IS NULL \
                        AND black_disconnected_at IS NULL \
                        AND end_reason IS NOT NULL \
                        AND ((end_reason = 'stalemate' AND winner IS NULL) \
                            OR (end_reason != 'stalemate' AND winner IS NOT NULL))) \
                    OR (status = 'expired' \
                        AND turn_started_at IS NULL \
                        AND white_disconnected_at IS NULL \
                        AND black_disconnected_at IS NULL \
                        AND winner IS NULL \
                        AND end_reason IS NULL)\
                )\
            );",
            None,
        )?;

        Ok(())
    }

    pub(super) fn load(&self) -> Result<Option<GameState>> {
        let rows = self
            .inner
            .sql()
            .exec(
                "SELECT \
                    revision, \
                    status, \
                    winner, \
                    end_reason, \
                    fen, \
                    created_at, \
                    white_disconnected_at, \
                    black_disconnected_at, \
                     turn_started_at, \
                     join_timeout_ms, \
                     first_move_timeout_ms, \
                     disconnect_timeout_ms, \
                     white_remaining_ms, \
                     black_remaining_ms \
                  FROM game WHERE id = 1;",
                None,
            )?
            .to_array::<SqlGameRow>()?;

        rows.into_iter().next().map(GameState::try_from).transpose()
    }

    pub(super) fn create_game(
        &self,
        game: Game,
        join_timeout_ms: i32,
        first_move_timeout_ms: i32,
        disconnect_timeout_ms: i32,
        white_remaining_ms: i32,
        black_remaining_ms: i32,
    ) -> Result<GameState> {
        if let Some(stored_game) = self.load()? {
            return Ok(stored_game);
        }

        let created_at = Date::now().as_millis() as i64;

        self.inner.sql().exec(
            "INSERT INTO game (\
                id, \
                revision, \
                status, \
                winner, \
                end_reason, \
                fen, \
                created_at, \
                join_timeout_ms, \
                first_move_timeout_ms, \
                disconnect_timeout_ms, \
                white_remaining_ms, \
                black_remaining_ms\
			) VALUES (1, 0, 'waiting', NULL, NULL, ?, ?, ?, ?, ?, ?, ?) \
             ON CONFLICT(id) DO NOTHING;",
            vec![
                game.fen().into(),
                created_at.into(),
                join_timeout_ms.into(),
                first_move_timeout_ms.into(),
                disconnect_timeout_ms.into(),
                white_remaining_ms.into(),
                black_remaining_ms.into(),
            ],
        )?;

        Ok(GameState {
            revision: 0,
            game,
            timeouts: GameTimeouts {
                join_timeout_ms,
                first_move_timeout_ms,
                disconnect_timeout_ms,
            },
            clock: GameClock {
                white_remaining_ms,
                black_remaining_ms,
            },
            lifecycle: GameLifecycle::Waiting { created_at },
        })
    }

    pub(super) fn save(&self, state: &GameState) -> Result<()> {
        self.inner.sql().exec(
            "UPDATE game SET \
                revision = ?, \
                status = ?, \
                winner = ?, \
                end_reason = ?, \
                fen = ?, \
                white_disconnected_at = ?, \
                black_disconnected_at = ?, \
                turn_started_at = ?, \
                white_remaining_ms = ?, \
                black_remaining_ms = ? \
             WHERE id = 1;",
            vec![
                (state.revision as i32).into(),
                GameStatus::from(state.lifecycle).as_str().into(),
                state.winner().map(Color::as_str).into(),
                state.end_reason().into(),
                state.game.fen().into(),
                state.white_disconnected_at().into(),
                state.black_disconnected_at().into(),
                state.turn_started_at().into(),
                state.clock.white_remaining_ms.into(),
                state.clock.black_remaining_ms.into(),
            ],
        )?;

        Ok(())
    }
}

impl EndReason {
    fn into_outcome(self, winner: Option<Color>) -> Result<GameOutcome> {
        match (self, winner) {
            (Self::Won(reason), Some(winner)) => Ok(GameOutcome::Won { winner, reason }),
            (Self::Drawn(reason), None) => Ok(GameOutcome::Drawn { reason }),
            _ => Err(worker::Error::RustError(
                "stored game outcome is invalid".to_string(),
            )),
        }
    }
}

impl TryFrom<SqlGameRow> for GameState {
    type Error = worker::Error;

    fn try_from(row: SqlGameRow) -> Result<Self> {
        let lifecycle = match row.status {
            GameStatus::Waiting => {
                if row.turn_started_at.is_some()
                    || row.white_disconnected_at.is_some()
                    || row.black_disconnected_at.is_some()
                {
                    return Err(worker::Error::RustError(
                        "waiting game row contains active connection fields".to_string(),
                    ));
                }

                GameLifecycle::Waiting {
                    created_at: row.created_at,
                }
            }
            GameStatus::Active => {
                let Some(turn_started_at) = row.turn_started_at else {
                    return Err(worker::Error::RustError(
                        "active game row must have turn_started_at".to_string(),
                    ));
                };

                GameLifecycle::Active {
                    turn_started_at,
                    white_disconnected_at: row.white_disconnected_at,
                    black_disconnected_at: row.black_disconnected_at,
                }
            }
            GameStatus::Ended => {
                let Some(reason) = row.end_reason else {
                    return Err(worker::Error::RustError(
                        "ended game row must have an end reason".to_string(),
                    ));
                };
                GameLifecycle::Ended(reason.into_outcome(row.winner)?)
            }
            GameStatus::Expired => GameLifecycle::Expired,
        };

        Ok(Self {
            revision: row.revision,
            game: Game::from_fen(&row.fen)
                .with_context(|| format!("failed to load stored game from FEN: {}", row.fen))
                .map_err(|error| worker::Error::RustError(error.to_string()))?,
            timeouts: GameTimeouts {
                join_timeout_ms: row.join_timeout_ms,
                first_move_timeout_ms: row.first_move_timeout_ms,
                disconnect_timeout_ms: row.disconnect_timeout_ms,
            },
            clock: GameClock {
                white_remaining_ms: row.white_remaining_ms,
                black_remaining_ms: row.black_remaining_ms,
            },
            lifecycle,
        })
    }
}

impl From<GameLifecycle> for GameStatus {
    fn from(lifecycle: GameLifecycle) -> Self {
        match lifecycle {
            GameLifecycle::Waiting { .. } => Self::Waiting,
            GameLifecycle::Active { .. } => Self::Active,
            GameLifecycle::Ended(_) => Self::Ended,
            GameLifecycle::Expired => Self::Expired,
        }
    }
}

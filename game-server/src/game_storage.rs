use anyhow::Context;
use serde::Deserialize;
use worker::{Date, Result, SqlStorageValue, Storage};

use crate::{
    game::Game,
    game_state::{GameClock, GameLifecycle, GameState, GameTimeouts},
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

#[derive(Debug, Deserialize)]
struct SqlGameRow {
    revision: u32,
    status: GameStatus,
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
    const fn as_str(self) -> &'static str {
        match self {
            Self::Waiting => "waiting",
            Self::Active => "active",
            Self::Ended => "ended",
            Self::Expired => "expired",
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
                fen TEXT NOT NULL, \
                created_at INTEGER NOT NULL, \
                white_disconnected_at INTEGER, \
                black_disconnected_at INTEGER, \
                turn_started_at INTEGER, \
                join_timeout_ms INTEGER NOT NULL, \
                first_move_timeout_ms INTEGER NOT NULL, \
                disconnect_timeout_ms INTEGER NOT NULL, \
                white_remaining_ms INTEGER NOT NULL, \
                black_remaining_ms INTEGER NOT NULL\
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
                fen, \
                created_at, \
                join_timeout_ms, \
                first_move_timeout_ms, \
                disconnect_timeout_ms, \
                white_remaining_ms, \
                black_remaining_ms\
             ) VALUES (1, 0, 'waiting', ?, ?, ?, ?, ?, ?, ?) \
             ON CONFLICT(id) DO NOTHING;",
            vec![
                SqlStorageValue::from(game.fen()),
                SqlStorageValue::from(created_at),
                SqlStorageValue::from(join_timeout_ms),
                SqlStorageValue::from(first_move_timeout_ms),
                SqlStorageValue::from(disconnect_timeout_ms),
                SqlStorageValue::from(white_remaining_ms),
                SqlStorageValue::from(black_remaining_ms),
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
                fen = ?, \
                white_disconnected_at = ?, \
                black_disconnected_at = ?, \
                turn_started_at = ?, \
                white_remaining_ms = ?, \
                black_remaining_ms = ? \
             WHERE id = 1;",
            vec![
                SqlStorageValue::from(state.revision as i32),
                SqlStorageValue::from(GameStatus::from(state.lifecycle).as_str()),
                SqlStorageValue::from(state.game.fen()),
                SqlStorageValue::from(state.white_disconnected_at()),
                SqlStorageValue::from(state.black_disconnected_at()),
                SqlStorageValue::from(state.turn_started_at()),
                SqlStorageValue::from(state.clock.white_remaining_ms),
                SqlStorageValue::from(state.clock.black_remaining_ms),
            ],
        )?;

        Ok(())
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
            GameStatus::Ended => GameLifecycle::Ended,
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

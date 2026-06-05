use anyhow::Context;
use serde::Deserialize;
use worker::{Result, SqlStorageValue, Storage as WorkerStorage};

use crate::game::Game;

pub(super) struct Storage {
    inner: WorkerStorage,
}

pub(super) struct StoredGame {
    pub(super) game: Game,
    pub(super) revision: u32,
}

#[derive(Debug, Deserialize)]
struct GameRow {
    fen: String,
    revision: u32,
}

impl Storage {
    pub(super) fn new(inner: WorkerStorage) -> Self {
        Self { inner }
    }

    pub(super) fn init(&self) -> Result<()> {
        self.inner.sql().exec(
            "CREATE TABLE IF NOT EXISTS game (\
                id INTEGER PRIMARY KEY CHECK (id = 1), \
                fen TEXT NOT NULL, \
                revision INTEGER NOT NULL\
            );",
            None,
        )?;

        Ok(())
    }

    pub(super) fn load(&self) -> Result<Option<StoredGame>> {
        let rows = self
            .inner
            .sql()
            .exec("SELECT fen, revision FROM game WHERE id = 1;", None)?
            .to_array::<GameRow>()?;

        rows.into_iter()
            .next()
            .map(StoredGame::try_from)
            .transpose()
    }

    pub(super) fn save(&self, game: &Game, revision: u32) -> Result<()> {
        self.inner.sql().exec(
            "INSERT INTO game (id, fen, revision) \
             VALUES (1, ?, ?) \
             ON CONFLICT(id) DO UPDATE SET \
                fen = excluded.fen, \
                revision = excluded.revision;",
            vec![
                SqlStorageValue::from(game.fen()),
                SqlStorageValue::from(revision as i32),
            ],
        )?;

        Ok(())
    }
}

impl TryFrom<GameRow> for StoredGame {
    type Error = worker::Error;

    fn try_from(row: GameRow) -> Result<Self> {
        Ok(Self {
            game: Game::from_fen(&row.fen)
                .with_context(|| format!("failed to load stored game from FEN: {}", row.fen))
                .map_err(|error| worker::Error::RustError(error.to_string()))?,
            revision: row.revision,
        })
    }
}

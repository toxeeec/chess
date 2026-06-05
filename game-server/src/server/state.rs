use std::fmt::Write;

use crate::{
    game::{Game, MakeMoveError, Player},
    moves::Move,
};

use super::messages::{ErrorMessage, SnapshotMessage};

pub(super) struct GameServerState {
    pub(super) game: Game,
    pub(super) revision: u32,
    cached_moves: String,
}

impl GameServerState {
    pub(super) fn new(game: Game, revision: u32) -> Self {
        let capacity = game.moves.len() * 5;
        let mut state = Self {
            game,
            revision,
            cached_moves: String::with_capacity(capacity),
        };
        state.sync_moves();

        state
    }

    pub(super) fn snapshot_message(&self) -> SnapshotMessage {
        SnapshotMessage {
            revision: self.revision,
            fen: self.game.fen(),
            legal_moves: self.legal_moves(),
        }
    }

    pub(super) fn legal_moves(&self) -> String {
        self.cached_moves.clone()
    }

    pub(super) fn make_move(&mut self, player: Player, mve: Move) -> Result<(), ErrorMessage> {
        match self.game.make_move(player, mve) {
            Ok(_) => {
                self.revision += 1;
                self.sync_moves();
                Ok(())
            }
            Err(error) => Err(match error {
                MakeMoveError::IllegalMove => ErrorMessage::IllegalMove,
                MakeMoveError::NotYourTurn => ErrorMessage::NotYourTurn,
            }),
        }
    }

    fn sync_moves(&mut self) {
        self.cached_moves.clear();

        let mut iter = self.game.moves.iter();
        if let Some(first) = iter.next() {
            write!(&mut self.cached_moves, "{}", first).unwrap();
            for mve in iter {
                write!(&mut self.cached_moves, " {}", mve).unwrap();
            }
        }
    }
}

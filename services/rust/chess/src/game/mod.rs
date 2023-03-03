use self::{board::Board, state::State};
use std::fmt::Debug;

mod board;
pub mod moves;
mod piece;
mod state;

#[derive(Default)]
pub struct Game {
    pub board: Board,
    pub state: State,
}

impl Debug for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.board, f)
    }
}

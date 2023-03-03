use self::board::Board;
use std::fmt::Debug;

mod board;
mod moves;
mod piece;
mod state;

#[derive(Default)]
pub struct Game {
    board: Board,
}

impl Debug for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.board, f)
    }
}

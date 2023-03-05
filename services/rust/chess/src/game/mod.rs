use self::{board::Board, moves::Move, state::State};
use std::fmt::Debug;

mod board;
pub mod moves;
mod piece;
mod state;

#[derive(Clone)]
pub struct Game {
    pub board: Board,
    pub state: State,
    pub moves: Vec<Move>,
    // TODO: add move counters
}

impl Game {
    pub fn make_move(&mut self, mov: Move) {
        if !self.moves.contains(&mov) {
            // TODO: return error
        }
        self.board.update(mov, self.state.white);
        self.state.update(mov);
        self.moves.clear();
        moves::generate(&mut self.moves, &self.board, self.state);
    }

    #[cfg(test)]
    fn perft(self, depth: u32) -> u32 {
        let mut nodes = 0;
        self.perft_inner(depth, &mut nodes);
        nodes
    }

    #[cfg(test)]
    fn perft_inner(self, depth: u32, nodes: &mut u32) {
        if depth == 1 {
            *nodes += self.moves.len() as u32;
            return;
        }
        for mov in &self.moves {
            let mut g = self.clone();
            g.make_move(*mov);
            g.perft_inner(depth - 1, nodes);
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        let board = Board::default();
        let state = State::default();
        let mut moves = Vec::with_capacity(32);
        moves::generate(&mut moves, &board, state);
        Self {
            board,
            state,
            moves,
        }
    }
}

impl Debug for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.board, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test]
    fn perft_tests() {
        assert_eq!(Game::default().perft(2), 400);
    }
}

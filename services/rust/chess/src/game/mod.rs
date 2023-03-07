use thiserror::Error;

use self::{board::Board, counter::Counter, moves::Move, state::State};
use std::fmt::Debug;

mod board;
mod counter;
mod fen;
pub mod moves;
mod piece;
mod state;

#[derive(Clone, PartialEq)]
pub struct Game {
    pub board: Board,
    pub state: State,
    pub counter: Counter,
    pub moves: Vec<Move>,
    pub result: Option<f32>,
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum MoveError {
    #[error("{0} is not a legal move")]
    Illegal(Move),
}

impl Game {
    pub fn make_move(&mut self, mov: Move) -> Result<(), MoveError> {
        if !self.moves.contains(&mov) {
            return Err(MoveError::Illegal(mov));
        }
        self.counter.update(mov, self.state.white, &self.board);
        self.board.update(mov, self.state.white);
        self.state.update(mov);
        self.moves.clear();
        let in_check = moves::generate(&mut self.moves, &self.board, self.state);
        self.set_result(in_check);
        Ok(())
    }

    pub fn set_result(&mut self, in_check: bool) {
        self.result = if self.moves.is_empty() {
            if in_check {
                Some(!self.state.white as u32 as f32)
            } else {
                Some(0.5)
            }
        } else {
            None
        }
    }

    #[cfg(test)]
    pub fn perft(self, depth: u32) -> u32 {
        let mut nodes = 0;
        self.perft_inner(depth, &mut nodes);
        nodes
    }

    #[cfg(debug_assertions)]
    pub fn divide(self, depth: u32) {
        let mut total = 0;
        for mov in &self.moves {
            let mut g = self.clone();
            let mut nodes = 0;
            g.make_move(*mov).unwrap();
            g.perft_inner(depth - 1, &mut nodes);
            total += nodes;
            println!("{}: {}", mov, nodes);
        }
        println!("Nodes: {}", total);
    }

    #[cfg(any(test, debug_assertions))]
    fn perft_inner(self, depth: u32, nodes: &mut u32) {
        if depth == 0 {
            *nodes += 1;
            return;
        }
        if depth == 1 {
            *nodes += self.moves.len() as u32;
            return;
        }
        for mov in &self.moves {
            let mut g = self.clone();
            g.make_move(*mov).unwrap();
            g.perft_inner(depth - 1, nodes);
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        let board = Board::default();
        let state = State::default();
        let counter = Counter::default();
        let mut moves = Vec::with_capacity(32);
        moves::generate(&mut moves, &board, state);
        Self {
            board,
            state,
            counter,
            moves,
            result: None,
        }
    }
}

impl Debug for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?}", self.board)?;
        writeln!(f, "{:?}", self.state)?;
        writeln!(f)?;
        for m in &self.moves {
            writeln!(f, "{:?}", m)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(Game::default(), 5 => 4865609; "startpost depth 5")]
    #[test_case("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -".parse().unwrap(), 5 => 193690690; "kiwi depth 5")]
    #[test_case("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ -".parse().unwrap(), 5 => 89941194; "promotion depth 5")]
    #[test_case("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -".parse().unwrap(), 5 => 674624; "pin depth 5")]
    #[test_case("8/8/8/kq1pP1K1/8/8/8/8 w - d6 0 1".parse().unwrap(), 1 => 9; "illegal ep")]
    fn perft_tests(game: Game, depth: u32) -> u32 {
        game.perft(depth)
    }

    #[test_case(Game::default() => None)]
    #[test_case("7k/6Q1/5K2/8/8/8/8/8 b - -".parse().unwrap()=> Some(1.0))]
    #[test_case("8/8/8/8/8/5k2/6q1/7K w - - 0 1".parse().unwrap() => Some(0.0))]
    #[test_case("6k1/8/5Q1K/8/8/8/8/8 b - - 0 1".parse().unwrap() => Some(0.5))]
    fn set_result_tests(game: Game) -> Option<f32> {
        game.result
    }
}

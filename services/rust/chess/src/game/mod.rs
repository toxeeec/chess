use bitboard::square::ParseSquareError;
use thiserror::Error;

use self::{
    board::Board,
    counter::Counter,
    moves::{list::List, Move},
    piece::ParsePieceError,
    state::State,
};
use std::{collections::HashMap, fmt::Debug};

mod board;
mod counter;
mod fen;
pub mod moves;
mod piece;
mod state;

#[derive(Clone)]
pub struct Game {
    pub board: Board,
    pub state: State,
    pub counter: Counter,
    pub result: Option<f64>,
    pub list: List,
    pub positions: HashMap<(Board, State), u8>,
    // TODO: add timer
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum MoveError {
    #[error("invalid move format")]
    Format,
    #[error("invalid square")]
    Square(#[from] ParseSquareError),
    #[error("invalid promotion piece")]
    PromotionPiece(#[from] ParsePieceError),
    #[error("{0} is not a legal move")]
    Illegal(String),
    #[error("the game has already finished")]
    Finished,
}

impl Game {
    pub fn new(board: Board, state: State, counter: Counter) -> Self {
        let mut game = Self {
            board,
            state,
            counter,
            list: List::default(),
            result: None,
            positions: HashMap::with_capacity(16),
        };
        game.positions.insert((board, state), 1);
        let in_check = moves::generate(&mut game);
        game.set_result::<false>(in_check);
        game
    }

    pub fn make_move(&mut self, move_str: &str) -> Result<(), MoveError> {
        let mov = self.list.find(move_str)?;
        self.make_move_inner::<false>(mov)
    }

    fn make_move_inner<const IS_PERFT: bool>(&mut self, mov: Move) -> Result<(), MoveError> {
        if self.result.is_some() {
            return Err(MoveError::Finished);
        }
        let irreversible = mov.is_irreversible(self.state.white, &self.board);
        self.counter.update(irreversible, self.state.white);
        self.board.update(mov, self.state.white);
        self.state.update(mov);
        if irreversible {
            self.positions.clear();
        }
        self.positions
            .entry((self.board, self.state))
            .and_modify(|x| *x += 1)
            .or_insert(1);
        let in_check = moves::generate(self);
        self.set_result::<IS_PERFT>(in_check);
        Ok(())
    }

    fn set_result<const IS_PERFT: bool>(&mut self, in_check: bool) {
        if self.list.0.is_empty() {
            if in_check {
                self.result = Some(!self.state.white as i32 as f64);
            } else {
                self.result = Some(0.5);
            }
            return;
        }
        if IS_PERFT {
            return;
        }
        if self.counter.half >= 100 {
            self.result = Some(0.5);
            return;
        }
        if !self.board.has_sufficient_material(true) && !self.board.has_sufficient_material(false) {
            self.result = Some(0.5);
            return;
        }
        if *self.positions.get(&(self.board, self.state)).unwrap_or(&0) >= 3 {
            self.result = Some(0.5);
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
        for mov in &self.list.0 {
            let mut g = self.clone();
            let mut nodes = 0;
            g.make_move_inner::<true>(*mov).unwrap();
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
            *nodes += self.list.0.len() as u32;
            return;
        }
        for mov in &self.list.0 {
            let mut g = self.clone();
            g.make_move_inner::<true>(*mov).unwrap();
            g.perft_inner(depth - 1, nodes);
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        let board = Board::default();
        let state = State::default();
        let counter = Counter::default();
        Self::new(board, state, counter)
    }
}

impl Debug for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?}", self.board)?;
        writeln!(f, "{:?}", self.state)?;
        writeln!(f, "{:?}", self.counter)?;
        writeln!(f, "Result: {:?}\n", self.result)?;
        for m in &self.list.0 {
            writeln!(f, "{:?}", m)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::moves::Type;
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
    #[test_case("7k/6Q1/5K2/8/8/8/8/8 b - -".parse().unwrap()=> Some(1.0); "white won")]
    #[test_case("8/8/8/8/8/5k2/6q1/7K w - - 0 1".parse().unwrap() => Some(0.0); "black won")]
    #[test_case("6k1/8/5Q1K/8/8/8/8/8 b - - 0 1".parse().unwrap() => Some(0.5); "stalemate")]
    #[test_case("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 100 1".parse().unwrap() => Some(0.5); "halfmove clock")]
    fn set_result_tests(game: Game) -> Option<f64> {
        game.result
    }

    #[test]
    fn repetition_test() {
        let mut game = Game::default();
        let _ = game.make_move_inner::<false>(Move::new(6.into(), 21.into(), Type::Quiet));
        let _ = game.make_move_inner::<false>(Move::new(62.into(), 45.into(), Type::Quiet));
        let _ = game.make_move_inner::<false>(Move::new(21.into(), 6.into(), Type::Quiet));
        let _ = game.make_move_inner::<false>(Move::new(45.into(), 62.into(), Type::Quiet));
        let _ = game.make_move_inner::<false>(Move::new(6.into(), 21.into(), Type::Quiet));
        let _ = game.make_move_inner::<false>(Move::new(62.into(), 45.into(), Type::Quiet));
        let _ = game.make_move_inner::<false>(Move::new(21.into(), 6.into(), Type::Quiet));
        let _ = game.make_move_inner::<false>(Move::new(45.into(), 62.into(), Type::Quiet));
        assert_eq!(game.result, Some(0.5));
    }
}

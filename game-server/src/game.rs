use crate::{board::Board, moves::Move, pawn::add_pawn_moves};

pub(super) struct Game {
    pub(super) board: Board,
    pub(super) moves: Vec<Move>,
}

impl Default for Game {
    fn default() -> Self {
        let board = Board::default();
        let mut moves = Vec::new();
        add_pawn_moves(&board, &mut moves);

        Self { board, moves }
    }
}

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::marker::ConstParamTy;

use crate::{
    bishop::add_bishop_moves,
    board::Board,
    king::add_king_moves,
    knight::add_knight_moves,
    moves::{Move, MoveList},
    pawn::add_pawn_moves,
    queen::add_queen_moves,
    rook::add_rook_moves,
};

#[derive(Clone, Copy, ConstParamTy, Debug, Deserialize, Eq, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub(super) enum Color {
    White,
    Black,
}

impl Color {
    pub(super) const fn opponent(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }

    pub(super) const fn fen_value(self) -> &'static str {
        match self {
            Self::White => "w",
            Self::Black => "b",
        }
    }

    pub(super) fn from_fen_value(value: &str) -> Result<Self> {
        match value {
            "w" => Ok(Self::White),
            "b" => Ok(Self::Black),
            _ => bail!("invalid FEN active color: {value}"),
        }
    }
}

pub(super) enum MakeMoveError {
    IllegalMove,
    NotYourTurn,
}

pub(super) struct Game {
    pub(super) board: Board,
    pub(super) turn: Color,
    pub(super) moves: MoveList,
}

impl Default for Game {
    fn default() -> Self {
        Self::new(Board::default(), Color::White)
    }
}

impl Game {
    pub(super) fn new(board: Board, turn: Color) -> Self {
        let mut game = Self {
            board,
            turn,
            moves: MoveList::default(),
        };
        game.add_moves();

        game
    }

    pub(super) fn from_fen(fen: &str) -> Result<Self> {
        let mut fields = fen.split_whitespace();
        let placement = fields.next().context("FEN must contain piece placement")?;
        let active_color = fields.next().context("FEN must contain active color")?;
        Ok(Self::new(
            Board::from_fen(placement)?,
            Color::from_fen_value(active_color)?,
        ))
    }

    pub(super) fn fen(&self) -> String {
        format!("{} {} - - 0 1", self.board.fen(), self.turn.fen_value())
    }

    pub(super) fn make_move(&mut self, color: Color, mve: Move) -> Result<(), MakeMoveError> {
        if color != self.turn {
            return Err(MakeMoveError::NotYourTurn);
        }

        if !self.moves.contains(mve) {
            return Err(MakeMoveError::IllegalMove);
        }

        self.board.make_move(mve);
        self.turn = self.turn.opponent();

        self.moves.clear();
        self.add_moves();

        Ok(())
    }

    fn add_moves(&mut self) {
        let occ = self.board.occupied();
        let empty = !occ;

        match self.turn {
            Color::White => {
                let blockers = self.board.occupancy::<{ Color::White }>();
                add_pawn_moves::<{ Color::White }>(&self.board, empty, &mut self.moves);
                add_knight_moves::<{ Color::White }>(&self.board, blockers, &mut self.moves);
                add_rook_moves::<{ Color::White }>(&self.board, occ, blockers, &mut self.moves);
                add_bishop_moves::<{ Color::White }>(&self.board, occ, blockers, &mut self.moves);
                add_queen_moves::<{ Color::White }>(&self.board, occ, blockers, &mut self.moves);
                add_king_moves::<{ Color::White }>(&self.board, blockers, &mut self.moves);
            }
            Color::Black => {
                let blockers = self.board.occupancy::<{ Color::Black }>();
                add_pawn_moves::<{ Color::Black }>(&self.board, empty, &mut self.moves);
                add_knight_moves::<{ Color::Black }>(&self.board, blockers, &mut self.moves);
                add_rook_moves::<{ Color::Black }>(&self.board, occ, blockers, &mut self.moves);
                add_bishop_moves::<{ Color::Black }>(&self.board, occ, blockers, &mut self.moves);
                add_queen_moves::<{ Color::Black }>(&self.board, occ, blockers, &mut self.moves);
                add_king_moves::<{ Color::Black }>(&self.board, blockers, &mut self.moves);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Color, Game};

    #[test]
    fn parses_white_and_black_active_color() {
        let white = Game::from_fen("7k/8/8/8/8/8/4P3/4K3 w - - 0 1").unwrap();
        let black = Game::from_fen("7k/3p4/8/8/8/8/8/4K3 b - - 0 1").unwrap();

        assert_eq!(white.turn, Color::White);
        assert_eq!(white.fen(), "7k/8/8/8/8/8/4P3/4K3 w - - 0 1");
        assert_eq!(black.turn, Color::Black);
        assert_eq!(black.fen(), "7k/3p4/8/8/8/8/8/4K3 b - - 0 1");
    }

    #[test]
    fn rejects_invalid_fen() {
        for fen in [
            "",
            "8/8/8/8/8/8/8/8",
            "8/8/8/8/8/8/8/8 x - - 0 1",
            "8/8/8/8/8/8/8 w - - 0 1",
        ] {
            assert!(Game::from_fen(fen).is_err(), "{fen} should be invalid");
        }
    }

    #[test]
    fn legal_move_updates_board_turn_and_move_count() {
        let mut game = Game::default();

        assert_eq!(game.moves.len(), 20);
        assert!(
            game.make_move(Color::White, "e2e3".parse().unwrap())
                .is_ok()
        );

        assert_eq!(
            game.fen(),
            "rnbqkbnr/pppppppp/8/8/8/4P3/PPPP1PPP/RNBQKBNR b - - 0 1"
        );
        assert_eq!(game.turn, Color::Black);
        assert_eq!(game.moves.len(), 20);
    }

    #[test]
    fn rejects_wrong_turn_without_changing_move_count() {
        let mut game = Game::default();
        let move_count = game.moves.len();

        assert!(
            game.make_move(Color::Black, "a7a6".parse().unwrap())
                .is_err()
        );

        assert_eq!(game.turn, Color::White);
        assert_eq!(game.moves.len(), move_count);
    }

    #[test]
    fn rejects_illegal_move_without_changing_move_count() {
        let mut game = Game::default();
        let move_count = game.moves.len();

        assert!(
            game.make_move(Color::White, "e2e5".parse().unwrap())
                .is_err()
        );

        assert_eq!(game.turn, Color::White);
        assert_eq!(game.moves.len(), move_count);
    }
}

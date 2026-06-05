use crate::{board::Board, moves::Move, pawn::add_pawn_moves};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub(super) enum Player {
    White,
    Black,
}

impl Player {
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
    pub(super) turn: Player,
    pub(super) moves: Vec<Move>,
}

impl Default for Game {
    fn default() -> Self {
        Self::new(Board::default(), Player::White)
    }
}

impl Game {
    pub(super) fn new(board: Board, turn: Player) -> Self {
        let mut game = Self {
            board,
            turn,
            moves: Vec::with_capacity(32),
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
            Player::from_fen_value(active_color)?,
        ))
    }

    pub(super) fn fen(&self) -> String {
        format!("{} {} - - 0 1", self.board.fen(), self.turn.fen_value())
    }

    pub(super) fn make_move(&mut self, player: Player, mve: Move) -> Result<(), MakeMoveError> {
        if player != self.turn {
            return Err(MakeMoveError::NotYourTurn);
        }

        if !self.moves.contains(&mve) {
            return Err(MakeMoveError::IllegalMove);
        }

        self.board.make_move(mve);
        self.turn = self.turn.opponent();

        self.moves.clear();
        self.add_moves();

        Ok(())
    }

    fn add_moves(&mut self) {
        match self.turn {
            Player::White => {
                add_pawn_moves::<true>(&self.board, &mut self.moves);
            }
            Player::Black => {
                add_pawn_moves::<false>(&self.board, &mut self.moves);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Game, Player};
    use crate::moves::Move;
    use std::str::FromStr;

    #[test]
    fn parses_white_and_black_active_color() {
        let white = Game::from_fen("8/8/8/8/8/8/4P3/8 w - - 0 1").unwrap();
        let black = Game::from_fen("8/3p4/8/8/8/8/8/8 b - - 0 1").unwrap();

        assert_eq!(white.turn, Player::White);
        assert_eq!(white.fen(), "8/8/8/8/8/8/4P3/8 w - - 0 1");
        assert_eq!(black.turn, Player::Black);
        assert_eq!(black.fen(), "8/3p4/8/8/8/8/8/8 b - - 0 1");
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

        assert_eq!(game.moves.len(), 16);
        assert!(
            game.make_move(Player::White, Move::from_str("e2e3").unwrap())
                .is_ok()
        );

        assert_eq!(
            game.fen(),
            "rnbqkbnr/pppppppp/8/8/8/4P3/PPPP1PPP/RNBQKBNR b - - 0 1"
        );
        assert_eq!(game.turn, Player::Black);
        assert_eq!(game.moves.len(), 16);
    }

    #[test]
    fn rejects_wrong_turn_without_changing_move_count() {
        let mut game = Game::default();
        let move_count = game.moves.len();

        assert!(
            game.make_move(Player::Black, Move::from_str("a7a6").unwrap())
                .is_err()
        );

        assert_eq!(game.turn, Player::White);
        assert_eq!(game.moves.len(), move_count);
    }

    #[test]
    fn rejects_illegal_move_without_changing_move_count() {
        let mut game = Game::default();
        let move_count = game.moves.len();

        assert!(
            game.make_move(Player::White, Move::from_str("e2e5").unwrap())
                .is_err()
        );

        assert_eq!(game.turn, Player::White);
        assert_eq!(game.moves.len(), move_count);
    }
}

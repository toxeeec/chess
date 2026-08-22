use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::{marker::ConstParamTy, ops::Not};

use crate::{
    attacks::{KingThreats, evasion_mask, king_threats},
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

const impl Not for Color {
    type Output = Self;

    fn not(self) -> Self::Output {
        self.opponent()
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

        self.board.make_move(self.turn, mve);
        self.turn = self.turn.opponent();

        self.moves.clear();
        self.add_moves();

        Ok(())
    }

    fn add_moves(&mut self) {
        match self.turn {
            Color::White => self.add_moves_for::<{ Color::White }>(),
            Color::Black => self.add_moves_for::<{ Color::Black }>(),
        }
    }

    fn add_moves_for<const COLOR: Color>(&mut self)
    where
        [(); { !COLOR } as usize]:,
        [(); { !(!COLOR) } as usize]:,
    {
        let blockers = self.board.occupancy::<COLOR>();
        let enemy = self.board.occupancy::<{ !COLOR }>();
        let occupied = blockers | enemy;
        let empty = !occupied;
        let KingThreats {
            attackers,
            forbidden,
            pin_rays,
        } = king_threats::<{ !COLOR }>(&self.board, occupied);

        let evasion_mask = evasion_mask(self.board.king_square::<COLOR>(), attackers);

        if !evasion_mask.empty() {
            add_pawn_moves::<COLOR>(
                &self.board,
                empty,
                enemy,
                evasion_mask,
                pin_rays,
                &mut self.moves,
            );
            add_knight_moves::<COLOR>(
                &self.board,
                blockers,
                evasion_mask,
                pin_rays,
                &mut self.moves,
            );
            add_bishop_moves::<COLOR>(
                &self.board,
                occupied,
                blockers,
                evasion_mask,
                pin_rays,
                &mut self.moves,
            );
            add_rook_moves::<COLOR>(
                &self.board,
                occupied,
                blockers,
                evasion_mask,
                pin_rays,
                &mut self.moves,
            );
            add_queen_moves::<COLOR>(
                &self.board,
                occupied,
                blockers,
                evasion_mask,
                pin_rays,
                &mut self.moves,
            );
        }

        add_king_moves::<COLOR>(&self.board, blockers, forbidden, &mut self.moves);
    }
}

#[cfg(test)]
mod tests {
    use crate::{square, test_utils::board};

    use super::{Color, Game};

    fn has_move(game: &Game, mve: &str) -> bool {
        game.moves.contains(mve.parse().unwrap())
    }

    #[test]
    fn parses_white_and_black_active_color() {
        let white = Game::from_fen("7k/8/8/8/8/8/4P3/4K3 w - - 0 1").unwrap();
        assert_eq!(white.turn, Color::White);
        assert_eq!(white.fen(), "7k/8/8/8/8/8/4P3/4K3 w - - 0 1");

        let black = Game::from_fen("7k/3p4/8/8/8/8/8/4K3 b - - 0 1").unwrap();
        assert_eq!(black.turn, Color::Black);
        assert_eq!(black.fen(), "7k/3p4/8/8/8/8/8/4K3 b - - 0 1");
    }

    #[test]
    fn rejects_invalid_fen() {
        for fen in [
            "",
            "8/8/8/8/8/8/8/8",
            "7k/8/8/8/8/8/8/4K3 x - - 0 1",
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
    fn non_sliding_check_allows_only_checker_captures() {
        let game = Game::new(
            board!(
                k . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . n . .
                P . . . . . B .
                . . . R K . . .
            ),
            Color::White,
        );

        assert!(has_move(&game, "g2f3"));
        assert!(!has_move(&game, "a2a3"));
        assert!(!has_move(&game, "d1d2"));
    }

    #[test]
    fn sliding_check_allows_checker_captures_and_blocks() {
        let game = Game::new(
            board!(
                . . . . r . . .
                . . . . . . . k
                . . . . . . . .
                . B . . . . . .
                . . . . . . . .
                . . . . . . . .
                P . . R . . . .
                . . . . K . . .
            ),
            Color::White,
        );

        assert!(has_move(&game, "b5e8"));
        assert!(has_move(&game, "d2e2"));
        assert!(!has_move(&game, "a2a3"));
    }

    #[test]
    fn double_check_generates_only_king_moves() {
        let game = Game::new(
            board!(
                . . . . r . . k
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . b . . . . . .
                . . . . . . . .
                . . . . . . . .
                Q . . . K . . .
            ),
            Color::White,
        );

        assert!(game.moves.len() > 0);
        assert!(game.moves.iter().all(|mve| mve.from == square!(e1)));
    }

    #[test]
    fn checked_king_cannot_retreat_on_ray_or_capture_defended_piece() {
        let ray = Game::new(
            board!(
                . . . . r . . k
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . K . . .
                . . . . . . . .
            ),
            Color::White,
        );
        assert!(!has_move(&ray, "e2e1"));

        let defended = Game::new(
            board!(
                . . . . . . . k
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                r . . . r . . .
                . . . . K . . .
            ),
            Color::White,
        );
        assert!(!has_move(&defended, "e1e2"));
    }

    #[test]
    fn pin_detection_restricts_moves_to_the_pin_ray() {
        let game = Game::new(
            board!(
                . . . . r . . k
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . R . . .
                . . . . K . . .
            ),
            Color::White,
        );
        assert!(has_move(&game, "e2e8"));
        assert!(has_move(&game, "e2e3"));
        assert!(!has_move(&game, "e2d2"));
        assert!(!has_move(&game, "e2f2"));
    }

    #[test]
    fn pawn_capture_removes_captured_piece() {
        let mut game = Game::new(
            board!(
                . . . . . . . k
                . . . . . . . .
                . . . . . . . .
                . . . p . . . .
                . . . . P . . .
                . . . . . . . .
                . . . . . . . .
                K . . . . . . .
            ),
            Color::White,
        );

        assert!(
            game.make_move(Color::White, "e4d5".parse().unwrap())
                .is_ok()
        );
        assert_eq!(game.fen(), "7k/8/8/3P4/8/8/8/K7 b - - 0 1");
    }

    #[test]
    fn promotes_pawns_to_the_selected_piece() {
        for (fen, mve, expected) in [
            (
                "7k/P7/8/8/8/8/8/K7 w - - 0 1",
                "a7a8q",
                "Q6k/8/8/8/8/8/8/K7 b - - 0 1",
            ),
            (
                "1r5k/P7/8/8/8/8/8/K7 w - - 0 1",
                "a7b8n",
                "1N5k/8/8/8/8/8/8/K7 b - - 0 1",
            ),
            (
                "7k/8/8/8/8/8/p7/7K b - - 0 1",
                "a2a1r",
                "7k/8/8/8/8/8/8/r6K w - - 0 1",
            ),
            (
                "7k/8/8/8/8/8/1p6/B6K b - - 0 1",
                "b2a1b",
                "7k/8/8/8/8/8/8/b6K w - - 0 1",
            ),
        ] {
            let mut game = Game::from_fen(fen).unwrap();

            assert!(game.make_move(game.turn, mve.parse().unwrap()).is_ok());
            assert_eq!(game.fen(), expected);
        }
    }

    #[test]
    fn promotion_requires_a_piece_choice() {
        let mut game = Game::from_fen("7k/P7/8/8/8/8/8/K7 w - - 0 1").unwrap();

        assert!(
            game.make_move(Color::White, "a7a8".parse().unwrap())
                .is_err()
        );
        assert_eq!(game.fen(), "7k/P7/8/8/8/8/8/K7 w - - 0 1");
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

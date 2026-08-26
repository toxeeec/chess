use anyhow::{Context, Result};

use crate::{
    attacks::{KingThreats, evasion_mask, king_threats},
    bishop::add_bishop_moves,
    board::Board,
    castling::CastlingRights,
    king::add_king_moves,
    knight::add_knight_moves,
    moves::{Move, MoveList},
    pawn::add_pawn_moves,
    queen::add_queen_moves,
    rook::add_rook_moves,
    state::{Color, EnPassant, State},
};

pub(super) enum MakeMoveError {
    IllegalMove,
    NotYourTurn,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum GameResult {
    Win { winner: Color },
}

pub(super) struct Game {
    pub(super) board: Board,
    pub(super) state: State,
    pub(super) moves: MoveList,
}

impl Default for Game {
    fn default() -> Self {
        Self::new(
            Board::default(),
            State::new(Color::White, CastlingRights::ALL),
        )
    }
}

impl Game {
    pub(super) fn new(board: Board, state: State) -> Self {
        let mut game = Self {
            board,
            state,
            moves: MoveList::default(),
        };
        game.generate_legal_moves();

        game
    }

    pub(super) fn from_fen(fen: &str) -> Result<Self> {
        let mut fields = fen.split_whitespace();
        let placement = fields.next().context("FEN must contain piece placement")?;

        let board = Board::from_fen(placement)?;
        let state = State::from_fen(&mut fields, &board)?;

        Ok(Self::new(board, state))
    }

    pub(super) fn fen(&self) -> String {
        format!("{} {}", self.board.fen(), self.state.fen())
    }

    pub(super) fn make_move(
        &mut self,
        color: Color,
        mve: Move,
    ) -> Result<Option<GameResult>, MakeMoveError> {
        if color != self.state.turn {
            return Err(MakeMoveError::NotYourTurn);
        }

        if !self.moves.contains(mve) {
            return Err(MakeMoveError::IllegalMove);
        }

        self.state.castling_rights.update(mve.from, mve.to);
        let is_pawn = (self.board.pawns::<{ Color::White }>()
            | self.board.pawns::<{ Color::Black }>())
        .contains(mve.from);
        let en_passant = if is_pawn && mve.from.rank().abs_diff(mve.to.rank()) == 2 {
            EnPassant::new(match self.state.turn {
                Color::White => mve.to.backward::<{ Color::White }, 1>(),
                Color::Black => mve.to.backward::<{ Color::Black }, 1>(),
            })
        } else {
            EnPassant::NONE
        };

        self.board.apply_move(self.state.turn, mve);
        self.state.en_passant = en_passant;
        self.state.turn = self.state.turn.opponent();

        self.moves.clear();
        let result = self.generate_legal_moves();
        Ok(result)
    }

    fn generate_legal_moves(&mut self) -> Option<GameResult> {
        match self.state.turn {
            Color::White => self.generate_legal_moves_for::<{ Color::White }>(),
            Color::Black => self.generate_legal_moves_for::<{ Color::Black }>(),
        }
    }

    fn generate_legal_moves_for<const COLOR: Color>(&mut self) -> Option<GameResult>
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
                self.state.en_passant,
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

        add_king_moves::<COLOR>(
            &self.board,
            occupied,
            blockers,
            attackers,
            forbidden,
            self.state.castling_rights,
            &mut self.moves,
        );

        (self.moves.is_empty() && !attackers.empty()).then_some(GameResult::Win { winner: !COLOR })
    }
}

#[cfg(test)]
mod tests {
    use crate::{square, test_utils::board};

    use super::{CastlingRights, Color, EnPassant, Game, GameResult, State};

    fn has_move(game: &Game, mve: &str) -> bool {
        game.moves.contains(mve.parse().unwrap())
    }

    #[test]
    fn parses_white_and_black_active_color() {
        let white = Game::from_fen("7k/8/8/8/8/8/4P3/4K3 w - - 0 1").unwrap();
        assert_eq!(white.state.turn, Color::White);
        assert_eq!(white.fen(), "7k/8/8/8/8/8/4P3/4K3 w - - 0 1");

        let black = Game::from_fen("7k/3p4/8/8/8/8/8/4K3 b - - 0 1").unwrap();
        assert_eq!(black.state.turn, Color::Black);
        assert_eq!(black.fen(), "7k/3p4/8/8/8/8/8/4K3 b - - 0 1");
    }

    #[test]
    fn parses_and_serializes_castling_rights() {
        for rights in ["-", "K", "Q", "k", "q", "KQkq", "qK"] {
            let game =
                Game::from_fen(&format!("r3k2r/8/8/8/8/8/8/R3K2R w {rights} - 0 1")).unwrap();
            let expected = if rights == "qK" { "Kq" } else { rights };

            assert_eq!(
                game.fen(),
                format!("r3k2r/8/8/8/8/8/8/R3K2R w {expected} - 0 1")
            );
        }
    }

    #[test]
    fn rejects_invalid_fen() {
        for fen in [
            "",
            "8/8/8/8/8/8/8/8",
            "7k/8/8/8/8/8/8/4K3 x - - 0 1",
            "8/8/8/8/8/8/8 w - - 0 1",
            "7k/8/8/8/8/8/8/4K3 w X - 0 1",
            "7k/8/8/8/8/8/8/4K3 w KK - 0 1",
            "7k/8/8/8/8/8/8/3K3R w K - 0 1",
            "r2k4/8/8/8/8/8/8/7K b q - 0 1",
            "7k/8/8/8/8/8/8/4K3 w K - 0 1",
            "7k/8/8/8/8/8/8/4K2R w Q - 0 1",
            "4k3/8/8/8/8/8/8/7K b k - 0 1",
            "4k2r/8/8/8/8/8/8/7K b q - 0 1",
            "7k/8/8/3p4/8/8/8/4K3 w - d3 0 1",
            "7k/8/3P4/3p4/8/8/8/4K3 w - d6 0 1",
            "7k/3n4/8/3p4/8/8/8/4K3 w - d6 0 1",
            "7k/8/8/8/8/8/3P4/4K3 b - d3 0 1",
            "7k/8/8/8/8/8/8/4K3 w - i6 0 1",
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
            "rnbqkbnr/pppppppp/8/8/8/4P3/PPPP1PPP/RNBQKBNR b KQkq - 0 1"
        );
        assert_eq!(game.state.turn, Color::Black);
        assert_eq!(game.moves.len(), 20);
    }

    #[test]
    fn detects_checkmate_after_a_move() {
        let mut game = Game::default();
        for (color, mve, winner) in [
            (Color::White, "f2f3", None),
            (Color::Black, "e7e5", None),
            (Color::White, "g2g4", None),
            (
                Color::Black,
                "d8h4",
                Some(GameResult::Win {
                    winner: Color::Black,
                }),
            ),
        ] {
            assert!(matches!(
                game.make_move(color, mve.parse().unwrap()),
                Ok(result) if result == winner
            ));
        }
    }

    #[test]
    fn castling_moves_the_rook_and_revokes_the_rights() {
        let mut game = Game::from_fen("4k3/8/8/8/8/8/8/R3K2R w KQ - 0 1").unwrap();

        assert!(
            game.make_move(Color::White, "e1g1".parse().unwrap())
                .is_ok()
        );
        assert_eq!(game.fen(), "4k3/8/8/8/8/8/8/R4RK1 b - - 0 1");
    }

    #[test]
    fn king_rook_moves_and_home_rook_captures_revoke_rights() {
        let mut rook = Game::from_fen("4k3/8/8/8/8/8/8/R3K2R w KQ - 0 1").unwrap();
        assert!(
            rook.make_move(Color::White, "h1h2".parse().unwrap())
                .is_ok()
        );
        assert_eq!(rook.fen(), "4k3/8/8/8/8/8/7R/R3K3 b Q - 0 1");

        let mut king = Game::from_fen("4k3/8/8/8/8/8/8/R3K2R w KQ - 0 1").unwrap();
        assert!(
            king.make_move(Color::White, "e1e2".parse().unwrap())
                .is_ok()
        );
        assert_eq!(king.fen(), "4k3/8/8/8/8/8/4K3/R6R b - - 0 1");

        let mut capture = Game::from_fen("r3k3/8/8/8/8/8/8/R3K3 b Qq - 0 1").unwrap();
        assert!(
            capture
                .make_move(Color::Black, "a8a1".parse().unwrap())
                .is_ok()
        );
        assert_eq!(capture.fen(), "4k3/8/8/8/8/8/8/r3K3 w - - 0 1");
    }

    #[test]
    fn non_sliding_check_allows_only_checker_captures() {
        let game = Game::new(
            board!(
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . n . .
                P . . . . . B .
                . . . R K . . .
            ),
            State::new(Color::White, CastlingRights::NONE),
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
                . . . . . . . .
                . . . . . . . .
                . B . . . . . .
                . . . . . . . .
                . . . . . . . .
                P . . R . . . .
                . . . . K . . .
            ),
            State::new(Color::White, CastlingRights::NONE),
        );

        assert!(has_move(&game, "b5e8"));
        assert!(has_move(&game, "d2e2"));
        assert!(!has_move(&game, "a2a3"));
    }

    #[test]
    fn double_check_generates_only_king_moves() {
        let game = Game::new(
            board!(
                . . . . r . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . b . . . . . .
                . . . . . . . .
                . . . . . . . .
                Q . . . K . . .
            ),
            State::new(Color::White, CastlingRights::NONE),
        );

        assert!(!game.moves.is_empty());
        assert!(game.moves.iter().all(|mve| mve.from == square!(e1)));
    }

    #[test]
    fn checked_king_cannot_retreat_on_ray_or_capture_defended_piece() {
        let ray = Game::new(
            board!(
                . . . . r . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . K . . .
                . . . . . . . .
            ),
            State::new(Color::White, CastlingRights::NONE),
        );
        assert!(!has_move(&ray, "e2e1"));

        let defended = Game::new(
            board!(
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                r . . . r . . .
                . . . . K . . .
            ),
            State::new(Color::White, CastlingRights::NONE),
        );
        assert!(!has_move(&defended, "e1e2"));
    }

    #[test]
    fn pin_detection_restricts_moves_to_the_pin_ray() {
        let game = Game::new(
            board!(
                . . . . r . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . R . . .
                . . . . K . . .
            ),
            State::new(Color::White, CastlingRights::NONE),
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
            State::new(Color::White, CastlingRights::NONE),
        );

        assert!(
            game.make_move(Color::White, "e4d5".parse().unwrap())
                .is_ok()
        );
        assert_eq!(game.fen(), "7k/8/8/3P4/8/8/8/K7 b - - 0 1");
    }

    #[test]
    fn generates_and_applies_en_passant_for_both_colors() {
        for (fen, mve, expected) in [
            (
                "4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1",
                "e5d6",
                "4k3/8/3P4/8/8/8/8/4K3 b - - 0 1",
            ),
            (
                "4k3/8/8/8/3Pp3/8/8/4K3 b - d3 0 1",
                "e4d3",
                "4k3/8/8/8/8/3p4/8/4K3 w - - 0 1",
            ),
        ] {
            let mut game = Game::from_fen(fen).unwrap();
            assert!(has_move(&game, mve));
            assert!(
                game.make_move(game.state.turn, mve.parse().unwrap())
                    .is_ok()
            );
            assert_eq!(game.fen(), expected);
        }
    }

    #[test]
    fn en_passant_legality_handles_checks_pins_and_two_removed_pawns() {
        for (name, board, turn, target, legal, illegal) in [
            (
                "two white pawns can capture",
                board!(
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . P p P . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . K . . .
                ),
                Color::White,
                square!(d6),
                &["c5d6", "e5d6"][..],
                &[][..],
            ),
            (
                "two black pawns can capture",
                board!(
                    . . . . . . k .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    p P p . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                ),
                Color::Black,
                square!(b3),
                &["a4b3", "c4b3"][..],
                &[][..],
            ),
            (
                "one of two pawns is file-pinned",
                board!(
                    . . r . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . P p P . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . K . . . . .
                ),
                Color::White,
                square!(d6),
                &["e5d6"][..],
                &["c5d6"][..],
            ),
            (
                "capture removes a checking pawn",
                board!(
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . p P . . .
                    . . . . K . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                ),
                Color::White,
                square!(d6),
                &["e5d6"][..],
                &[][..],
            ),
            (
                "capturing exposes a horizontal rook attack",
                board!(
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    r . . . . p P K
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                ),
                Color::White,
                square!(f6),
                &[][..],
                &["g5f6"][..],
            ),
            (
                "mirrored horizontal rook attack",
                board!(
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    R . p P . . k .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                ),
                Color::Black,
                square!(d3),
                &[][..],
                &["c4d3"][..],
            ),
            (
                "capturing exposes a diagonal bishop attack",
                board!(
                    . . . . . . b .
                    . . . . . . . .
                    . . . . . . . .
                    . . . p P . . .
                    . . . . . . . .
                    . . . . . . . .
                    K . . . . . . .
                    . . . . . . . .
                ),
                Color::White,
                square!(d6),
                &[][..],
                &["e5d6"][..],
            ),
        ] {
            let mut state = State::new(turn, CastlingRights::NONE);
            state.en_passant = EnPassant::new(target);
            let game = Game::new(board, state);
            for mve in legal {
                assert!(has_move(&game, mve), "{name}: {mve} should be legal");
            }
            for mve in illegal {
                assert!(!has_move(&game, mve), "{name}: {mve} should be illegal");
            }
        }
    }

    #[test]
    fn double_push_sets_en_passant_target_and_next_move_clears_it() {
        let mut game = Game::from_fen("4k3/3p4/8/4P3/8/8/8/4K3 b - - 0 1").unwrap();

        assert!(
            game.make_move(Color::Black, "d7d5".parse().unwrap())
                .is_ok()
        );
        assert_eq!(game.fen(), "4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1");
        assert!(has_move(&game, "e5d6"));

        assert!(
            game.make_move(Color::White, "e1e2".parse().unwrap())
                .is_ok()
        );
        assert_eq!(game.fen(), "4k3/8/8/3pP3/8/8/4K3/8 b - - 0 1");
    }

    #[test]
    fn standard_fen_keeps_an_en_passant_target_without_a_capturer() {
        let mut game = Game::default();
        assert!(
            game.make_move(Color::White, "e2e4".parse().unwrap())
                .is_ok()
        );

        assert_eq!(
            game.fen(),
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1"
        );
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

            assert!(
                game.make_move(game.state.turn, mve.parse().unwrap())
                    .is_ok()
            );
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

        assert_eq!(game.state.turn, Color::White);
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

        assert_eq!(game.state.turn, Color::White);
        assert_eq!(game.moves.len(), move_count);
    }
}

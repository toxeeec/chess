use anyhow::{Context, Result};

use crate::{
    attacks::{KingThreats, evasion_mask, king_threats},
    bishop::add_bishop_moves,
    board::Board,
    castling::CastlingRights,
    king::add_king_moves,
    knight::add_knight_moves,
    moves::{Move, MoveKind, MoveList, UciMove},
    pawn::add_pawn_moves,
    queen::add_queen_moves,
    rook::add_rook_moves,
    state::{Color, EnPassant, OPPONENT, State},
};

#[cfg(any(test, feature = "benchmark"))]
use crate::board::BoardUndo;

pub(super) enum MakeMoveError {
    IllegalMove,
    NotYourTurn,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum GameResult {
    Win { winner: Color },
    Draw,
}

pub struct Game {
    pub(super) board: Board,
    pub(super) state: State,
    pub(super) moves: MoveList,
}

#[derive(Clone, Copy)]
pub(super) struct Undo {
    #[cfg(any(test, feature = "benchmark"))]
    state: State,
    #[cfg(any(test, feature = "benchmark"))]
    board: BoardUndo,
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
        generate_legal_moves(&game.board, game.state, &mut game.moves);

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
        mve: UciMove,
    ) -> Result<Option<GameResult>, MakeMoveError> {
        if color != self.state.turn {
            return Err(MakeMoveError::NotYourTurn);
        }

        let mve = self.moves.resolve(mve).ok_or(MakeMoveError::IllegalMove)?;
        // SAFETY: `resolve` returns a move generated for the current board and state.
        unsafe { make_move(&mut self.board, &mut self.state, mve) };
        Ok(generate_legal_moves(
            &self.board,
            self.state,
            &mut self.moves,
        ))
    }
}

pub(super) fn generate_legal_moves(
    board: &Board,
    state: State,
    moves: &mut MoveList,
) -> Option<GameResult> {
    moves.clear();
    match state.turn {
        Color::White => generate_legal_moves_for::<{ Color::White }>(board, state, moves),
        Color::Black => generate_legal_moves_for::<{ Color::Black }>(board, state, moves),
    }
}

fn generate_legal_moves_for<const COLOR: Color>(
    board: &Board,
    state: State,
    moves: &mut MoveList,
) -> Option<GameResult> {
    let blockers = board.occupancy::<COLOR>();
    let enemy = board.occupancy::<{ OPPONENT::<COLOR> }>();
    let occupied = blockers | enemy;
    let empty = !occupied;
    let KingThreats {
        attackers,
        forbidden,
        pin_rays,
    } = king_threats::<{ OPPONENT::<COLOR> }>(board, occupied, blockers);
    let evasion_mask = evasion_mask(board.king_square::<COLOR>(), attackers);

    if !evasion_mask.empty() {
        add_pawn_moves::<COLOR>(
            board,
            empty,
            enemy,
            evasion_mask,
            pin_rays,
            state.en_passant,
            moves,
        );
        add_knight_moves::<COLOR>(board, blockers, enemy, evasion_mask, pin_rays, moves);
        add_bishop_moves::<COLOR>(
            board,
            occupied,
            blockers,
            enemy,
            evasion_mask,
            pin_rays,
            moves,
        );
        add_rook_moves::<COLOR>(
            board,
            occupied,
            blockers,
            enemy,
            evasion_mask,
            pin_rays,
            moves,
        );
        add_queen_moves::<COLOR>(
            board,
            occupied,
            blockers,
            enemy,
            evasion_mask,
            pin_rays,
            moves,
        );
    }

    add_king_moves::<COLOR>(
        board,
        occupied,
        enemy,
        attackers,
        forbidden,
        state.castling_rights,
        moves,
    );

    if moves.is_empty() {
        Some(if attackers.empty() {
            GameResult::Draw
        } else {
            GameResult::Win { winner: !COLOR }
        })
    } else {
        None
    }
}

/// # Safety
///
/// `mve` must have been generated for the current board and state.
pub(super) unsafe fn make_move(board: &mut Board, state: &mut State, mve: Move) -> Undo {
    let previous_state = *state;
    // SAFETY: Guaranteed by `make_move`'s safety contract.
    let _board_undo = unsafe { board.make_move(previous_state.turn, mve) };
    let from = mve.from();
    let to = mve.to();

    state.castling_rights.update(from, to);
    state.en_passant = if mve.kind() == MoveKind::DoublePush {
        EnPassant::new(match previous_state.turn {
            Color::White => to.backward::<{ Color::White }, 1>(),
            Color::Black => to.backward::<{ Color::Black }, 1>(),
        })
    } else {
        EnPassant::NONE
    };
    state.turn = previous_state.turn.opponent();

    Undo {
        #[cfg(any(test, feature = "benchmark"))]
        state: previous_state,
        #[cfg(any(test, feature = "benchmark"))]
        board: _board_undo,
    }
}

#[cfg(any(test, feature = "benchmark"))]
pub(super) fn unmake_move(board: &mut Board, state: &mut State, mve: Move, undo: Undo) {
    board.unmake_move(undo.state.turn, mve, undo.board);
    *state = undo.state;
}

#[cfg(test)]
mod tests {
    use crate::{square, test_utils::board};

    use super::{
        CastlingRights, Color, EnPassant, Game, GameResult, MoveKind, MoveList, State,
        generate_legal_moves, make_move, unmake_move,
    };

    fn has_move(game: &Game, mve: &str) -> bool {
        game.moves.resolve(mve.parse().unwrap()).is_some()
    }

    fn assert_perft(fen: &str, expected: &[u64]) {
        let game = Game::from_fen(fen).unwrap();
        for (index, nodes) in expected.iter().copied().enumerate() {
            let depth = index as u32 + 1;
            assert_eq!(game.perft(depth), nodes, "perft depth {depth}");
        }
    }

    #[test]
    fn make_unmake_restores_every_move_type() {
        for (fen, mve, kind) in [
            (
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                "e2e4",
                MoveKind::DoublePush,
            ),
            (
                "4k3/8/8/3p4/4P3/8/8/4K3 w - - 0 1",
                "e4d5",
                MoveKind::Capture,
            ),
            (
                "4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1",
                "e5d6",
                MoveKind::EnPassant,
            ),
            (
                "4k3/8/8/8/3Pp3/8/8/4K3 b - d3 0 1",
                "e4d3",
                MoveKind::EnPassant,
            ),
            (
                "4k3/8/8/8/8/8/8/R3K2R w KQ - 0 1",
                "e1g1",
                MoveKind::CastleKing,
            ),
            (
                "4k3/8/8/8/8/8/8/R3K2R w KQ - 0 1",
                "e1c1",
                MoveKind::CastleQueen,
            ),
            (
                "r3k2r/8/8/8/8/8/8/4K3 b kq - 0 1",
                "e8g8",
                MoveKind::CastleKing,
            ),
            (
                "r3k2r/8/8/8/8/8/8/4K3 b kq - 0 1",
                "e8c8",
                MoveKind::CastleQueen,
            ),
            (
                "7k/P7/8/8/8/8/8/K7 w - - 0 1",
                "a7a8n",
                MoveKind::PromoteKnight,
            ),
            (
                "1r5k/P7/8/8/8/8/8/K7 w - - 0 1",
                "a7b8q",
                MoveKind::CapturePromoteQueen,
            ),
            (
                "7k/8/8/8/8/8/p7/7K b - - 0 1",
                "a2a1r",
                MoveKind::PromoteRook,
            ),
            (
                "7k/8/8/8/8/8/1p6/B6K b - - 0 1",
                "b2a1b",
                MoveKind::CapturePromoteBishop,
            ),
            (
                "r3k3/8/8/8/8/8/8/R3K3 b Qq - 0 1",
                "a8a1",
                MoveKind::Capture,
            ),
            ("4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1", "e1e2", MoveKind::Quiet),
        ] {
            let game = Game::from_fen(fen).unwrap();
            let input = mve.parse().unwrap();
            let resolved = game.moves.resolve(input);
            assert!(resolved.is_some(), "{input} must be legal in {fen}");
            let mve = resolved.unwrap();
            assert_eq!(mve.kind(), kind);

            let mut board = game.board;
            let mut state = game.state;
            // SAFETY: `mve` was resolved from the moves generated for this board and state.
            let undo = unsafe { make_move(&mut board, &mut state, mve) };
            unmake_move(&mut board, &mut state, mve, undo);

            assert_eq!(format!("{} {}", board.fen(), state.fen()), game.fen());
            let mut restored_moves = MoveList::default();
            generate_legal_moves(&board, state, &mut restored_moves);
            assert_eq!(
                restored_moves.iter().collect::<Vec<_>>(),
                game.moves.iter().collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn perft_initial_position() {
        assert_perft(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            &[20, 400, 8_902, 197_281, 4_865_609],
        );
    }

    #[test]
    fn perft_kiwipete() {
        assert_perft(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            &[48, 2_039, 97_862, 4_085_603],
        );
    }

    #[test]
    fn perft_endgame_with_en_passant() {
        assert_perft(
            "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
            &[14, 191, 2_812, 43_238, 674_624, 11_030_083],
        );
    }

    #[test]
    fn perft_promotions_and_castling() {
        assert_perft(
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
            &[6, 264, 9_467, 422_333, 15_833_292],
        );
    }

    #[test]
    fn perft_tactical_promotions() {
        assert_perft(
            "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 0 1",
            &[44, 1_486, 62_379, 2_103_487],
        );
    }

    #[test]
    fn perft_pins_and_discovered_attacks() {
        assert_perft(
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 1",
            &[46, 2_079, 89_890, 3_894_594],
        );
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
    fn detects_stalemate_after_a_move() {
        let mut game = Game::from_fen("7k/5K2/8/6Q1/8/8/8/8 w - - 0 1").unwrap();

        assert!(matches!(
            game.make_move(Color::White, "g5g6".parse().unwrap()),
            Ok(Some(GameResult::Draw))
        ));
        assert!(game.moves.is_empty());
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
    fn castling_gives_check() {
        for (fen, castle, evasion) in [
            ("5k2/p7/8/8/8/8/8/4K2R w K - 0 1", "e1g1", "f8e7"),
            ("3k4/p7/8/8/8/8/8/R3K3 w Q - 0 1", "e1c1", "d8e7"),
        ] {
            let mut game = Game::from_fen(fen).unwrap();
            assert!(
                game.make_move(Color::White, castle.parse().unwrap())
                    .is_ok()
            );

            assert!(has_move(&game, evasion));
            assert!(!has_move(&game, "a7a6"));
            assert!(!has_move(&game, "a7a5"));
        }
    }

    #[test]
    fn replacing_a_captured_home_rook_does_not_restore_castling_rights() {
        let mut game = Game::from_fen("4k3/8/8/8/8/8/Rb6/R3K3 b Q - 0 1").unwrap();

        for (color, mve) in [
            (Color::Black, "b2a1"),
            (Color::White, "a2a1"),
            (Color::Black, "e8e7"),
        ] {
            assert!(game.make_move(color, mve.parse().unwrap()).is_ok());
        }

        assert_eq!(game.state.castling_rights.fen(), "-");
        assert!(!has_move(&game, "e1c1"));
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
        assert!(game.moves.iter().all(|mve| mve.from() == square!(e1)));
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
    fn en_passant_evades_a_check_caused_by_a_double_push() {
        for (push, en_passant) in [("d7d5", "e5d6"), ("f7f5", "e5f6")] {
            let mut game = Game::from_fen("4k3/3p1p2/8/4P3/4K3/8/8/8 b - - 0 1").unwrap();

            assert!(game.make_move(Color::Black, push.parse().unwrap()).is_ok());

            assert!(has_move(&game, en_passant));
        }
    }

    #[test]
    fn en_passant_can_open_a_discovered_check() {
        let board = board!(
            . . . . . . . .
            p . . . . k . .
            . . . . . . . .
            . . P p . . . .
            . . B . . . . .
            . K . . . . . .
            . . . . . . . .
            . . . . . . . .
        );
        let mut state = State::new(Color::White, CastlingRights::NONE);
        state.en_passant = EnPassant::new(square!(d6));
        let mut game = Game::new(board, state);

        assert!(
            game.make_move(Color::White, "c5d6".parse().unwrap())
                .is_ok()
        );

        assert!(has_move(&game, "f7e8"));
        assert!(!has_move(&game, "a7a6"));
        assert!(!has_move(&game, "a7a5"));
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
    fn promotion_can_evade_check() {
        let game = Game::from_fen("2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1").unwrap();

        for mve in ["e7e8q", "e7e8r", "e7e8b", "e7e8n"] {
            assert!(has_move(&game, mve));
        }
    }

    #[test]
    fn knight_underpromotion_gives_check() {
        let board = board!(
            . . . . . . . .
            P . k . . . . p
            K . . . . . . .
            . . . . . . . .
            . . . . . . . .
            . . . . . . . .
            . . . . . . . .
            . . . . . . . .
        );

        let mut knight = Game::new(board, State::new(Color::White, CastlingRights::NONE));
        assert!(
            knight
                .make_move(Color::White, "a7a8n".parse().unwrap())
                .is_ok()
        );
        assert!(!has_move(&knight, "h7h6"));
        assert!(!has_move(&knight, "h7h5"));

        let mut queen = Game::new(board, State::new(Color::White, CastlingRights::NONE));
        assert!(
            queen
                .make_move(Color::White, "a7a8q".parse().unwrap())
                .is_ok()
        );
        assert!(has_move(&queen, "h7h6"));
        assert!(has_move(&queen, "h7h5"));
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

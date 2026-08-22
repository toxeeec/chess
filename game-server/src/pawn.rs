use crate::{
    attacks::PinRays,
    bitboard::Bitboard,
    board::Board,
    game::Color,
    moves::{Move, MoveList, PromotionPiece},
};

pub(super) fn add_pawn_moves<const COLOR: Color>(
    board: &Board,
    empty: Bitboard,
    enemies: Bitboard,
    evasion_mask: Bitboard,
    pin_rays: PinRays,
    list: &mut MoveList,
) {
    let pawns = board.pawns::<COLOR>();
    let pinned = pin_rays.pinned_pieces(pawns);
    let unpinned = pawns & !pinned;
    let diagonal = pawns & pin_rays.diagonal;
    let orthogonal = pawns & pin_rays.orthogonal;
    let promotion_rank = Bitboard::relative_rank::<COLOR>(8);

    let pushes = (unpinned.forward::<COLOR, 1>()
        | (orthogonal.forward::<COLOR, 1>() & pin_rays.orthogonal))
        & empty;
    let single_pushes = pushes & !promotion_rank;
    let double_pushes = ((single_pushes & Bitboard::relative_rank::<COLOR>(3))
        .forward::<COLOR, 1>())
        & empty
        & evasion_mask;
    let single_pushes = single_pushes & evasion_mask;

    let west_captures = (unpinned.forward_west::<COLOR>()
        | (diagonal.forward_west::<COLOR>() & pin_rays.diagonal))
        & enemies
        & evasion_mask;
    let east_captures = (unpinned.forward_east::<COLOR>()
        | (diagonal.forward_east::<COLOR>() & pin_rays.diagonal))
        & enemies
        & evasion_mask;
    let west_capture_promotions = west_captures & promotion_rank;
    let east_capture_promotions = east_captures & promotion_rank;
    let west_captures = west_captures & !promotion_rank;
    let east_captures = east_captures & !promotion_rank;

    let quiet_promotions = pushes & promotion_rank & evasion_mask;

    list.extend(
        single_pushes
            .map(|to| Move::new(to.backward::<COLOR, 1>(), to, None))
            .chain(double_pushes.map(|to| Move::new(to.backward::<COLOR, 2>(), to, None)))
            .chain(west_captures.map(|to| Move::new(to.backward_east::<COLOR>(), to, None)))
            .chain(east_captures.map(|to| Move::new(to.backward_west::<COLOR>(), to, None))),
    );
    list.extend(
        quiet_promotions
            .map(|to| (to.backward::<COLOR, 1>(), to))
            .chain(
                west_capture_promotions
                    .map(|to| (to.backward_east::<COLOR>(), to))
                    .chain(east_capture_promotions.map(|to| (to.backward_west::<COLOR>(), to))),
            )
            .flat_map(|(from, to)| {
                PromotionPiece::ALL.map(|piece| Move::new(from, to, Some(piece)))
            }),
    );
}

#[cfg(test)]
mod tests {
    use crate::{
        attacks::PinRays,
        bitboard::Bitboard,
        board::Board,
        game::Color,
        moves::MoveList,
        squares,
        test_utils::{MoveCase, assert_move_case, assert_move_cases, board, moves},
    };

    use super::add_pawn_moves;

    fn pawn_moves<const COLOR: Color>(board: Board, pin_rays: PinRays) -> MoveList {
        let mut moves = MoveList::default();
        let blockers = board.occupancy::<COLOR>();
        let occupied = board.occupied();

        add_pawn_moves::<COLOR>(
            &board,
            !occupied,
            occupied & !blockers,
            Bitboard::FULL,
            pin_rays,
            &mut moves,
        );

        moves
    }

    #[test]
    fn generates_white_pawn_pushes() {
        assert_move_cases(
            [
                MoveCase {
                    name: "white pawn single and double push",
                    board: board!(
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . P . . .
                        . . . . . . . .
                    ),
                    moves: moves!(
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . x . . .
                        . . . . x . . .
                        . . . . o . . .
                        . . . . . . . .
                    ),
                },
                MoveCase {
                    name: "white pawn blocked",
                    board: board!(
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . p . . .
                        . . . . P . . .
                        . . . . . . . .
                    ),
                    moves: moves!(
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . o . . .
                        . . . . . . . .
                    ),
                },
                MoveCase {
                    name: "white pawn captures enemies and ignores friendly pieces",
                    board: board!(
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . N . p . .
                        . . . . P . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                    ),
                    moves: moves!(
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . x x . .
                        . . . . o . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                    ),
                },
                MoveCase {
                    name: "white pawn captures without wrapping from a-file",
                    board: board!(
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . p . . . . . p
                        P . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                    ),
                    moves: moves!(
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        x x . . . . . .
                        o . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                    ),
                },
            ],
            |board| pawn_moves::<{ Color::White }>(board, PinRays::EMPTY),
        );
    }

    #[test]
    fn generates_black_pawn_pushes() {
        assert_move_cases(
            [
                MoveCase {
                    name: "black pawn single and double push",
                    board: board!(
                        . . . . . . . .
                        . . . . p . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                    ),
                    moves: moves!(
                        . . . . . . . .
                        . . . . o . . .
                        . . . . x . . .
                        . . . . x . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                    ),
                },
                MoveCase {
                    name: "black pawn blocked",
                    board: board!(
                        . . . . . . . .
                        . . . . p . . .
                        . . . . P . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                    ),
                    moves: moves!(
                        . . . . . . . .
                        . . . . o . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                    ),
                },
                MoveCase {
                    name: "black pawn captures enemies and ignores friendly pieces",
                    board: board!(
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . p . . .
                        . . . n . P . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                    ),
                    moves: moves!(
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . o . . .
                        . . . . x x . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                    ),
                },
                MoveCase {
                    name: "black pawn captures without wrapping from h-file",
                    board: board!(
                        . . . . . . . .
                        . . . . . . . p
                        P . . . . . P .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                    ),
                    moves: moves!(
                        . . . . . . . .
                        . . . . . . . o
                        . . . . . . x x
                        . . . . . . . x
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                    ),
                },
            ],
            |board| pawn_moves::<{ Color::Black }>(board, PinRays::EMPTY),
        );
    }

    #[test]
    fn generates_all_quiet_and_capture_promotions() {
        let white = pawn_moves::<{ Color::White }>(
            board!(
                r . n . . . . .
                . P . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
            ),
            PinRays::EMPTY,
        );
        assert_eq!(
            white.iter().map(|mve| mve.to_string()).collect::<Vec<_>>(),
            [
                "b7b8q", "b7b8r", "b7b8b", "b7b8n", "b7a8q", "b7a8r", "b7a8b", "b7a8n", "b7c8q",
                "b7c8r", "b7c8b", "b7c8n",
            ]
        );

        let black = pawn_moves::<{ Color::Black }>(
            board!(
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . p .
                . . . . . B . R
            ),
            PinRays::EMPTY,
        );
        assert_eq!(
            black.iter().map(|mve| mve.to_string()).collect::<Vec<_>>(),
            [
                "g2g1q", "g2g1r", "g2g1b", "g2g1n", "g2f1q", "g2f1r", "g2f1b", "g2f1n", "g2h1q",
                "g2h1r", "g2h1b", "g2h1n",
            ]
        );
    }

    #[test]
    fn allows_file_pinned_white_pawns_to_push() {
        let pin_rays = PinRays::orthogonal(Bitboard::from(squares![e2, e3, e4, e5, e6, e7, e8]));

        assert_move_case(
            MoveCase {
                name: "file-pinned white pawn",
                board: board!(
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . P . . .
                    . . . . . . . .
                ),
                moves: moves!(
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . x . . .
                    . . . . x . . .
                    . . . . o . . .
                    . . . . . . . .
                ),
            },
            |board| pawn_moves::<{ Color::White }>(board, pin_rays),
        );
    }

    #[test]
    fn allows_diagonally_pinned_white_pawns_to_capture() {
        let pin_rays = PinRays::diagonal(Bitboard::from(squares![c2, d3, e4]));

        assert_move_case(
            MoveCase {
                name: "diagonally pinned white pawn",
                board: board!(
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . b . . .
                    . . . P . . . .
                    . . . . . . . .
                    . . . . . . . .
                ),
                moves: moves!(
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . x . . .
                    . . . o . . . .
                    . . . . . . . .
                    . . . . . . . .
                ),
            },
            |board| pawn_moves::<{ Color::White }>(board, pin_rays),
        );
    }

    #[test]
    fn allows_file_pinned_black_pawns_to_push() {
        let pin_rays = PinRays::orthogonal(Bitboard::from(squares![e1, e2, e3, e4, e5, e6, e7]));

        assert_move_case(
            MoveCase {
                name: "file-pinned black pawn",
                board: board!(
                    . . . . . . . .
                    . . . . p . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                ),
                moves: moves!(
                    . . . . . . . .
                    . . . . o . . .
                    . . . . x . . .
                    . . . . x . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                ),
            },
            |board| pawn_moves::<{ Color::Black }>(board, pin_rays),
        );
    }

    #[test]
    fn allows_diagonally_pinned_black_pawns_to_capture() {
        let pin_rays = PinRays::diagonal(Bitboard::from(squares![c7, d6, e5]));

        assert_move_case(
            MoveCase {
                name: "diagonally pinned black pawn",
                board: board!(
                    . . . . . . . .
                    . . . . . . . .
                    . . . p . . . .
                    . . . . B . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                ),
                moves: moves!(
                    . . . . . . . .
                    . . . . . . . .
                    . . . o . . . .
                    . . . . x . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                ),
            },
            |board| pawn_moves::<{ Color::Black }>(board, pin_rays),
        );
    }
}

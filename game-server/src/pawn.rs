use crate::{
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
    list: &mut MoveList,
) {
    let pawns = board.pawns::<COLOR>();
    let promotion_rank = Bitboard::relative_rank::<COLOR>(8);

    let single_pushes = pawns.forward::<COLOR, 1>() & empty & !promotion_rank;
    let double_pushes = ((single_pushes & Bitboard::relative_rank::<COLOR>(3))
        .forward::<COLOR, 1>())
        & empty
        & evasion_mask;
    let single_pushes = single_pushes & evasion_mask;

    let west_captures = pawns.forward_west::<COLOR>() & enemies & !promotion_rank & evasion_mask;
    let east_captures = pawns.forward_east::<COLOR>() & enemies & !promotion_rank & evasion_mask;

    let quiet_promotions = pawns.forward::<COLOR, 1>() & empty & promotion_rank & evasion_mask;
    let west_capture_promotions =
        pawns.forward_west::<COLOR>() & enemies & promotion_rank & evasion_mask;
    let east_capture_promotions =
        pawns.forward_east::<COLOR>() & enemies & promotion_rank & evasion_mask;

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
        bitboard::Bitboard,
        board::Board,
        game::Color,
        moves::MoveList,
        test_utils::{MoveCase, assert_move_cases, board, moves},
    };

    use super::add_pawn_moves;

    fn pawn_moves<const COLOR: Color>(board: Board) -> MoveList {
        let mut moves = MoveList::default();
        let blockers = board.occupancy::<COLOR>();
        let occupied = board.occupied();

        add_pawn_moves::<COLOR>(
            &board,
            !occupied,
            occupied & !blockers,
            Bitboard::FULL,
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
            pawn_moves::<{ Color::White }>,
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
            pawn_moves::<{ Color::Black }>,
        );
    }

    #[test]
    fn generates_all_quiet_and_capture_promotions() {
        let white = pawn_moves::<{ Color::White }>(board!(
            r . n . . . . .
            . P . . . . . .
            . . . . . . . .
            . . . . . . . .
            . . . . . . . .
            . . . . . . . .
            . . . . . . . .
            . . . . . . . .
        ));
        let black = pawn_moves::<{ Color::Black }>(board!(
            . . . . . . . .
            . . . . . . . .
            . . . . . . . .
            . . . . . . . .
            . . . . . . . .
            . . . . . . . .
            . . . . . . p .
            . . . . . B . R
        ));

        assert_eq!(
            white.iter().map(|mve| mve.to_string()).collect::<Vec<_>>(),
            [
                "b7b8q", "b7b8r", "b7b8b", "b7b8n", "b7a8q", "b7a8r", "b7a8b", "b7a8n", "b7c8q",
                "b7c8r", "b7c8b", "b7c8n",
            ]
        );
        assert_eq!(
            black.iter().map(|mve| mve.to_string()).collect::<Vec<_>>(),
            [
                "g2g1q", "g2g1r", "g2g1b", "g2g1n", "g2f1q", "g2f1r", "g2f1b", "g2f1n", "g2h1q",
                "g2h1r", "g2h1b", "g2h1n",
            ]
        );
    }
}

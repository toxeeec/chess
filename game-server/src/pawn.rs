use crate::{
    attacks::PinRays,
    bitboard::Bitboard,
    board::Board,
    magics::{bishop_attacks, rook_attacks},
    moves::{Move, MoveKind, MoveList},
    square::Square,
    state::{Color, EnPassant, OPPONENT},
};

pub(super) fn add_pawn_moves<const COLOR: Color>(
    board: &Board,
    empty: Bitboard,
    enemies: Bitboard,
    evasion_mask: Bitboard,
    pin_rays: PinRays,
    en_passant: EnPassant,
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

    list.extend(single_pushes.map(|to| Move::new(to.backward::<COLOR, 1>(), to, MoveKind::Quiet)));
    list.extend(
        double_pushes.map(|to| Move::new(to.backward::<COLOR, 2>(), to, MoveKind::DoublePush)),
    );

    add_pawn_captures::<COLOR>(west_captures, east_captures, list);
    add_pawn_promotions::<COLOR>(
        quiet_promotions,
        west_capture_promotions,
        east_capture_promotions,
        list,
    );

    if let Some(to) = en_passant.target() {
        let target = Bitboard::from(to);
        let captured = to.backward::<COLOR, 1>();
        let occupied = !empty;
        let west = pawns.forward_west::<COLOR>() & target;
        let east = pawns.forward_east::<COLOR>() & target;

        let evades_check = !(evasion_mask & (target | Bitboard::from(captured))).empty();
        if evades_check {
            if !west.empty() {
                let from = to.backward_east::<COLOR>();
                if en_passant_is_legal::<COLOR>(board, occupied, from, to, captured) {
                    list.push(Move::new(from, to, MoveKind::EnPassant));
                }
            }
            if !east.empty() {
                let from = to.backward_west::<COLOR>();
                if en_passant_is_legal::<COLOR>(board, occupied, from, to, captured) {
                    list.push(Move::new(from, to, MoveKind::EnPassant));
                }
            }
        }
    }
}

fn add_pawn_captures<const COLOR: Color>(
    west_captures: Bitboard,
    east_captures: Bitboard,
    list: &mut MoveList,
) {
    list.extend(west_captures.map(|to| {
        let from = to.backward_east::<COLOR>();
        Move::new(from, to, MoveKind::Capture)
    }));
    list.extend(east_captures.map(|to| {
        let from = to.backward_west::<COLOR>();
        Move::new(from, to, MoveKind::Capture)
    }));
}

fn add_pawn_promotions<const COLOR: Color>(
    quiet_promotions: Bitboard,
    west_capture_promotions: Bitboard,
    east_capture_promotions: Bitboard,
    list: &mut MoveList,
) {
    for to in quiet_promotions {
        let from = to.backward::<COLOR, 1>();
        list.extend(MoveKind::QUIET_PROMOTIONS.map(|kind| Move::new(from, to, kind)));
    }

    for to in west_capture_promotions {
        let from = to.backward_east::<COLOR>();
        list.extend(MoveKind::CAPTURE_PROMOTIONS.map(|kind| Move::new(from, to, kind)));
    }

    for to in east_capture_promotions {
        let from = to.backward_west::<COLOR>();
        list.extend(MoveKind::CAPTURE_PROMOTIONS.map(|kind| Move::new(from, to, kind)));
    }
}

fn en_passant_is_legal<const COLOR: Color>(
    board: &Board,
    occupied: Bitboard,
    from: Square,
    to: Square,
    captured: Square,
) -> bool {
    let king = board.king_square::<COLOR>();
    let occupied = (occupied & !Bitboard::from([from, captured])) | Bitboard::from(to);
    let diagonal_sliders =
        board.bishops::<{ OPPONENT::<COLOR> }>() | board.queens::<{ OPPONENT::<COLOR> }>();
    let orthogonal_sliders =
        board.rooks::<{ OPPONENT::<COLOR> }>() | board.queens::<{ OPPONENT::<COLOR> }>();

    (bishop_attacks(king, occupied) & diagonal_sliders).empty()
        && (rook_attacks(king, occupied) & orthogonal_sliders).empty()
}

#[cfg(test)]
mod tests {
    use crate::{
        attacks::PinRays,
        bitboard::Bitboard,
        board::Board,
        moves::MoveList,
        squares,
        state::{Color, EnPassant},
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
            EnPassant::NONE,
            &mut moves,
        );

        moves
    }

    fn white_pawn_moves(board: Board, pin_rays: PinRays) -> MoveList {
        pawn_moves::<{ Color::White }>(board, pin_rays)
    }

    fn black_pawn_moves(board: Board, pin_rays: PinRays) -> MoveList {
        pawn_moves::<{ Color::Black }>(board, pin_rays)
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
            |board| white_pawn_moves(board, PinRays::EMPTY),
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
            |board| black_pawn_moves(board, PinRays::EMPTY),
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
            |board| white_pawn_moves(board, pin_rays),
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
            |board| white_pawn_moves(board, pin_rays),
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
            |board| black_pawn_moves(board, pin_rays),
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
            |board| black_pawn_moves(board, pin_rays),
        );
    }
}

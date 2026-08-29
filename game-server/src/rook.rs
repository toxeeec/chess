use crate::{
    attacks::PinRays, bitboard::Bitboard, board::Board, magics::rook_attacks, moves::MoveList,
    state::Color,
};

pub(super) fn add_rook_moves<const COLOR: Color>(
    board: &Board,
    occupied: Bitboard,
    blockers: Bitboard,
    enemies: Bitboard,
    evasion_mask: Bitboard,
    pin_rays: PinRays,
    list: &mut MoveList,
) {
    let rooks = board.rooks::<COLOR>();
    let pinned = pin_rays.pinned_pieces(blockers);
    let targets = !blockers & evasion_mask;

    for from in rooks & !pinned {
        let moves = rook_attacks(from, occupied) & targets;
        board.add_normal_moves(list, from, moves, enemies);
    }
    for from in rooks & pin_rays.orthogonal {
        let moves = rook_attacks(from, occupied) & targets & pin_rays.orthogonal;
        board.add_normal_moves(list, from, moves, enemies);
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        attacks::PinRays,
        bitboard::Bitboard,
        board::Board,
        moves::MoveList,
        squares,
        state::Color,
        test_utils::{MoveCase, assert_move_case, assert_move_cases, board, moves},
    };

    use super::add_rook_moves;

    fn rook_moves(board: Board, pin_rays: PinRays) -> MoveList {
        let mut moves = MoveList::default();
        let blockers = board.occupancy::<{ Color::White }>();
        let enemies = board.occupancy::<{ Color::Black }>();
        let occupied = blockers | enemies;

        add_rook_moves::<{ Color::White }>(
            &board,
            occupied,
            blockers,
            enemies,
            Bitboard::FULL,
            pin_rays,
            &mut moves,
        );

        moves
    }

    #[test]
    fn generates_rook_moves() {
        assert_move_cases(
            [
                MoveCase {
                    name: "rook from center on empty board",
                    board: board!(
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . R . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                    ),
                    moves: moves!(
                        . . . x . . . .
                        . . . x . . . .
                        . . . x . . . .
                        . . . x . . . .
                        x x x o x x x x
                        . . . x . . . .
                        . . . x . . . .
                        . . . x . . . .
                    ),
                },
                MoveCase {
                    name: "rook excludes own blockers and includes enemy blockers",
                    board: board!(
                        . . . . . . . .
                        . . . . . . . .
                        . . . P . . . .
                        . . . . . . . .
                        . P . R . . p .
                        . . . . . . . .
                        . . . p . . . .
                        . . . . . . . .
                    ),
                    moves: moves!(
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . x . . . .
                        . . x o x x x .
                        . . . x . . . .
                        . . . x . . . .
                        . . . . . . . .
                    ),
                },
            ],
            |board| rook_moves(board, PinRays::EMPTY),
        );
    }

    #[test]
    fn allows_rooks_to_move_along_orthogonal_pin_rays() {
        let pin_rays = PinRays::orthogonal(Bitboard::from(squares![e2, e3, e4, e5, e6, e7, e8]));

        assert_move_case(
            MoveCase {
                name: "orthogonally pinned rook",
                board: board!(
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . R . . .
                    . . . . . . . .
                ),
                moves: moves!(
                    . . . . x . . .
                    . . . . x . . .
                    . . . . x . . .
                    . . . . x . . .
                    . . . . x . . .
                    . . . . x . . .
                    . . . . o . . .
                    . . . . . . . .
                ),
            },
            |board| rook_moves(board, pin_rays),
        );
    }

    #[test]
    fn excludes_diagonally_pinned_rooks() {
        let pin_rays = PinRays::diagonal(Bitboard::from(squares![d3, e4, f5, g6, h7]));

        assert_move_case(
            MoveCase {
                name: "diagonally pinned rook",
                board: board!(
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . R . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                ),
                moves: moves!(
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . o . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                ),
            },
            |board| rook_moves(board, pin_rays),
        );
    }
}

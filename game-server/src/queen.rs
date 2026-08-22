use crate::{
    attacks::PinRays,
    bitboard::Bitboard,
    board::Board,
    game::Color,
    magics::{bishop_attacks, rook_attacks},
    moves::{Move, MoveList},
};

pub(super) fn add_queen_moves<const COLOR: Color>(
    board: &Board,
    occupied: Bitboard,
    blockers: Bitboard,
    evasion_mask: Bitboard,
    pin_rays: PinRays,
    list: &mut MoveList,
) {
    let queens = board.queens::<COLOR>();
    let pinned = pin_rays.pinned_pieces(blockers);
    let targets = !blockers & evasion_mask;

    for from in queens & !pinned {
        let moves = (rook_attacks(from, occupied) | bishop_attacks(from, occupied)) & targets;
        list.extend(moves.map(|to| Move::new(from, to, None)));
    }
    for from in queens & pin_rays.diagonal {
        let moves = bishop_attacks(from, occupied) & targets & pin_rays.diagonal;
        list.extend(moves.map(|to| Move::new(from, to, None)));
    }
    for from in queens & pin_rays.orthogonal {
        let moves = rook_attacks(from, occupied) & targets & pin_rays.orthogonal;
        list.extend(moves.map(|to| Move::new(from, to, None)));
    }
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

    use super::add_queen_moves;

    fn queen_moves(board: Board, pin_rays: PinRays) -> MoveList {
        let mut moves = MoveList::default();

        add_queen_moves::<{ Color::White }>(
            &board,
            board.occupied(),
            board.occupancy::<{ Color::White }>(),
            Bitboard::FULL,
            pin_rays,
            &mut moves,
        );

        moves
    }

    #[test]
    fn generates_queen_moves() {
        assert_move_cases(
            [
                MoveCase {
                    name: "queen from center on empty board",
                    board: board!(
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . Q . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                    ),
                    moves: moves!(
                        . . . x . . . x
                        x . . x . . x .
                        . x . x . x . .
                        . . x x x . . .
                        x x x o x x x x
                        . . x x x . . .
                        . x . x . x . .
                        x . . x . . x .
                    ),
                },
                MoveCase {
                    name: "queen excludes own blockers and includes enemy blockers",
                    board: board!(
                        . . . . . . . .
                        . . . . . . . .
                        . P . P . p . .
                        . . P . p . . .
                        . P . Q . . p .
                        . . p . P . . .
                        . . . p . . . .
                        . . . . . . . .
                    ),
                    moves: moves!(
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . x x . . .
                        . . x o x x x .
                        . . x x . . . .
                        . . . x . . . .
                        . . . . . . . .
                    ),
                },
            ],
            |board| queen_moves(board, PinRays::EMPTY),
        );
    }

    #[test]
    fn allows_queens_to_move_along_diagonal_pin_rays() {
        let pin_rays = PinRays::diagonal(Bitboard::from(squares![d3, e4, f5, g6, h7]));

        assert_move_case(
            MoveCase {
                name: "diagonally pinned queen",
                board: board!(
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . Q . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                ),
                moves: moves!(
                    . . . . . . . .
                    . . . . . . . x
                    . . . . . . x .
                    . . . . . o . .
                    . . . . x . . .
                    . . . x . . . .
                    . . . . . . . .
                    . . . . . . . .
                ),
            },
            |board| queen_moves(board, pin_rays),
        );
    }

    #[test]
    fn allows_queens_to_move_along_orthogonal_pin_rays() {
        let pin_rays = PinRays::orthogonal(Bitboard::from(squares![e2, e3, e4, e5, e6, e7, e8]));

        assert_move_case(
            MoveCase {
                name: "orthogonally pinned queen",
                board: board!(
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . Q . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                ),
                moves: moves!(
                    . . . . x . . .
                    . . . . x . . .
                    . . . . x . . .
                    . . . . x . . .
                    . . . . o . . .
                    . . . . x . . .
                    . . . . x . . .
                    . . . . . . . .
                ),
            },
            |board| queen_moves(board, pin_rays),
        );
    }
}

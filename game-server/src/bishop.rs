use crate::{
    attacks::PinRays,
    bitboard::Bitboard,
    board::Board,
    magics::bishop_attacks,
    moves::{Move, MoveList},
    state::Color,
};

pub(super) fn add_bishop_moves<const COLOR: Color>(
    board: &Board,
    occupied: Bitboard,
    blockers: Bitboard,
    evasion_mask: Bitboard,
    pin_rays: PinRays,
    list: &mut MoveList,
) {
    let bishops = board.bishops::<COLOR>();
    let pinned = pin_rays.pinned_pieces(blockers);
    let targets = !blockers & evasion_mask;

    for from in bishops & !pinned {
        let moves = bishop_attacks(from, occupied) & targets;
        list.extend(moves.map(|to| Move::new(from, to, None)));
    }
    for from in bishops & pin_rays.diagonal {
        let moves = bishop_attacks(from, occupied) & targets & pin_rays.diagonal;
        list.extend(moves.map(|to| Move::new(from, to, None)));
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

    use super::add_bishop_moves;

    fn bishop_moves(board: Board, pin_rays: PinRays) -> MoveList {
        let mut moves = MoveList::default();

        add_bishop_moves::<{ Color::White }>(
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
    fn generates_bishop_moves() {
        assert_move_cases(
            [
                MoveCase {
                    name: "bishop from center on empty board",
                    board: board!(
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . B . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                    ),
                    moves: moves!(
                        . . . . . . . x
                        x . . . . . x .
                        . x . . . x . .
                        . . x . x . . .
                        . . . o . . . .
                        . . x . x . . .
                        . x . . . x . .
                        x . . . . . x .
                    ),
                },
                MoveCase {
                    name: "bishop excludes own blockers and includes enemy blockers",
                    board: board!(
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . P . p . . .
                        . . . B . . . .
                        . . p . P . . .
                        . . . . . . . .
                        . . . . . . . .
                    ),
                    moves: moves!(
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . x . . .
                        . . . o . . . .
                        . . x . . . . .
                        . . . . . . . .
                        . . . . . . . .
                    ),
                },
            ],
            |board| bishop_moves(board, PinRays::EMPTY),
        );
    }

    #[test]
    fn allows_bishops_to_move_along_diagonal_pin_rays() {
        let pin_rays = PinRays::diagonal(Bitboard::from(squares![d3, e4, f5, g6, h7]));

        assert_move_case(
            MoveCase {
                name: "diagonally pinned bishop",
                board: board!(
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . B . .
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
            |board| bishop_moves(board, pin_rays),
        );
    }

    #[test]
    fn excludes_orthogonally_pinned_bishops() {
        let pin_rays = PinRays::orthogonal(Bitboard::from(squares![e2, e3, e4, e5, e6, e7, e8]));

        assert_move_case(
            MoveCase {
                name: "orthogonally pinned bishop",
                board: board!(
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . B . . .
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
            |board| bishop_moves(board, pin_rays),
        );
    }
}

use crate::{
    bitboard::Bitboard,
    board::Board,
    game::Color,
    magics::rook_attacks,
    moves::{Move, MoveList},
};

pub(super) fn add_rook_moves<const COLOR: Color>(
    board: &Board,
    occupied: Bitboard,
    blockers: Bitboard,
    list: &mut MoveList,
) {
    for from in board.rooks::<COLOR>() {
        let moves = rook_attacks(from, occupied) & !blockers;
        list.extend(moves.map(|to| Move::new(from, to, None)));
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        board::Board,
        game::Color,
        moves::MoveList,
        test_utils::{MoveCase, assert_move_cases, board, moves},
    };

    use super::add_rook_moves;

    fn rook_moves(board: Board) -> MoveList {
        let mut moves = MoveList::default();

        add_rook_moves::<{ Color::White }>(
            &board,
            board.occupied(),
            board.occupancy::<{ Color::White }>(),
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
                        . P . R . . p p
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
            rook_moves,
        );
    }
}

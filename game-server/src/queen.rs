use crate::{
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
    list: &mut MoveList,
) {
    for from in board.queens::<COLOR>() {
        let moves = (rook_attacks(from, occupied) | bishop_attacks(from, occupied))
            & !blockers
            & evasion_mask;
        list.extend(moves.map(|to| Move::new(from, to, None)));
    }
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

    use super::add_queen_moves;

    fn queen_moves(board: Board) -> MoveList {
        let mut moves = MoveList::default();

        add_queen_moves::<{ Color::White }>(
            &board,
            board.occupied(),
            board.occupancy::<{ Color::White }>(),
            Bitboard::FULL,
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
            queen_moves,
        );
    }
}

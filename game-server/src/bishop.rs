use crate::{
    bitboard::Bitboard,
    board::Board,
    game::Color,
    magics::bishop_attacks,
    moves::{Move, MoveList},
};

pub(super) fn add_bishop_moves<const COLOR: Color>(
    board: &Board,
    occupied: Bitboard,
    blockers: Bitboard,
    evasion_mask: Bitboard,
    list: &mut MoveList,
) {
    for from in board.bishops::<COLOR>() {
        let moves = bishop_attacks(from, occupied) & !blockers & evasion_mask;
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

    use super::add_bishop_moves;

    fn bishop_moves(board: Board) -> MoveList {
        let mut moves = MoveList::default();

        add_bishop_moves::<{ Color::White }>(
            &board,
            board.occupied(),
            board.occupancy::<{ Color::White }>(),
            Bitboard::FULL,
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
                        . . . . . p . .
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
            bishop_moves,
        );
    }
}

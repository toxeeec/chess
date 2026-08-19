use crate::{
    bitboard,
    bitboard::{Bitboard, Direction},
    board::Board,
    game::Color,
    moves::{Move, MoveList},
};

pub(super) fn add_knight_moves<const COLOR: Color>(
    board: &Board,
    blockers: Bitboard,
    list: &mut MoveList,
) {
    let knights = board.knights::<COLOR>();

    for from in knights {
        let moves = KNIGHT_ATTACKS[from] & !blockers;
        list.extend(moves.map(|to| Move::new(from, to, None)));
    }
}

pub(super) const KNIGHT_ATTACKS: [Bitboard; 64] = {
    let mut attacks = [Bitboard::EMPTY; 64];
    let mut square = 0;

    while square < 64 {
        let bb = bitboard!(square);
        attacks[square] = bb.shift::<{ Direction::Nne }>()
            | bb.shift::<{ Direction::Nnw }>()
            | bb.shift::<{ Direction::Nee }>()
            | bb.shift::<{ Direction::Nww }>()
            | bb.shift::<{ Direction::Sse }>()
            | bb.shift::<{ Direction::Ssw }>()
            | bb.shift::<{ Direction::See }>()
            | bb.shift::<{ Direction::Sww }>();
        square += 1;
    }

    attacks
};

#[cfg(test)]
mod tests {
    use crate::{
        board::Board,
        game::Color,
        moves::MoveList,
        test_utils::{MoveCase, assert_move_cases, board, moves},
    };

    use super::add_knight_moves;

    fn knight_moves(board: Board) -> MoveList {
        let mut moves = MoveList::default();

        add_knight_moves::<{ Color::White }>(
            &board,
            board.occupancy::<{ Color::White }>(),
            &mut moves,
        );

        moves
    }

    #[test]
    fn generates_knight_moves() {
        assert_move_cases(
            [
                MoveCase {
                    name: "knight from center on empty board",
                    board: board!(
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . N . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                    ),
                    moves: moves!(
                        . . . . . . . .
                        . . . . . . . .
                        . . x . x . . .
                        . x . . . x . .
                        . . . o . . . .
                        . x . . . x . .
                        . . x . x . . .
                        . . . . . . . .
                    ),
                },
                MoveCase {
                    name: "knight from corner",
                    board: board!(
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        N . . . . . . .
                    ),
                    moves: moves!(
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . x . . . . . .
                        . . x . . . . .
                        o . . . . . . .
                    ),
                },
                MoveCase {
                    name: "knight excludes own blockers and includes enemy blockers",
                    board: board!(
                        . . . . . . . .
                        . . . . . . . .
                        . . p . P . . .
                        . . . . . P . .
                        . . . N . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                    ),
                    moves: moves!(
                        . . . . . . . .
                        . . . . . . . .
                        . . x . . . . .
                        . x . . . . . .
                        . . . o . . . .
                        . x . . . x . .
                        . . x . x . . .
                        . . . . . . . .
                    ),
                },
            ],
            knight_moves,
        );
    }
}

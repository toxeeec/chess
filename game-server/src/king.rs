use crate::{
    bitboard,
    bitboard::{Bitboard, Direction},
    board::Board,
    game::Color,
    moves::{Move, MoveList},
};

pub(super) fn add_king_moves<const COLOR: Color>(
    board: &Board,
    blockers: Bitboard,
    forbidden: Bitboard,
    list: &mut MoveList,
) {
    let mut king = board.king::<COLOR>();

    debug_assert_eq!(king.len(), 1);

    let from = unsafe { king.next().unwrap_unchecked() };
    let moves = KING_ATTACKS[from] & !(blockers | forbidden);
    list.extend(moves.map(|to| Move::new(from, to, None)));
}

pub(super) const KING_ATTACKS: [Bitboard; 64] = {
    let mut attacks = [Bitboard::EMPTY; 64];
    let mut square = 0;

    while square < 64 {
        let bb = bitboard!(square);
        attacks[square] = bb.shift::<{ Direction::North }>()
            | bb.shift::<{ Direction::South }>()
            | bb.shift::<{ Direction::East }>()
            | bb.shift::<{ Direction::West }>()
            | bb.shift::<{ Direction::Northeast }>()
            | bb.shift::<{ Direction::Northwest }>()
            | bb.shift::<{ Direction::Southeast }>()
            | bb.shift::<{ Direction::Southwest }>();
        square += 1;
    }

    attacks
};

#[cfg(test)]
mod tests {
    use crate::{
        attacks::king_forbidden_squares,
        board::Board,
        game::Color,
        moves::MoveList,
        test_utils::{MoveCase, assert_move_cases, board, moves},
    };

    use super::add_king_moves;

    fn king_moves(board: Board) -> MoveList {
        let mut moves = MoveList::default();
        let occupied = board.occupied();
        let forbidden = king_forbidden_squares::<{ Color::Black }>(&board, occupied);

        add_king_moves::<{ Color::White }>(
            &board,
            board.occupancy::<{ Color::White }>(),
            forbidden,
            &mut moves,
        );

        moves
    }

    #[test]
    fn generates_king_moves() {
        assert_move_cases(
            [
                MoveCase {
                    name: "king from center on empty board",
                    board: board!(
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . K . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                    ),
                    moves: moves!(
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . x x x . . .
                        . . x o x . . .
                        . . x x x . . .
                        . . . . . . . .
                        . . . . . . . .
                    ),
                },
                MoveCase {
                    name: "king from corner",
                    board: board!(
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        K . . . . . . .
                    ),
                    moves: moves!(
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        x x . . . . . .
                        o x . . . . . .
                    ),
                },
                MoveCase {
                    name: "king excludes own blockers and includes enemy blockers",
                    board: board!(
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . P K p . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                    ),
                    moves: moves!(
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . x x x . .
                        . . . . o x . .
                        . . . x . x . .
                        . . . . . . . .
                        . . . . . . . .
                    ),
                },
                MoveCase {
                    name: "king cannot retreat along a checking slider ray",
                    board: board!(
                        . . . . r . . .
                        . . . . K . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                    ),
                    moves: moves!(
                        . . . . x . . .
                        . . . x o x . .
                        . . . x . x . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                    ),
                },
                MoveCase {
                    name: "king cannot capture a defended enemy piece",
                    board: board!(
                        . . . . . r . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . K p . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                    ),
                    moves: moves!(
                        . . . . . . . .
                        . . . . . . . .
                        . . . . . . . .
                        . . . x x . . .
                        . . . x o . . .
                        . . . x . x . .
                        . . . . . . . .
                        . . . . . . . .
                    ),
                },
            ],
            king_moves,
        );
    }
}

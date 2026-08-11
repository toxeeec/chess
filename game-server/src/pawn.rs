use crate::{
    bitboard::Bitboard,
    board::Board,
    game::Color,
    moves::{Move, MoveList},
};

pub(super) fn add_pawn_moves<const COLOR: Color>(
    board: &Board,
    empty: Bitboard,
    enemies: Bitboard,
    list: &mut MoveList,
) {
    let pawns = board.pawns::<COLOR>();

    let single_pushes =
        ((pawns & !Bitboard::relative_rank::<COLOR>(7)).forward::<COLOR, 1>()) & empty;
    let double_pushes =
        ((single_pushes & Bitboard::relative_rank::<COLOR>(3)).forward::<COLOR, 1>()) & empty;
    let west_captures = pawns.forward_west::<COLOR>() & enemies;
    let east_captures = pawns.forward_east::<COLOR>() & enemies;

    list.reserve(
        single_pushes.len() + double_pushes.len() + west_captures.len() + east_captures.len(),
    );
    list.extend(single_pushes.map(|to| Move::new(to.backward::<COLOR, 1>(), to)));
    list.extend(double_pushes.map(|to| Move::new(to.backward::<COLOR, 2>(), to)));
    list.extend(west_captures.map(|to| Move::new(to.backward_east::<COLOR>(), to)));
    list.extend(east_captures.map(|to| Move::new(to.backward_west::<COLOR>(), to)));
}

#[cfg(test)]
mod tests {
    use crate::{
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

        add_pawn_moves::<COLOR>(&board, !occupied, occupied & !blockers, &mut moves);

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
}

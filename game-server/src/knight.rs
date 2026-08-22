use crate::{
    attacks::PinRays,
    bitboard::{Bitboard, Direction},
    board::Board,
    game::Color,
    moves::{Move, MoveList},
    square::Square,
};

pub(super) fn add_knight_moves<const COLOR: Color>(
    board: &Board,
    blockers: Bitboard,
    evasion_mask: Bitboard,
    pin_rays: PinRays,
    list: &mut MoveList,
) {
    let knights = board.knights::<COLOR>() & !pin_rays.pinned_pieces(blockers);

    for from in knights {
        let moves = KNIGHT_ATTACKS[from] & !blockers & evasion_mask;
        list.extend(moves.map(|to| Move::new(from, to, None)));
    }
}

pub(super) const KNIGHT_ATTACKS: [Bitboard; 64] = {
    let mut attacks = [Bitboard::EMPTY; 64];
    let mut square = 0;

    while square < 64 {
        let bb = Bitboard::from(Square::new(square as u32));
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
        attacks::PinRays,
        bitboard::Bitboard,
        board::Board,
        game::Color,
        moves::MoveList,
        squares,
        test_utils::{MoveCase, assert_move_case, assert_move_cases, board, moves},
    };

    use super::add_knight_moves;

    fn knight_moves(board: Board, pin_rays: PinRays) -> MoveList {
        let mut moves = MoveList::default();

        add_knight_moves::<{ Color::White }>(
            &board,
            board.occupancy::<{ Color::White }>(),
            Bitboard::FULL,
            pin_rays,
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
            |board| knight_moves(board, PinRays::EMPTY),
        );
    }

    #[test]
    fn excludes_pinned_knights() {
        let pin_rays = PinRays::orthogonal(Bitboard::from(squares![e2, e3, e4, e5, e6, e7, e8]));

        assert_move_case(
            MoveCase {
                name: "pinned knight",
                board: board!(
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . . . . .
                    . . . . N . . .
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
            |board| knight_moves(board, pin_rays),
        );
    }
}

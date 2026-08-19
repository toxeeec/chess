use crate::{
    bitboard::Bitboard,
    board::Board,
    game::Color,
    king::KING_ATTACKS,
    knight::KNIGHT_ATTACKS,
    magics::{bishop_attacks, rook_attacks},
};

pub(super) fn king_forbidden_squares<const ENEMY: Color>(
    board: &Board,
    occupied: Bitboard,
) -> Bitboard
where
    [(); { !ENEMY } as usize]:,
{
    let occupied = occupied & !board.king::<{ !ENEMY }>();
    let pawns = board.pawns::<ENEMY>();
    let mut forbidden = pawns.forward_west::<ENEMY>() | pawns.forward_east::<ENEMY>();

    for square in board.knights::<ENEMY>() {
        forbidden |= KNIGHT_ATTACKS[square];
    }
    for square in board.bishops::<ENEMY>() {
        forbidden |= bishop_attacks(square, occupied);
    }
    for square in board.rooks::<ENEMY>() {
        forbidden |= rook_attacks(square, occupied);
    }
    for square in board.queens::<ENEMY>() {
        forbidden |= bishop_attacks(square, occupied) | rook_attacks(square, occupied);
    }
    for square in board.king::<ENEMY>() {
        forbidden |= KING_ATTACKS[square];
    }

    forbidden
}

#[cfg(test)]
mod tests {
    use crate::{game::Color, squares, test_utils::board};

    use super::king_forbidden_squares;

    #[test]
    fn king_forbidden_squares_include_attacks_from_every_piece_type() {
        let board = board!(
            q . . . . . . k
            . . . . . . . .
            b . . . . n . .
            . . . p . . . r
            . . . . K . . .
            . . . . . . . .
            . . . . . . . .
            . . . . . . . .
        );
        let forbidden = king_forbidden_squares::<{ Color::Black }>(&board, board.occupied());

        for square in squares![c4, e4, e2, e5, d8, g7] {
            assert!(forbidden.contains(square), "{square:?} should be forbidden");
        }
    }
}

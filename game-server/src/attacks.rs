use crate::{
    bitboard::Bitboard,
    board::Board,
    game::Color,
    king::KING_ATTACKS,
    knight::KNIGHT_ATTACKS,
    magics::{bishop_attacks, rook_attacks},
    square::Square,
};

pub(super) struct KingThreats {
    pub(super) attackers: Bitboard,
    pub(super) forbidden: Bitboard,
}

pub(super) fn king_threats<const ENEMY: Color>(board: &Board, occupied: Bitboard) -> KingThreats
where
    [(); { !ENEMY } as usize]:,
{
    let king = board.king::<{ !ENEMY }>();
    let king_square = board.king_square::<{ !ENEMY }>();
    let occupied = occupied & !king;
    let diagonal_sliders = board.bishops::<ENEMY>() | board.queens::<ENEMY>();
    let orthogonal_sliders = board.rooks::<ENEMY>() | board.queens::<ENEMY>();
    let pawn_attackers = (king.forward_west::<{ !ENEMY }>() | king.forward_east::<{ !ENEMY }>())
        & board.pawns::<ENEMY>();

    let attackers = pawn_attackers
        | (KNIGHT_ATTACKS[king_square] & board.knights::<ENEMY>())
        | (bishop_attacks(king_square, occupied) & diagonal_sliders)
        | (rook_attacks(king_square, occupied) & orthogonal_sliders);
    let pawns = board.pawns::<ENEMY>();
    let mut forbidden = pawns.forward_west::<ENEMY>() | pawns.forward_east::<ENEMY>();

    for square in board.knights::<ENEMY>() {
        forbidden |= KNIGHT_ATTACKS[square];
    }
    for square in diagonal_sliders {
        forbidden |= bishop_attacks(square, occupied);
    }
    for square in orthogonal_sliders {
        forbidden |= rook_attacks(square, occupied);
    }
    for square in board.king::<ENEMY>() {
        forbidden |= KING_ATTACKS[square];
    }

    KingThreats {
        attackers,
        forbidden,
    }
}

pub(super) fn evasion_mask(king: Square, mut attackers: Bitboard) -> Bitboard {
    if attackers.empty() {
        return Bitboard::FULL;
    }
    if attackers.len() > 1 {
        return Bitboard::EMPTY;
    }

    let attacker = unsafe { attackers.next().unwrap_unchecked() };

    EVASION_MASKS[king][attacker]
}

static EVASION_MASKS: [[Bitboard; 64]; 64] = {
    let mut masks = [[Bitboard::EMPTY; 64]; 64];
    let mut from = 0;

    while from < 64 {
        let from_file = from % 8;
        let from_rank = from / 8;
        let mut to = 0;

        while to < 64 {
            let to_file = to % 8;
            let to_rank = to / 8;
            let file_delta = to_file as i32 - from_file as i32;
            let rank_delta = to_rank as i32 - from_rank as i32;
            let aligned =
                file_delta == 0 || rank_delta == 0 || file_delta.abs() == rank_delta.abs();
            let mut squares = 1 << to;

            if aligned && from != to {
                let file_step = file_delta.signum();
                let rank_step = rank_delta.signum();
                let mut file = from_file as i32 + file_step;
                let mut rank = from_rank as i32 + rank_step;

                while file != to_file as i32 || rank != to_rank as i32 {
                    squares |= 1 << (rank * 8 + file);
                    file += file_step;
                    rank += rank_step;
                }
            }

            masks[from][to] = Bitboard::new(squares);
            to += 1;
        }
        from += 1;
    }

    masks
};

#[cfg(test)]
mod tests {
    use crate::{
        bitboard::Bitboard, board::Board, game::Color, square, squares, test_utils::board,
    };

    use super::{EVASION_MASKS, king_threats};

    #[test]
    fn evasion_masks_include_attacker_and_intermediate_squares() {
        for (from, to, expected) in [
            (
                square!(a1),
                square!(a8),
                Bitboard::from(squares![a2, a3, a4, a5, a6, a7, a8]),
            ),
            (
                square!(a4),
                square!(h4),
                Bitboard::from(squares![b4, c4, d4, e4, f4, g4, h4]),
            ),
            (
                square!(a1),
                square!(h8),
                Bitboard::from(squares![b2, c3, d4, e5, f6, g7, h8]),
            ),
            (
                square!(h1),
                square!(a8),
                Bitboard::from(squares![g2, f3, e4, d5, c6, b7, a8]),
            ),
            (square!(a1), square!(a2), Bitboard::from(square!(a2))),
            (square!(a1), square!(c2), Bitboard::from(square!(c2))),
        ] {
            assert_eq!(EVASION_MASKS[from][to], expected);
        }
    }

    #[test]
    fn king_attackers_find_pawns_for_each_color() {
        let white = Board::from_fen("7k/8/8/3p4/4K3/8/8/8").unwrap();
        let black = Board::from_fen("8/8/8/4k3/3P4/8/8/7K").unwrap();

        assert_eq!(
            king_threats::<{ Color::Black }>(&white, white.occupied()).attackers,
            Bitboard::from(square!(d5))
        );
        assert_eq!(
            king_threats::<{ Color::White }>(&black, black.occupied()).attackers,
            Bitboard::from(square!(d4))
        );
    }

    #[test]
    fn king_attackers_find_knights_and_sliders() {
        for (placement, expected) in [
            ("7k/8/8/2n5/4K3/8/8/8", square!(c5)),
            ("b6k/8/8/8/4K3/8/8/8", square!(a8)),
            ("4r2k/8/8/8/4K3/8/8/8", square!(e8)),
            ("7k/7q/8/8/4K3/8/8/8", square!(h7)),
            ("7k/8/8/8/q3K3/8/8/8", square!(a4)),
        ] {
            let board = Board::from_fen(placement).unwrap();
            assert_eq!(
                king_threats::<{ Color::Black }>(&board, board.occupied()).attackers,
                Bitboard::from(expected),
                "failed for {placement}"
            );
        }
    }

    #[test]
    fn king_attackers_respect_blockers_and_preserve_multiple_attackers() {
        let blocked = Board::from_fen("b6k/8/2p5/8/4K3/8/8/8").unwrap();
        let double = Board::from_fen("4r2k/8/8/2n5/4K3/8/8/8").unwrap();
        let quiet = Board::from_fen("7k/8/8/8/4K3/8/8/8").unwrap();

        assert_eq!(
            king_threats::<{ Color::Black }>(&blocked, blocked.occupied()).attackers,
            Bitboard::EMPTY
        );
        assert_eq!(
            king_threats::<{ Color::Black }>(&double, double.occupied()).attackers,
            Bitboard::from(squares![e8, c5])
        );
        assert_eq!(
            king_threats::<{ Color::Black }>(&quiet, quiet.occupied()).attackers,
            Bitboard::EMPTY
        );
    }

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
        let forbidden = king_threats::<{ Color::Black }>(&board, board.occupied()).forbidden;

        for square in squares![c4, e4, e2, e5, d8, g7] {
            assert!(forbidden.contains(square), "{square:?} should be forbidden");
        }
    }
}

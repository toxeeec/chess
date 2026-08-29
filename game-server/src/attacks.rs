use crate::{
    bitboard::Bitboard,
    board::Board,
    king::KING_ATTACKS,
    knight::KNIGHT_ATTACKS,
    magics::{bishop_attacks, rook_attacks},
    square::Square,
    state::{Color, OPPONENT},
};

pub(super) struct KingThreats {
    pub(super) attackers: Bitboard,
    pub(super) forbidden: Bitboard,
    pub(super) pin_rays: PinRays,
}

#[derive(Clone, Copy)]
pub(super) struct PinRays {
    pub(super) diagonal: Bitboard,
    pub(super) orthogonal: Bitboard,
}

impl PinRays {
    #[cfg(test)]
    pub(super) const EMPTY: Self = Self {
        diagonal: Bitboard::EMPTY,
        orthogonal: Bitboard::EMPTY,
    };

    pub(super) fn pinned_pieces(self, pieces: Bitboard) -> Bitboard {
        (self.diagonal | self.orthogonal) & pieces
    }

    #[cfg(test)]
    pub(super) fn diagonal(ray: Bitboard) -> Self {
        Self {
            diagonal: ray,
            orthogonal: Bitboard::EMPTY,
        }
    }

    #[cfg(test)]
    pub(super) fn orthogonal(ray: Bitboard) -> Self {
        Self {
            diagonal: Bitboard::EMPTY,
            orthogonal: ray,
        }
    }
}

pub(super) fn king_threats<const ENEMY: Color>(
    board: &Board,
    occupied: Bitboard,
    blockers: Bitboard,
) -> KingThreats {
    let king = board.king::<{ OPPONENT::<ENEMY> }>();
    let king_square = board.king_square::<{ OPPONENT::<ENEMY> }>();
    let occupied = occupied & !king;

    let diagonal_sliders = board.bishops::<ENEMY>() | board.queens::<ENEMY>();
    let orthogonal_sliders = board.rooks::<ENEMY>() | board.queens::<ENEMY>();
    let diagonal_attacks = bishop_attacks(king_square, occupied);
    let orthogonal_attacks = rook_attacks(king_square, occupied);

    let pawn_attackers = (king.forward_west::<{ OPPONENT::<ENEMY> }>()
        | king.forward_east::<{ OPPONENT::<ENEMY> }>())
        & board.pawns::<ENEMY>();
    let attackers = pawn_attackers
        | (KNIGHT_ATTACKS[king_square] & board.knights::<ENEMY>())
        | (diagonal_attacks & diagonal_sliders)
        | (orthogonal_attacks & orthogonal_sliders);

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

    let diagonal_blockers = diagonal_attacks & blockers;
    let orthogonal_blockers = orthogonal_attacks & blockers;
    let diagonal_pinners =
        bishop_attacks(king_square, occupied & !diagonal_blockers) & diagonal_sliders;
    let orthogonal_pinners =
        rook_attacks(king_square, occupied & !orthogonal_blockers) & orthogonal_sliders;
    let mut diagonal_pins = Bitboard::EMPTY;
    let mut orthogonal_pins = Bitboard::EMPTY;

    for pinner in diagonal_pinners {
        diagonal_pins |= RAY_MASKS[king_square][pinner];
    }
    for pinner in orthogonal_pinners {
        orthogonal_pins |= RAY_MASKS[king_square][pinner];
    }

    KingThreats {
        attackers,
        forbidden,
        pin_rays: PinRays {
            diagonal: diagonal_pins,
            orthogonal: orthogonal_pins,
        },
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

    RAY_MASKS[king][attacker]
}

static RAY_MASKS: [[Bitboard; 64]; 64] = {
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
    use crate::{bitboard::Bitboard, square, squares, state::Color, test_utils::board};

    use super::{RAY_MASKS, king_threats};

    #[test]
    fn ray_masks_include_destination_and_intermediate_squares() {
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
            assert_eq!(RAY_MASKS[from][to], expected);
        }
    }

    #[test]
    fn king_attackers_find_every_piece_type() {
        let board = board!(
            . . . . r . . .
            . . . . . . . q
            . . . . . . . .
            . . n p . . . .
            q . . . K . . .
            . . . . . . . .
            . . . . . . . .
            . b . . . . . .
        );
        let occupied = board.occupied();
        let blockers = board.occupancy::<{ Color::White }>();

        assert_eq!(
            king_threats::<{ Color::Black }>(&board, occupied, blockers).attackers,
            Bitboard::from(squares![b1, a4, c5, d5, e8, h7])
        );
    }

    #[test]
    fn king_attackers_respect_blockers_and_preserve_multiple_attackers() {
        let blocked = board!(
            b . . . . . . .
            . . . . . . . .
            . . p . . . . .
            . . . . . . . .
            . . . . K . . .
            . . . . . . . .
            . . . . . . . .
            . . . . . . . .
        );
        let occupied = blocked.occupied();
        let blockers = blocked.occupancy::<{ Color::White }>();
        assert_eq!(
            king_threats::<{ Color::Black }>(&blocked, occupied, blockers).attackers,
            Bitboard::EMPTY
        );

        let double = board!(
            . . . . r . . .
            . . . . . . . .
            . . . . . . . .
            . . n . . . . .
            . . . . K . . .
            . . . . . . . .
            . . . . . . . .
            . . . . . . . .
        );
        let occupied = double.occupied();
        let blockers = double.occupancy::<{ Color::White }>();
        assert_eq!(
            king_threats::<{ Color::Black }>(&double, occupied, blockers).attackers,
            Bitboard::from(squares![e8, c5])
        );

        let quiet = board!(
            . . . . . . . .
            . . . . . . . .
            . . . . . . . .
            . . . . . . . .
            . . . . K . . .
            . . . . . . . .
            . . . . . . . .
            . . . . . . . .
        );
        let occupied = quiet.occupied();
        let blockers = quiet.occupancy::<{ Color::White }>();
        assert_eq!(
            king_threats::<{ Color::Black }>(&quiet, occupied, blockers).attackers,
            Bitboard::EMPTY
        );
    }

    #[test]
    fn king_threats_collect_pin_rays_by_slider_type() {
        let board = board!(
            . . . . r . . .
            . . . . . . . .
            . . . . . . . .
            b . . . . . . .
            . . . . . . . .
            . . . . . . . .
            . . . B R . . .
            . . . . K . . .
        );
        let occupied = board.occupied();
        let blockers = board.occupancy::<{ Color::White }>();
        let threats = king_threats::<{ Color::Black }>(&board, occupied, blockers);

        assert_eq!(
            threats.pin_rays.diagonal,
            Bitboard::from(squares![d2, c3, b4, a5])
        );
        assert_eq!(
            threats.pin_rays.orthogonal,
            Bitboard::from(squares![e2, e3, e4, e5, e6, e7, e8])
        );

        let double_blocked = board!(
            . . . . r . . .
            . . . . . . . .
            . . . . . . . .
            . . . . . . . .
            . . . . . . . .
            . . . . B . . .
            . . . . R . . .
            . . . . K . . .
        );
        let occupied = double_blocked.occupied();
        let blockers = double_blocked.occupancy::<{ Color::White }>();
        let threats = king_threats::<{ Color::Black }>(&double_blocked, occupied, blockers);

        assert_eq!(threats.pin_rays.orthogonal, Bitboard::EMPTY);
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
        let occupied = board.occupied();
        let blockers = board.occupancy::<{ Color::White }>();
        let forbidden = king_threats::<{ Color::Black }>(&board, occupied, blockers).forbidden;

        for square in squares![c4, e4, e2, e5, d8, g7] {
            assert!(forbidden.contains(square), "{square:?} should be forbidden");
        }
    }
}

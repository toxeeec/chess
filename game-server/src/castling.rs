use std::fmt;

use anyhow::{Result, bail};

use crate::{
    bitboard::Bitboard,
    board::Board,
    game::Color,
    moves::{Move, MoveList},
    square,
    square::Square,
    squares,
};

#[derive(Clone, Copy)]
pub(super) struct CastlingRights(u8);

struct CastlingConfig {
    from: Square,
    queen_right: CastlingRights,
    king_right: CastlingRights,
    queen_rook: Square,
    king_rook: Square,
    queen_empty: Bitboard,
    king_empty: Bitboard,
    queen_safe: Bitboard,
    king_safe: Bitboard,
    queen_to: Square,
    king_to: Square,
}

impl CastlingRights {
    const WHITE_QUEENSIDE: Self = Self(1 << 0);
    const WHITE_KINGSIDE: Self = Self(1 << 1);
    const BLACK_QUEENSIDE: Self = Self(1 << 2);
    const BLACK_KINGSIDE: Self = Self(1 << 3);
    pub(super) const NONE: Self = Self(0);
    pub(super) const ALL: Self = Self(0b1111);

    pub(super) fn from_fen(value: &str, board: &Board) -> Result<Self> {
        if value == "-" {
            return Ok(Self::NONE);
        }

        let mut rights = Self::NONE;
        for right in value.bytes() {
            let castling_right = match right {
                b'K' => Self::WHITE_KINGSIDE,
                b'Q' => Self::WHITE_QUEENSIDE,
                b'k' => Self::BLACK_KINGSIDE,
                b'q' => Self::BLACK_QUEENSIDE,
                _ => bail!("invalid FEN castling right: {}", right as char),
            };

            if rights.contains(castling_right) {
                bail!("duplicate FEN castling right: {}", right as char);
            }
            rights.0 |= castling_right.0;
        }

        if rights.0 & (Self::WHITE_QUEENSIDE.0 | Self::WHITE_KINGSIDE.0) != 0
            && board.king_square::<{ Color::White }>() != CastlingConfig::WHITE.from
        {
            bail!("white castling rights require the white king on e1");
        }
        if rights.0 & (Self::BLACK_QUEENSIDE.0 | Self::BLACK_KINGSIDE.0) != 0
            && board.king_square::<{ Color::Black }>() != CastlingConfig::BLACK.from
        {
            bail!("black castling rights require the black king on e8");
        }
        let white_rooks = board.rooks::<{ Color::White }>();
        if rights.contains(Self::WHITE_QUEENSIDE)
            && !white_rooks.contains(CastlingConfig::WHITE.queen_rook)
        {
            bail!("white queenside castling rights require a white rook on a1");
        }
        if rights.contains(Self::WHITE_KINGSIDE)
            && !white_rooks.contains(CastlingConfig::WHITE.king_rook)
        {
            bail!("white kingside castling rights require a white rook on h1");
        }
        let black_rooks = board.rooks::<{ Color::Black }>();
        if rights.contains(Self::BLACK_QUEENSIDE)
            && !black_rooks.contains(CastlingConfig::BLACK.queen_rook)
        {
            bail!("black queenside castling rights require a black rook on a8");
        }
        if rights.contains(Self::BLACK_KINGSIDE)
            && !black_rooks.contains(CastlingConfig::BLACK.king_rook)
        {
            bail!("black kingside castling rights require a black rook on h8");
        }

        Ok(rights)
    }

    fn contains(self, rights: Self) -> bool {
        self.0 & rights.0 != 0
    }

    pub(super) fn update(&mut self, from: Square, to: Square) {
        self.0 &= RETAIN_CASTLING_RIGHTS[from] & RETAIN_CASTLING_RIGHTS[to];
    }
}

impl CastlingConfig {
    const WHITE: Self = Self {
        from: square!(e1),
        queen_right: CastlingRights::WHITE_QUEENSIDE,
        king_right: CastlingRights::WHITE_KINGSIDE,
        queen_rook: square!(a1),
        king_rook: square!(h1),
        queen_empty: squares![b1, c1, d1].into(),
        king_empty: squares![f1, g1].into(),
        queen_safe: squares![c1, d1].into(),
        king_safe: squares![f1, g1].into(),
        queen_to: square!(c1),
        king_to: square!(g1),
    };

    const BLACK: Self = Self {
        from: square!(e8),
        queen_right: CastlingRights::BLACK_QUEENSIDE,
        king_right: CastlingRights::BLACK_KINGSIDE,
        queen_rook: square!(a8),
        king_rook: square!(h8),
        queen_empty: squares![b8, c8, d8].into(),
        king_empty: squares![f8, g8].into(),
        queen_safe: squares![c8, d8].into(),
        king_safe: squares![f8, g8].into(),
        queen_to: square!(c8),
        king_to: square!(g8),
    };
}

const RETAIN_CASTLING_RIGHTS: [u8; 64] = {
    let mut masks = [CastlingRights::ALL.0; 64];
    masks[square!(a1)] = !CastlingRights::WHITE_QUEENSIDE.0;
    masks[square!(e1)] = !(CastlingRights::WHITE_QUEENSIDE.0 | CastlingRights::WHITE_KINGSIDE.0);
    masks[square!(h1)] = !CastlingRights::WHITE_KINGSIDE.0;
    masks[square!(a8)] = !CastlingRights::BLACK_QUEENSIDE.0;
    masks[square!(e8)] = !(CastlingRights::BLACK_QUEENSIDE.0 | CastlingRights::BLACK_KINGSIDE.0);
    masks[square!(h8)] = !CastlingRights::BLACK_KINGSIDE.0;
    masks
};

pub(super) fn add_castling_moves<const COLOR: Color>(
    board: &Board,
    occupied: Bitboard,
    attackers: Bitboard,
    forbidden: Bitboard,
    rights: CastlingRights,
    list: &mut MoveList,
) {
    let config = match COLOR {
        Color::White => CastlingConfig::WHITE,
        Color::Black => CastlingConfig::BLACK,
    };
    if rights.0 & (config.queen_right.0 | config.king_right.0) == 0 {
        return;
    }

    let from = config.from;
    debug_assert_eq!(board.king_square::<COLOR>(), from);
    let ready = attackers.empty();
    debug_assert!(
        !rights.contains(config.queen_right) || board.rooks::<COLOR>().contains(config.queen_rook)
    );
    debug_assert!(
        !rights.contains(config.king_right) || board.rooks::<COLOR>().contains(config.king_rook)
    );
    let queen_side = ready
        & rights.contains(config.queen_right)
        & (occupied & config.queen_empty).empty()
        & (forbidden & config.queen_safe).empty();
    let king_side = ready
        & rights.contains(config.king_right)
        & (occupied & config.king_empty).empty()
        & (forbidden & config.king_safe).empty();
    let castles = Bitboard::new(
        ((queen_side as u64) << usize::from(config.queen_to))
            | ((king_side as u64) << usize::from(config.king_to)),
    );
    list.extend(castles.map(|to| Move::new(from, to, None)));
}

impl fmt::Display for CastlingRights {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == 0 {
            return f.write_str("-");
        }

        for (castling_right, right) in [
            (Self::WHITE_KINGSIDE, 'K'),
            (Self::WHITE_QUEENSIDE, 'Q'),
            (Self::BLACK_KINGSIDE, 'k'),
            (Self::BLACK_QUEENSIDE, 'q'),
        ] {
            if self.contains(castling_right) {
                write!(f, "{right}")?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        castling::CastlingRights,
        game::{Color, Game, State},
        test_utils::board,
    };

    fn has_move(game: &Game, mve: &str) -> bool {
        game.moves.contains(mve.parse().unwrap())
    }

    #[test]
    fn generates_castling_moves_only_when_the_path_is_clear_and_safe() {
        let clear = Game::new(
            board!(
                r . . . k . . r
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                R . . . K . . R
            ),
            State::new(Color::White, CastlingRights::ALL),
        );
        assert!(has_move(&clear, "e1c1"));
        assert!(has_move(&clear, "e1g1"));

        let no_rights = Game::new(
            board!(
                r . . . k . . r
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                R . . . K . . R
            ),
            State::new(Color::White, CastlingRights::NONE),
        );
        assert!(!has_move(&no_rights, "e1c1"));
        assert!(!has_move(&no_rights, "e1g1"));

        let blocked = Game::new(
            board!(
                . . . . k . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                R N . . K . . R
            ),
            State::new(Color::White, CastlingRights::ALL),
        );
        assert!(!has_move(&blocked, "e1c1"));
        assert!(has_move(&blocked, "e1g1"));

        let in_check = Game::new(
            board!(
                . . . . r . . k
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                R . . . K . . R
            ),
            State::new(Color::White, CastlingRights::ALL),
        );
        assert!(!has_move(&in_check, "e1c1"));
        assert!(!has_move(&in_check, "e1g1"));

        let attacked_transit = Game::new(
            board!(
                . . . . k r . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                R . . . K . . R
            ),
            State::new(Color::White, CastlingRights::ALL),
        );
        assert!(has_move(&attacked_transit, "e1c1"));
        assert!(!has_move(&attacked_transit, "e1g1"));

        let black = Game::new(
            board!(
                r . . . k . . r
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . K . . .
            ),
            State::new(Color::Black, CastlingRights::ALL),
        );
        assert!(has_move(&black, "e8c8"));
        assert!(has_move(&black, "e8g8"));
    }
}

use std::fmt;

use anyhow::{Result, bail};

use crate::{
    bitboard::Bitboard,
    moves::{Move, PromotionPiece},
    square,
    square::Square,
    squares,
    state::Color,
};

pub(super) struct Board {
    white_pawns: Bitboard,
    white_rooks: Bitboard,
    white_knights: Bitboard,
    white_bishops: Bitboard,
    white_queens: Bitboard,
    white_king: Bitboard,
    black_pawns: Bitboard,
    black_rooks: Bitboard,
    black_knights: Bitboard,
    black_bishops: Bitboard,
    black_queens: Bitboard,
    black_king: Bitboard,
}

impl Default for Board {
    fn default() -> Self {
        let white_pawns = Bitboard::from(squares![a2, b2, c2, d2, e2, f2, g2, h2]);
        let white_rooks = Bitboard::from(squares![a1, h1]);
        let white_knights = Bitboard::from(squares![b1, g1]);
        let white_bishops = Bitboard::from(squares![c1, f1]);
        let white_queens = Bitboard::from(square!(d1));
        let white_king = Bitboard::from(square!(e1));

        let black_pawns = Bitboard::from(squares![a7, b7, c7, d7, e7, f7, g7, h7]);
        let black_rooks = Bitboard::from(squares![a8, h8]);
        let black_knights = Bitboard::from(squares![b8, g8]);
        let black_bishops = Bitboard::from(squares![c8, f8]);
        let black_queens = Bitboard::from(square!(d8));
        let black_king = Bitboard::from(square!(e8));

        Self {
            white_pawns,
            white_rooks,
            white_knights,
            white_bishops,
            white_queens,
            white_king,
            black_pawns,
            black_rooks,
            black_knights,
            black_bishops,
            black_queens,
            black_king,
        }
    }
}

impl Board {
    pub(super) fn from_fen(placement: &str) -> Result<Self> {
        let board = Self::parse_fen_placement(placement)?;

        if board.king::<{ Color::White }>().len() != 1
            || board.king::<{ Color::Black }>().len() != 1
        {
            bail!("FEN position must contain exactly one king of each color");
        }

        Ok(board)
    }

    fn parse_fen_placement(placement: &str) -> Result<Self> {
        let mut board = Self {
            white_pawns: Bitboard::EMPTY,
            white_rooks: Bitboard::EMPTY,
            white_knights: Bitboard::EMPTY,
            white_bishops: Bitboard::EMPTY,
            white_queens: Bitboard::EMPTY,
            white_king: Bitboard::EMPTY,
            black_pawns: Bitboard::EMPTY,
            black_rooks: Bitboard::EMPTY,
            black_knights: Bitboard::EMPTY,
            black_bishops: Bitboard::EMPTY,
            black_queens: Bitboard::EMPTY,
            black_king: Bitboard::EMPTY,
        };

        let ranks = placement.split('/').collect::<Vec<_>>();
        if ranks.len() != 8 {
            bail!("FEN piece placement must contain 8 ranks");
        }

        for (rank_index, rank) in ranks.into_iter().enumerate() {
            let board_rank = 7 - rank_index as u32;
            let mut file = 0;

            for piece in rank.chars() {
                if let Some(empty) = piece.to_digit(10) {
                    if !(1..=8).contains(&empty) {
                        bail!("FEN empty square count must be between 1 and 8");
                    }
                    file += empty;
                    continue;
                }

                if file >= 8 {
                    bail!("FEN rank contains too many squares");
                }

                let square = Square::new(board_rank * 8 + file);
                board.add_piece(piece, square)?;
                file += 1;
            }

            if file != 8 {
                bail!("FEN rank does not contain 8 squares");
            }
        }

        Ok(board)
    }

    #[cfg(test)]
    pub(super) fn from_ascii(ascii: &str) -> Self {
        let mut fen = String::new();
        let squares = ascii
            .chars()
            .filter(|square| !square.is_whitespace())
            .collect::<Vec<_>>();

        assert_eq!(squares.len(), 64, "board must contain 64 squares");

        for (index, square) in squares.into_iter().enumerate() {
            if index > 0 && index % 8 == 0 {
                fen.push('/');
            }

            match square {
                '.' => match fen.pop() {
                    Some(previous) if previous.is_ascii_digit() => {
                        fen.push(char::from(previous as u8 + 1));
                    }
                    Some(previous) => {
                        fen.push(previous);
                        fen.push('1');
                    }
                    None => fen.push('1'),
                },
                piece => fen.push(piece),
            }
        }

        Self::parse_fen_placement(&fen).expect("board! generated invalid FEN")
    }

    pub(super) const fn pawns<const COLOR: Color>(&self) -> Bitboard {
        match COLOR {
            Color::White => self.white_pawns,
            Color::Black => self.black_pawns,
        }
    }

    pub(super) const fn knights<const COLOR: Color>(&self) -> Bitboard {
        match COLOR {
            Color::White => self.white_knights,
            Color::Black => self.black_knights,
        }
    }

    pub(super) const fn rooks<const COLOR: Color>(&self) -> Bitboard {
        match COLOR {
            Color::White => self.white_rooks,
            Color::Black => self.black_rooks,
        }
    }

    pub(super) const fn bishops<const COLOR: Color>(&self) -> Bitboard {
        match COLOR {
            Color::White => self.white_bishops,
            Color::Black => self.black_bishops,
        }
    }

    pub(super) const fn queens<const COLOR: Color>(&self) -> Bitboard {
        match COLOR {
            Color::White => self.white_queens,
            Color::Black => self.black_queens,
        }
    }

    pub(super) const fn king<const COLOR: Color>(&self) -> Bitboard {
        match COLOR {
            Color::White => self.white_king,
            Color::Black => self.black_king,
        }
    }

    pub(super) fn king_square<const COLOR: Color>(&self) -> Square {
        let mut king = self.king::<COLOR>();
        debug_assert_eq!(king.len(), 1);

        unsafe { king.next().unwrap_unchecked() }
    }

    pub(super) const fn occupancy<const COLOR: Color>(&self) -> Bitboard {
        match COLOR {
            Color::White => {
                self.white_pawns
                    | self.white_rooks
                    | self.white_knights
                    | self.white_bishops
                    | self.white_queens
                    | self.white_king
            }
            Color::Black => {
                self.black_pawns
                    | self.black_rooks
                    | self.black_knights
                    | self.black_bishops
                    | self.black_queens
                    | self.black_king
            }
        }
    }

    pub(super) fn apply_move(&mut self, color: Color, mve: Move) {
        const CASTLING_ROOK_MOVES: [[(Square, Square); 2]; 2] = [
            [(square!(a1), square!(d1)), (square!(h1), square!(f1))],
            [(square!(a8), square!(d8)), (square!(h8), square!(f8))],
        ];

        let is_castling = (self.white_king | self.black_king).contains(mve.from)
            & (mve.from.file().abs_diff(mve.to.file()) == 2);
        let is_en_passant = (self.white_pawns | self.black_pawns).contains(mve.from)
            & (mve.from.file() != mve.to.file())
            & !self.occupied().contains(mve.to);

        if is_en_passant {
            let captured = match color {
                Color::White => mve.to.backward::<{ Color::White }, 1>(),
                Color::Black => mve.to.backward::<{ Color::Black }, 1>(),
            };
            self.white_pawns &= !Bitboard::from(captured);
            self.black_pawns &= !Bitboard::from(captured);
        }

        self.for_each_bb_mut(|piece| {
            piece.apply_move(mve.from, mve.to);
        });

        if is_castling {
            let king_side = (mve.to.file() > mve.from.file()) as usize;
            let (rook_from, rook_to) = CASTLING_ROOK_MOVES[color as usize][king_side];
            match color {
                Color::White => self.white_rooks.apply_move(rook_from, rook_to),
                Color::Black => self.black_rooks.apply_move(rook_from, rook_to),
            }
        }

        if let Some(promotion) = mve.promotion {
            self.promote(color, promotion, mve.to);
        }
    }

    pub(super) fn fen(&self) -> String {
        let mut fen = String::new();

        for rank in (0..8).rev() {
            let mut empty = 0;

            for file in 0..8 {
                let sq = Square::new(rank * 8 + file);
                match self.piece_at(sq) {
                    Some(piece) => {
                        if empty > 0 {
                            fen.push(
                                char::from_digit(empty, 10).expect("empty count fits in digit"),
                            );
                            empty = 0;
                        }
                        fen.push(piece);
                    }
                    None => empty += 1,
                }
            }

            if empty > 0 {
                fen.push(char::from_digit(empty, 10).expect("empty count fits in digit"));
            }
            if rank > 0 {
                fen.push('/');
            }
        }

        fen
    }

    pub(super) const fn occupied(&self) -> Bitboard {
        self.occupancy::<{ Color::White }>() | self.occupancy::<{ Color::Black }>()
    }

    fn for_each_bb_mut(&mut self, mut f: impl FnMut(&mut Bitboard)) {
        f(&mut self.white_pawns);
        f(&mut self.white_rooks);
        f(&mut self.white_knights);
        f(&mut self.white_bishops);
        f(&mut self.white_queens);
        f(&mut self.white_king);
        f(&mut self.black_pawns);
        f(&mut self.black_rooks);
        f(&mut self.black_knights);
        f(&mut self.black_bishops);
        f(&mut self.black_queens);
        f(&mut self.black_king);
    }

    fn promote(&mut self, color: Color, promotion: PromotionPiece, to: Square) {
        let (pawns, promoted) = match (color, promotion) {
            (Color::White, PromotionPiece::Queen) => {
                (&mut self.white_pawns, &mut self.white_queens)
            }
            (Color::White, PromotionPiece::Rook) => (&mut self.white_pawns, &mut self.white_rooks),
            (Color::White, PromotionPiece::Bishop) => {
                (&mut self.white_pawns, &mut self.white_bishops)
            }
            (Color::White, PromotionPiece::Knight) => {
                (&mut self.white_pawns, &mut self.white_knights)
            }
            (Color::Black, PromotionPiece::Queen) => {
                (&mut self.black_pawns, &mut self.black_queens)
            }
            (Color::Black, PromotionPiece::Rook) => (&mut self.black_pawns, &mut self.black_rooks),
            (Color::Black, PromotionPiece::Bishop) => {
                (&mut self.black_pawns, &mut self.black_bishops)
            }
            (Color::Black, PromotionPiece::Knight) => {
                (&mut self.black_pawns, &mut self.black_knights)
            }
        };

        let to = Bitboard::from(to);
        *pawns &= !to;
        *promoted |= to;
    }

    fn add_piece(&mut self, piece: char, square: Square) -> Result<()> {
        let bitboard = match piece {
            'P' => &mut self.white_pawns,
            'R' => &mut self.white_rooks,
            'N' => &mut self.white_knights,
            'B' => &mut self.white_bishops,
            'Q' => &mut self.white_queens,
            'K' => &mut self.white_king,
            'p' => &mut self.black_pawns,
            'r' => &mut self.black_rooks,
            'n' => &mut self.black_knights,
            'b' => &mut self.black_bishops,
            'q' => &mut self.black_queens,
            'k' => &mut self.black_king,
            _ => bail!("invalid FEN piece: {piece}"),
        };

        *bitboard |= square;
        Ok(())
    }

    fn piece_at(&self, sq: Square) -> Option<char> {
        let pieces = [
            (self.black_pawns, 'p'),
            (self.black_rooks, 'r'),
            (self.black_knights, 'n'),
            (self.black_bishops, 'b'),
            (self.black_queens, 'q'),
            (self.black_king, 'k'),
            (self.white_pawns, 'P'),
            (self.white_rooks, 'R'),
            (self.white_knights, 'N'),
            (self.white_bishops, 'B'),
            (self.white_queens, 'Q'),
            (self.white_king, 'K'),
        ];

        pieces
            .iter()
            .find_map(|(bb, piece)| bb.contains(sq).then_some(*piece))
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rank in (0..8).rev() {
            write!(f, "{}  ", rank + 1)?;

            for file in 0..8 {
                let sq = Square::new(rank * 8 + file);
                let piece = self.piece_at(sq).unwrap_or('.');

                write!(f, "{piece}")?;

                if file < 7 {
                    write!(f, " ")?;
                }
            }
            writeln!(f)?;
        }
        writeln!(f, "\n   A B C D E F G H")
    }
}

#[cfg(test)]
mod tests {
    use super::Board;

    #[test]
    fn default_board_fen() {
        assert_eq!(
            Board::default().fen(),
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR"
        );
    }

    #[test]
    fn board_roundtrips_through_fen() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";

        assert_eq!(Board::from_fen(fen).unwrap().fen(), fen);
    }

    #[test]
    fn parses_mixed_piece_fen() {
        let fen = "8/3k4/2p5/8/4P3/8/3K4/8";

        assert_eq!(Board::from_fen(fen).unwrap().fen(), fen);
    }

    #[test]
    fn rejects_invalid_fen_placements() {
        for fen in [
            "8/8/8/8/8/8/8",
            "8/8/8/8/8/8/8/8/8",
            "8/8/8/8/8/8/8/7",
            "8/8/8/8/8/8/8/9",
            "8/8/8/8/8/8/8/0",
            "8/8/8/8/8/8/8/7X",
            "8/8/8/8/8/8/8/8P",
            "8/8/8/8/8/8/8/8",
            "7k/8/8/8/8/8/8/8",
            "8/8/8/8/8/8/8/4K3",
            "k6k/8/8/8/8/8/8/4K3",
            "7k/8/8/8/8/8/8/K6K",
        ] {
            assert!(Board::from_fen(fen).is_err(), "{fen} should be invalid");
        }
    }
}

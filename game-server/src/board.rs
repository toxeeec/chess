use crate::{bitboard, bitboard::Bitboard, moves::Move, square::Square};
use anyhow::{Result, bail};
use std::fmt;

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
        let white_pawns = bitboard![8, 9, 10, 11, 12, 13, 14, 15];
        let white_rooks = bitboard![0, 7];
        let white_knights = bitboard![1, 6];
        let white_bishops = bitboard![2, 5];
        let white_queens = bitboard![3];
        let white_king = bitboard![4];

        let black_pawns = bitboard![48, 49, 50, 51, 52, 53, 54, 55];
        let black_rooks = bitboard![56, 63];
        let black_knights = bitboard![57, 62];
        let black_bishops = bitboard![58, 61];
        let black_queens = bitboard![59];
        let black_king = bitboard![60];

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

    pub(super) fn empty(&self) -> Bitboard {
        !self.occupied()
    }

    pub(super) const fn pawns<const IS_WHITE: bool>(&self) -> Bitboard {
        if IS_WHITE {
            self.white_pawns
        } else {
            self.black_pawns
        }
    }

    pub(super) fn make_move(&mut self, mve: Move) {
        self.for_each_bb_mut(|piece| {
            piece.apply_move(mve.from, mve.to);
        });
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

    fn occupied(&self) -> Bitboard {
        self.white_pawns
            | self.white_rooks
            | self.white_knights
            | self.white_bishops
            | self.white_queens
            | self.white_king
            | self.black_pawns
            | self.black_rooks
            | self.black_knights
            | self.black_bishops
            | self.black_queens
            | self.black_king
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
        ] {
            assert!(Board::from_fen(fen).is_err(), "{fen} should be invalid");
        }
    }
}

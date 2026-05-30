use crate::{bitboard, bitboard::Bitboard, square::Square};
use std::fmt;

pub(super) struct Board {
    black_pawns: Bitboard,
    black_rooks: Bitboard,
    black_knights: Bitboard,
    black_bishops: Bitboard,
    black_queens: Bitboard,
    black_king: Bitboard,
    white_pawns: Bitboard,
    white_rooks: Bitboard,
    white_knights: Bitboard,
    white_bishops: Bitboard,
    white_queens: Bitboard,
    white_king: Bitboard,
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
            black_pawns,
            black_rooks,
            black_knights,
            black_bishops,
            black_queens,
            black_king,
            white_pawns,
            white_rooks,
            white_knights,
            white_bishops,
            white_queens,
            white_king,
        }
    }
}

impl Board {
    pub(super) fn white_pawns(&self) -> Bitboard {
        self.white_pawns
    }

    pub(super) fn occupied(&self) -> Bitboard {
        self.black_pawns
            | self.black_rooks
            | self.black_knights
            | self.black_bishops
            | self.black_queens
            | self.black_king
            | self.white_pawns
            | self.white_rooks
            | self.white_knights
            | self.white_bishops
            | self.white_queens
            | self.white_king
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
}

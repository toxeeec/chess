use std::{
    fmt,
    ops::{Index, IndexMut},
};

use anyhow::{Result, bail};

use crate::{
    bitboard::Bitboard,
    moves::{Move, PromotionPiece},
    square,
    square::Square,
    squares,
    state::Color,
};

const CASTLING_ROOK_MOVES: [[(Square, Square); 2]; 2] = [
    [(square!(a1), square!(d1)), (square!(h1), square!(f1))],
    [(square!(a8), square!(d8)), (square!(h8), square!(f8))],
];

#[derive(Clone, Copy)]
pub(super) struct Board {
    pieces: [[Bitboard; 6]; 2],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Piece {
    Pawn = 0,
    Rook = 1,
    Knight = 2,
    Bishop = 3,
    Queen = 4,
    King = 5,
}

impl Piece {
    const ALL: [Self; 6] = [
        Self::Pawn,
        Self::Rook,
        Self::Knight,
        Self::Bishop,
        Self::Queen,
        Self::King,
    ];
    const FEN: [[char; 6]; 2] = [
        ['P', 'R', 'N', 'B', 'Q', 'K'],
        ['p', 'r', 'n', 'b', 'q', 'k'],
    ];

    fn from_fen(value: char) -> Option<(Color, Self)> {
        let index = Self::FEN
            .iter()
            .flatten()
            .position(|piece| *piece == value)?;
        let piece_count = Self::ALL.len();

        Some((
            Color::ALL[index / piece_count],
            Self::ALL[index % piece_count],
        ))
    }

    fn fen(self, color: Color) -> char {
        Self::FEN[color][self]
    }
}

#[derive(Clone, Copy)]
pub(super) struct BoardUndo {
    pub(super) moved: Piece,
    #[cfg(test)]
    captured: Option<Piece>,
}

impl Default for Board {
    fn default() -> Self {
        Self {
            pieces: [
                [
                    Bitboard::from(squares![a2, b2, c2, d2, e2, f2, g2, h2]),
                    Bitboard::from(squares![a1, h1]),
                    Bitboard::from(squares![b1, g1]),
                    Bitboard::from(squares![c1, f1]),
                    Bitboard::from(square!(d1)),
                    Bitboard::from(square!(e1)),
                ],
                [
                    Bitboard::from(squares![a7, b7, c7, d7, e7, f7, g7, h7]),
                    Bitboard::from(squares![a8, h8]),
                    Bitboard::from(squares![b8, g8]),
                    Bitboard::from(squares![c8, f8]),
                    Bitboard::from(square!(d8)),
                    Bitboard::from(square!(e8)),
                ],
            ],
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
            pieces: [[Bitboard::EMPTY; 6]; 2],
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

    pub(super) fn pawns<const COLOR: Color>(&self) -> Bitboard {
        self.pieces[COLOR][Piece::Pawn]
    }

    pub(super) fn knights<const COLOR: Color>(&self) -> Bitboard {
        self.pieces[COLOR][Piece::Knight]
    }

    pub(super) fn rooks<const COLOR: Color>(&self) -> Bitboard {
        self.pieces[COLOR][Piece::Rook]
    }

    pub(super) fn bishops<const COLOR: Color>(&self) -> Bitboard {
        self.pieces[COLOR][Piece::Bishop]
    }

    pub(super) fn queens<const COLOR: Color>(&self) -> Bitboard {
        self.pieces[COLOR][Piece::Queen]
    }

    pub(super) fn king<const COLOR: Color>(&self) -> Bitboard {
        self.pieces[COLOR][Piece::King]
    }

    pub(super) fn king_square<const COLOR: Color>(&self) -> Square {
        let mut king = self.king::<COLOR>();
        debug_assert_eq!(king.len(), 1);

        unsafe { king.next().unwrap_unchecked() }
    }

    pub(super) fn occupancy<const COLOR: Color>(&self) -> Bitboard {
        self.pieces[COLOR][Piece::Pawn]
            | self.pieces[COLOR][Piece::Rook]
            | self.pieces[COLOR][Piece::Knight]
            | self.pieces[COLOR][Piece::Bishop]
            | self.pieces[COLOR][Piece::Queen]
            | self.pieces[COLOR][Piece::King]
    }

    pub(super) fn make_move(
        &mut self,
        color: Color,
        mve: Move,
        en_passant: Option<Square>,
    ) -> BoardUndo {
        let moved = self
            .piece_at_for(color, mve.from)
            .expect("legal move must have a moving piece");
        let is_en_passant = moved == Piece::Pawn && Some(mve.to) == en_passant;
        let captured_square = if is_en_passant {
            match color {
                Color::White => mve.to.backward::<{ Color::White }, 1>(),
                Color::Black => mve.to.backward::<{ Color::Black }, 1>(),
            }
        } else {
            mve.to
        };
        let captured = self.piece_at_for(color.opponent(), captured_square);
        if let Some(captured) = captured {
            self.pieces[color.opponent()][captured] &= !Bitboard::from(captured_square);
        }

        self.pieces[color][moved].apply_move(mve.from, mve.to);

        if moved == Piece::King && mve.from.file().abs_diff(mve.to.file()) == 2 {
            let king_side = (mve.to.file() > mve.from.file()) as usize;
            let (rook_from, rook_to) = CASTLING_ROOK_MOVES[color][king_side];
            self.pieces[color][Piece::Rook].apply_move(rook_from, rook_to);
        }

        if let Some(promotion) = mve.promotion {
            debug_assert_eq!(moved, Piece::Pawn);
            self.pieces[color][Piece::Pawn] &= !Bitboard::from(mve.to);
            self.pieces[color][Piece::from(promotion)] |= mve.to;
        }

        BoardUndo {
            moved,
            #[cfg(test)]
            captured,
        }
    }

    #[cfg(test)]
    pub(super) fn unmake_move(
        &mut self,
        color: Color,
        mve: Move,
        en_passant: Option<Square>,
        undo: BoardUndo,
    ) {
        if undo.moved == Piece::King && mve.from.file().abs_diff(mve.to.file()) == 2 {
            let king_side = (mve.to.file() > mve.from.file()) as usize;
            let (rook_from, rook_to) = CASTLING_ROOK_MOVES[color][king_side];
            self.pieces[color][Piece::Rook].apply_move(rook_to, rook_from);
        }

        if let Some(promotion) = mve.promotion {
            self.pieces[color][Piece::from(promotion)] &= !Bitboard::from(mve.to);
            self.pieces[color][Piece::Pawn] |= mve.from;
        } else {
            self.pieces[color][undo.moved].apply_move(mve.to, mve.from);
        }

        if let Some(captured) = undo.captured {
            let captured_square = if undo.moved == Piece::Pawn && Some(mve.to) == en_passant {
                match color {
                    Color::White => mve.to.backward::<{ Color::White }, 1>(),
                    Color::Black => mve.to.backward::<{ Color::Black }, 1>(),
                }
            } else {
                mve.to
            };
            self.pieces[color.opponent()][captured] |= captured_square;
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

    pub(super) fn occupied(&self) -> Bitboard {
        self.occupancy::<{ Color::White }>() | self.occupancy::<{ Color::Black }>()
    }

    fn piece_at_for(&self, color: Color, square: Square) -> Option<Piece> {
        Piece::ALL
            .into_iter()
            .find(|piece| self.pieces[color][*piece].contains(square))
    }

    fn add_piece(&mut self, piece: char, square: Square) -> Result<()> {
        let (color, piece) =
            Piece::from_fen(piece).ok_or_else(|| anyhow::anyhow!("invalid FEN piece: {piece}"))?;

        self.pieces[color][piece] |= square;
        Ok(())
    }

    fn piece_at(&self, sq: Square) -> Option<char> {
        for color in [Color::Black, Color::White] {
            for piece in Piece::ALL {
                if self.pieces[color][piece].contains(sq) {
                    return Some(piece.fen(color));
                }
            }
        }

        None
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

impl From<PromotionPiece> for Piece {
    fn from(promotion: PromotionPiece) -> Self {
        match promotion {
            PromotionPiece::Queen => Self::Queen,
            PromotionPiece::Rook => Self::Rook,
            PromotionPiece::Bishop => Self::Bishop,
            PromotionPiece::Knight => Self::Knight,
        }
    }
}

const impl<T> Index<Piece> for [T; 6] {
    type Output = T;

    fn index(&self, piece: Piece) -> &Self::Output {
        &self[piece as usize]
    }
}

const impl<T> IndexMut<Piece> for [T; 6] {
    fn index_mut(&mut self, piece: Piece) -> &mut Self::Output {
        &mut self[piece as usize]
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

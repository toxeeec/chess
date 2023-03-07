use enum_iterator::all;
use std::fmt::Debug;

use bitboard::{bb, shift::Direction, square::Square, Bitboard};

use super::{
    moves::{Move, Type},
    piece::Piece,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Board {
    pub pieces: [Bitboard; 12],

    pub white: Bitboard,
    pub black: Bitboard,
    pub occ: Bitboard,
}

impl Board {
    pub fn empty() -> Self {
        Self {
            pieces: Default::default(),
            white: Bitboard::default(),
            black: Bitboard::default(),
            occ: Bitboard::default(),
        }
    }

    pub const fn get<const PIECE: Piece>(&self, is_white: bool) -> Bitboard {
        self.pieces[6 * is_white as usize + PIECE as usize]
    }

    pub const fn get_mut(&mut self, piece: Piece, is_white: bool) -> &mut Bitboard {
        &mut self.pieces[6 * is_white as usize + piece as usize]
    }

    pub const fn enemy(&self, is_white: bool) -> Bitboard {
        if is_white {
            self.black
        } else {
            self.white
        }
    }

    pub const fn own(&self, is_white: bool) -> Bitboard {
        if is_white {
            self.white
        } else {
            self.black
        }
    }

    pub const fn enemy_or_empty(&self, is_white: bool) -> Bitboard {
        self.enemy(is_white) | !self.occ
    }

    fn piece_at(&self, sq: Square) -> Piece {
        for (i, bb) in self.pieces.into_iter().enumerate() {
            if bb.contains(sq) {
                return num::FromPrimitive::from_usize(i % 6).unwrap();
            }
        }
        unreachable!("No piece at given square");
    }

    fn clear(&mut self, sq: Square, is_white: bool) {
        for piece in all::<Piece>() {
            self.get_mut(piece, is_white).clear(sq);
        }
    }

    pub fn set_white(&mut self) {
        self.white = Bitboard::default();
        for bb in self.pieces.into_iter().skip(6) {
            self.white |= bb;
        }
    }

    pub fn set_black(&mut self) {
        self.black = Bitboard::default();
        for bb in self.pieces.into_iter().take(6) {
            self.black |= bb;
        }
    }

    pub fn set_occ(&mut self) {
        self.occ = self.white | self.black;
    }

    pub fn update(&mut self, mov: Move, is_white: bool) {
        let from = mov.from();
        let mut to = mov.to();
        let typ = mov.typ();
        let piece = match typ {
            Type::Quiet | Type::Capture => self.piece_at(from),
            Type::KingCastle | Type::QueenCastle => Piece::King,
            _ => Piece::Pawn,
        };

        match typ.promotion_piece() {
            Some(promotion_piece) => {
                self.get_mut(Piece::Pawn, is_white).clear(from);
                self.get_mut(promotion_piece, is_white).set(to);
            }
            None => {
                *self.get_mut(piece, is_white) ^= bb![from.0, to.0];
            }
        };

        if typ.is_capture() {
            if typ == Type::EnPassant {
                let dir = if is_white {
                    Direction::South
                } else {
                    Direction::North
                };
                to = to.shifted_by(dir).unwrap()
            }
            self.clear(to, !is_white);
        }

        if typ == Type::KingCastle {
            let mov = if is_white { bb![5, 7] } else { bb![61, 63] };
            *self.get_mut(Piece::Rook, is_white) ^= mov;
        } else if typ == Type::QueenCastle {
            let mov = if is_white { bb![0, 3] } else { bb![56, 59] };
            *self.get_mut(Piece::Rook, is_white) ^= mov;
        }

        self.set_white();
        self.set_black();
        self.set_occ();
    }
}

impl Default for Board {
    fn default() -> Self {
        let wp = bb![8, 9, 10, 11, 12, 13, 14, 15];
        let wr = bb![0, 7];
        let wn = bb![1, 6];
        let wb = bb![2, 5];
        let wq = bb![3];
        let wk = bb![4];

        let bp = bb![48, 49, 50, 51, 52, 53, 54, 55];
        let br = bb![56, 63];
        let bn = bb![57, 62];
        let bb = bb![58, 61];
        let bq = bb![59];
        let bk = bb![60];

        let white = wp | wr | wn | wb | wq | wk;
        let black = bp | br | bn | bb | bq | bk;
        let occ = white | black;

        Self {
            pieces: [bp, br, bn, bb, bq, bk, wp, wr, wn, wb, wq, wk],
            white,
            black,
            occ,
        }
    }
}

impl Debug for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const PIECES: [char; 12] = ['p', 'r', 'n', 'b', 'q', 'k', 'P', 'R', 'N', 'B', 'Q', 'K'];
        let mut squares = ['.'; 64];
        for (i, sq) in squares.iter_mut().enumerate() {
            for (j, bb) in self.pieces.iter().enumerate() {
                if bb.contains((i as u32).into()) {
                    *sq = PIECES[j];
                }
            }
        }
        for (i, row) in squares.chunks(8).enumerate().rev() {
            writeln!(
                f,
                "{}  {}",
                i + 1,
                String::from_iter(row.iter().intersperse(&' '))
            )?;
        }
        writeln!(f, "\n   A B C D E F G H")
    }
}

use std::fmt::Debug;

use super::{bitboard::Bitboard, piece::Piece};

pub struct Board {
    pub pieces: [Bitboard; 12],

    pub white: Bitboard,
    pub black: Bitboard,
    pub occ: Bitboard,
}

impl Board {
    pub const fn get<const PIECE: Piece>(&self, white: bool) -> Bitboard {
        self.pieces[PIECE as usize * white as usize]
    }

    pub const fn empty(&self) -> Bitboard {
        !self.occ
    }

    pub const fn enemy(&self, white: bool) -> Bitboard {
        if white {
            self.black
        } else {
            self.white
        }
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
                if bb.contains(i as u32) {
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

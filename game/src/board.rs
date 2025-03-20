use std::fmt;

use itertools::{Itertools, intersperse};

use crate::bitboard::Bitboard;

pub(super) struct Board([Bitboard; 12]);

impl Default for Board {
    fn default() -> Self {
        let wr = Bitboard::new([0, 7]);
        let wn = Bitboard::new([1, 6]);
        let wb = Bitboard::new([2, 5]);
        let wq = Bitboard::new([3]);
        let wk = Bitboard::new([4]);
        let wp = Bitboard::new([8, 9, 10, 11, 12, 13, 14, 15]);

        let br = Bitboard::new([56, 63]);
        let bn = Bitboard::new([57, 62]);
        let bb = Bitboard::new([58, 61]);
        let bq = Bitboard::new([59]);
        let bk = Bitboard::new([60]);
        let bp = Bitboard::new([48, 49, 50, 51, 52, 53, 54, 55]);

        Self([br, bn, bb, bq, bk, bp, wr, wn, wb, wq, wk, wp])
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const PIECES: [char; 12] = ['r', 'n', 'b', 'q', 'k', 'p', 'R', 'N', 'B', 'Q', 'K', 'P'];
        let mut squares = ['.'; 64];
        for (i, sq) in squares.iter_mut().enumerate() {
            for (j, bb) in self.0.into_iter().enumerate() {
                if bb.contains((i as u32).into()) {
                    *sq = PIECES[j];
                    break;
                }
            }
        }

        writeln!(
            f,
            "{}",
            squares
                .chunks(8)
                .enumerate()
                .rev()
                .format_with("\n", |(i, row), f| {
                    f(&format_args!(
                        "{}  {}",
                        i + 1,
                        String::from_iter(intersperse(row, &' '))
                    ))
                })
        )?;
        write!(f, "\n   A B C D E F G H")
    }
}

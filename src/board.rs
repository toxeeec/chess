use crate::{bb, bitboard::Bitboard};
use std::fmt;

pub(super) struct Board {
    pieces: [Bitboard; 12],
}

impl Default for Board {
    #[inline(always)]
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

        Self {
            pieces: [bp, br, bn, bb, bq, bk, wp, wr, wn, wb, wq, wk],
        }
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const PIECES: [char; 12] = ['p', 'r', 'n', 'b', 'q', 'k', 'P', 'R', 'N', 'B', 'Q', 'K'];

        Bitboard::FULL
            .map(|sq| {
                self.pieces
                    .iter()
                    .position(|bb| bb.contains(sq))
                    .map_or('.', |i| PIECES[i])
            })
            .array_chunks::<8>()
            .map(|row| String::from_iter(row.iter().intersperse(&' ')))
            .enumerate()
            .rev()
            .try_for_each(|(i, row)| writeln!(f, "{}  {}", i + 1, row))?;

        writeln!(f, "\n   A B C D E F G H")
    }
}

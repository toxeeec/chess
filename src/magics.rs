// https://github.com/official-stockfish/Stockfish/blob/master/src/bitboard.cpp

use crate::{
    bitboard::{Bitboard, Direction},
    square::Square,
};
use once_cell::sync::Lazy;

static ROOK_MAGICS: Lazy<[Magic; 64]> = Lazy::new(magics::<true>);
static BISHOP_MAGICS: Lazy<[Magic; 64]> = Lazy::new(magics::<false>);

#[derive(Clone, Copy, Debug)]
struct Magic {
    mask: Bitboard,
    magic: u64,
    attacks: &'static [Bitboard],
    shift: u32,
}

struct Prng {
    seed: u64,
}

impl Prng {
    #[inline(always)]
    const fn new(seed: u64) -> Self {
        Self { seed }
    }

    #[inline(always)]
    const fn rand64(&mut self) -> u64 {
        self.seed ^= self.seed >> 12;
        self.seed ^= self.seed << 25;
        self.seed ^= self.seed >> 27;
        self.seed.wrapping_mul(2685821657736338717)
    }

    #[inline(always)]
    const fn sparse_rand(&mut self) -> u64 {
        self.rand64() & self.rand64() & self.rand64()
    }
}

const DISTANCES: [[u8; 64]; 64] = {
    let mut distances = [[0; 64]; 64];
    let mut sq1 = Square::ZERO;
    while sq1.is_valid() {
        let mut sq2 = Square::ZERO;
        while sq2.is_valid() {
            let (sq1_rank, sq2_rank) = (sq1.rank(), sq2.rank());
            let (sq1_file, sq2_file) = (sq1.file(), sq2.file());
            let rank_diff = sq1_rank.abs_diff(sq2_rank);
            let file_diff = sq1_file.abs_diff(sq2_file);
            distances[sq1.0 as usize][sq2.0 as usize] = if rank_diff > file_diff {
                rank_diff as u8
            } else {
                file_diff as u8
            };
            sq2 = sq2.next();
        }
        sq1 = sq1.next();
    }
    distances
};

#[inline(always)]
const fn attacks_bb<const IS_ROOK: bool>(sq: Square, occ: Bitboard) -> Bitboard {
    let dirs: [Direction; 4] = if IS_ROOK {
        [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ]
    } else {
        [
            Direction::NorthEast,
            Direction::SouthEast,
            Direction::SouthWest,
            Direction::NorthWest,
        ]
    };

    let mut attacks = Bitboard::EMPTY;
    for &dir in &dirs {
        let mut prev = sq;
        let mut curr = sq.shifted(dir);
        while let Some(curr_sq) = curr
            && DISTANCES[curr_sq.0 as usize][prev.0 as usize] == 1
        {
            attacks = attacks.with_square(curr_sq);
            if occ.contains(curr_sq) {
                break;
            }
            prev = curr_sq;
            curr = curr_sq.shifted(dir);
        }
    }
    attacks
}

#[inline(always)]
const fn relevant_occupancies<const IS_ROOK: bool>() -> [Bitboard; 64] {
    let mut occ = [Bitboard::EMPTY; 64];
    let mut sq = Square::ZERO;
    while sq.is_valid() {
        let edges = ((Bitboard::RANK_1.or(Bitboard::RANK_8)).xor(Bitboard::rank(sq)))
            .or((Bitboard::FILE_A.or(Bitboard::FILE_H)).xor(Bitboard::file(sq)));
        occ[sq.0 as usize] = attacks_bb::<IS_ROOK>(sq, Bitboard::EMPTY).xor(edges);
        sq = sq.next();
    }
    occ
}

#[inline(always)]
fn magics<const IS_ROOK: bool>() -> [Magic; 64] {
    const SEEDS: [u64; 8] = [728, 10316, 55013, 32803, 12281, 15100, 16645, 255];

    let mut occs = [Bitboard::EMPTY; 64 * 64];
    let mut reference = [Bitboard::EMPTY; 64 * 64];
    let mut epoch = [0; 64 * 64];
    let mut count = 0;

    array_init::array_init(|sq| {
        let mask = relevant_occupancies::<IS_ROOK>()[sq];
        let shift = 64 - mask.0.count_ones();
        let mut size = 0;
        let mut bb = Bitboard::EMPTY;
        loop {
            occs[size] = bb;
            reference[size] = attacks_bb::<IS_ROOK>(Square::new(sq as u32), occs[size]);
            size += 1;
            bb.0 = bb.0.wrapping_sub(mask.0) & mask.0;
            if bb.is_empty() {
                break;
            }
        }

        let mut rng = Prng::new(SEEDS[sq / 8]);

        let mut magic: u64 = 0;
        let attacks = Box::leak(vec![Bitboard::EMPTY; size].into_boxed_slice());

        let mut i = 0;
        while i < size {
            magic = 0;
            while (magic.wrapping_mul(mask.0) >> 56).count_ones() < 6 {
                magic = rng.sparse_rand();
            }

            count += 1;
            i = 0;
            while i < size {
                let idx = ((occs[i].and(mask).0).wrapping_mul(magic) >> shift) as usize;

                if epoch[idx] < count {
                    epoch[idx] = count;
                    attacks[idx] = reference[i];
                } else if attacks[idx] != reference[i] {
                    break;
                }
                i += 1;
            }
        }

        Magic {
            mask,
            magic,
            attacks,
            shift,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bb;
    use test_case::test_case;

    #[test_case(0, 0 => 0)]
    #[test_case(63, 0 => 7)]
    #[test_case(0, 33 => 4)]
    #[test_case(12, 0 => 4)]
    fn distances_tests(sq1: usize, sq2: usize) -> u8 {
        DISTANCES[sq1][sq2]
    }

    #[test_case(0, Bitboard::EMPTY => bb![1, 2, 3, 4, 5, 6, 7, 8, 16, 24, 32, 40, 48, 56])]
    #[test_case(0, bb![1, 8] => bb![1, 8])]
    fn attacks_rook_tests(sq: u32, occ: Bitboard) -> Bitboard {
        attacks_bb::<true>(sq.into(), occ)
    }

    #[test_case(0, Bitboard::EMPTY => bb![9, 18, 27, 36, 45, 54, 63])]
    #[test_case(0, bb![9] => bb![9])]
    fn attacks_bishop_tests(sq: u32, occ: Bitboard) -> Bitboard {
        attacks_bb::<false>(sq.into(), occ)
    }

    #[test_case(0 => bb![1, 2, 3, 4, 5, 6, 8, 16, 24, 32, 40, 48])]
    #[test_case(9 => bb![10, 11, 12, 13, 14, 17, 25, 33, 41, 49])]
    fn rook_relevant_occupancies_tests(sq: usize) -> Bitboard {
        relevant_occupancies::<true>()[sq]
    }

    #[test_case(0 => bb![9, 18, 27, 36, 45, 54])]
    #[test_case(9 => bb![18, 27, 36, 45, 54])]
    fn bishop_relevant_occupancies_tests(sq: usize) -> Bitboard {
        relevant_occupancies::<false>()[sq]
    }
}

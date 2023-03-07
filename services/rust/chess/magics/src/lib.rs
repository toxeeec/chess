// https://github.com/official-stockfish/Stockfish/blob/master/src/bitboard.cpp

#![feature(
    const_cmp,
    const_trait_impl,
    const_mut_refs,
    let_chains,
    const_for,
    const_intoiterator_identity,
    const_convert
)]

mod prng;

use bitboard::{
    shift::Direction, square::Square, squares::squares, Bitboard, FILE_A, FILE_H, RANK_1, RANK_8,
};
use prng::Prng;
use quote::{quote, ToTokens, TokenStreamExt};
use std::cmp::max;

#[derive(Default, Debug, Clone, Copy)]
pub struct Magic {
    pub mask: u64,
    pub magic: u64,
    pub shift: u32,
    pub attacks_index: usize,
}

pub fn rook_magics() -> ([Magic; 64], [bitboard::Bitboard; ROOK_ATTACKS]) {
    magics(ROOK_RELEVANT_OCCUPANCIES, attacks_bb::<true>)
}
pub fn bishop_magics() -> ([Magic; 64], [bitboard::Bitboard; BISHOP_ATTACKS]) {
    magics(BISHOP_RELEVANT_OCCUPANCIES, attacks_bb::<false>)
}

impl ToTokens for Magic {
    fn to_tokens(&self, tokens: &mut quote::__private::TokenStream) {
        let mask = self.mask;
        let magic = self.magic;
        let shift = self.shift;
        let attacks_index = self.attacks_index;
        tokens.append_all(quote! {
            Magic {
                mask: #mask,
                magic: #magic,
                shift: #shift,
                attacks_index: #attacks_index,
            }
        })
    }
}

const DISTANCES: [[u8; 64]; 64] = {
    let mut distances = [[0; 64]; 64];
    for sq1 in squares() {
        for sq2 in squares() {
            let (sq1_rank, sq2_rank) = (sq1.rank(), sq2.rank());
            let (sq1_file, sq2_file) = (sq1.file(), sq2.file());
            distances[sq1.0 as usize][sq2.0 as usize] =
                max(sq1_rank.abs_diff(sq2_rank), sq1_file.abs_diff(sq2_file)) as u8;
        }
    }
    distances
};

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
    let mut attacks = Bitboard::default();
    let mut i = 0;
    while i < 4 {
        let dir = dirs[i];
        let mut prev = sq;
        let mut curr = sq.shifted_by(dir);
        while let Some(curr_sq) = curr && DISTANCES[curr_sq.0 as usize][prev.0 as usize] == 1 {
            attacks |= curr_sq.into();
            if occ.contains(curr_sq) {
                break;
            }
            prev = curr_sq;
            curr = curr_sq.shifted_by(dir);
        }
        i += 1;
    }
    attacks
}

const fn relevant_occupancies<const IS_ROOK: bool>() -> [u64; 64] {
    let mut occ = [0; 64];
    for sq in squares() {
        let edges =
            ((RANK_1 | RANK_8) & !Bitboard::rank(sq)) | ((FILE_A | FILE_H) & !Bitboard::file(sq));
        occ[sq.0 as usize] = (attacks_bb::<IS_ROOK>(sq, Bitboard::default()) & !edges).0;
    }
    occ
}

const ROOK_RELEVANT_OCCUPANCIES: [u64; 64] = relevant_occupancies::<true>();
const BISHOP_RELEVANT_OCCUPANCIES: [u64; 64] = relevant_occupancies::<false>();

fn magics<const N: usize>(
    relevant_occupancies: [u64; 64],
    attacks_bb: fn(Square, Bitboard) -> Bitboard,
) -> ([Magic; 64], [Bitboard; N]) {
    const SEEDS: [u64; 8] = [728, 10316, 55013, 32803, 12281, 15100, 16645, 255];
    let mut magics = [Magic::default(); 64];
    let mut occs = [Bitboard::default(); 4096];
    let mut reference = [Bitboard::default(); 4096];
    let mut attacks = [Bitboard::default(); N];
    let mut attacks_index = 0;

    for (sq, mask) in relevant_occupancies.into_iter().enumerate() {
        let shift = mask.count_ones();
        let mut size = 0;
        let mut bb = 0;
        loop {
            occs[size] = Bitboard(bb);
            reference[size] = attacks_bb((sq as u32).into(), occs[size]);
            size += 1;
            bb = bb.wrapping_sub(mask) & mask;
            if bb == 0 {
                break;
            }
        }

        let mut rng = Prng::new(SEEDS[sq / 8]);
        let sq_attacks = &mut attacks[attacks_index..attacks_index + size];
        let mut magic;
        loop {
            magic = rng.sparse_rand();
            sq_attacks.fill(Bitboard::default());
            let mut i = 0;
            while i < size {
                let j = occs[i].0.wrapping_mul(magic).wrapping_shr(64 - shift) as usize;
                let attack = sq_attacks[j];
                if !attack.is_empty() && attack != reference[i] {
                    break;
                }
                sq_attacks[j] = reference[i];
                i += 1;
            }
            if i == size {
                break;
            }
        }
        magics[sq] = Magic {
            mask,
            magic,
            shift,
            attacks_index,
        };
        attacks_index += size;
    }
    (magics, attacks)
}

const ROOK_ATTACKS: usize = 102400;
const BISHOP_ATTACKS: usize = 5248;

#[cfg(test)]
mod tests {
    use super::*;
    use bitboard::bb;
    use test_case::test_case;

    #[test_case(0, 0 => 0)]
    #[test_case(63, 0 => 7)]
    #[test_case(0, 33 => 4)]
    #[test_case(12, 0 => 4)]
    fn distances_tests(sq1: usize, sq2: usize) -> u8 {
        DISTANCES[sq1][sq2]
    }

    #[test_case(0, Bitboard::default() => bb![1, 2, 3, 4, 5, 6, 7, 8, 16, 24, 32, 40, 48, 56])]
    #[test_case(0, bb![1, 8] => bb![1, 8])]
    fn attacks_rook_tests(sq: u32, occ: Bitboard) -> Bitboard {
        attacks_bb::<true>(sq.into(), occ)
    }

    #[test_case(0, Bitboard::default() => bb![9, 18, 27, 36, 45, 54, 63])]
    #[test_case(0, bb![9] => bb![9])]
    fn attacks_bishop_tests(sq: u32, occ: Bitboard) -> Bitboard {
        attacks_bb::<false>(sq.into(), occ)
    }

    #[test_case(0 => bb![1, 2, 3, 4, 5, 6, 8, 16, 24, 32, 40, 48].0)]
    #[test_case(9 => bb![10, 11, 12, 13, 14, 17, 25, 33, 41, 49].0)]
    fn rook_relevant_occupancies_tests(sq: usize) -> u64 {
        ROOK_RELEVANT_OCCUPANCIES[sq]
    }

    #[test_case(0 => bb![9, 18, 27, 36, 45, 54].0)]
    #[test_case(9 => bb![18, 27, 36, 45, 54].0)]
    fn bishop_relevant_occupancies_tests(sq: usize) -> u64 {
        BISHOP_RELEVANT_OCCUPANCIES[sq]
    }
}

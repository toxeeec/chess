use crate::{bitboard::Bitboard, square::Square};

#[derive(Clone, Copy)]
struct MagicEntry {
    mask: u64,
    magic: u64,
    shift: u32,
    offset: usize,
}

include!(concat!(env!("OUT_DIR"), "/rook_magics.rs"));
include!(concat!(env!("OUT_DIR"), "/bishop_magics.rs"));

pub(super) fn rook_attacks(square: Square, occupied: Bitboard) -> Bitboard {
    let entry = ROOK_MAGICS[usize::from(square)];
    let index = ((occupied.0 & entry.mask).wrapping_mul(entry.magic) >> entry.shift) as usize;
    ROOK_ATTACKS[entry.offset + index]
}

pub(super) fn bishop_attacks(square: Square, occupied: Bitboard) -> Bitboard {
    let entry = BISHOP_MAGICS[usize::from(square)];
    let index = ((occupied.0 & entry.mask).wrapping_mul(entry.magic) >> entry.shift) as usize;
    BISHOP_ATTACKS[entry.offset + index]
}

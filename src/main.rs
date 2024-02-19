#![allow(incomplete_features)]
#![feature(
    adt_const_params,
    const_for,
    const_mut_refs,
    derive_const,
    effects,
    exact_size_is_empty,
    iter_array_chunks,
    iter_intersperse,
    let_chains
)]

use crate::board::Board;

mod bitboard;
mod board;
mod magics;
mod square;

fn main() {
    println!("{:?}", Board::default());
}

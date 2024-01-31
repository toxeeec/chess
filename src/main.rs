#![feature(derive_const, effects, iter_array_chunks, iter_intersperse)]

use crate::board::Board;

mod bitboard;
mod board;

fn main() {
    println!("{:?}", Board::default());
}

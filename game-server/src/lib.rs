#![allow(incomplete_features)]
#![feature(
    adt_const_params,
    const_trait_impl,
    const_convert,
    const_index,
    const_ops,
    generic_const_args,
    generic_const_items,
    macroless_generic_const_args,
    min_generic_const_args,
    trusted_len
)]

mod attacks;
#[cfg(feature = "benchmark")]
mod benchmark;
mod bishop;
mod bitboard;
mod board;
mod castling;
mod game;
mod king;
mod knight;
mod magics;
mod moves;
mod pawn;
#[cfg(any(test, feature = "benchmark"))]
mod perft;
mod queen;
mod rook;
mod server;
mod square;
mod state;
#[cfg(test)]
mod test_utils;

#[cfg(feature = "benchmark")]
pub use benchmark::{GameReplay, Perft};
pub use game::Game;
pub use server::{GameServer, SnapshotMessage};

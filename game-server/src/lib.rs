#![allow(incomplete_features)]
#![feature(
    adt_const_params,
    const_trait_impl,
    const_convert,
    const_index,
    const_ops,
    generic_const_exprs,
    trusted_len
)]

mod attacks;
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
mod queen;
mod rook;
mod server;
mod square;
mod state;
#[cfg(test)]
mod test_utils;

pub use server::{GameServer, SnapshotMessage};

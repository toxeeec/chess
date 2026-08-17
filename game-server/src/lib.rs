#![feature(
    adt_const_params,
    const_trait_impl,
    const_convert,
    const_ops,
    const_range,
    trusted_len
)]

mod bishop;
mod bitboard;
mod board;
mod game;
mod game_state;
mod game_storage;
mod king;
mod knight;
mod magics;
mod moves;
mod pawn;
mod queen;
mod rook;
mod server;
mod square;
#[cfg(test)]
mod test_utils;

pub use server::{GameServer, SnapshotMessage};

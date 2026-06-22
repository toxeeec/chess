#![feature(
    adt_const_params,
    const_trait_impl,
    const_convert,
    const_ops,
    const_range
)]

mod bitboard;
mod board;
mod game;
mod game_state;
mod game_storage;
mod king;
mod knight;
mod moves;
mod pawn;
mod server;
mod square;

pub use server::{GameServer, SnapshotMessage};

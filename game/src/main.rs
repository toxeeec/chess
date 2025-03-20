mod bitboard;
mod board;

use board::Board;

fn main() {
    let board = Board::default();
    println!("{:?}", board);
}

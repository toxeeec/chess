mod bitboard;

use bitboard::Bitboard;

fn main() {
    let bb = Bitboard::new([0, 1, 2, 3, 4, 5, 6, 7]);
    println!("{:?}", bb);
}

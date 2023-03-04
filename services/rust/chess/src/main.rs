use chess::game::{moves, Game};

fn main() {
    let game = Game::default();
    let mut list = Vec::new();
    moves::generate(&mut list, &game.board, game.state);
    println!("{:?}", list);
    println!("{}", list.len());
    println!("{:?}", game);
}

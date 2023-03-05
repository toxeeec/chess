use chess::game::{
    moves::{Move, Type},
    Game,
};

fn main() {
    let mut game = Game::default();
    println!("{:?}", game);
    game.make_move(Move::new(8, 16, Type::Quiet));
    println!("{:?}", game);
    println!("{:?}", game.moves);
    println!("{:?}", game.moves.len());
}

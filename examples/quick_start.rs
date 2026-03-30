#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

use spooky_chess::color::Color;
use spooky_chess::game::StandardGame;

fn main() {
    let mut game = StandardGame::standard();
    assert_eq!(game.make_move_from_san("e4"), Ok(true));
    assert_eq!(game.make_move_from_san("e5"), Ok(true));

    println!("{}", game.turn() == Color::White);
    println!("{}", game.to_fen());
}

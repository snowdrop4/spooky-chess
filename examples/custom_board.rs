#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

use std::error::Error;

use spooky_chess::color::Color;
use spooky_chess::encode;
use spooky_chess::game::Game;

const CUSTOM_FEN: &str = "rnbkqr/pppppp/6/6/PPPPPP/RNBKQR w - - 0 1";

fn main() -> Result<(), Box<dyn Error>> {
    let mut game = Game::<6, 6>::new(CUSTOM_FEN, true)?;

    println!(
        "Board shape (height, width): ({}, {})",
        game.height(),
        game.width()
    );
    println!("Initial FEN: {}", game.to_fen());
    println!("White pieces: {}", game.pieces(Color::White).len());
    println!("Black pieces: {}", game.pieces(Color::Black).len());
    println!(
        "Total actions for 6x6: {}",
        encode::get_total_actions(game.width(), game.height())
    );

    let mut legal_moves: Vec<_> = game
        .legal_moves()
        .into_iter()
        .map(|mv| mv.to_lan())
        .collect();
    legal_moves.sort();
    println!(
        "Legal moves from the starting 6x6 position: {}",
        legal_moves.len()
    );
    println!("First 10 legal moves: {}", legal_moves[..10].join(", "));

    let mv = game.move_from_lan("a2a3")?;
    assert!(game.make_move(&mv));
    println!("After a2a3:");
    println!("{}", game.to_fen());

    Ok(())
}

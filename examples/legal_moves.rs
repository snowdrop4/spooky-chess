#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

use std::error::Error;

use spooky_chess::color::Color;
use spooky_chess::game::StandardGame;
use spooky_chess::position::Position;

fn side_name(color: Color) -> &'static str {
    match color {
        Color::White => "White",
        Color::Black => "Black",
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut game = StandardGame::standard();
    assert_eq!(game.make_move_from_san("e4"), Ok(true));
    assert_eq!(game.make_move_from_san("e5"), Ok(true));

    println!("Position after 1.e4 e5");
    println!("Turn: {}", side_name(game.turn()));
    println!("In check: {}", game.is_check());

    let legal_moves = game.legal_moves();
    let mut legal_rows: Vec<_> = legal_moves
        .iter()
        .map(|mv| (game.move_to_san(mv), mv.to_lan()))
        .collect();
    legal_rows.sort();
    println!("{} legal moves:", legal_rows.len());
    for (san, lan) in legal_rows {
        println!("  {san:<6} {lan}");
    }

    let knight = Position::from_algebraic("g1")?;
    let knight_moves = game.legal_moves_for_position(&knight);
    let mut knight_rows: Vec<_> = knight_moves
        .iter()
        .map(|mv| (game.move_to_san(mv), mv.to_lan()))
        .collect();
    knight_rows.sort();
    println!();
    println!("Moves from g1:");
    for (san, lan) in knight_rows {
        println!("  {san:<6} {lan}");
    }

    Ok(())
}

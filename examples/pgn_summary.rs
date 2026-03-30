#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

use std::error::Error;
use std::fs;

use spooky_chess::pgn::{PgnGame, parse_pgn};

const PGN_PATH: &str = "pgn/example/multi_game.pgn";

fn print_game_summary(index: usize, pgn_game: &PgnGame) {
    let mut final_game = pgn_game.final_game.clone();

    println!();
    println!(
        "Game {index}: {} vs {}",
        pgn_game.headers.white().unwrap_or("?"),
        pgn_game.headers.black().unwrap_or("?"),
    );
    println!("  event={}", pgn_game.headers.event().unwrap_or("?"));
    println!("  result={}", pgn_game.result);
    println!("  ply={}", pgn_game.moves.len());
    println!(
        "  starting_fen={}",
        pgn_game.starting_fen().unwrap_or("standard start")
    );
    println!("  final_fen={}", final_game.to_fen());
}

fn main() -> Result<(), Box<dyn Error>> {
    let pgn = fs::read_to_string(PGN_PATH)?;
    let games = parse_pgn(&pgn)?;
    println!("Loaded {} game(s) from {PGN_PATH}", games.len());

    for (index, pgn_game) in games.iter().enumerate() {
        print_game_summary(index + 1, pgn_game);
    }

    Ok(())
}

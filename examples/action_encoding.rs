#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

use spooky_chess::encode;
use spooky_chess::game::StandardGame;

fn main() {
    let mut game = StandardGame::standard();
    let (data, num_planes, height, width) = encode::encode_game_planes(&mut game);

    println!("Encoded game planes: {num_planes} x {height} x {width}");
    println!("Flat data length: {}", data.len());
    println!(
        "Action planes: {}",
        encode::get_move_planes_count(game.width(), game.height())
    );
    println!(
        "Total actions: {}",
        encode::get_total_actions(game.width(), game.height())
    );

    let legal_actions: Vec<_> = game
        .legal_moves()
        .into_iter()
        .filter_map(|mv| game.encode_action(&mv))
        .collect();
    println!(
        "Legal actions in the current position: {}",
        legal_actions.len()
    );
    println!("Legal action -> move pairs:");
    for action in &legal_actions {
        let decoded = game
            .decode_action(*action)
            .expect("legal action should decode");
        println!("  {action:>4} -> {}", decoded.to_lan());
    }

    let mv = game
        .move_from_lan("e2e4")
        .expect("expected e2e4 to parse from the initial position");
    let action = game
        .encode_action(&mv)
        .expect("expected e2e4 to encode on an 8x8 board");
    assert!(legal_actions.contains(&action));

    let decoded = game
        .decode_action(action)
        .expect("encoded action should decode");
    println!();
    println!("Move {} encodes to action {action}", mv.to_lan());
    println!(
        "Action {action} decodes to {} ({})",
        decoded.to_lan(),
        game.move_to_san(&decoded)
    );

    let mut next_game = game.clone();
    assert!(next_game.apply_action(action));
    println!("FEN after applying that action:");
    println!("{}", next_game.to_fen());
}

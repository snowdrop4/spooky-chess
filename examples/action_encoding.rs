#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

use spooky_chess::encode;
use spooky_chess::game::StandardGame;

fn show_alphazero_roundtrip(game: &StandardGame, mv: &spooky_chess::r#move::Move) {
    let total_actions = game.alphazero_total_actions();
    println!();
    println!("AlphaZero");
    println!("Total actions: {total_actions}");

    let mut legal_game = game.clone();
    let legal_actions = legal_game.legal_alphazero_action_indices();
    println!(
        "Legal actions in the current position: {}",
        legal_actions.len()
    );
    println!("Legal action -> move pairs:");
    for action in &legal_actions {
        let decoded = game
            .decode_alphazero_action(*action)
            .expect("legal AlphaZero action should decode");
        println!("  {action:>4} -> {}", decoded.to_lan());
    }

    let action = game
        .encode_alphazero_action(mv)
        .expect("expected move to encode in AlphaZero action space");
    assert!(legal_actions.contains(&action));

    let decoded = game
        .decode_alphazero_action(action)
        .expect("encoded AlphaZero action should decode");
    println!();
    println!("Move {} encodes to AlphaZero action {action}", mv.to_lan());
    let mut san_game = game.clone();
    println!(
        "AlphaZero action {action} decodes to {} ({})",
        decoded.to_lan(),
        san_game.move_to_san(&decoded)
    );

    let mut next_game = game.clone();
    assert!(next_game.apply_alphazero_action(action));
    println!("FEN after applying that AlphaZero action:");
    println!("{}", next_game.to_fen());
}

fn show_maia2_roundtrip(game: &StandardGame, mv: &spooky_chess::r#move::Move) {
    let total_actions = game
        .maia2_total_actions()
        .expect("MAIA2 should be available on a standard 8x8 board");
    println!();
    println!("MAIA2");
    println!("Total actions: {total_actions}");

    let mut legal_game = game.clone();
    let legal_actions = legal_game.legal_maia2_action_indices();
    println!(
        "Legal actions in the current position: {}",
        legal_actions.len()
    );
    println!("Legal action -> move pairs:");
    for action in &legal_actions {
        let decoded = game
            .decode_maia2_action(*action)
            .expect("legal MAIA2 action should decode");
        println!("  {action:>4} -> {}", decoded.to_lan());
    }

    let action = game
        .encode_maia2_action(mv)
        .expect("expected move to encode in MAIA2 action space");
    assert!(legal_actions.contains(&action));

    let decoded = game
        .decode_maia2_action(action)
        .expect("encoded MAIA2 action should decode");
    println!();
    println!("Move {} encodes to MAIA2 action {action}", mv.to_lan());
    let mut san_game = game.clone();
    println!(
        "MAIA2 action {action} decodes to {} ({})",
        decoded.to_lan(),
        san_game.move_to_san(&decoded)
    );

    let mut next_game = game.clone();
    assert!(next_game.apply_maia2_action(action));
    println!("FEN after applying that MAIA2 action:");
    println!("{}", next_game.to_fen());
}

fn main() {
    let mut game = StandardGame::standard();
    let (data, num_planes, height, width) = encode::encode_spatial_game_planes(&mut game);

    println!("Encoded game planes: {num_planes} x {height} x {width}");
    println!("Flat data length: {}", data.len());
    println!(
        "AlphaZero action planes: {}",
        encode::get_alphazero_move_planes_count(game.width(), game.height())
    );

    let mv = game
        .move_from_lan("e2e4")
        .expect("expected e2e4 to parse from the initial position");
    show_alphazero_roundtrip(&game, &mv);
    show_maia2_roundtrip(&game, &mv);
}

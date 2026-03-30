import spooky_chess as sc


def main() -> None:
    game = sc.Game.standard()
    data, num_planes, height, width = game.encode_game_planes()

    print(f"Encoded game planes: {num_planes} x {height} x {width}")
    print(f"Flat data length: {len(data)}")
    print(f"Action planes: {game.action_planes_count()}")
    print(f"Total actions: {game.total_actions()}")

    legal_actions = game.legal_action_indices()
    print(f"Legal actions in the current position: {len(legal_actions)}")
    print("Legal action -> move pairs:")
    for action in legal_actions:
        decoded = game.decode_action(action)
        assert decoded is not None
        print(f"  {action:>4} -> {decoded.to_lan()}")

    move_ = game.move_from_lan("e2e4")
    action = move_.encode(game.width(), game.height())
    assert action is not None
    assert action in legal_actions

    decoded = game.decode_action(action)
    assert decoded is not None
    print()
    print(f"Move {move_.to_lan()} encodes to action {action}")
    print(f"Action {action} decodes to {decoded.to_lan()} ({game.move_to_san(decoded)})")

    next_game = game.clone()
    assert next_game.apply_action(action)
    print("FEN after applying that action:")
    print(next_game.to_fen())


if __name__ == "__main__":
    main()

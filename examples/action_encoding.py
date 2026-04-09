import spooky_chess as sc


def show_alphazero_roundtrip(game: sc.Game, move_: sc.Move) -> None:
    total_actions = game.alphazero_total_actions()
    print()
    print("AlphaZero")
    print(f"Total actions: {total_actions}")

    legal_actions = game.legal_alphazero_action_indices()
    print(f"Legal actions in the current position: {len(legal_actions)}")
    print("Legal action -> move pairs:")
    for action in legal_actions:
        decoded = game.decode_alphazero_action(action)
        assert decoded is not None
        print(f"  {action:>4} -> {decoded.to_lan()}")

    action = move_.encode_alphazero_action(game.turn(), game.width(), game.height())
    assert action is not None
    assert action in legal_actions

    decoded = game.decode_alphazero_action(action)
    assert decoded is not None
    print()
    print(f"Move {move_.to_lan()} encodes to AlphaZero action {action}")
    print(f"AlphaZero action {action} decodes to {decoded.to_lan()} ({game.move_to_san(decoded)})")

    next_game = game.clone()
    assert next_game.apply_alphazero_action(action)
    print("FEN after applying that AlphaZero action:")
    print(next_game.to_fen())


def show_maia2_roundtrip(game: sc.Game, move_: sc.Move) -> None:
    total_actions = game.maia2_total_actions()
    assert total_actions is not None
    print()
    print("MAIA2")
    print(f"Total actions: {total_actions}")

    legal_actions = game.legal_maia2_action_indices()
    print(f"Legal actions in the current position: {len(legal_actions)}")
    print("Legal action -> move pairs:")
    for action in legal_actions:
        decoded = game.decode_maia2_action(action)
        assert decoded is not None
        print(f"  {action:>4} -> {decoded.to_lan()}")

    action = move_.encode_maia2_action(game.turn())
    assert action is not None
    assert action in legal_actions

    decoded = game.decode_maia2_action(action)
    assert decoded is not None
    print()
    print(f"Move {move_.to_lan()} encodes to MAIA2 action {action}")
    print(f"MAIA2 action {action} decodes to {decoded.to_lan()} ({game.move_to_san(decoded)})")

    next_game = game.clone()
    assert next_game.apply_maia2_action(action)
    print("FEN after applying that MAIA2 action:")
    print(next_game.to_fen())


def main() -> None:
    game = sc.Game.standard()
    data, num_planes, height, width = game.encode_game_planes()

    print(f"Encoded game planes: {num_planes} x {height} x {width}")
    print(f"Flat data length: {len(data)}")
    print(f"AlphaZero action planes: {game.alphazero_action_planes_count()}")

    move_ = game.move_from_lan("e2e4")
    show_alphazero_roundtrip(game, move_)
    show_maia2_roundtrip(game, move_)


if __name__ == "__main__":
    main()

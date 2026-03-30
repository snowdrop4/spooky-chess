import spooky_chess as sc


def main() -> None:
    game = sc.Game.standard()
    assert game.make_move_from_san("e4")
    assert game.make_move_from_san("e5")

    print(game.turn() == sc.WHITE)
    print(game.to_fen())


if __name__ == "__main__":
    main()

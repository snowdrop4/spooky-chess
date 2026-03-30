import spooky_chess as sc


def side_name(color: int) -> str:
    return "White" if color == sc.WHITE else "Black"


def main() -> None:
    game = sc.Game.standard()
    assert game.make_move_from_san("e4")
    assert game.make_move_from_san("e5")

    print("Position after 1.e4 e5")
    print(f"Turn: {side_name(game.turn())}")
    print(f"In check: {game.is_check()}")

    legal_rows = sorted((game.move_to_san(move_), move_.to_lan()) for move_ in game.legal_moves())
    print(f"{len(legal_rows)} legal moves:")
    for san, lan in legal_rows:
        print(f"  {san:<6} {lan}")

    knight = sc.Position.from_algebraic("g1")
    knight_rows = sorted(
        (game.move_to_san(move_), move_.to_lan()) for move_ in game.legal_moves_for_position(knight.col(), knight.row())
    )
    print()
    print("Moves from g1:")
    for san, lan in knight_rows:
        print(f"  {san:<6} {lan}")


if __name__ == "__main__":
    main()

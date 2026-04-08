import spooky_chess as sc

CUSTOM_FEN = "rnbkqr/pppppp/6/6/PPPPPP/RNBKQR w - - 0 1"


def main() -> None:
    game = sc.Game(width=6, height=6, fen=CUSTOM_FEN, castling_enabled=True)

    print(f"Board shape (height, width): {game.board_shape()}")
    print(f"Initial FEN: {game.to_fen()}")
    print(f"White pieces: {len(game.pieces(sc.WHITE))}")
    print(f"Black pieces: {len(game.pieces(sc.BLACK))}")
    print(f"AlphaZero total actions for 6x6: {game.alphazero_total_actions()}")

    legal_moves = sorted(move_.to_lan() for move_ in game.legal_moves())
    print(f"Legal moves from the starting 6x6 position: {len(legal_moves)}")
    print(f"First 10 legal moves: {', '.join(legal_moves[:10])}")

    move_ = game.move_from_lan("a2a3")
    assert game.make_move(move_)
    print("After a2a3:")
    print(game.to_fen())


if __name__ == "__main__":
    main()

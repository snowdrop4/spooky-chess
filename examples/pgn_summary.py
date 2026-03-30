from pathlib import Path

import spooky_chess as sc

PGN_PATH = Path("pgn/example/multi_game.pgn")


def print_game_summary(index: int, pgn_game: sc.PgnGame) -> None:
    final_game = pgn_game.game()

    print()
    print(f"Game {index}: {pgn_game.white() or '?'} vs {pgn_game.black() or '?'}")
    print(f"  event={pgn_game.event() or '?'}")
    print(f"  result={pgn_game.result()}")
    print(f"  ply={len(pgn_game.moves())}")
    print(f"  starting_fen={pgn_game.starting_fen() or 'standard start'}")
    print(f"  final_fen={final_game.to_fen()}")


def main() -> None:
    games = sc.parse_pgn(PGN_PATH.read_text())
    print(f"Loaded {len(games)} game(s) from {PGN_PATH}")

    for index, pgn_game in enumerate(games, start=1):
        print_game_summary(index, pgn_game)


if __name__ == "__main__":
    main()

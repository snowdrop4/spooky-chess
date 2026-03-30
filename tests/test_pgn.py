import spooky_chess

SCHOLARS_MATE_PGN = """[Event "Test"]
[Site "Test"]
[Date "2024.01.01"]
[White "Player1"]
[Black "Player2"]
[Result "1-0"]

1. e4 e5 2. Bc4 Nc6 3. Qh5 Nf6 4. Qxf7# 1-0
"""


def test_pgn_headers_can_be_modified_and_serialized() -> None:
    game = spooky_chess.parse_pgn(SCHOLARS_MATE_PGN)[0]

    game.set_header("event", "Updated Event")
    game.set_header("Annotator", "spooky_chess")
    game.set_header("result", "0-1")

    assert game.event() == "Updated Event"
    assert game.header("annotator") == "spooky_chess"
    assert game.result() == "0-1"

    assert game.remove_header("site") is True
    assert game.remove_header("site") is False
    assert game.site() is None

    serialized = game.to_pgn()

    assert serialized == str(game)
    assert '[Event "Updated Event"]' in serialized
    assert '[Annotator "spooky_chess"]' in serialized
    assert '[Result "0-1"]' in serialized
    assert '[Site "' not in serialized
    assert serialized.strip().endswith("0-1")

    round_tripped = spooky_chess.parse_pgn(serialized)[0]
    assert round_tripped.event() == "Updated Event"
    assert round_tripped.header("Annotator") == "spooky_chess"
    assert round_tripped.site() is None
    assert round_tripped.result() == "0-1"
    assert len(round_tripped.moves()) == len(game.moves())

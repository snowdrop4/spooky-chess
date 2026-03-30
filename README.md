# spooky_chess 🎃👻

Rust and Python library for the game of Chess.

# Features

- Drive external engines with [Universal Chess Interface](https://en.wikipedia.org/wiki/Universal_Chess_Interface).
- Variable board sizes from 6x6 to 16x16.
- Relatively fast.
- Out-of-the-box support for DL/ML (action encoding and decoding methods).
- Consistent interface with [spooky-connect4](https://github.com/snowdrop4/spooky-connect4) and [spooky-go](https://github.com/snowdrop4/spooky-go).

# Performance

Measured with a Threadripper 9980x, and 6400 MT/s CL36 DDR5. Python 3.14.

```fish
> cd tests/comparison && cargo run --release && cd -
50000 random game playouts
  spooky_chess (Rust Bindings):
    moves:   4936906
    time:    2.14s
    moves/s: 2311096.73
```

```fish
> uv run python -m pytest -k test_compare_random_game_playout -s --run-slow
50000 random game playouts
  spooky_chess (Python Bindings):
    moves:   4936141
    time:    6.66s
    moves/s: 740666.24
  python-chess:
    moves:   4935315
    time:    113.33s
    moves/s: 43548.00
  Speedup: 17.01x
```

# Validity

Fuzz-tested against python-chess, with 5 million random playouts.

# Install

## Rust

```fish
cargo add spooky_chess
```

## Python

```fish
uv add spooky-chess
```

Includes type hints.

# Examples

Quick start:

Rust:

```fish
cargo run --example quick_start
```

Python:

```fish
uv run python examples/quick_start.py
```

More Rust examples:

- `cargo run --example pgn_summary`
- `cargo run --example legal_moves`
- `cargo run --example action_encoding`
- `cargo run --example custom_board`

More Python examples:

- `uv run python examples/pgn_summary.py`
- `uv run python examples/legal_moves.py`
- `uv run python examples/action_encoding.py`
- `uv run python examples/custom_board.py`

PGN + UCI analysis:

Rust:

```fish
cargo run --example analyse_pgn
```

Python:

```fish
uv run python examples/analyse_pgn.py
```

# Develop

### Tests

- `fish run_tests.fish`
    - `fish run_python_tests.fish`
    - `fish run_rust_tests.fish`

### Lints

- `fish run_lints.fish`

### Docs

- `uv run --with sphinx==9.1.0 python -m sphinx -b html -E docs docs/_build/html`

### Performance

- `fish run_benchmark.fish`
- `fish run_profile.fish`

# See Also

* spooky-chess
* [spooky-connect4](https://github.com/snowdrop4/spooky-connect4)
* [spooky-go](https://github.com/snowdrop4/spooky-go)

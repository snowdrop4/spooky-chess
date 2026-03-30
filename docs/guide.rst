Guide
=====

Installation
------------

Install the published package with ``uv``:

.. code-block:: console

   uv add spooky-chess

Overview
--------

The package includes:

* ``Game``, ``Move``, ``Piece``, and ``Position`` for board state and move handling.
* ``parse_pgn`` and ``PgnGame`` for PGN parsing.
* ``UciEngine`` and ``SearchResult`` for talking to external UCI engines such as Stockfish.
* Action/state encoding helpers and constants for ML-oriented workflows.

Examples
--------

Quick Start
^^^^^^^^^^^

Create a standard game, apply SAN moves directly, and inspect the resulting position:

.. literalinclude:: ../examples/quick_start.py
   :language: python
   :caption: examples/quick_start.py

PGN + UCI Analysis
^^^^^^^^^^^^^^^^^^

The repository includes a complete example that replays a PGN and queries a UCI engine at each position:

.. literalinclude:: ../examples/analyse_pgn.py
   :language: python
   :caption: examples/analyse_pgn.py

PGN Summary
^^^^^^^^^^^

Parse one or more PGN games and print headers, results, and final FENs:

.. literalinclude:: ../examples/pgn_summary.py
   :language: python
   :caption: examples/pgn_summary.py

Legal Moves
^^^^^^^^^^^

Inspect legal moves in SAN and LAN from a live position:

.. literalinclude:: ../examples/legal_moves.py
   :language: python
   :caption: examples/legal_moves.py

Action Encoding
^^^^^^^^^^^^^^^

Show input-plane encoding and action-index round-tripping:

.. literalinclude:: ../examples/action_encoding.py
   :language: python
   :caption: examples/action_encoding.py

Custom Board
^^^^^^^^^^^^

Create and play on a 6x6 board:

.. literalinclude:: ../examples/custom_board.py
   :language: python
   :caption: examples/custom_board.py

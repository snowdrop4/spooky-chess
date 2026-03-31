from typing import Final

import numpy as np
import numpy.typing as npt

WHITE: Final[int]
"""Side-to-move constant for White."""

BLACK: Final[int]
"""Side-to-move constant for Black."""

TOTAL_INPUT_PLANES: Final[int]
"""Total number of input planes for the encoder."""

HISTORY_LENGTH: Final[int]
"""Number of positions in the game history to encode."""

PIECE_PLANES: Final[int]
"""Number of piece planes per history step."""

CONSTANT_PLANES: Final[int]
"""Number of constant planes."""

NUM_DIRECTIONS: Final[int]
"""Number of sliding directions."""

NUM_KNIGHT_DELTAS: Final[int]
"""Number of knight move patterns."""

NUM_UNDERPROMO_DIRECTIONS: Final[int]
"""Number of underpromotion directions."""

NUM_UNDERPROMO_PIECES: Final[int]
"""Number of underpromotion piece types."""

NUM_PROMOTION_ORIENTATIONS: Final[int]
"""Number of promotion orientations."""

def augment_symmetries(
    states: npt.NDArray[np.float32],
    policies: npt.NDArray[np.float32],
    values: npt.NDArray[np.float32],
    opponent_policies: npt.NDArray[np.float32],
    opponent_policy_masks: npt.NDArray[np.float32],
) -> tuple[
    npt.NDArray[np.float32],
    npt.NDArray[np.float32],
    npt.NDArray[np.float32],
    npt.NDArray[np.float32],
    npt.NDArray[np.float32],
]:
    """Augment a chess training batch with its horizontal mirror."""

def parse_pgn(pgn: str) -> list[PgnGame]:
    """Parse one or more PGN games from a string."""

class Game:
    """A mutable chess game on a `width x height` board."""

    def __init__(self, width: int, height: int, fen: str, castling_enabled: bool) -> None:
        """Create a game from FEN."""

    @staticmethod
    def standard() -> Game:
        """Create the standard initial 8x8 chess position."""

    def turn(self) -> int:
        """Return the side to move as `WHITE` or `BLACK`."""

    def fullmove_number(self) -> int:
        """Return the fullmove number."""

    def halfmove_clock(self) -> int:
        """Return the halfmove clock."""

    def ply(self) -> int:
        """Return the number of applied moves in history."""

    def castling_enabled(self) -> bool:
        """Return whether castling rules are enabled."""

    def has_kingside_castling_rights(self, color: int) -> bool:
        """Return whether `color` may castle kingside."""

    def has_queenside_castling_rights(self, color: int) -> bool:
        """Return whether `color` may castle queenside."""

    def make_move(self, move_: Move) -> bool:
        """Try to apply `move_`. Returns `True` if it is legal and applied."""

    def make_move_unchecked(self, move_: Move) -> None:
        """Apply a move that is already known to be legal."""

    def move_history(self) -> list[Move]:
        """Return the applied move history."""

    def unmake_move(self) -> bool:
        """Undo the last applied move. Returns `False` if history is empty."""

    def is_legal_move(self, move_: Move) -> bool:
        """Return whether `move_` is legal in the current position."""

    def legal_moves(self) -> list[Move]:
        """Return all legal moves for the side to move."""

    def pseudo_legal_moves(self) -> list[Move]:
        """Return pseudo-legal moves from the current position. They may still leave the king in check."""

    def legal_moves_for_position(self, col: int, row: int) -> list[Move]:
        """Return legal moves for one source square."""

    def move_to_lan(self, move_: Move) -> str:
        """Format a move as LAN."""

    def move_from_lan(self, lan: str) -> Move:
        """Parse LAN in the current position."""

    def move_to_san(self, move_: Move) -> str:
        """Format a move as SAN in the current position."""

    def move_from_san(self, san: str) -> Move:
        """Parse SAN in the current position."""

    def is_check(self) -> bool:
        """Return whether the side to move is in check."""

    def is_checkmate(self) -> bool:
        """Return whether the side to move is checkmated."""

    def is_stalemate(self) -> bool:
        """Return whether the side to move is stalemated."""

    def is_over(self) -> bool:
        """Return whether the game is over."""

    def width(self) -> int:
        """Return the board width."""

    def height(self) -> int:
        """Return the board height."""

    def get_piece(self, col: int, row: int) -> Piece | None:
        """Return the piece at `(col, row)`, if any."""

    def pieces(self, color: int) -> list[tuple[Position, Piece]]:
        """Return all pieces of one color."""

    def set_piece(self, col: int, row: int, piece: Piece | None = None) -> None:
        """Set a piece directly on the board without updating move history."""

    def piece_count(self, piece_type: str, color: int) -> int:
        """Return the count for one piece kind and color."""

    def __getitem__(self, key: str | tuple[int, int]) -> Piece | None:
        """Look up a piece by algebraic square or `(col, row)`."""

    def legal_action_indices(self) -> list[int]:
        """Return encoded action indices for all legal moves."""

    def apply_action(self, action: int) -> bool:
        """Decode and apply an action without legality checking. Returns `False` if it cannot be decoded."""

    def encode_game_planes(self) -> tuple[list[float], int, int, int]:
        """Encode the game as flat planes: `(data, num_planes, height, width)`."""

    def action_planes_count(self) -> int:
        """Return the number of move-policy planes for this board size."""

    def decode_action(self, action: int) -> Move | None:
        """Decode an action index into a move."""

    def total_actions(self) -> int:
        """Return the total number of action indices for this board size."""

    def board_shape(self) -> tuple[int, int]:
        """Return the board shape as `(height, width)`."""

    def input_plane_count(self) -> int:
        """Return the total number of input planes."""

    def reward_absolute(self) -> float:
        """Encode the current outcome as `1.0`, `-1.0`, or `0.0`."""

    def reward_from_perspective(self, perspective: int) -> float:
        """Encode the current outcome from one side's perspective."""

    def is_insufficient_material(self) -> bool:
        """Return whether neither side has mating material."""

    def has_legal_en_passant(self) -> bool:
        """Return whether the current en passant square is legally capturable."""

    def en_passant_square(self) -> Position | None:
        """Return the current en passant target square, if any."""

    def outcome(self) -> GameOutcome | None:
        """Return the current game outcome, if the game is over."""

    def turn_state(self) -> TurnState:
        """Return either the current legal moves or the terminal outcome."""

    def to_fen(self) -> str:
        """Serialize the current position as FEN."""

    def clone(self) -> Game:
        """Return a copy of the game."""

    def state_hash(self) -> int:
        """Return a hash of the full game state, including history-conditioned NN state."""

    def transposition_hash(self) -> int:
        """Return a hash of the full game state, but without any move history."""

    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __hash__(self) -> int: ...

class Move:
    """A chess move."""

    @staticmethod
    def from_rowcol(src_col: int, src_row: int, dst_col: int, dst_row: int) -> Move:
        """Create a move from zero-based coordinates."""

    @classmethod
    def from_lan(cls, lan: str, board_width: int, board_height: int) -> Move:
        """Parse LAN like `e2e4` or `a7a8q`."""

    @property
    def src(self) -> Position:
        """Source square."""

    @property
    def dst(self) -> Position:
        """Destination square."""

    def src_square(self) -> tuple[int, int]:
        """Return the source square as `(col, row)`."""

    def dst_square(self) -> tuple[int, int]:
        """Return the destination square as `(col, row)`."""

    def promotion(self) -> str | None:
        """Return the promotion piece, if any."""

    def to_lan(self) -> str:
        """Format the move as LAN."""

    @property
    def is_capture(self) -> bool:
        """Whether the move captures a piece."""

    @property
    def is_castling(self) -> bool:
        """Whether the move castles."""

    @property
    def is_en_passant(self) -> bool:
        """Whether the move captures en passant."""

    @property
    def is_promotion(self) -> bool:
        """Whether the move promotes a pawn."""

    @property
    def is_check(self) -> bool:
        """Whether the move gives check."""

    @property
    def is_double_push(self) -> bool:
        """Whether the move is a two-square pawn push."""

    def encode(self, width: int, height: int) -> int | None:
        """Encode the move as an action index for a board size."""

    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: Move) -> bool: ...
    def __hash__(self) -> int: ...

class Piece:
    """A colored chess piece."""

    def __init__(self, piece_type: str, color: int) -> None:
        """Create a piece from a piece kind and color."""

    def piece_type(self) -> str:
        """Return the lowercase FEN character for the piece kind."""

    def color(self) -> int:
        """Return the piece color as `WHITE` or `BLACK`."""

    @property
    def is_white(self) -> bool:
        """Whether the piece is white."""

    @property
    def is_black(self) -> bool:
        """Whether the piece is black."""

    def symbol(self) -> str:
        """Return the FEN character for the piece."""

    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: Piece) -> bool: ...
    def __hash__(self) -> int: ...

class Position:
    """A zero-based board position."""

    def __init__(self, col: int, row: int) -> None:
        """Create a position from zero-based coordinates."""

    @classmethod
    def from_algebraic(cls, s: str) -> Position:
        """Parse algebraic notation like `e4`."""

    def to_algebraic(self) -> str:
        """Format the position as algebraic notation like `e4`."""

    def col(self) -> int:
        """Return the zero-based file."""

    def row(self) -> int:
        """Return the zero-based rank."""

    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: Position) -> bool: ...
    def __hash__(self) -> int: ...

class GameOutcome:
    """The final result of a game."""

    def winner(self) -> int | None:
        """Return the winning color, if any."""

    def encode_winner_absolute(self) -> float:
        """Encode white win as `1.0`, black win as `-1.0`, and draws as `0.0`."""

    def encode_winner_from_perspective(self, perspective: int) -> float:
        """Encode the outcome from one side's perspective."""

    def is_draw(self) -> bool:
        """Return whether the outcome is a draw."""

    def is_checkmate(self) -> bool:
        """Return whether the outcome is a win by checkmate."""

    def is_stalemate(self) -> bool:
        """Return whether the outcome is a stalemate."""

    def is_insufficient_material(self) -> bool:
        """Return whether the outcome is insufficient material."""

    def is_threefold_repetition(self) -> bool:
        """Return whether the outcome is threefold repetition."""

    def is_fifty_move_rule(self) -> bool:
        """Return whether the outcome is the fifty-move rule."""

    def reason(self) -> str:
        """Return the outcome name."""

    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: GameOutcome) -> bool: ...
    def __hash__(self) -> int: ...

class TurnState:
    """The current turn state."""

    def is_over(self) -> bool:
        """Return whether the game is over."""

    def outcome(self) -> GameOutcome | None:
        """Return the terminal outcome, if any."""

    def legal_moves(self) -> list[Move]:
        """Return the current legal moves, or an empty list if the game is over."""

    def __repr__(self) -> str: ...

class PgnGame:
    """A parsed PGN game."""

    def headers(self) -> dict[str, str]:
        """Return raw PGN headers."""

    def header(self, key: str) -> str | None:
        """Return a header value by key, case-insensitively."""

    def white(self) -> str | None:
        """Return the `White` header."""

    def black(self) -> str | None:
        """Return the `Black` header."""

    def event(self) -> str | None:
        """Return the `Event` header."""

    def site(self) -> str | None:
        """Return the `Site` header."""

    def date(self) -> str | None:
        """Return the `Date` header."""

    def result(self) -> str:
        """Return the PGN result token."""

    def moves(self) -> list[Move]:
        """Return the moves in internal move form."""

    def starting_fen(self) -> str | None:
        """Return the starting FEN if the PGN uses `SetUp` and `FEN`."""

    def starting_game(self) -> Game:
        """Build the starting game described by the PGN."""

    def game(self) -> Game:
        """Return the final board state after all moves."""

    def __repr__(self) -> str: ...

class SearchResult:
    """Result of a UCI `go` command."""

    best_move: Move
    """The engine's chosen move."""

    best_move_lan: str
    """The engine's chosen move in LAN form."""

    ponder_move: Move | None
    """The engine's ponder move, if any."""

    ponder_move_lan: str | None
    """The engine's ponder move in LAN form."""

    score_cp: int | None
    """Centipawn score from the deepest `info` line, if present."""

    score_mate: int | None
    """Mate distance score from the deepest `info` line, if present."""

    depth: int | None
    """Search depth from the deepest `info` line, if present."""

    nodes: int | None
    """Node count from the deepest `info` line, if present."""

    pv: list[str]
    """Principal variation as LAN moves."""

    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...

class UciEngine:
    """A wrapper around an external UCI engine process."""

    def __init__(
        self,
        program: str,
        args: list[str] = ...,
    ) -> None:
        """Spawn a UCI engine process and perform the UCI handshake."""

    def engine_name(self) -> str | None:
        """Return the engine name reported during the UCI handshake."""

    def engine_author(self) -> str | None:
        """Return the engine author reported during the UCI handshake."""

    def set_option(self, name: str, value: str) -> None:
        """Send `setoption name <name> value <value>` to the engine."""

    def is_ready(self) -> None:
        """Send `isready` and wait for `readyok`."""

    def new_game(self) -> None:
        """Tell the engine a new game is starting and reset internal state."""

    def new_game_from_fen(self, fen: str) -> None:
        """Tell the engine a new game is starting and initialize it from a FEN."""

    def set_position_startpos(self) -> None:
        """Reset the internal game to the standard starting position."""

    def set_position_fen(self, fen: str) -> None:
        """Reset the internal game to a position given by FEN."""

    def set_position_pgn_start(self, pgn_game: PgnGame) -> None:
        """Reset the internal game to the starting position described by a PGN game."""

    def make_move(self, move_: Move) -> bool:
        """Apply `move_` to the mirrored game state. Returns `True` if it is legal."""

    def make_move_lan(self, lan: str) -> bool:
        """Parse a LAN move in the current position, then apply it."""

    def go_depth(self, depth: int) -> SearchResult:
        """Search with a depth limit."""

    def go_movetime(self, ms: int) -> SearchResult:
        """Search with a time limit in milliseconds."""

    def go_clock(self, wtime: int, btime: int, winc: int, binc: int) -> SearchResult:
        """Search with clock parameters."""

    def go_bestmove_depth(self, depth: int) -> Move:
        """Search to a depth, apply the best move, and return it."""

    def go_bestmove_movetime(self, ms: int) -> Move:
        """Search for a time limit, apply the best move, and return it."""

    def turn(self) -> int:
        """Return the side to move as `WHITE` or `BLACK`."""

    def fullmove_number(self) -> int:
        """Return the fullmove number."""

    def halfmove_clock(self) -> int:
        """Return the halfmove clock."""

    def castling_enabled(self) -> bool:
        """Return whether castling rules are enabled."""

    def has_kingside_castling_rights(self, color: int) -> bool:
        """Return whether `color` may castle kingside."""

    def has_queenside_castling_rights(self, color: int) -> bool:
        """Return whether `color` may castle queenside."""

    def is_check(self) -> bool:
        """Return whether the side to move is in check."""

    def is_over(self) -> bool:
        """Return whether the game is over."""

    def outcome(self) -> GameOutcome | None:
        """Return the current outcome, if the game is over."""

    def turn_state(self) -> TurnState:
        """Return the current turn state."""

    def is_checkmate(self) -> bool:
        """Return whether the side to move is checkmated."""

    def is_stalemate(self) -> bool:
        """Return whether the side to move is stalemated."""

    def is_insufficient_material(self) -> bool:
        """Return whether neither side has mating material."""

    def has_legal_en_passant(self) -> bool:
        """Return whether the current en passant square is legally capturable."""

    def en_passant_square(self) -> Position | None:
        """Return the current en passant target square, if any."""

    def legal_moves(self) -> list[Move]:
        """Return all legal moves from the current position."""

    def pseudo_legal_moves(self) -> list[Move]:
        """Return pseudo-legal moves from the current position. They may still leave the king in check."""

    def legal_moves_for_position(self, col: int, row: int) -> list[Move]:
        """Return legal moves for one source square."""

    def is_legal_move(self, move_: Move) -> bool:
        """Return whether `move_` is legal in the current position."""

    def move_to_lan(self, move_: Move) -> str:
        """Format a move as LAN."""

    def move_from_lan(self, lan: str) -> Move:
        """Parse LAN in the current position."""

    def move_to_san(self, move_: Move) -> str:
        """Format a move as SAN."""

    def move_from_san(self, san: str) -> Move:
        """Parse SAN in the current position."""

    def width(self) -> int:
        """Return the board width."""

    def height(self) -> int:
        """Return the board height."""

    def get_piece(self, col: int, row: int) -> Piece | None:
        """Return the piece at `(col, row)`, if any."""

    def to_fen(self) -> str:
        """Serialize the current position as FEN."""

    def undo(self) -> None:
        """Undo the last move."""

    def send_command(self, cmd: str) -> str:
        """Send a raw UCI command and return the response line."""

    def quit(self) -> None:
        """Send `quit` to the engine."""

    def __enter__(self) -> UciEngine: ...
    def __exit__(self, exc_type: type | None, exc_value: BaseException | None, traceback: object | None) -> bool: ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...

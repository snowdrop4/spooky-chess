//! Neural-network state and action encoders.

use std::cmp::Reverse;
use std::collections::HashMap;
use std::sync::OnceLock;

use crate::color::Color;
use crate::directions::{KNIGHT_DELTAS, direction_index};
use crate::game::Game;
use crate::r#move::{Move, MoveFlags};
use crate::pieces::PieceType;
use crate::position::Position;

/// Number of piece planes per history step: 6 for white and 6 for black.
pub const PIECE_PLANES: usize = 6 + 6;

/// Number of constant planes: 2 repetition, 1 side-to-move, 1 move count,
/// 4 castling, and 1 no-progress plane.
pub const CONSTANT_PLANES: usize = 2 + 1 + 1 + 4 + 1;

/// Number of positions in the game history to encode.
pub const HISTORY_LENGTH: usize = 8;

/// Total number of board-shaped spatial planes returned by
/// [`encode_spatial_game_planes`].
pub const SPATIAL_INPUT_PLANES: usize = (HISTORY_LENGTH * PIECE_PLANES) + CONSTANT_PLANES;

/// Total number of separate non-spatial global input features.
///
/// Chess does not currently expose any, so this is `0`.
pub const GLOBAL_INPUT_FEATURES: usize = 0;

/// Number of sliding directions: N, NE, E, SE, S, SW, W, NW.
pub const NUM_DIRECTIONS: usize = 8;

/// Number of knight move patterns.
pub const NUM_KNIGHT_DELTAS: usize = 8;

/// Number of underpromotion directions: left capture, straight, and right capture.
pub const NUM_UNDERPROMO_DIRECTIONS: usize = 3;

/// Number of underpromotion piece types: knight, bishop, and rook.
pub const NUM_UNDERPROMO_PIECES: usize = 3;

/// Number of underpromotion orientations in AlphaZero's action space.
///
/// AlphaZero normalizes moves into the current side-to-move's frame, so there
/// is only one forward underpromotion orientation.
pub const NUM_PROMOTION_ORIENTATIONS: usize = 1;

/// Total MAIA2 action indices for standard 8x8 chess.
pub const MAIA2_TOTAL_ACTIONS_STANDARD: usize = 1880;

/// Normalization divisor for fullmove number in the spatial encoder planes.
const FULLMOVE_SCALE: f32 = 100.0;

/// Normalization divisor for halfmove clock (no-progress count) in the spatial
/// encoder planes.
const HALFMOVE_SCALE: f32 = 50.0;

const STANDARD_BOARD_SIZE: usize = 8;

struct Maia2ActionTable {
    moves: Vec<String>,
    indices: HashMap<String, usize>,
}

static MAIA2_ACTION_TABLE: OnceLock<Maia2ActionTable> = OnceLock::new();

fn is_standard_board(width: usize, height: usize) -> bool {
    width == STANDARD_BOARD_SIZE && height == STANDARD_BOARD_SIZE
}

fn mirror_position_vertically(pos: Position, height: usize) -> Position {
    Position::new(pos.col, (height - 1 - usize::from(pos.row)) as u8)
}

fn mirror_move_for_turn(move_: &Move, turn: Color, height: usize) -> Move {
    if turn == Color::White {
        *move_
    } else {
        Move {
            src: mirror_position_vertically(move_.src, height),
            dst: mirror_position_vertically(move_.dst, height),
            flags: move_.flags,
            promotion: move_.promotion,
        }
    }
}

fn build_maia2_action_table() -> Maia2ActionTable {
    const QUEEN_DELTAS: [(i32, i32); 8] = [
        (0, 1),
        (1, 1),
        (1, 0),
        (1, -1),
        (0, -1),
        (-1, -1),
        (-1, 0),
        (-1, 1),
    ];
    const MAIA2_PROMOTION_ORDER: [PieceType; 4] = [
        PieceType::Queen,
        PieceType::Rook,
        PieceType::Bishop,
        PieceType::Knight,
    ];

    let mut moves = Vec::with_capacity(MAIA2_TOTAL_ACTIONS_STANDARD);

    for row in 0..STANDARD_BOARD_SIZE {
        for col in 0..STANDARD_BOARD_SIZE {
            let src = Position::from_usize(col, row);

            let mut queen_targets = Vec::new();
            for (dx, dy) in QUEEN_DELTAS {
                let mut dst_col = col as i32 + dx;
                let mut dst_row = row as i32 + dy;
                while (0..STANDARD_BOARD_SIZE as i32).contains(&dst_col)
                    && (0..STANDARD_BOARD_SIZE as i32).contains(&dst_row)
                {
                    queen_targets.push(Position::new(dst_col as u8, dst_row as u8));
                    dst_col += dx;
                    dst_row += dy;
                }
            }
            queen_targets.sort_by_key(|pos| Reverse(pos.to_index(STANDARD_BOARD_SIZE)));
            for dst in queen_targets {
                moves.push(Move::from_position(src, dst, MoveFlags::empty()).to_lan());
            }

            let mut knight_targets: Vec<_> = KNIGHT_DELTAS
                .iter()
                .filter_map(|&(dx, dy)| {
                    let dst_col = col as i32 + dx;
                    let dst_row = row as i32 + dy;
                    ((0..STANDARD_BOARD_SIZE as i32).contains(&dst_col)
                        && (0..STANDARD_BOARD_SIZE as i32).contains(&dst_row))
                    .then(|| Position::new(dst_col as u8, dst_row as u8))
                })
                .collect();
            knight_targets.sort_by_key(|pos| Reverse(pos.to_index(STANDARD_BOARD_SIZE)));
            for dst in knight_targets {
                moves.push(Move::from_position(src, dst, MoveFlags::empty()).to_lan());
            }
        }
    }

    for file in b'a'..=b'h' {
        let src_file = file as char;

        for piece in MAIA2_PROMOTION_ORDER {
            moves.push(format!("{src_file}7{src_file}8{}", piece.to_char()));
        }

        if file != b'a' {
            let left_file = (file - 1) as char;
            for piece in MAIA2_PROMOTION_ORDER {
                moves.push(format!("{src_file}7{left_file}8{}", piece.to_char()));
            }
        }

        if file != b'h' {
            let right_file = (file + 1) as char;
            for piece in MAIA2_PROMOTION_ORDER {
                moves.push(format!("{src_file}7{right_file}8{}", piece.to_char()));
            }
        }
    }

    debug_assert_eq!(
        moves.len(),
        MAIA2_TOTAL_ACTIONS_STANDARD,
        "unexpected MAIA2 action count",
    );

    let indices = moves
        .iter()
        .enumerate()
        .map(|(idx, move_uci)| (move_uci.clone(), idx))
        .collect();

    Maia2ActionTable { moves, indices }
}

fn maia2_action_table() -> &'static Maia2ActionTable {
    MAIA2_ACTION_TABLE.get_or_init(build_maia2_action_table)
}

/// Encode the board-shaped portion of the game state into a flat `f32` array.
///
/// Returns `(flat_data, num_planes, height, width)` in row-major order.
///
/// Any separate non-spatial global features are tracked outside this tensor via
/// [`GLOBAL_INPUT_FEATURES`]. Chess currently exposes no such features.
#[hotpath::measure]
pub fn encode_spatial_game_planes<const W: usize, const H: usize>(
    game: &mut Game<W, H>,
) -> (Vec<f32>, usize, usize, usize)
where
    [(); (W * H).div_ceil(64)]:,
{
    let num_planes = SPATIAL_INPUT_PLANES;
    let board_size = H * W;
    let total_size = num_planes * board_size;
    let mut data = vec![0.0f32; total_size];

    let perspective = game.turn();
    let opponent = perspective.opposite();

    let history_len = game.move_count();
    let steps_back = (HISTORY_LENGTH - 1).min(history_len);

    let moves_to_replay: Vec<Move> = game.move_history()[(history_len - steps_back)..]
        .iter()
        .map(|e| e.mv)
        .collect();

    // T=0: current position
    fill_chess_planes::<W, H>(&mut data, game, perspective, 0);

    // T=1..steps_back: walk backward through history
    for t in 1..=steps_back {
        game.unmake_move();
        fill_chess_planes::<W, H>(&mut data, game, perspective, t);
    }

    // Replay saved moves to restore game state
    for mv in &moves_to_replay {
        game.make_move_unchecked(mv);
    }

    debug_assert_eq!(
        game.move_count(),
        history_len,
        "game state not fully restored after encode: move_count {} != original {}",
        game.move_count(),
        history_len,
    );

    // Constant plane layout (relative to constant_start):
    const PLANE_REPETITION_1: usize = 0;
    const PLANE_REPETITION_2: usize = 1;
    const PLANE_COLOR: usize = 2;
    const PLANE_MOVE_COUNT: usize = 3;
    const PLANE_P1_KINGSIDE: usize = 4;
    const PLANE_P1_QUEENSIDE: usize = 5;
    const PLANE_P2_KINGSIDE: usize = 6;
    const PLANE_P2_QUEENSIDE: usize = 7;
    const PLANE_NO_PROGRESS: usize = 8;

    let constant_start = HISTORY_LENGTH * PIECE_PLANES;

    // Repetition count planes - zeros for now (PLANE_REPETITION_1, PLANE_REPETITION_2)
    let _ = (PLANE_REPETITION_1, PLANE_REPETITION_2);

    // Color plane
    let color_value = if perspective == Color::White {
        1.0
    } else {
        0.0
    };
    fill_constant_plane(
        &mut data,
        constant_start + PLANE_COLOR,
        color_value,
        board_size,
    );

    // Total move count plane
    let move_count = game.fullmove_number() as f32 / FULLMOVE_SCALE;
    fill_constant_plane(
        &mut data,
        constant_start + PLANE_MOVE_COUNT,
        move_count,
        board_size,
    );

    // Castling rights (4 planes)
    let castling_rights = game.castling_rights();

    let p1_kingside = if castling_rights.has_kingside(perspective) {
        1.0
    } else {
        0.0
    };
    fill_constant_plane(
        &mut data,
        constant_start + PLANE_P1_KINGSIDE,
        p1_kingside,
        board_size,
    );

    let p1_queenside = if castling_rights.has_queenside(perspective) {
        1.0
    } else {
        0.0
    };
    fill_constant_plane(
        &mut data,
        constant_start + PLANE_P1_QUEENSIDE,
        p1_queenside,
        board_size,
    );

    let p2_kingside = if castling_rights.has_kingside(opponent) {
        1.0
    } else {
        0.0
    };
    fill_constant_plane(
        &mut data,
        constant_start + PLANE_P2_KINGSIDE,
        p2_kingside,
        board_size,
    );

    let p2_queenside = if castling_rights.has_queenside(opponent) {
        1.0
    } else {
        0.0
    };
    fill_constant_plane(
        &mut data,
        constant_start + PLANE_P2_QUEENSIDE,
        p2_queenside,
        board_size,
    );

    // No-progress count plane
    let no_progress = game.halfmove_clock() as f32 / HALFMOVE_SCALE;
    fill_constant_plane(
        &mut data,
        constant_start + PLANE_NO_PROGRESS,
        no_progress,
        board_size,
    );

    (data, num_planes, H, W)
}

#[hotpath::measure]
fn fill_constant_plane(data: &mut [f32], plane: usize, value: f32, board_size: usize) {
    let offset = plane * board_size;
    data[offset..offset + board_size].fill(value);
}

#[inline]
fn piece_type_plane_index(pt: PieceType) -> usize {
    match pt {
        PieceType::Pawn => 0,
        PieceType::Knight => 1,
        PieceType::Bishop => 2,
        PieceType::Rook => 3,
        PieceType::Queen => 4,
        PieceType::King => 5,
    }
}

#[hotpath::measure]
fn fill_chess_planes<const W: usize, const H: usize>(
    data: &mut [f32],
    game: &Game<W, H>,
    perspective: Color,
    t: usize,
) where
    [(); (W * H).div_ceil(64)]:,
{
    let board_size = H * W;
    debug_assert!(
        t < HISTORY_LENGTH,
        "history timestep t={} exceeds HISTORY_LENGTH={}",
        t,
        HISTORY_LENGTH,
    );
    let base_plane = t * PIECE_PLANES;

    for (pos, piece) in game.pieces_iter(perspective) {
        let plane_idx = piece_type_plane_index(piece.piece_type);
        let offset = (base_plane + plane_idx) * board_size;
        let idx = pos.to_index(W);
        debug_assert!(
            idx < board_size,
            "piece position index {} exceeds board_size {}",
            idx,
            board_size,
        );
        data[offset + idx] = 1.0;
    }

    for (pos, piece) in game.pieces_iter(perspective.opposite()) {
        let plane_idx = piece_type_plane_index(piece.piece_type);
        let offset = (base_plane + 6 + plane_idx) * board_size;
        let idx = pos.to_index(W);
        debug_assert!(
            idx < board_size,
            "piece position index {} exceeds board_size {}",
            idx,
            board_size,
        );
        data[offset + idx] = 1.0;
    }
}

/// Encodes a move using AlphaZero's side-to-move-oriented action space.
///
/// Black-to-move positions are mirrored vertically so that pawn advances always
/// move "forward" in the shared policy vocabulary.
#[hotpath::measure]
pub fn encode_alphazero_action(
    move_: &Move,
    turn: Color,
    width: usize,
    height: usize,
) -> Option<usize> {
    debug_assert!(
        usize::from(move_.src.col) < width && usize::from(move_.src.row) < height,
        "encode_alphazero_action: move src ({},{}) out of bounds for {}x{} board",
        move_.src.col,
        move_.src.row,
        width,
        height,
    );
    debug_assert!(
        usize::from(move_.dst.col) < width && usize::from(move_.dst.row) < height,
        "encode_alphazero_action: move dst ({},{}) out of bounds for {}x{} board",
        move_.dst.col,
        move_.dst.row,
        width,
        height,
    );

    if !move_.src.is_valid(width, height) || !move_.dst.is_valid(width, height) {
        return None;
    }

    let normalized_move = mirror_move_for_turn(move_, turn, height);
    let plane = encode_alphazero_move_plane(&normalized_move, width, height)?;
    let board_size = width * height;
    let src_index =
        usize::from(normalized_move.src.row) * width + usize::from(normalized_move.src.col);
    Some(plane * board_size + src_index)
}

/// Decodes an AlphaZero action index into a move.
///
/// The decoded move is mirrored back into the current side-to-move's board
/// orientation.
#[hotpath::measure]
pub(crate) fn decode_alphazero_action(
    action: usize,
    turn: Color,
    width: usize,
    height: usize,
) -> Option<Move> {
    let board_size = width * height;
    let plane_idx = action / board_size;
    let src_index = action % board_size;
    let src_col = src_index % width;
    let src_row = src_index / width;

    let (dx, dy, promo) = decode_alphazero_move_plane(plane_idx, width, height)?;

    let dst_col_i = src_col as i32 + dx;
    let dst_row_i = src_row as i32 + dy;
    if dst_col_i < 0 || dst_row_i < 0 {
        return None;
    }

    let (dst_col, dst_row) = (dst_col_i as usize, dst_row_i as usize);
    if dst_col >= width || dst_row >= height {
        return None;
    }

    let normalized_move = Move {
        src: Position::from_usize(src_col, src_row),
        dst: Position::from_usize(dst_col, dst_row),
        flags: MoveFlags::empty(),
        promotion: promo,
    };
    Some(mirror_move_for_turn(&normalized_move, turn, height))
}

/// Encodes a move using MAIA2's white-oriented action space.
///
/// Only standard 8x8 chess is supported. Black-to-move positions are mirrored
/// vertically before indexing into the shared white-oriented vocabulary.
///
/// Returns `None` on non-standard boards or for moves outside MAIA2's
/// vocabulary.
#[hotpath::measure]
pub fn encode_maia2_action(
    move_: &Move,
    turn: Color,
    width: usize,
    height: usize,
) -> Option<usize> {
    if !is_standard_board(width, height)
        || !move_.src.is_valid(width, height)
        || !move_.dst.is_valid(width, height)
    {
        return None;
    }

    let maia2_move = mirror_move_for_turn(move_, turn, STANDARD_BOARD_SIZE);
    let maia2_lan = maia2_move.to_lan();
    maia2_action_table()
        .indices
        .get(maia2_lan.as_str())
        .copied()
}

/// Decodes a MAIA2 action index into a move.
///
/// The returned move is mirrored back into the current side-to-move's
/// orientation. Only standard 8x8 chess is supported.
///
/// Returns `None` for non-standard boards or out-of-range action indices.
#[hotpath::measure]
pub fn decode_maia2_action(
    action: usize,
    turn: Color,
    width: usize,
    height: usize,
) -> Option<Move> {
    if !is_standard_board(width, height) {
        return None;
    }

    let maia2_lan = maia2_action_table().moves.get(action)?;
    let maia2_move = Move::from_lan(maia2_lan, STANDARD_BOARD_SIZE, STANDARD_BOARD_SIZE).ok()?;
    Some(mirror_move_for_turn(&maia2_move, turn, STANDARD_BOARD_SIZE))
}

/// Returns the total number of AlphaZero action indices for a board size.
#[hotpath::measure]
pub fn get_alphazero_total_actions(width: usize, height: usize) -> usize {
    get_alphazero_move_planes_count(width, height) * width * height
}

/// Returns the MAIA2 action count for a board size.
///
/// Only standard 8x8 chess is supported.
#[hotpath::measure]
pub fn get_maia2_total_actions(width: usize, height: usize) -> Option<usize> {
    is_standard_board(width, height).then_some(MAIA2_TOTAL_ACTIONS_STANDARD)
}

/// Encode a move as an AlphaZero move-plane index.
///
/// The move must already be normalized into the side-to-move's frame.
/// AlphaZero move planes encode:
/// - Horizontal/vertical/diagonal moves, for all non-knight pieces,
///   in 8 directions (N, NE, E, SE, S, SW, W, NW) up to max distance
/// - L-shaped moves for knights, in 8 directions
/// - Forward underpromotions (3 directions × 3 piece types, excluding queen)
#[hotpath::measure]
pub(crate) fn encode_alphazero_move_plane(
    move_: &Move,
    width: usize,
    height: usize,
) -> Option<usize> {
    let src = move_.src;
    let dst = move_.dst;
    let dx = dst.col as i32 - src.col as i32;
    let dy = dst.row as i32 - src.row as i32;

    let max_distance = width.max(height) - 1;

    // L-shaped moves for knights
    for (i, &(kdx, kdy)) in KNIGHT_DELTAS.iter().enumerate() {
        if dx == kdx && dy == kdy {
            let knight_planes_start = NUM_DIRECTIONS * max_distance;
            return Some(knight_planes_start + i);
        }
    }

    // Underpromotions (only for non-queen promotions).
    // The move has already been normalized into the current player's
    // perspective, so underpromotions are always a single forward step.
    if let Some(promo) = move_.promotion
        && promo != PieceType::Queen
        && dy == 1
    {
        let direction_idx = if dx == -1 {
            0 // left diagonal
        } else if dx == 0 {
            1 // straight
        } else if dx == 1 {
            2 // right diagonal
        } else {
            return None;
        };

        let piece_idx = match promo {
            PieceType::Knight => 0,
            PieceType::Bishop => 1,
            PieceType::Rook => 2,
            _ => return None,
        };

        let knight_planes_start = NUM_DIRECTIONS * max_distance;
        let underpromo_planes_start = knight_planes_start + NUM_KNIGHT_DELTAS;
        return Some(underpromo_planes_start + direction_idx * NUM_UNDERPROMO_PIECES + piece_idx);
    }

    // Horizontal/vertical/diagonal moves for all non-knight pieces
    // Verify it's actually a straight/diagonal move (not an arbitrary direction)
    let is_straight_or_diagonal = (dx == 0) != (dy == 0)  // straight
        || (dx.abs() == dy.abs() && dx != 0); // diagonal

    let direction = if is_straight_or_diagonal {
        direction_index(dx, dy)
    } else {
        None
    };

    direction.and_then(|dir| {
        let distance = dx.abs().max(dy.abs()) as usize;
        if distance > 0 && distance <= max_distance {
            Some(dir * max_distance + (distance - 1))
        } else {
            None
        }
    })
}

/// Decode an AlphaZero move plane back to normalized move deltas.
#[hotpath::measure]
pub(crate) fn decode_alphazero_move_plane(
    plane_idx: usize,
    width: usize,
    height: usize,
) -> Option<(i32, i32, Option<PieceType>)> {
    let max_distance = width.max(height) - 1;
    let straight_diagonal_planes = NUM_DIRECTIONS * max_distance;
    let knight_planes_start = straight_diagonal_planes;
    let underpromo_planes_start = knight_planes_start + NUM_KNIGHT_DELTAS;

    if plane_idx < straight_diagonal_planes {
        // Horizontal/vertical/diagonal moves for all non-knight pieces
        let direction = plane_idx / max_distance;
        let distance = (plane_idx % max_distance) + 1;

        let (dx, dy) = match direction {
            0 => (0, distance as i32),                     // N
            1 => (distance as i32, distance as i32),       // NE
            2 => (distance as i32, 0),                     // E
            3 => (distance as i32, -(distance as i32)),    // SE
            4 => (0, -(distance as i32)),                  // S
            5 => (-(distance as i32), -(distance as i32)), // SW
            6 => (-(distance as i32), 0),                  // W
            7 => (-(distance as i32), distance as i32),    // NW
            _ => return None,
        };

        Some((dx, dy, None))
    } else if plane_idx < underpromo_planes_start {
        // L-shaped moves for knights
        let knight_idx = plane_idx - knight_planes_start;
        KNIGHT_DELTAS
            .get(knight_idx)
            .map(|&(dx, dy)| (dx, dy, None))
    } else {
        // Underpromotion
        let underpromo_idx = plane_idx - underpromo_planes_start;
        let total_underpromo_planes = NUM_UNDERPROMO_DIRECTIONS * NUM_UNDERPROMO_PIECES;
        if underpromo_idx < total_underpromo_planes {
            let direction_idx = underpromo_idx / NUM_UNDERPROMO_PIECES;
            let piece_idx = underpromo_idx % NUM_UNDERPROMO_PIECES;

            let dx = match direction_idx {
                0 => -1, // left diagonal
                1 => 0,  // straight
                2 => 1,  // right diagonal
                _ => return None,
            };

            let promo = match piece_idx {
                0 => Some(PieceType::Knight),
                1 => Some(PieceType::Bishop),
                2 => Some(PieceType::Rook),
                _ => return None,
            };

            Some((dx, 1, promo))
        } else {
            None
        }
    }
}

/// Returns the total number of AlphaZero move-policy planes for a board size.
#[hotpath::measure]
pub fn get_alphazero_move_planes_count(width: usize, height: usize) -> usize {
    let max_distance = width.max(height) - 1;
    let straight_diagonal_planes = NUM_DIRECTIONS * max_distance;
    let knight_planes = NUM_KNIGHT_DELTAS;
    let underpromo_planes = NUM_UNDERPROMO_DIRECTIONS * NUM_UNDERPROMO_PIECES;

    straight_diagonal_planes + knight_planes + underpromo_planes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::position::Position;

    fn get_plane_value(
        data: &[f32],
        plane: usize,
        row: usize,
        col: usize,
        height: usize,
        width: usize,
    ) -> f32 {
        data[plane * height * width + row * width + col]
    }

    #[test]
    fn test_standard_game_encode_initial_position() {
        let mut game = Game::standard();
        let (data, num_planes, height, width) = encode_spatial_game_planes(&mut game);

        // Should have SPATIAL_INPUT_PLANES planes
        assert_eq!(num_planes, SPATIAL_INPUT_PLANES);
        assert_eq!(height, 8);
        assert_eq!(width, 8);
        assert_eq!(data.len(), num_planes * height * width);

        // Check white pawns (plane 0) - should be on row 1
        for col in 0..8 {
            assert_eq!(
                get_plane_value(&data, 0, 1, col, height, width),
                1.0,
                "White pawn at row 1, col {}",
                col
            );
        }

        // Check white king (plane 5) at e1 (col 4, row 0)
        assert_eq!(
            get_plane_value(&data, 5, 0, 4, height, width),
            1.0,
            "White king at e1"
        );
    }

    #[test]
    fn test_standard_game_encode_game() {
        let mut game = Game::standard();
        let (data, num_planes, height, width) = encode_spatial_game_planes(&mut game);

        // Should have SPATIAL_INPUT_PLANES planes
        assert_eq!(num_planes, SPATIAL_INPUT_PLANES);
        assert_eq!(height, 8);
        assert_eq!(width, 8);
        assert_eq!(data.len(), num_planes * height * width);

        // Color plane should be all 1.0 (white's turn)
        let color_plane_idx = HISTORY_LENGTH * PIECE_PLANES + 2; // After board history and repetitions
        assert_eq!(
            get_plane_value(&data, color_plane_idx, 0, 0, height, width),
            1.0
        );
    }

    #[test]
    fn test_encode_alphazero_move_plane_horizontal_vertical() {
        use crate::r#move::MoveFlags;

        // Test vertical move (rook moving north)
        let move_north =
            Move::from_position(Position::new(3, 0), Position::new(3, 4), MoveFlags::empty());
        let encoded = encode_alphazero_move_plane(&move_north, 8, 8);
        assert_eq!(encoded, Some(3)); // North direction, distance 4

        // Test horizontal move (rook moving east)
        let move_east =
            Move::from_position(Position::new(0, 3), Position::new(5, 3), MoveFlags::empty());
        let encoded = encode_alphazero_move_plane(&move_east, 8, 8);
        assert_eq!(encoded, Some(2 * 7 + 4)); // East direction, distance 5
    }

    #[test]
    fn test_encode_alphazero_move_plane_diagonal() {
        use crate::r#move::MoveFlags;

        // Test diagonal move (bishop moving NE)
        let move_ne =
            Move::from_position(Position::new(1, 1), Position::new(4, 4), MoveFlags::empty());
        let encoded = encode_alphazero_move_plane(&move_ne, 8, 8);
        assert_eq!(encoded, Some(7 + 2)); // NE direction, distance 3

        // Test diagonal move (bishop moving SW)
        let move_sw =
            Move::from_position(Position::new(5, 5), Position::new(3, 3), MoveFlags::empty());
        let encoded = encode_alphazero_move_plane(&move_sw, 8, 8);
        assert_eq!(encoded, Some(5 * 7 + 1)); // SW direction, distance 2
    }

    #[test]
    fn test_encode_alphazero_move_plane_knight() {
        use crate::r#move::MoveFlags;

        // Test knight move (1, 2)
        let move_knight =
            Move::from_position(Position::new(3, 3), Position::new(4, 5), MoveFlags::empty());
        let encoded = encode_alphazero_move_plane(&move_knight, 8, 8);
        assert_eq!(encoded, Some(8 * 7)); // First knight pattern

        // Test knight move (2, -1)
        let move_knight2 =
            Move::from_position(Position::new(3, 3), Position::new(5, 2), MoveFlags::empty());
        let encoded = encode_alphazero_move_plane(&move_knight2, 8, 8);
        assert_eq!(encoded, Some(8 * 7 + 2)); // Third knight pattern
    }

    #[test]
    fn test_encode_alphazero_move_plane_underpromotion() {
        use crate::r#move::MoveFlags;

        // Test straight underpromotion to knight (forward)
        let move_promo = Move::from_position_with_promotion(
            Position::new(3, 6),
            Position::new(3, 7),
            MoveFlags::PROMOTION,
            PieceType::Knight,
        );
        let encoded = encode_alphazero_move_plane(&move_promo, 8, 8);
        assert_eq!(encoded, Some((8 * 7 + 8) + 3)); // Forward, straight, knight

        // Test diagonal underpromotion to rook (forward)
        let move_promo2 = Move::from_position_with_promotion(
            Position::new(3, 6),
            Position::new(4, 7),
            MoveFlags::PROMOTION,
            PieceType::Rook,
        );
        let encoded = encode_alphazero_move_plane(&move_promo2, 8, 8);
        assert_eq!(encoded, Some((8 * 7 + 8) + 2 * 3 + 2)); // Forward, right diagonal, rook
    }

    #[test]
    fn test_alphazero_action_black_underpromotion_is_mirrored() {
        let white_move =
            Move::from_lan("d7d8b", 8, 8).expect("failed to parse white underpromotion");
        let black_move =
            Move::from_lan("d2d1b", 8, 8).expect("failed to parse black underpromotion");

        let white_action = encode_alphazero_action(&white_move, Color::White, 8, 8)
            .expect("failed to encode white underpromotion");
        let black_action = encode_alphazero_action(&black_move, Color::Black, 8, 8)
            .expect("failed to encode mirrored black underpromotion");

        assert_eq!(white_action, black_action);
    }

    #[test]
    fn test_encode_alphazero_move_plane_queen_promotion() {
        use crate::r#move::MoveFlags;

        // Queen promotions should use regular straight/diagonal encoding
        let move_promo = Move::from_position_with_promotion(
            Position::new(3, 6),
            Position::new(3, 7),
            MoveFlags::PROMOTION,
            PieceType::Queen,
        );
        let encoded = encode_alphazero_move_plane(&move_promo, 8, 8);
        assert_eq!(encoded, Some(0)); // North direction, distance 1
    }

    #[test]
    fn test_decode_alphazero_move_plane_horizontal_vertical() {
        // North, distance 4
        let decoded = decode_alphazero_move_plane(3, 8, 8);
        assert_eq!(decoded, Some((0, 4, None)));

        // East, distance 5
        let decoded = decode_alphazero_move_plane(2 * 7 + 4, 8, 8);
        assert_eq!(decoded, Some((5, 0, None)));

        // South, distance 2
        let decoded = decode_alphazero_move_plane(4 * 7 + 1, 8, 8);
        assert_eq!(decoded, Some((0, -2, None)));
    }

    #[test]
    fn test_decode_alphazero_move_plane_diagonal() {
        // NE, distance 3
        let decoded = decode_alphazero_move_plane(7 + 2, 8, 8);
        assert_eq!(decoded, Some((3, 3, None)));

        // SW, distance 2
        let decoded = decode_alphazero_move_plane(5 * 7 + 1, 8, 8);
        assert_eq!(decoded, Some((-2, -2, None)));
    }

    #[test]
    fn test_decode_alphazero_move_plane_knight() {
        // First knight pattern (1, 2)
        let decoded = decode_alphazero_move_plane(8 * 7, 8, 8);
        assert_eq!(decoded, Some((1, 2, None)));

        // Third knight pattern (2, -1)
        let decoded = decode_alphazero_move_plane(8 * 7 + 2, 8, 8);
        assert_eq!(decoded, Some((2, -1, None)));
    }

    #[test]
    fn test_decode_alphazero_move_plane_underpromotion() {
        // Forward, straight, knight
        let decoded = decode_alphazero_move_plane(8 * 7 + 8 + 3, 8, 8);
        assert_eq!(decoded, Some((0, 1, Some(PieceType::Knight))));

        // Forward, right diagonal, rook
        let decoded = decode_alphazero_move_plane(8 * 7 + 8 + 2 * 3 + 2, 8, 8);
        assert_eq!(decoded, Some((1, 1, Some(PieceType::Rook))));
    }

    #[test]
    fn test_encode_decode_alphazero_move_plane_roundtrip() {
        use crate::r#move::MoveFlags;

        let moves = vec![
            Move::from_position(Position::new(0, 0), Position::new(0, 5), MoveFlags::empty()),
            Move::from_position(Position::new(2, 2), Position::new(5, 5), MoveFlags::empty()),
            Move::from_position(Position::new(3, 3), Position::new(4, 5), MoveFlags::empty()),
            Move::from_position_with_promotion(
                Position::new(3, 6),
                Position::new(3, 7),
                MoveFlags::PROMOTION,
                PieceType::Bishop,
            ),
        ];

        for move_ in moves {
            let encoded = encode_alphazero_move_plane(&move_, 8, 8).expect("Failed to encode move");
            let (dx, dy, promo) =
                decode_alphazero_move_plane(encoded, 8, 8).expect("Failed to decode");

            assert_eq!(dx, move_.dst.col as i32 - move_.src.col as i32);
            assert_eq!(dy, move_.dst.row as i32 - move_.src.row as i32);
            assert_eq!(promo, move_.promotion.filter(|&p| p != PieceType::Queen));
        }
    }

    #[test]
    fn test_get_alphazero_move_planes_count() {
        // For 2x2 board: (8 * 1) + 8 + 9 = 25
        assert_eq!(get_alphazero_move_planes_count(2, 2), 25);

        // For 6x6 board: (8 * 5) + 8 + 9 = 57
        assert_eq!(get_alphazero_move_planes_count(6, 6), 57);

        // For 8x8 board: (8 * 7) + 8 + 9 = 73
        assert_eq!(get_alphazero_move_planes_count(8, 8), 73);
    }

    #[test]
    fn test_get_alphazero_total_actions() {
        // For 8x8 board: 73 * 64 = 4672
        assert_eq!(get_alphazero_total_actions(8, 8), 4672);

        // For 6x6 board: 57 * 36 = 2052
        assert_eq!(get_alphazero_total_actions(6, 6), 2052);
    }

    #[test]
    fn test_alphazero_action_black_mirroring() {
        let white_move =
            Move::from_lan("e2e4", 8, 8).expect("failed to parse white AlphaZero move");
        let black_move =
            Move::from_lan("e7e5", 8, 8).expect("failed to parse black AlphaZero move");
        let white_action = encode_alphazero_action(&white_move, Color::White, 8, 8)
            .expect("failed to encode white AlphaZero action");
        let black_action = encode_alphazero_action(&black_move, Color::Black, 8, 8)
            .expect("failed to encode black AlphaZero action");

        assert_eq!(white_action, black_action);
        assert_eq!(
            decode_alphazero_action(white_action, Color::Black, 8, 8)
                .expect("failed to decode mirrored black AlphaZero action")
                .to_lan(),
            "e7e5"
        );
    }

    fn maia2_reference_pawn_promotions() -> Vec<String> {
        const MAIA2_PROMOTION_ORDER: [PieceType; 4] = [
            PieceType::Queen,
            PieceType::Rook,
            PieceType::Bishop,
            PieceType::Knight,
        ];

        let mut promotions = Vec::with_capacity(88);
        for file in b'a'..=b'h' {
            let src_file = file as char;

            for piece in MAIA2_PROMOTION_ORDER {
                promotions.push(format!("{src_file}7{src_file}8{}", piece.to_char()));
            }

            if file != b'a' {
                let left_file = (file - 1) as char;
                for piece in MAIA2_PROMOTION_ORDER {
                    promotions.push(format!("{src_file}7{left_file}8{}", piece.to_char()));
                }
            }

            if file != b'h' {
                let right_file = (file + 1) as char;
                for piece in MAIA2_PROMOTION_ORDER {
                    promotions.push(format!("{src_file}7{right_file}8{}", piece.to_char()));
                }
            }
        }

        promotions
    }

    #[test]
    fn test_get_maia2_total_actions() {
        assert_eq!(
            get_maia2_total_actions(8, 8),
            Some(MAIA2_TOTAL_ACTIONS_STANDARD)
        );
        assert_eq!(get_maia2_total_actions(6, 6), None);
    }

    fn maia2_reference_moves() -> Vec<String> {
        const QUEEN_DELTAS: [(i32, i32); 8] = [
            (0, 1),
            (1, 1),
            (1, 0),
            (1, -1),
            (0, -1),
            (-1, -1),
            (-1, 0),
            (-1, 1),
        ];

        let mut moves = Vec::with_capacity(MAIA2_TOTAL_ACTIONS_STANDARD);

        for rank in 0..STANDARD_BOARD_SIZE {
            for file in 0..STANDARD_BOARD_SIZE {
                let src = Position::from_usize(file, rank);

                let mut queen_targets = Vec::new();
                for (dx, dy) in QUEEN_DELTAS {
                    let mut dst_file = file as i32 + dx;
                    let mut dst_rank = rank as i32 + dy;
                    while (0..STANDARD_BOARD_SIZE as i32).contains(&dst_file)
                        && (0..STANDARD_BOARD_SIZE as i32).contains(&dst_rank)
                    {
                        queen_targets.push(Position::new(dst_file as u8, dst_rank as u8));
                        dst_file += dx;
                        dst_rank += dy;
                    }
                }
                queen_targets.sort_by_key(|pos| Reverse(pos.to_index(STANDARD_BOARD_SIZE)));
                for dst in queen_targets {
                    moves.push(Move::from_position(src, dst, MoveFlags::empty()).to_lan());
                }

                let mut knight_targets: Vec<_> = KNIGHT_DELTAS
                    .iter()
                    .filter_map(|&(dx, dy)| {
                        let dst_file = file as i32 + dx;
                        let dst_rank = rank as i32 + dy;
                        ((0..STANDARD_BOARD_SIZE as i32).contains(&dst_file)
                            && (0..STANDARD_BOARD_SIZE as i32).contains(&dst_rank))
                        .then(|| Position::new(dst_file as u8, dst_rank as u8))
                    })
                    .collect();
                knight_targets.sort_by_key(|pos| Reverse(pos.to_index(STANDARD_BOARD_SIZE)));
                for dst in knight_targets {
                    moves.push(Move::from_position(src, dst, MoveFlags::empty()).to_lan());
                }
            }
        }

        moves.extend(maia2_reference_pawn_promotions());
        moves
    }

    #[test]
    fn test_maia2_action_order_matches_utils_py() {
        let expected = maia2_reference_moves();
        assert_eq!(expected.len(), MAIA2_TOTAL_ACTIONS_STANDARD);

        let actual: Vec<_> = (0..MAIA2_TOTAL_ACTIONS_STANDARD)
            .map(|action| {
                decode_maia2_action(action, Color::White, 8, 8)
                    .expect("missing MAIA2 move")
                    .to_lan()
            })
            .collect();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_maia2_action_order_matches_spot_checks() {
        assert_eq!(
            decode_maia2_action(0, Color::White, 8, 8)
                .expect("missing first MAIA2 move")
                .to_lan(),
            "a1h8"
        );
        assert_eq!(
            decode_maia2_action(1, Color::White, 8, 8)
                .expect("missing second MAIA2 move")
                .to_lan(),
            "a1a8"
        );
        assert_eq!(
            decode_maia2_action(21, Color::White, 8, 8)
                .expect("missing first MAIA2 knight move")
                .to_lan(),
            "a1b3"
        );
        assert_eq!(
            decode_maia2_action(22, Color::White, 8, 8)
                .expect("missing second MAIA2 knight move")
                .to_lan(),
            "a1c2"
        );

        let promotion_start = MAIA2_TOTAL_ACTIONS_STANDARD - 88;
        assert_eq!(
            decode_maia2_action(promotion_start, Color::White, 8, 8)
                .expect("missing first promotion move")
                .to_lan(),
            "a7a8q"
        );
        assert_eq!(
            decode_maia2_action(promotion_start + 1, Color::White, 8, 8)
                .expect("missing second promotion move")
                .to_lan(),
            "a7a8r"
        );
        assert_eq!(
            decode_maia2_action(promotion_start + 4, Color::White, 8, 8)
                .expect("missing first capture-promotion move")
                .to_lan(),
            "a7b8q"
        );
        assert_eq!(
            decode_maia2_action(MAIA2_TOTAL_ACTIONS_STANDARD - 1, Color::White, 8, 8)
                .expect("missing last promotion move")
                .to_lan(),
            "h7g8n"
        );
    }

    #[test]
    fn test_maia2_action_black_mirroring() {
        let white_move = Move::from_lan("e2e4", 8, 8).expect("failed to parse white MAIA2 move");
        let black_move = Move::from_lan("e7e5", 8, 8).expect("failed to parse black MAIA2 move");
        let white_action = encode_maia2_action(&white_move, Color::White, 8, 8)
            .expect("failed to encode white MAIA2 action");
        let black_action = encode_maia2_action(&black_move, Color::Black, 8, 8)
            .expect("failed to encode black MAIA2 action");

        assert_eq!(white_action, black_action);
        assert_eq!(
            decode_maia2_action(white_action, Color::Black, 8, 8)
                .expect("failed to decode mirrored black action")
                .to_lan(),
            "e7e5"
        );
    }

    #[test]
    fn test_maia2_action_promotion_order_and_mirroring() {
        let white_q = Move::from_lan("a7a8q", 8, 8).expect("failed to parse a7a8q");
        let white_r = Move::from_lan("a7a8r", 8, 8).expect("failed to parse a7a8r");
        let white_b = Move::from_lan("a7a8b", 8, 8).expect("failed to parse a7a8b");
        let white_n = Move::from_lan("a7a8n", 8, 8).expect("failed to parse a7a8n");
        let black_q = Move::from_lan("a2a1q", 8, 8).expect("failed to parse a2a1q");

        let q_idx =
            encode_maia2_action(&white_q, Color::White, 8, 8).expect("failed to encode a7a8q");
        let r_idx =
            encode_maia2_action(&white_r, Color::White, 8, 8).expect("failed to encode a7a8r");
        let b_idx =
            encode_maia2_action(&white_b, Color::White, 8, 8).expect("failed to encode a7a8b");
        let n_idx =
            encode_maia2_action(&white_n, Color::White, 8, 8).expect("failed to encode a7a8n");
        let mirrored_black_idx = encode_maia2_action(&black_q, Color::Black, 8, 8)
            .expect("failed to encode mirrored black promotion");

        let promotion_start = MAIA2_TOTAL_ACTIONS_STANDARD - 88;
        assert_eq!(q_idx, promotion_start);
        assert_eq!(r_idx, promotion_start + 1);
        assert_eq!(b_idx, promotion_start + 2);
        assert_eq!(n_idx, promotion_start + 3);
        assert_eq!(mirrored_black_idx, q_idx);
    }

    #[test]
    fn test_fuzz_move_encoding_random_games() {
        use rand::SeedableRng;
        use rand::prelude::IndexedRandom;
        use rand::rngs::SmallRng;
        use std::sync::Arc;
        use std::sync::atomic::{AtomicU64, Ordering};
        use std::thread;

        let num_games = 5_000;
        let num_threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);
        let games_per_thread = num_games / num_threads;

        let total_moves_played = Arc::new(AtomicU64::new(0));
        let total_moves_tested = Arc::new(AtomicU64::new(0));

        let mut handles = vec![];

        for thread_id in 0..num_threads {
            let moves_played = Arc::clone(&total_moves_played);
            let moves_tested = Arc::clone(&total_moves_tested);

            let handle = thread::spawn(move || {
                let mut rng = SmallRng::seed_from_u64(thread_id as u64);
                let mut thread_moves_played = 0u64;
                let mut thread_moves_tested = 0u64;

                for _game_num in 0..games_per_thread {
                    let mut game = Game::standard();
                    let max_moves = 200;

                    for _move_num in 0..max_moves {
                        if game.is_over() {
                            break;
                        }

                        let legal_moves = game.legal_moves();
                        if legal_moves.is_empty() {
                            break;
                        }

                        // Test encoding for all legal moves
                        let width = 8;
                        let height = 8;
                        let turn = game.turn();
                        let total_actions = get_alphazero_total_actions(width, height);
                        let mut seen_actions = std::collections::HashSet::new();

                        for move_ in &legal_moves {
                            let normalized_move = mirror_move_for_turn(move_, turn, height);

                            // Test plane encoding
                            let encoded =
                                encode_alphazero_move_plane(&normalized_move, width, height);
                            assert!(
                                encoded.is_some(),
                                "Failed to encode move {} in position {}",
                                move_.to_lan(),
                                game.to_fen()
                            );

                            let plane_idx = encoded.expect(
                                "test_fuzz_move_encoding_random_games: failed to encode move plane",
                            );
                            let decoded = decode_alphazero_move_plane(plane_idx, width, height);
                            assert!(
                                decoded.is_some(),
                                "Failed to decode plane {} for move {}",
                                plane_idx,
                                move_.to_lan()
                            );

                            let (dx, dy, promo) = decoded.expect(
                                "test_fuzz_move_encoding_random_games: failed to decode move plane",
                            );

                            // Verify deltas
                            let expected_dx =
                                normalized_move.dst.col as i32 - normalized_move.src.col as i32;
                            let expected_dy =
                                normalized_move.dst.row as i32 - normalized_move.src.row as i32;

                            assert_eq!(
                                dx,
                                expected_dx,
                                "Move {}: decoded dx {} != expected {}",
                                move_.to_lan(),
                                dx,
                                expected_dx
                            );
                            assert_eq!(
                                dy,
                                expected_dy,
                                "Move {}: decoded dy {} != expected {}",
                                move_.to_lan(),
                                dy,
                                expected_dy
                            );

                            // Verify promotion (queen promotions should decode as None)
                            if let Some(move_promo) = move_.promotion {
                                if move_promo != PieceType::Queen {
                                    assert_eq!(
                                        promo,
                                        Some(move_promo),
                                        "Move {}: decoded promotion {:?} != expected {:?}",
                                        move_.to_lan(),
                                        promo,
                                        Some(move_promo)
                                    );
                                } else {
                                    assert_eq!(
                                        promo,
                                        None,
                                        "Move {}: queen promotion should decode as None, got {:?}",
                                        move_.to_lan(),
                                        promo
                                    );
                                }
                            } else {
                                assert_eq!(
                                    promo,
                                    None,
                                    "Move {}: expected no promotion, got {:?}",
                                    move_.to_lan(),
                                    promo
                                );
                            }

                            // Test full action encoding
                            let action = encode_alphazero_action(move_, turn, width, height);
                            assert!(
                                action.is_some(),
                                "Failed to encode action for move {} in position {}",
                                move_.to_lan(),
                                game.to_fen()
                            );
                            let action_idx = action.expect("test_fuzz_move_encoding_random_games: failed to encode full action");
                            assert!(
                                action_idx < total_actions,
                                "Action index {} out of range (total: {}) for move {}",
                                action_idx,
                                total_actions,
                                move_.to_lan()
                            );

                            // Verify no action collisions
                            assert!(
                                seen_actions.insert(action_idx),
                                "Action collision: action {} for move {} in position {}",
                                action_idx,
                                move_.to_lan(),
                                game.to_fen()
                            );

                            // Verify action roundtrip via normalized plane/src decomposition.
                            let decoded_plane = action_idx / (width * height);
                            let src_index = action_idx % (width * height);
                            let decoded_src_col = src_index % width;
                            let decoded_src_row = src_index / width;
                            assert_eq!(
                                decoded_src_col,
                                usize::from(normalized_move.src.col),
                                "Action roundtrip: src_col mismatch for move {}",
                                move_.to_lan()
                            );
                            assert_eq!(
                                decoded_src_row,
                                usize::from(normalized_move.src.row),
                                "Action roundtrip: src_row mismatch for move {}",
                                move_.to_lan()
                            );
                            assert_eq!(
                                decoded_plane,
                                plane_idx,
                                "Action roundtrip: plane mismatch for move {}",
                                move_.to_lan()
                            );

                            let decoded_move =
                                decode_alphazero_action(action_idx, turn, width, height)
                                    .expect("failed to decode full AlphaZero action");
                            assert_eq!(
                                decoded_move.src,
                                move_.src,
                                "Action roundtrip: src mismatch for move {}",
                                move_.to_lan()
                            );
                            assert_eq!(
                                decoded_move.dst,
                                move_.dst,
                                "Action roundtrip: dst mismatch for move {}",
                                move_.to_lan()
                            );
                            assert_eq!(
                                decoded_move.promotion,
                                move_.promotion.filter(|&p| p != PieceType::Queen),
                                "Action roundtrip: normalized promotion mismatch for move {}",
                                move_.to_lan()
                            );

                            let finalized_decoded_move = game
                                .decode_alphazero_action(action_idx)
                                .expect("failed to decode finalized AlphaZero action");
                            assert_eq!(
                                finalized_decoded_move.promotion,
                                move_.promotion,
                                "Action roundtrip: promotion mismatch for move {}",
                                move_.to_lan()
                            );

                            thread_moves_tested += 1;
                        }

                        // Make a random move
                        let chosen_move = legal_moves.choose(&mut rng).expect(
                            "test_fuzz_move_encoding_random_games: legal moves must not be empty",
                        );
                        game.make_move_unchecked(chosen_move);

                        thread_moves_played += 1;
                    }
                }

                moves_played.fetch_add(thread_moves_played, Ordering::Relaxed);
                moves_tested.fetch_add(thread_moves_tested, Ordering::Relaxed);
            });

            handles.push(handle);
        }

        for handle in handles {
            handle
                .join()
                .expect("test_fuzz_move_encoding_random_games: worker thread panicked");
        }

        let final_moves_played = total_moves_played.load(Ordering::Relaxed);
        let final_moves_tested = total_moves_tested.load(Ordering::Relaxed);

        println!(
            "\nMove Encoding Fuzz Test (Rust):\n  Games: {}\n  Threads: {}\n  Moves played: {}\n  Moves tested: {}",
            num_games, num_threads, final_moves_played, final_moves_tested
        );

        assert!(final_moves_played > 0, "No moves were played");
        assert!(final_moves_tested > 0, "No moves were tested");
    }
}

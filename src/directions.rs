//! Direction tables used by move generation and encoding.

/// Orthogonal `(dx, dy)` directions.
pub const ORTHOGONAL: [(i32, i32); 4] = [(0, 1), (0, -1), (1, 0), (-1, 0)];

/// Diagonal `(dx, dy)` directions.
pub const DIAGONAL: [(i32, i32); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];

/// All eight queen directions in encoding order.
pub const ALL_DIRS: [(i32, i32); 8] = [
    (0, 1),
    (0, -1),
    (1, 0),
    (-1, 0),
    (1, 1),
    (1, -1),
    (-1, 1),
    (-1, -1),
];

/// Maps a direction to its move-plane index.
pub fn direction_index(dx: i32, dy: i32) -> Option<usize> {
    match (dx.signum(), dy.signum()) {
        (0, 1) => Some(0),   // N
        (1, 1) => Some(1),   // NE
        (1, 0) => Some(2),   // E
        (1, -1) => Some(3),  // SE
        (0, -1) => Some(4),  // S
        (-1, -1) => Some(5), // SW
        (-1, 0) => Some(6),  // W
        (-1, 1) => Some(7),  // NW
        _ => None,
    }
}

/// Knight move deltas in encoding order.
pub const KNIGHT_DELTAS: [(i32, i32); 8] = [
    (1, 2),
    (2, 1),
    (2, -1),
    (1, -2),
    (-1, -2),
    (-2, -1),
    (-2, 1),
    (-1, 2),
];

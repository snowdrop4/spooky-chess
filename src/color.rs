//! Piece and side colors.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(i8)]
/// A chess side.
pub enum Color {
    /// White.
    White = 1,
    /// Black.
    Black = -1,
}

#[hotpath::measure_all]
impl Color {
    #[inline]
    /// Returns the opposite color.
    pub fn opposite(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    #[inline]
    /// Converts `1` to white and `-1` to black.
    pub fn from_int(i: i8) -> Option<Color> {
        match i {
            1 => Some(Color::White),
            -1 => Some(Color::Black),
            _ => None,
        }
    }
}

#[hotpath::measure_all]
impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Color::White => "White",
            Color::Black => "Black",
        };
        write!(f, "{}", s)
    }
}

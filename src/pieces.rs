//! Piece kinds and colored pieces.

use crate::color::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// A chess piece kind.
pub enum PieceType {
    /// Pawn.
    Pawn,
    /// Knight.
    Knight,
    /// Bishop.
    Bishop,
    /// Rook.
    Rook,
    /// Queen.
    Queen,
    /// King.
    King,
}

impl PieceType {
    /// Promotion targets in this crate's default order.
    pub const PROMOTABLE: [PieceType; 4] = [
        PieceType::Queen,
        PieceType::Knight,
        PieceType::Bishop,
        PieceType::Rook,
    ];

    /// The default promotion piece.
    pub const DEFAULT_PROMOTION: PieceType = PieceType::Queen;

    /// Returns the lowercase FEN character for the piece type.
    pub fn to_char(self) -> char {
        match self {
            PieceType::Pawn => 'p',
            PieceType::Knight => 'n',
            PieceType::Bishop => 'b',
            PieceType::Rook => 'r',
            PieceType::Queen => 'q',
            PieceType::King => 'k',
        }
    }

    /// Parses a FEN piece character, ignoring case.
    pub fn from_char(c: char) -> Option<Self> {
        match c.to_ascii_lowercase() {
            'p' => Some(PieceType::Pawn),
            'n' => Some(PieceType::Knight),
            'b' => Some(PieceType::Bishop),
            'r' => Some(PieceType::Rook),
            'q' => Some(PieceType::Queen),
            'k' => Some(PieceType::King),
            _ => None,
        }
    }

    /// Returns the SAN piece letter.
    pub fn to_san_char(self) -> char {
        match self {
            PieceType::Pawn => 'P',
            PieceType::Knight => 'N',
            PieceType::Bishop => 'B',
            PieceType::Rook => 'R',
            PieceType::Queen => 'Q',
            PieceType::King => 'K',
        }
    }

    /// Parses a SAN piece letter.
    pub fn from_san_char(c: char) -> Option<Self> {
        match c {
            'N' => Some(PieceType::Knight),
            'B' => Some(PieceType::Bishop),
            'R' => Some(PieceType::Rook),
            'Q' => Some(PieceType::Queen),
            'K' => Some(PieceType::King),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// A colored chess piece.
pub struct Piece {
    /// The piece kind.
    pub piece_type: PieceType,
    /// The piece color.
    pub color: Color,
}

#[hotpath::measure_all]
impl Piece {
    /// Creates a piece.
    pub fn new(piece_type: PieceType, color: Color) -> Self {
        Piece { piece_type, color }
    }

    /// Returns the FEN character for the piece.
    pub fn to_char(&self) -> char {
        let c = self.piece_type.to_char();
        match self.color {
            Color::White => c.to_ascii_uppercase(),
            Color::Black => c,
        }
    }

    /// Parses a FEN piece character.
    pub fn from_char(c: char) -> Option<Self> {
        let color = if c.is_ascii_uppercase() {
            Color::White
        } else {
            Color::Black
        };
        PieceType::from_char(c).map(|pt| Piece::new(pt, color))
    }
}

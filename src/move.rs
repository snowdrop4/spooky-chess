//! Move values and LAN parsing.

use crate::pieces::PieceType;
use crate::position::Position;
use bitflags::bitflags;

fn parse_square_prefix(s: &str, start: usize) -> Result<(Position, usize), String> {
    let bytes = s.as_bytes();
    if start >= bytes.len() {
        return Err("Invalid LAN move".to_string());
    }

    let file = bytes[start] as char;
    if !file.is_ascii_lowercase() {
        return Err("Invalid file character".to_string());
    }

    let mut end = start + 1;
    while end < bytes.len() && bytes[end].is_ascii_digit() {
        end += 1;
    }

    if end == start + 1 {
        return Err("Invalid row number".to_string());
    }

    Ok((Position::from_algebraic(&s[start..end])?, end))
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    /// Extra move metadata inferred from the board state.
    pub struct MoveFlags: u8 {
        /// The move captures a piece.
        const CAPTURE = 0b00000001;
        /// The move is a two-square pawn push.
        const DOUBLE_PUSH = 0b00000010;
        /// The move captures en passant.
        const EN_PASSANT = 0b00000100;
        /// The move castles.
        const CASTLE = 0b00001000;
        /// The move promotes a pawn.
        const PROMOTION = 0b00010000;
        /// The move gives check.
        const CHECK = 0b00100000;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// A chess move.
///
/// ```
/// use spooky_chess::r#move::Move;
///
/// let mv = Move::from_lan("a9a10", 10, 10).unwrap();
/// assert_eq!(mv.to_lan(), "a9a10");
/// ```
pub struct Move {
    /// Source square.
    pub src: Position,
    /// Destination square.
    pub dst: Position,
    /// Move metadata.
    pub flags: MoveFlags,
    /// Promotion target, if any.
    pub promotion: Option<PieceType>,
}

#[hotpath::measure_all]
impl Move {
    /// Creates a move without promotion.
    pub fn from_position(src: Position, dst: Position, flags: MoveFlags) -> Self {
        Move {
            src,
            dst,
            flags,
            promotion: None,
        }
    }

    /// Creates a promotion move.
    pub fn from_position_with_promotion(
        src: Position,
        dst: Position,
        flags: MoveFlags,
        promotion: PieceType,
    ) -> Self {
        Move {
            src,
            dst,
            flags: flags | MoveFlags::PROMOTION,
            promotion: Some(promotion),
        }
    }

    /// Parses LAN like `e2e4` or `a7a8q`.
    pub fn from_lan(lan: &str, board_width: usize, board_height: usize) -> Result<Self, String> {
        if lan.len() < 4 {
            return Err("Invalid LAN move".to_string());
        }

        let (src, next) = parse_square_prefix(lan, 0)?;
        let (dst, next) = parse_square_prefix(lan, next)?;

        if !src.is_valid(board_width, board_height) || !dst.is_valid(board_width, board_height) {
            return Err("Move positions out of bounds".to_string());
        }

        let mut move_ = Move::from_position(src, dst, MoveFlags::empty());

        if next < lan.len() {
            if next + 1 != lan.len() {
                return Err("Invalid LAN move".to_string());
            }

            let promo_char = lan[next..]
                .chars()
                .next()
                .expect("Failed to get promotion character from LAN string");

            let promotion = PieceType::from_char(promo_char)
                .ok_or_else(|| "Invalid promotion piece".to_string())?;

            move_.promotion = Some(promotion);
            move_.flags |= MoveFlags::PROMOTION;
        }

        Ok(move_)
    }

    /// Returns the rook movement implied by a castling move.
    pub(crate) fn castling_rook_positions(&self, board_width: usize) -> (Position, Position) {
        debug_assert!(self.flags.contains(MoveFlags::CASTLE));
        if self.dst.col > self.src.col {
            // Kingside
            (
                Position::from_usize(board_width - 1, usize::from(self.src.row)),
                Position::new(self.dst.col - 1, self.dst.row),
            )
        } else {
            // Queenside
            (
                Position::new(0, self.src.row),
                Position::new(self.dst.col + 1, self.dst.row),
            )
        }
    }

    /// Formats the move as LAN.
    pub fn to_lan(&self) -> String {
        let mut lan = format!("{}{}", self.src.to_algebraic(), self.dst.to_algebraic());

        if let Some(promo) = self.promotion {
            lan.push(promo.to_char());
        }

        lan
    }
}

#[hotpath::measure_all]
impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_lan())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lan_roundtrips_multi_digit_ranks() {
        let mv = Move::from_position(Position::new(0, 8), Position::new(0, 9), MoveFlags::empty());
        assert_eq!(mv.to_lan(), "a9a10");

        let parsed = Move::from_lan("a9a10", 10, 10)
            .expect("lan_roundtrips_multi_digit_ranks: failed to parse a9a10");
        assert_eq!(parsed.src, Position::new(0, 8));
        assert_eq!(parsed.dst, Position::new(0, 9));
    }

    #[test]
    fn lan_parses_multi_digit_promotion_ranks() {
        let parsed = Move::from_lan("a15a16q", 16, 16)
            .expect("lan_parses_multi_digit_promotion_ranks: failed to parse a15a16q");
        assert_eq!(parsed.src, Position::new(0, 14));
        assert_eq!(parsed.dst, Position::new(0, 15));
        assert_eq!(parsed.promotion, Some(PieceType::Queen));
    }
}

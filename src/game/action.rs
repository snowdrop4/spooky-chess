use crate::r#move::{Move, MoveFlags};
use crate::pieces::PieceType;
use crate::position::Position;

use super::Game;

impl<const W: usize, const H: usize> Game<W, H>
where
    [(); (W * H).div_ceil(64)]:,
{
    /// Decodes an action index into a move.
    pub fn decode_action(&self, action: usize) -> Option<Move> {
        let board_size = W * H;

        let plane_idx = action / board_size;
        let src_index = action % board_size;
        let src_col = src_index % W;
        let src_row = src_index / W;

        let (dx, dy, promo) = crate::encode::decode_move_plane(plane_idx, W, H)?;

        let dst_col_i = src_col as i32 + dx;
        let dst_row_i = src_row as i32 + dy;
        if dst_col_i < 0 || dst_row_i < 0 {
            return None;
        }
        let (dst_col, dst_row) = (dst_col_i as usize, dst_row_i as usize);
        if dst_col >= W || dst_row >= H {
            return None;
        }

        let src = Position::from_usize(src_col, src_row);
        let dst = Position::from_usize(dst_col, dst_row);
        let piece = self.board.get_piece(&src)?;

        let mut flags = self.infer_move_flags(&src, &dst, &piece);

        // Promotion
        let promotion = if let Some(promo_piece) = promo {
            flags |= MoveFlags::PROMOTION;
            Some(promo_piece)
        } else if piece.piece_type == PieceType::Pawn && (dst_row == 0 || dst_row == H - 1) {
            flags |= MoveFlags::PROMOTION;
            Some(PieceType::DEFAULT_PROMOTION)
        } else {
            None
        };

        Some(Move {
            src,
            dst,
            flags,
            promotion,
        })
    }

    /// Decodes and applies an action without legality checking.
    ///
    /// Returns `false` if the action cannot be decoded for the current
    /// position. Otherwise returns `true` after applying the decoded move.
    pub fn apply_action(&mut self, action: usize) -> bool {
        let mv = match self.decode_action(action) {
            Some(mv) => mv,
            None => return false,
        };
        self.make_move_unchecked(&mv);
        true
    }

    /// Encodes a move as an action index for this board size.
    pub fn encode_action(&self, mv: &Move) -> Option<usize> {
        crate::encode::encode_action(mv, W, H)
    }
}

use crate::r#move::{Move, MoveFlags};
use crate::pieces::PieceType;

use super::Game;

impl<const W: usize, const H: usize> Game<W, H>
where
    [(); (W * H).div_ceil(64)]:,
{
    fn finalize_decoded_move(&self, mv: Move) -> Option<Move> {
        let src = mv.src;
        let dst = mv.dst;
        let piece = self.board.get_piece(&src)?;

        let mut flags = self.infer_move_flags(&src, &dst, &piece);

        let promotion = if let Some(promo_piece) = mv.promotion {
            flags |= MoveFlags::PROMOTION;
            Some(promo_piece)
        } else if piece.piece_type == PieceType::Pawn
            && (dst.row == 0 || usize::from(dst.row) == H - 1)
        {
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

    /// Decodes an AlphaZero action index into a move for the current position.
    pub fn decode_alphazero_action(&self, action: usize) -> Option<Move> {
        let mv = crate::encode::decode_alphazero_action(action, self.turn(), W, H)?;
        self.finalize_decoded_move(mv)
    }

    /// Decodes and applies an AlphaZero action without legality checking.
    ///
    /// Returns `false` if the action cannot be decoded for the current
    /// position. Otherwise returns `true` after applying the decoded move.
    pub fn apply_alphazero_action(&mut self, action: usize) -> bool {
        let mv = match self.decode_alphazero_action(action) {
            Some(mv) => mv,
            None => return false,
        };
        self.make_move_unchecked(&mv);
        true
    }

    /// Encodes a move using AlphaZero's action space.
    pub fn encode_alphazero_action(&self, mv: &Move) -> Option<usize> {
        crate::encode::encode_alphazero_action(mv, self.turn(), W, H)
    }

    /// Encodes a move using MAIA2's action space.
    ///
    /// Only standard 8x8 chess is supported.
    pub fn encode_maia2_action(&self, mv: &Move) -> Option<usize> {
        crate::encode::encode_maia2_action(mv, self.turn(), W, H)
    }

    /// Decodes a MAIA2 action index into a move for the current position.
    ///
    /// Only standard 8x8 chess is supported.
    pub fn decode_maia2_action(&self, action: usize) -> Option<Move> {
        let mv = crate::encode::decode_maia2_action(action, self.turn(), W, H)?;
        self.finalize_decoded_move(mv)
    }

    /// Decodes and applies a MAIA2 action without legality checking.
    ///
    /// Returns `false` if the action cannot be decoded for the current
    /// position.
    pub fn apply_maia2_action(&mut self, action: usize) -> bool {
        let mv = match self.decode_maia2_action(action) {
            Some(mv) => mv,
            None => return false,
        };
        self.make_move_unchecked(&mv);
        true
    }

    /// Returns encoded AlphaZero actions for all legal moves.
    pub fn legal_alphazero_action_indices(&mut self) -> Vec<usize> {
        self.legal_moves()
            .into_iter()
            .filter_map(|mv| self.encode_alphazero_action(&mv))
            .collect()
    }

    /// Returns encoded MAIA2 actions for all legal moves.
    ///
    /// Returns an empty vector on non-standard board sizes.
    pub fn legal_maia2_action_indices(&mut self) -> Vec<usize> {
        self.legal_moves()
            .into_iter()
            .filter_map(|mv| self.encode_maia2_action(&mv))
            .collect()
    }

    /// Returns the total AlphaZero action count for this board size.
    pub fn alphazero_total_actions(&self) -> usize {
        crate::encode::get_alphazero_total_actions(W, H)
    }

    /// Returns the total MAIA2 action count for this board size.
    ///
    /// Only standard 8x8 chess is supported.
    pub fn maia2_total_actions(&self) -> Option<usize> {
        crate::encode::get_maia2_total_actions(W, H)
    }
}

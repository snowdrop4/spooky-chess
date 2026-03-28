//! Game results and turn-state snapshots.

use crate::color::Color;
use crate::r#move::Move;
use smallvec::SmallVec;
use std::fmt;

/// A move list with a stack-friendly inline capacity.
pub type MoveList = SmallVec<[Move; 256]>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// The final result of a game.
pub enum GameOutcome {
    /// White won.
    WhiteWin,
    /// Black won.
    BlackWin,
    /// Draw by stalemate.
    Stalemate,
    /// Draw by insufficient material.
    InsufficientMaterial,
    /// Draw by repetition.
    ThreefoldRepetition,
    /// Draw by the fifty-move rule.
    FiftyMoveRule,
    /// Some other drawn result.
    Other,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// The current turn state.
pub enum TurnState {
    /// The game is over with the given outcome.
    Over(GameOutcome),
    /// The game is ongoing with the current legal moves.
    Ongoing(MoveList),
}

#[hotpath::measure_all]
impl GameOutcome {
    /// Returns the winning color, if any.
    pub fn winner(&self) -> Option<Color> {
        match self {
            GameOutcome::WhiteWin => Some(Color::White),
            GameOutcome::BlackWin => Some(Color::Black),
            _ => None,
        }
    }

    /// Encodes white win as `1.0`, black win as `-1.0`, and draws as `0.0`.
    pub fn encode_winner_absolute(&self) -> f32 {
        match self {
            GameOutcome::WhiteWin => 1.0,
            GameOutcome::BlackWin => -1.0,
            _ => 0.0,
        }
    }

    /// Encodes the outcome from one side's perspective.
    pub fn encode_winner_from_perspective(&self, perspective: Color) -> f32 {
        match perspective {
            Color::White => match self {
                GameOutcome::WhiteWin => 1.0,
                GameOutcome::BlackWin => -1.0,
                _ => 0.0,
            },
            Color::Black => match self {
                GameOutcome::WhiteWin => -1.0,
                GameOutcome::BlackWin => 1.0,
                _ => 0.0,
            },
        }
    }

    /// Returns whether the outcome is a draw.
    pub fn is_draw(&self) -> bool {
        !matches!(self, GameOutcome::WhiteWin | GameOutcome::BlackWin)
    }
}

#[hotpath::measure_all]
impl fmt::Display for GameOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            GameOutcome::WhiteWin => "white_win",
            GameOutcome::BlackWin => "black_win",
            GameOutcome::Stalemate => "stalemate",
            GameOutcome::InsufficientMaterial => "insufficient_material",
            GameOutcome::ThreefoldRepetition => "threefold_repetition",
            GameOutcome::FiftyMoveRule => "fifty_move_rule",
            GameOutcome::Other => "other_draw",
        };
        write!(f, "{}", s)
    }
}

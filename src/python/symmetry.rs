use numpy::ndarray::{Array1, Array2, Array4, s};
use numpy::{
    IntoPyArray, PyArray1, PyArray2, PyArray4, PyReadonlyArray1, PyReadonlyArray2, PyReadonlyArray4,
};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::encode;

const DIRECTION_MIRROR: [usize; 8] = [0, 7, 6, 5, 4, 3, 2, 1];
const KNIGHT_MIRROR: [usize; 8] = [7, 6, 5, 4, 3, 2, 1, 0];
const UNDERPROMO_DIR_MIRROR: [usize; 3] = [2, 1, 0];

fn validate_batch_shapes(
    states_shape: &[usize],
    policies_shape: &[usize],
    values_shape: &[usize],
    opponent_policies_shape: &[usize],
    opponent_policy_masks_shape: &[usize],
) -> PyResult<(usize, usize, usize, usize)> {
    if states_shape.len() != 4 {
        return Err(PyValueError::new_err(
            "states must have shape [batch, planes, height, width]",
        ));
    }
    if policies_shape.len() != 2 {
        return Err(PyValueError::new_err(
            "policies must have shape [batch, actions]",
        ));
    }
    if values_shape.len() != 1 {
        return Err(PyValueError::new_err("values must have shape [batch]"));
    }
    if opponent_policies_shape.len() != 2 {
        return Err(PyValueError::new_err(
            "opponent_policies must have shape [batch, actions]",
        ));
    }
    if opponent_policy_masks_shape.len() != 1 {
        return Err(PyValueError::new_err(
            "opponent_policy_masks must have shape [batch]",
        ));
    }

    let sample_count = states_shape[0];
    let plane_count = states_shape[1];
    let height = states_shape[2];
    let width = states_shape[3];

    if plane_count != encode::SPATIAL_INPUT_PLANES {
        return Err(PyValueError::new_err(
            "states plane count does not match chess spatial encoder planes",
        ));
    }

    if policies_shape[0] != sample_count
        || values_shape[0] != sample_count
        || opponent_policies_shape[0] != sample_count
        || opponent_policy_masks_shape[0] != sample_count
    {
        return Err(PyValueError::new_err(
            "all inputs must have the same batch dimension",
        ));
    }

    if policies_shape[1] != opponent_policies_shape[1] {
        return Err(PyValueError::new_err(
            "policies and opponent_policies must have the same action dimension",
        ));
    }

    let total_actions = encode::get_alphazero_total_actions(width, height);
    if policies_shape[1] != total_actions {
        return Err(PyValueError::new_err(
            "policy action count does not match the chess board dimensions",
        ));
    }

    Ok((sample_count, plane_count, height, width))
}

fn mirrored_action_index(action_idx: usize, board_width: usize, board_height: usize) -> usize {
    let max_distance = board_width.max(board_height) - 1;
    let board_size = board_width * board_height;
    let straight_diagonal_planes = encode::NUM_DIRECTIONS * max_distance;
    let knight_planes_start = straight_diagonal_planes;
    let underpromo_planes_start = knight_planes_start + encode::NUM_KNIGHT_DELTAS;
    let plane = action_idx / board_size;
    let src_index = action_idx % board_size;
    let src_row = src_index / board_width;
    let src_col = src_index % board_width;
    let mirrored_col = board_width - 1 - src_col;
    let mirrored_src = src_row * board_width + mirrored_col;

    let mirrored_plane = if plane < straight_diagonal_planes {
        let direction = plane / max_distance;
        let distance = plane % max_distance;
        DIRECTION_MIRROR[direction] * max_distance + distance
    } else if plane < underpromo_planes_start {
        let knight_idx = plane - knight_planes_start;
        knight_planes_start + KNIGHT_MIRROR[knight_idx]
    } else {
        let underpromo_idx = plane - underpromo_planes_start;
        let dir_idx = underpromo_idx / encode::NUM_UNDERPROMO_PIECES;
        let piece_idx = underpromo_idx % encode::NUM_UNDERPROMO_PIECES;
        underpromo_planes_start
            + UNDERPROMO_DIR_MIRROR[dir_idx] * encode::NUM_UNDERPROMO_PIECES
            + piece_idx
    };

    mirrored_plane * board_size + mirrored_src
}

fn mirrored_plane_index(plane_idx: usize) -> usize {
    let castling_start = encode::HISTORY_LENGTH * encode::PIECE_PLANES + 4;
    let own_kingside = castling_start;
    let own_queenside = castling_start + 1;
    let opponent_kingside = castling_start + 2;
    let opponent_queenside = castling_start + 3;

    match plane_idx {
        idx if idx == own_kingside => own_queenside,
        idx if idx == own_queenside => own_kingside,
        idx if idx == opponent_kingside => opponent_queenside,
        idx if idx == opponent_queenside => opponent_kingside,
        idx => idx,
    }
}

/// Augment a batch of spatial chess states and policies with their horizontal
/// mirror.
#[pyfunction]
pub fn augment_symmetries<'py>(
    py: Python<'py>,
    states: PyReadonlyArray4<'py, f32>,
    policies: PyReadonlyArray2<'py, f32>,
    values: PyReadonlyArray1<'py, f32>,
    opponent_policies: PyReadonlyArray2<'py, f32>,
    opponent_policy_masks: PyReadonlyArray1<'py, f32>,
) -> PyResult<(
    Bound<'py, PyArray4<f32>>,
    Bound<'py, PyArray2<f32>>,
    Bound<'py, PyArray1<f32>>,
    Bound<'py, PyArray2<f32>>,
    Bound<'py, PyArray1<f32>>,
)> {
    let states = states.as_array();
    let policies = policies.as_array();
    let values = values.as_array();
    let opponent_policies = opponent_policies.as_array();
    let opponent_policy_masks = opponent_policy_masks.as_array();

    let (sample_count, plane_count, height, width) = validate_batch_shapes(
        states.shape(),
        policies.shape(),
        values.shape(),
        opponent_policies.shape(),
        opponent_policy_masks.shape(),
    )?;

    let action_size = policies.shape()[1];
    let augmented_sample_count = sample_count * 2;
    let mirrored_policy_indices = (0..action_size)
        .map(|action_idx| mirrored_action_index(action_idx, width, height))
        .collect::<Vec<usize>>();

    let mut augmented_states =
        Array4::<f32>::zeros((augmented_sample_count, plane_count, height, width));
    augmented_states
        .slice_mut(s![0..sample_count, .., .., ..])
        .assign(&states);

    let mut augmented_policies = Array2::<f32>::zeros((augmented_sample_count, action_size));
    augmented_policies
        .slice_mut(s![0..sample_count, ..])
        .assign(&policies);

    let mut augmented_values = Array1::<f32>::zeros(augmented_sample_count);
    augmented_values
        .slice_mut(s![0..sample_count])
        .assign(&values);
    augmented_values
        .slice_mut(s![sample_count..])
        .assign(&values);

    let mut augmented_opponent_policies =
        Array2::<f32>::zeros((augmented_sample_count, action_size));
    augmented_opponent_policies
        .slice_mut(s![0..sample_count, ..])
        .assign(&opponent_policies);

    let mut augmented_opponent_policy_masks = Array1::<f32>::zeros(augmented_sample_count);
    augmented_opponent_policy_masks
        .slice_mut(s![0..sample_count])
        .assign(&opponent_policy_masks);
    augmented_opponent_policy_masks
        .slice_mut(s![sample_count..])
        .assign(&opponent_policy_masks);

    for sample_idx in 0..sample_count {
        let mirrored_sample_idx = sample_count + sample_idx;

        for plane_idx in 0..plane_count {
            let mirrored_plane_idx = mirrored_plane_index(plane_idx);
            for row_idx in 0..height {
                for col_idx in 0..width {
                    let mirrored_col_idx = width - 1 - col_idx;
                    augmented_states[[
                        mirrored_sample_idx,
                        mirrored_plane_idx,
                        row_idx,
                        mirrored_col_idx,
                    ]] = states[[sample_idx, plane_idx, row_idx, col_idx]];
                }
            }
        }

        for source_action_idx in 0..action_size {
            let mirrored_action_idx = mirrored_policy_indices[source_action_idx];
            augmented_policies[[mirrored_sample_idx, mirrored_action_idx]] =
                policies[[sample_idx, source_action_idx]];
            augmented_opponent_policies[[mirrored_sample_idx, mirrored_action_idx]] =
                opponent_policies[[sample_idx, source_action_idx]];
        }
    }

    Ok((
        augmented_states.into_pyarray(py),
        augmented_policies.into_pyarray(py),
        augmented_values.into_pyarray(py),
        augmented_opponent_policies.into_pyarray(py),
        augmented_opponent_policy_masks.into_pyarray(py),
    ))
}

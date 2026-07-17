//! Optional smoothing boundary.  Phase 6 retains filtered predicted moments
//! so a Rauch--Tung--Striebel implementation can be added without changing
//! the durable artifact schema.

use crate::results::StateEstimatePoint;

pub fn filtered_points_are_smoothing_ready(points: &[StateEstimatePoint]) -> bool {
    points
        .iter()
        .all(|p| !p.predicted_covariance.is_empty() && !p.filtered_covariance.is_empty())
}

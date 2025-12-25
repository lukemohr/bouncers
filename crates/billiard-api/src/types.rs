use serde::{Deserialize, Serialize};

use billiard_core::dynamics::simulation::CollisionResult;
use billiard_core::dynamics::state::BoundaryState;
use billiard_core::geometry::table_spec::TableSpec;

/// Request payload for POST /simulate.
///
/// - `table`: geometric description of the billiard table.
/// - `initial_state`: starting collision state (boundary component, arc-length s, angle).
/// - `max_steps`: maximum number of collisions to simulate.
/// - `epsilon`: small threshold to skip self-intersections near the current bounce.
#[derive(Debug, Deserialize)]
pub struct SimulateRequest {
    pub table: TableSpec,
    pub initial_state: BoundaryStateDto,
    pub max_steps: usize,
    pub epsilon: f64,
}

/// API representation of a boundary-based state.
///
/// This mirrors billiard_core::dynamics::state::BoundaryState.
#[derive(Debug, Deserialize)]
pub struct BoundaryStateDto {
    pub component_index: usize,
    pub s: f64,
    pub theta: f64,
}

/// Collision information returned by the simulation.
///
/// Mirrors billiard_core::dynamics::simulation::CollisionResult, but tailored
/// for JSON responses (no Vec2, just x/y).
#[derive(Debug, Serialize)]
pub struct CollisionDto {
    pub step: usize,
    pub component_index: usize,
    pub segment_index: usize,
    pub s: f64,
    pub theta: f64,
    pub x: f64,
    pub y: f64,
}

/// Response payload for POST /simulate.
///
/// A trajectory is just a list of collision records.
#[derive(Debug, Serialize)]
pub struct SimulateResponse {
    pub collisions: Vec<CollisionDto>,
}

/// Convert API boundary state into core type.
impl BoundaryStateDto {
    pub fn into_core(self) -> BoundaryState {
        BoundaryState {
            component_index: self.component_index,
            s: self.s,
            theta: self.theta,
        }
    }
}

/// Convert core collision result into API DTO.
impl CollisionDto {
    pub fn from_core(step: usize, c: &CollisionResult) -> Self {
        CollisionDto {
            step,
            component_index: c.component_index,
            segment_index: c.segment_index,
            s: c.s,
            theta: c.theta,
            x: c.hit_point.x,
            y: c.hit_point.y,
        }
    }
}

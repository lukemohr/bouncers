use axum::{Json, response::IntoResponse};
use serde::Serialize;
use tracing::{info, instrument};

use crate::error::{ApiError, ApiResult};
use crate::types::{CollisionDto, SimulateRequest, SimulateResponse};

use billiard_core::dynamics::simulation::run_trajectory;

/// Health check endpoint for GET /health.
///
/// Returns a small JSON object indicating that the service is up.
pub async fn health() -> ApiResult<impl IntoResponse> {
    #[derive(Serialize)]
    struct HealthBody {
        status: &'static str,
    }

    let body = HealthBody { status: "ok" };
    Ok(Json(body))
}

/// Simulation endpoint for POST /simulate.
///
/// Instrumented with tracing to log incoming parameters and timing.
#[instrument(skip(req))]
pub async fn simulate(Json(req): Json<SimulateRequest>) -> ApiResult<impl IntoResponse> {
    info!(
        max_steps = req.max_steps,
        epsilon = req.epsilon,
        "Received simulation request"
    );

    // Basic validation
    if req.max_steps == 0 {
        return Err(ApiError::BadRequest(
            "max_steps must be greater than 0".to_string(),
        ));
    }

    if !req.epsilon.is_finite() || req.epsilon <= 0.0 {
        return Err(ApiError::BadRequest(
            "epsilon must be positive and finite".to_string(),
        ));
    }

    // Build internal table representation
    let table = req.table.to_billiard_table();

    // Convert initial state
    let initial_state = req.initial_state.into_core();

    info!(
        component_index = initial_state.component_index,
        s = initial_state.s,
        theta = initial_state.theta,
        "Starting trajectory"
    );

    // Run the trajectory using the core engine
    let collisions_core = run_trajectory(&table, &initial_state, req.max_steps, req.epsilon);

    let collision_count = collisions_core.len();

    // Map to DTOs
    let collisions_dto: Vec<CollisionDto> = collisions_core
        .iter()
        .enumerate()
        .map(|(step, c)| CollisionDto::from_core(step, c))
        .collect();

    info!(collisions = collision_count, "Simulation completed");

    // Wrap in response type
    let response = SimulateResponse {
        collisions: collisions_dto,
    };

    Ok(Json(response))
}

use crate::dynamics::intersection::Ray;
use crate::dynamics::state::{BoundaryState, WorldState};
use crate::geometry::boundary::BilliardTable;
use crate::geometry::primitives::Vec2;

#[derive(Clone, Copy, Debug)]
pub struct CollisionResult {
    pub component_index: usize,
    pub segment_index: usize,
    pub s: f64,     // new boundary arc-length parameter
    pub theta: f64, // new outgoing angle after reflection
    pub hit_point: Vec2,
}

impl CollisionResult {
    pub fn new(
        component_index: usize,
        segment_index: usize,
        s: f64,
        theta: f64,
        hit_point: Vec2,
    ) -> Self {
        Self {
            component_index,
            segment_index,
            s,
            theta,
            hit_point,
        }
    }
}

/// Find the next collision on the table from the boundary state.
///
/// Steps:
/// 1. Convert the boundary state to a world-space state (position + direction).
/// 2. Cast a ray from that position along the direction.
/// 3. Intersect the ray with the table to find the nearest collision.
/// 4. Compute the reflection using the inward normal at the hit point.
/// 5. Convert the reflected world state back into a boundary-based state.
/// 6. Return the new boundary state and the collision point.
pub fn next_collision_from_boundary_state(
    table: &BilliardTable,
    bs: &BoundaryState,
    epsilon: f64,
) -> Option<CollisionResult> {
    let ws = bs.to_world(table);

    let ray = Ray {
        origin: ws.position,
        direction: ws.direction,
    };

    let intersection = ray.intersect_table(table, epsilon)?;
    let component_index = intersection.component_index;
    let segment_index = intersection.segment_index;
    let local_t = intersection.local_t;
    let ray_t = intersection.ray_parameter;

    let component = table.component(component_index);
    let new_s = component.global_s_from_segment_local(segment_index, local_t);

    // Hit point from ray parameter
    let v_in = ws
        .direction
        .try_normalized()
        .expect("World direction should not be near-zero.");
    let hit_point = ws.position + v_in * ray_t;

    // Get inward normal from boundary at that s
    let (_check_point, inward_normal) = component.point_and_inward_normal_at(new_s);

    let n = inward_normal
        .try_normalized()
        .expect("Inward normal should not be near-zero.");

    let dot_vn = v_in.dot(n);
    let v_out = v_in - n * (2.0 * dot_vn);

    let outgoing_world = WorldState {
        position: hit_point,
        direction: v_out,
    };

    let outgoing_bs = outgoing_world.to_boundary(table, component_index, new_s);

    Some(CollisionResult::new(
        outgoing_bs.component_index,
        segment_index,
        outgoing_bs.s,
        outgoing_bs.theta,
        hit_point,
    ))
}

/// Simulate a billiard trajectory by iterating boundary collisions.
///
/// Starts from an initial boundary state and repeatedly applies
/// `next_collision_from_boundary_state`, collecting each collision.
///
/// Stops early if:
/// - `next_collision_from_boundary_state` returns `None`, or
/// - `max_steps` collisions have been generated.
pub fn run_trajectory(
    table: &BilliardTable,
    initial: &BoundaryState,
    max_steps: usize,
    epsilon: f64,
) -> Vec<CollisionResult> {
    let mut collisions = Vec::with_capacity(max_steps);
    let mut current = *initial;

    for _ in 0..max_steps {
        let collision = match next_collision_from_boundary_state(table, &current, epsilon) {
            Some(c) => c,
            None => break,
        };

        current = BoundaryState {
            component_index: collision.component_index,
            s: collision.s,
            theta: collision.theta,
        };

        collisions.push(collision);
    }

    collisions
}

#[cfg(test)]
mod tests {
    use super::next_collision_from_boundary_state;
    use crate::dynamics::state::BoundaryState;
    use crate::geometry::boundary::{BilliardTable, BoundaryComponent};
    use crate::geometry::primitives::Vec2;
    use crate::geometry::segments::{BoundarySegment, LineSegment};

    fn unit_square_table() -> BilliardTable {
        // Bottom: (0,0) -> (1,0)
        let bottom =
            BoundarySegment::Line(LineSegment::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0)));
        // Right: (1,0) -> (1,1)
        let right =
            BoundarySegment::Line(LineSegment::new(Vec2::new(1.0, 0.0), Vec2::new(1.0, 1.0)));
        // Top: (1,1) -> (0,1)
        let top = BoundarySegment::Line(LineSegment::new(Vec2::new(1.0, 1.0), Vec2::new(0.0, 1.0)));
        // Left: (0,1) -> (0,0)
        let left =
            BoundarySegment::Line(LineSegment::new(Vec2::new(0.0, 1.0), Vec2::new(0.0, 0.0)));

        let outer = BoundaryComponent::new("outer", vec![bottom, right, top, left]);

        BilliardTable {
            outer,
            obstacles: Vec::new(),
        }
    }

    #[test]
    fn next_collision_in_unit_square_from_bottom_edge() {
        let table = unit_square_table();

        // On the bottom edge, arc-length s runs from 0 to 1.
        // s = 0.5 is the midpoint (0.5, 0).
        let s0 = 0.5;

        // BoundaryState on bottom edge (component 0), at s=0.5,
        // theta = +π/2 → direction straight inward (upwards).
        let bs0 = BoundaryState {
            component_index: 0,
            s: s0,
            theta: std::f64::consts::FRAC_PI_2,
        };

        let epsilon = 1e-8;
        let result = next_collision_from_boundary_state(&table, &bs0, epsilon);

        assert!(
            result.is_some(),
            "Expected a collision result in closed unit square"
        );
        let result = result.unwrap();

        // We expect to hit the top edge (segment index 2 in our construction).
        assert_eq!(
            result.component_index, 0,
            "Expected to hit outer boundary component"
        );
        assert_eq!(
            result.segment_index, 2,
            "Expected to hit top edge (segment index 2)"
        );

        // Hit point should be approximately (0.5, 1.0).
        let hp = result.hit_point;
        assert!(
            (hp.x - 0.5).abs() < 1e-10,
            "Unexpected hit_point.x: got {}, expected 0.5",
            hp.x
        );
        assert!(
            (hp.y - 1.0).abs() < 1e-10,
            "Unexpected hit_point.y: got {}, expected 1.0",
            hp.y
        );
    }
}

#[cfg(test)]
mod trajectory_tests {
    use super::run_trajectory;
    use crate::dynamics::state::BoundaryState;
    use crate::geometry::boundary::{BilliardTable, BoundaryComponent};
    use crate::geometry::primitives::Vec2;
    use crate::geometry::segments::{BoundarySegment, LineSegment};

    fn unit_square_table() -> BilliardTable {
        let bottom =
            BoundarySegment::Line(LineSegment::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0)));
        let right =
            BoundarySegment::Line(LineSegment::new(Vec2::new(1.0, 0.0), Vec2::new(1.0, 1.0)));
        let top = BoundarySegment::Line(LineSegment::new(Vec2::new(1.0, 1.0), Vec2::new(0.0, 1.0)));
        let left =
            BoundarySegment::Line(LineSegment::new(Vec2::new(0.0, 1.0), Vec2::new(0.0, 0.0)));

        let outer = BoundaryComponent::new("outer", vec![bottom, right, top, left]);
        BilliardTable {
            outer,
            obstacles: Vec::new(),
        }
    }

    #[test]
    fn vertical_orbit_in_unit_square() {
        let table = unit_square_table();

        // Start on the bottom edge at x = 0.5, pointing straight up.
        // bottom edge tangent = (1,0), inward = (0,1),
        // theta = +π/2 => direction (0,1).
        let initial = BoundaryState {
            component_index: 0,
            s: 0.5,
            theta: std::f64::consts::FRAC_PI_2,
        };

        let epsilon = 1e-8;
        let traj = run_trajectory(&table, &initial, 4, epsilon);

        // We asked for 4 steps; there should be exactly 4 collisions in a closed square.
        assert_eq!(traj.len(), 4, "Expected 4 collisions, got {}", traj.len());

        // First hit: top edge at (0.5, 1.0)
        let c1 = &traj[0];
        assert_eq!(c1.component_index, 0);
        assert_eq!(c1.segment_index, 2); // top edge in our construction
        assert!((c1.hit_point.x - 0.5).abs() < 1e-10);
        assert!((c1.hit_point.y - 1.0).abs() < 1e-10);

        // Second hit: back to bottom edge at (0.5, 0.0)
        let c2 = &traj[1];
        assert_eq!(c2.segment_index, 0); // bottom edge
        assert!((c2.hit_point.x - 0.5).abs() < 1e-10);
        assert!((c2.hit_point.y - 0.0).abs() < 1e-10);

        // Third hit: top again
        let c3 = &traj[2];
        assert_eq!(c3.segment_index, 2);
        assert!((c3.hit_point.x - 0.5).abs() < 1e-10);
        assert!((c3.hit_point.y - 1.0).abs() < 1e-10);

        // Fourth hit: bottom again
        let c4 = &traj[3];
        assert_eq!(c4.segment_index, 0);
        assert!((c4.hit_point.x - 0.5).abs() < 1e-10);
        assert!((c4.hit_point.y - 0.0).abs() < 1e-10);
    }
}

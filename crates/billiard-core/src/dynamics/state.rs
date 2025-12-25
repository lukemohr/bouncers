use crate::geometry::boundary::BilliardTable;
use crate::geometry::primitives::Vec2;

/// A collision state on the billiard boundary (Poincaré section).
///
/// This encodes one bounce:
/// - which boundary component we are on,
/// - where along that component (arc-length),
/// - and the outgoing angle relative to the local tangent.
#[derive(Clone, Copy, Debug)]
pub struct BoundaryState {
    /// Index of the boundary component:
    /// 0 = outer boundary, 1.. = obstacles.
    pub component_index: usize,

    /// Arc-length parameter on that component, in [0, component_length).
    pub s: f64,

    /// Outgoing angle relative to the local tangent, in radians.
    ///
    /// Convention suggestion:
    /// - theta = 0: along the tangent direction,
    /// - theta > 0: rotate from tangent toward inward normal,
    /// - theta < 0: rotate from tangent toward outward side.
    pub theta: f64,
}

/// World-space representation of a moving billiard particle.
///
/// This does not itself know which boundary component it comes from; it is
/// just the instantaneous position and direction in ℝ².
pub struct WorldState {
    /// World-space position of the particle.
    pub position: Vec2,

    /// World-space direction of motion (ideally unit-length).
    pub direction: Vec2,
}

impl BoundaryState {
    /// Convert this boundary state to a world-space state using the table geometry.
    pub fn to_world(&self, table: &BilliardTable) -> WorldState {
        let component = table.component(self.component_index);

        let (position, tangent) = component.point_and_tangent_at(self.s);
        let (_p, inward_normal) = component.point_and_inward_normal_at(self.s);

        // Assume tangent/inward_normal are unit and orthogonal (by construction)
        let cos_theta = self.theta.cos();
        let sin_theta = self.theta.sin();

        let direction = tangent * cos_theta + inward_normal * sin_theta;

        WorldState {
            position,
            direction,
        }
    }
}

impl WorldState {
    /// Construct a boundary-based state from this world state, given:
    /// - which component and arc-length parameter its position corresponds to,
    /// - and a sign convention for theta.
    pub fn to_boundary(
        &self,
        table: &BilliardTable,
        component_index: usize,
        s: f64,
    ) -> BoundaryState {
        let component = table.component(component_index);
        let (_point, tangent) = component.point_and_tangent_at(s);

        let t_hat = tangent
            .try_normalized()
            .expect("Tangent should not be near-zero.");
        let d_hat = self
            .direction
            .try_normalized()
            .expect("Direction should not be near-zero.");

        // Signed angle between t_hat and d_hat.
        let dot = (t_hat.dot(d_hat)).clamp(-1.0, 1.0); // avoid NaN from rounding
        let cross_z = t_hat.x * d_hat.y - t_hat.y * d_hat.x;
        let theta = cross_z.atan2(dot); // atan2(y, x) → angle in (-π, π]

        BoundaryState {
            component_index,
            s,
            theta,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BoundaryState;
    use crate::geometry::boundary::{BilliardTable, BoundaryComponent};
    use crate::geometry::primitives::Vec2;
    use crate::geometry::segments::{BoundarySegment, LineSegment};

    /// Helper: build a simple horizontal outer boundary from (0,0) to (1,0),
    /// treated as a single-segment "table".
    fn simple_horizontal_table() -> BilliardTable {
        let seg = BoundarySegment::Line(LineSegment::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0)));
        let outer = BoundaryComponent::new("outer", vec![seg]);
        BilliardTable {
            outer,
            obstacles: Vec::new(),
        }
    }

    #[test]
    fn boundary_state_to_world_and_back_preserves_theta() {
        let table = simple_horizontal_table();

        // On our segment, s = 0.5 should be the midpoint (0.5, 0), with tangent (1, 0)
        // and inward normal (0, 1). We'll test a few theta values.
        let s = 0.5;
        let component_index = 0;

        let thetas = [0.0, 0.3, -0.5, 1.0];

        for &theta in &thetas {
            let bs = BoundaryState {
                component_index,
                s,
                theta,
            };

            let ws = bs.to_world(&table);
            let bs2 = ws.to_boundary(&table, component_index, s);

            // Theta should be preserved up to numerical tolerance.
            let diff = (bs2.theta - theta).abs();
            assert!(
                diff < 1e-10,
                "Round-trip theta mismatch: original = {}, recovered = {}, diff = {}",
                theta,
                bs2.theta,
                diff
            );
        }
    }
}

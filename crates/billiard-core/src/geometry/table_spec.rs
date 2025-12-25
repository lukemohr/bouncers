use super::primitives::Vec2;
use crate::geometry::boundary::{BilliardTable, BoundaryComponent};
use crate::geometry::segments::{BoundarySegment, LineSegment};
use serde::{Deserialize, Serialize};

/// A polyline describing a single closed boundary component, given by its vertices.
///
/// The polyline is assumed to be:
/// - Ordered (vertices[0] -> vertices[1] -> ...),
/// - Closed: the last vertex connects back to the first to form a loop,
/// - Counterclockwise (CCW) for the outer boundary.
///
/// Internal obstacles may later use clockwise orientation, but we won't enforce
/// that yet.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PolylineSpec {
    /// Human-readable name, e.g. "outer", "obstacle_1", etc.
    pub name: String,

    /// The vertices of the polyline, in world coordinates.
    ///
    /// At least 3 vertices are required to form a closed boundary.
    pub vertices: Vec<Vec2>,
}

/// A serializable description of a billiard table.
///
/// This is the shape you'll send from the frontend / store in the DB.
/// It can be converted into a `BilliardTable` using a helper function.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TableSpec {
    /// The outer boundary polyline.
    pub outer: PolylineSpec,

    /// Internal obstacles, each given as a closed polyline.
    pub obstacles: Vec<PolylineSpec>,
}

impl PolylineSpec {
    /// Convert this polyline into a `BoundaryComponent` built from line segments.
    ///
    /// Each consecutive pair of vertices (v[i], v[i+1]) becomes a line segment.
    /// Additionally, the last vertex is connected back to the first to close the loop.
    ///
    /// # Panics
    /// Panics if there are fewer than 3 vertices.
    pub fn to_boundary_component(&self) -> BoundaryComponent {
        let mut verts = self.vertices.clone();
        if verts.len() >= 2 && verts.first() == verts.last() {
            verts.pop();
        }

        let num_vertices = verts.len();
        assert!(
            num_vertices >= 3,
            "There must be at least three distinct vertices to construct a closed boundary."
        );

        let segments = (0..num_vertices)
            .map(|idx| {
                let start = verts[idx];
                let end = verts[(idx + 1) % num_vertices];
                let line_seg = LineSegment::new(start, end);
                BoundarySegment::Line(line_seg)
            })
            .collect();

        BoundaryComponent::new(self.name.clone(), segments)
    }
}

impl TableSpec {
    /// Convert this `TableSpec` into an internal `BilliardTable` representation.
    ///
    /// This constructs:
    /// - One `BoundaryComponent` for the outer polyline,
    /// - One `BoundaryComponent` per obstacle polyline.
    pub fn to_billiard_table(&self) -> BilliardTable {
        let outer_bc = self.outer.to_boundary_component();
        let obstacles_bc = self
            .obstacles
            .iter()
            .map(|poly| poly.to_boundary_component())
            .collect();
        BilliardTable {
            outer: outer_bc,
            obstacles: obstacles_bc,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PolylineSpec, TableSpec};
    use crate::geometry::boundary::BoundaryComponent;
    use crate::geometry::primitives::Vec2;

    fn unit_square_polyline(name: &str) -> PolylineSpec {
        PolylineSpec {
            name: name.to_string(),
            vertices: vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(0.0, 1.0),
            ],
        }
    }

    #[test]
    fn polyline_to_boundary_component_unit_square() {
        let poly = unit_square_polyline("outer");
        let bc = poly.to_boundary_component();

        // Lengths:
        // 4 edges of length 1 => total = 4
        assert!((bc.length() - 4.0).abs() < 1e-12);

        // Sample points at s = 0.5, 1.5, 2.5, 3.5 should be the midpoints of edges:
        let (p0, _) = bc.point_and_tangent_at(0.5);
        assert!((p0.x - 0.5).abs() < 1e-12);
        assert!((p0.y - 0.0).abs() < 1e-12);

        let (p1, _) = bc.point_and_tangent_at(1.5);
        assert!((p1.x - 1.0).abs() < 1e-12);
        assert!((p1.y - 0.5).abs() < 1e-12);

        let (p2, _) = bc.point_and_tangent_at(2.5);
        assert!((p2.x - 0.5).abs() < 1e-12);
        assert!((p2.y - 1.0).abs() < 1e-12);

        let (p3, _) = bc.point_and_tangent_at(3.5);
        assert!((p3.x - 0.0).abs() < 1e-12);
        assert!((p3.y - 0.5).abs() < 1e-12);
    }

    #[test]
    fn table_spec_to_billiard_table() {
        let outer = unit_square_polyline("outer");
        let obstacles = Vec::new();

        let spec = TableSpec { outer, obstacles };

        let table = spec.to_billiard_table();

        // Outer boundary should match what we know about the unit square.
        let bc: &BoundaryComponent = &table.outer;
        assert!((bc.length() - 4.0).abs() < 1e-12);

        // Spot-check a point:
        let (p, _) = bc.point_and_tangent_at(0.5);
        assert!((p.x - 0.5).abs() < 1e-12);
        assert!((p.y - 0.0).abs() < 1e-12);
    }
}

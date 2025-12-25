use super::primitives::Vec2;
use crate::geometry::boundary::{BilliardTable, BoundaryComponent};
use crate::geometry::segments::{BoundarySegment, CircularArcSegment, LineSegment};
use serde::{Deserialize, Serialize};

/// Serializable description of a single boundary segment.
///
/// This mirrors your internal `BoundarySegment` but is structured to be
/// JSON-friendly for the frontend and database.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SegmentSpec {
    /// Straight line between two points.
    Line { start: Vec2, end: Vec2 },

    /// Circular arc on a circle defined by center + radius.
    ///
    /// The arc runs from `start_angle` to `end_angle` in radians, with
    /// parameterization direction given by `ccw`.
    CircularArc {
        center: Vec2,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
        ccw: bool,
    },
}

/// Serializable description of a closed boundary component.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BoundarySpec {
    pub name: String,
    pub segments: Vec<SegmentSpec>,
}

/// A serializable description of a billiard table.
///
/// This is the shape you'll send from the frontend / store in the DB.
/// It can be converted into a `BilliardTable` using a helper function.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TableSpec {
    /// The outer boundary.
    pub outer: BoundarySpec,

    /// Internal obstacles.
    pub obstacles: Vec<BoundarySpec>,
}

impl BoundarySpec {
    /// Convert this serializable boundary spec into an internal BoundaryComponent.
    ///
    /// Each `SegmentSpec` variant is mapped to the corresponding `BoundarySegment`.
    ///
    /// # Panics
    /// Panics if the segments do not form a closed loop or contain degenerate geometry.
    pub fn to_boundary_component(&self) -> BoundaryComponent {
        let bdry_segments: Vec<BoundarySegment> = self
            .segments
            .iter()
            .map(|seg| match seg {
                SegmentSpec::Line { start, end } => {
                    BoundarySegment::Line(LineSegment::new(*start, *end))
                }
                SegmentSpec::CircularArc {
                    center,
                    radius,
                    start_angle,
                    end_angle,
                    ccw,
                } => BoundarySegment::CircularArc(CircularArcSegment::new(
                    *center,
                    *radius,
                    *start_angle,
                    *end_angle,
                    *ccw,
                )),
            })
            .collect();
        BoundaryComponent::new(self.name.clone(), bdry_segments)
    }
}

impl TableSpec {
    /// Convert this `TableSpec` into an internal `BilliardTable` representation.
    pub fn to_billiard_table(&self) -> BilliardTable {
        let outer_bc = self.outer.to_boundary_component();
        let obstacles_bc = self
            .obstacles
            .iter()
            .map(|bdry| bdry.to_boundary_component())
            .collect();
        BilliardTable {
            outer: outer_bc,
            obstacles: obstacles_bc,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BoundarySpec, SegmentSpec, TableSpec};
    use crate::geometry::boundary::{BilliardTable, BoundaryComponent};
    use crate::geometry::primitives::Vec2;
    use serde_json;
    use std::f64::consts::PI;

    // --- Helpers ---

    fn unit_square_boundary_spec(name: &str) -> BoundarySpec {
        BoundarySpec {
            name: name.to_string(),
            segments: vec![
                SegmentSpec::Line {
                    start: Vec2::new(0.0, 0.0),
                    end: Vec2::new(1.0, 0.0),
                },
                SegmentSpec::Line {
                    start: Vec2::new(1.0, 0.0),
                    end: Vec2::new(1.0, 1.0),
                },
                SegmentSpec::Line {
                    start: Vec2::new(1.0, 1.0),
                    end: Vec2::new(0.0, 1.0),
                },
                SegmentSpec::Line {
                    start: Vec2::new(0.0, 1.0),
                    end: Vec2::new(0.0, 0.0),
                },
            ],
        }
    }

    // --- BoundarySpec tests (lines) ---

    #[test]
    fn boundary_spec_line_segments_unit_square() {
        let spec = unit_square_boundary_spec("outer");
        let bc: BoundaryComponent = spec.to_boundary_component();

        // 4 edges of length 1 ⇒ total length = 4
        assert!((bc.length() - 4.0).abs() < 1e-12);

        // Sample midpoints at s = 0.5, 1.5, 2.5, 3.5
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

    // --- BoundarySpec tests (circular arc) ---

    #[test]
    fn boundary_spec_circular_arc_quarter_circle() {
        use std::f64::consts::FRAC_PI_2;

        let spec = BoundarySpec {
            name: "quarter_circle".to_string(),
            segments: vec![SegmentSpec::CircularArc {
                center: Vec2::new(0.0, 0.0),
                radius: 1.0,
                start_angle: 0.0,
                end_angle: FRAC_PI_2,
                ccw: true,
            }],
        };

        let bc: BoundaryComponent = spec.to_boundary_component();

        let expected_len = FRAC_PI_2;
        assert!((bc.length() - expected_len).abs() < 1e-12);

        // At s = 0: point should be (1, 0); tangent should point along +y.
        let (p0, t0) = bc.point_and_tangent_at(0.0);
        assert!((p0.x - 1.0).abs() < 1e-12);
        assert!((p0.y - 0.0).abs() < 1e-12);
        assert!((t0.x - 0.0).abs() < 1e-12);
        assert!((t0.y - 1.0).abs() < 1e-12);

        // Sample just *before* the end to avoid wrapping back to 0.
        let eps = 1e-6;
        let (p1, t1) = bc.point_and_tangent_at(expected_len - eps);

        // Point should be approximately (0, 1).
        assert!(p1.x.abs() < 1e-3, "p1.x = {}", p1.x);
        assert!((p1.y - 1.0).abs() < 1e-3, "p1.y = {}", p1.y);

        // Tangent near the end should point roughly along -x.
        assert!((t1.x + 1.0).abs() < 1e-3, "t1.x = {}", t1.x);
        assert!(t1.y.abs() < 1e-3, "t1.y = {}", t1.y);
    }

    // --- TableSpec → BilliardTable tests ---

    #[test]
    fn table_spec_to_billiard_table_square_no_obstacles() {
        let outer = unit_square_boundary_spec("outer");
        let obstacles = Vec::<BoundarySpec>::new();

        let spec = TableSpec { outer, obstacles };

        let table: BilliardTable = spec.to_billiard_table();
        let bc: &BoundaryComponent = &table.outer;

        // Outer boundary is still the unit square
        assert!((bc.length() - 4.0).abs() < 1e-12);

        let (p, _) = bc.point_and_tangent_at(0.5);
        assert!((p.x - 0.5).abs() < 1e-12);
        assert!((p.y - 0.0).abs() < 1e-12);

        // No obstacles expected
        assert_eq!(table.obstacles.len(), 0);
    }

    #[test]
    fn table_spec_with_circular_obstacle() {
        let outer = unit_square_boundary_spec("outer");

        // Single circular obstacle: full circle (0 → 2π)
        let obstacle = BoundarySpec {
            name: "circle_obstacle".to_string(),
            segments: vec![SegmentSpec::CircularArc {
                center: Vec2::new(0.5, 0.5),
                radius: 0.2,
                start_angle: 0.0,
                end_angle: 2.0 * PI,
                ccw: true,
            }],
        };

        let spec = TableSpec {
            outer,
            obstacles: vec![obstacle],
        };

        let table: BilliardTable = spec.to_billiard_table();

        // Outer boundary still square
        assert!((table.outer.length() - 4.0).abs() < 1e-12);

        // One obstacle
        assert_eq!(table.obstacles.len(), 1);

        let obs0 = &table.obstacles[0];
        // Circumference of radius 0.2 circle: 2πr
        let expected_len = 2.0 * PI * 0.2;
        assert!((obs0.length() - expected_len).abs() < 1e-8);
    }

    // --- Serde roundtrip tests ---

    #[test]
    fn segment_spec_serde_roundtrip() {
        // Line segment
        let line = SegmentSpec::Line {
            start: Vec2::new(0.0, 0.0),
            end: Vec2::new(1.0, 0.0),
        };

        let json_line = serde_json::to_string(&line).expect("serialize line");
        let line_back: SegmentSpec = serde_json::from_str(&json_line).expect("deserialize line");

        match line_back {
            SegmentSpec::Line { start, end } => {
                assert!((start.x - 0.0).abs() < 1e-12);
                assert!((start.y - 0.0).abs() < 1e-12);
                assert!((end.x - 1.0).abs() < 1e-12);
                assert!((end.y - 0.0).abs() < 1e-12);
            }
            _ => panic!("Expected SegmentSpec::Line after roundtrip"),
        }

        // Circular arc segment
        let arc = SegmentSpec::CircularArc {
            center: Vec2::new(0.5, 0.5),
            radius: 0.2,
            start_angle: 0.0,
            end_angle: PI,
            ccw: true,
        };

        let json_arc = serde_json::to_string(&arc).expect("serialize arc");
        let arc_back: SegmentSpec = serde_json::from_str(&json_arc).expect("deserialize arc");

        match arc_back {
            SegmentSpec::CircularArc {
                center,
                radius,
                start_angle,
                end_angle,
                ccw,
            } => {
                assert!((center.x - 0.5).abs() < 1e-12);
                assert!((center.y - 0.5).abs() < 1e-12);
                assert!((radius - 0.2).abs() < 1e-12);
                assert!((start_angle - 0.0).abs() < 1e-12);
                assert!((end_angle - PI).abs() < 1e-12);
                assert!(ccw);
            }
            _ => panic!("Expected SegmentSpec::CircularArc after roundtrip"),
        }
    }

    #[test]
    fn boundary_spec_serde_roundtrip() {
        let spec = unit_square_boundary_spec("outer");

        let json = serde_json::to_string(&spec).expect("serialize boundary spec");
        let spec_back: BoundarySpec =
            serde_json::from_str(&json).expect("deserialize boundary spec");

        assert_eq!(spec_back.name, "outer");
        assert_eq!(spec_back.segments.len(), 4);
    }

    #[test]
    fn table_spec_serde_roundtrip() {
        let outer = unit_square_boundary_spec("outer");
        let obstacle = BoundarySpec {
            name: "circle_obstacle".to_string(),
            segments: vec![SegmentSpec::CircularArc {
                center: Vec2::new(0.5, 0.5),
                radius: 0.3,
                start_angle: 0.0,
                end_angle: 2.0 * PI,
                ccw: true,
            }],
        };

        let spec = TableSpec {
            outer,
            obstacles: vec![obstacle],
        };

        let json = serde_json::to_string(&spec).expect("serialize table spec");
        let spec_back: TableSpec = serde_json::from_str(&json).expect("deserialize table spec");

        assert_eq!(spec_back.obstacles.len(), 1);
        assert_eq!(spec_back.obstacles[0].name, "circle_obstacle");
    }
}

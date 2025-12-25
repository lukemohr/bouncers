use std::f64::consts::TAU;

use billiard_core::geometry::boundary::{BilliardTable, BoundaryComponent};
use billiard_core::geometry::primitives::Vec2;
use billiard_core::geometry::segments::{BoundarySegment, CircularArcSegment, LineSegment};

/// A simple unit square outer table with no obstacles.
#[allow(dead_code)]
pub fn unit_square_table() -> BilliardTable {
    // Bottom: (0,0) -> (1,0)
    let bottom = BoundarySegment::Line(LineSegment::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0)));
    // Right: (1,0) -> (1,1)
    let right = BoundarySegment::Line(LineSegment::new(Vec2::new(1.0, 0.0), Vec2::new(1.0, 1.0)));
    // Top: (1,1) -> (0,1)
    let top = BoundarySegment::Line(LineSegment::new(Vec2::new(1.0, 1.0), Vec2::new(0.0, 1.0)));
    // Left: (0,1) -> (0,0)
    let left = BoundarySegment::Line(LineSegment::new(Vec2::new(0.0, 1.0), Vec2::new(0.0, 0.0)));

    let outer = BoundaryComponent::new("outer", vec![bottom, right, top, left]);

    BilliardTable {
        outer,
        obstacles: Vec::new(),
    }
}

/// A Sinai-style table: square outer boundary + circular obstacle.
pub fn sinai_table() -> BilliardTable {
    // TODO: outer = square, obstacles = single circle approximated or single CircularArc?
    // Bottom: (0,0) -> (1,0)
    let bottom = BoundarySegment::Line(LineSegment::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0)));
    // Right: (1,0) -> (1,1)
    let right = BoundarySegment::Line(LineSegment::new(Vec2::new(1.0, 0.0), Vec2::new(1.0, 1.0)));
    // Top: (1,1) -> (0,1)
    let top = BoundarySegment::Line(LineSegment::new(Vec2::new(1.0, 1.0), Vec2::new(0.0, 1.0)));
    // Left: (0,1) -> (0,0)
    let left = BoundarySegment::Line(LineSegment::new(Vec2::new(0.0, 1.0), Vec2::new(0.0, 0.0)));

    let outer = BoundaryComponent::new("outer", vec![bottom, right, top, left]);

    let circle = BoundarySegment::CircularArc(CircularArcSegment::new(
        Vec2::new(0.5, 0.5),
        0.2,
        0.,
        TAU,
        true,
    ));
    let obstacles = vec![BoundaryComponent::new("sinai", vec![circle])];

    BilliardTable { outer, obstacles }
}

use super::primitives::Vec2;

/// A straight line segment from `start` to `end`.
///
/// The segment is oriented: arc-length parameter increases from `start`
/// toward `end`.
#[derive(Clone, Copy, Debug)]
pub struct LineSegment {
    pub start: Vec2,
    pub end: Vec2,
}

impl LineSegment {
    /// Constructs a new line segment from `start` to `end`.
    pub fn new(start: Vec2, end: Vec2) -> Self {
        Self { start, end }
    }

    /// Returns the total arc length of this segment.
    pub fn length(&self) -> f64 {
        let v = self.end - self.start;
        v.length()
    }

    /// Returns the point at local arc-length parameter `t` along the segment.
    ///
    /// Precondition: 0.0 <= t <= self.length().
    pub fn point_at(&self, t: f64) -> Vec2 {
        self.start + (t / self.length()) * (self.end - self.start)
    }

    /// Returns the unit tangent vector at local parameter `t`.
    ///
    /// For a line, this is constant along the segment and aligned with
    /// (end - start).
    pub fn tangent_at(&self, _t: f64) -> Vec2 {
        (self.end - self.start).normalized()
    }
}

/// A circular arc segment between two angles on a circle.
///
/// The segment is specified by:
/// - a circle center,
/// - a radius,
/// - a start angle and end angle (in radians),
/// - and an orientation (CCW vs CW).
///
/// The parameter `t` for `point_at(t)` is arc-length [0, length()].
#[derive(Clone, Copy, Debug)]
pub struct CircularArcSegment {
    pub center: Vec2,
    pub radius: f64,
    pub start_angle: f64,
    pub end_angle: f64,
    pub ccw: bool,
    pub start: Vec2,
    pub end: Vec2,
}

impl CircularArcSegment {
    /// Constructs a new circular arc segment.
    pub fn new(center: Vec2, radius: f64, start_angle: f64, end_angle: f64, ccw: bool) -> Self {
        assert!(radius > 0., "Radius must be positive.");
        let start = center
            + radius
                * Vec2 {
                    x: start_angle.cos(),
                    y: start_angle.sin(),
                };
        let end = center
            + radius
                * Vec2 {
                    x: end_angle.cos(),
                    y: end_angle.sin(),
                };
        Self {
            center,
            radius,
            start_angle,
            end_angle,
            ccw,
            start,
            end,
        }
    }

    /// Returns the total arc length of this segment.
    pub fn length(&self) -> f64 {
        self.radius * (self.end_angle - self.start_angle).abs()
    }

    /// Returns the point at local arc-length parameter `t` along the segment.
    ///
    /// Precondition: 0.0 <= t <= self.length().
    pub fn point_at(&self, t: f64) -> Vec2 {
        let del_theta = t / self.radius;
        let theta = if self.ccw {
            self.start_angle + del_theta
        } else {
            self.start_angle - del_theta
        };
        self.center + self.radius * Vec2::new(theta.cos(), theta.sin())
    }

    /// Returns the unit tangent vector at local parameter `t`.
    pub fn tangent_at(&self, t: f64) -> Vec2 {
        let del_theta = t / self.radius;
        if self.ccw {
            let theta = self.start_angle + del_theta;
            Vec2::new(-theta.sin(), theta.cos())
        } else {
            let theta = self.start_angle - del_theta;
            Vec2::new(theta.sin(), -theta.cos())
        }
    }
}

/// A boundary segment of any supported kind.
///
/// For now we only support line segments. Later we can add circular arcs
/// or other parametric curves.
#[derive(Clone, Copy, Debug)]
pub enum BoundarySegment {
    Line(LineSegment),
    CircularArc(CircularArcSegment),
    // ...
}

impl BoundarySegment {
    /// Returns the total arc length of this segment.
    pub fn length(&self) -> f64 {
        match self {
            BoundarySegment::Line(seg) => seg.length(),
            BoundarySegment::CircularArc(seg) => seg.length(),
        }
    }

    /// Returns the world-space point at local arc-length parameter `t`.
    ///
    /// Precondition: 0.0 <= t <= self.length().
    pub fn point_at(&self, t: f64) -> Vec2 {
        match self {
            BoundarySegment::Line(seg) => seg.point_at(t),
            BoundarySegment::CircularArc(seg) => seg.point_at(t),
        }
    }

    /// Returns the unit tangent vector at local parameter `t`.
    pub fn tangent_at(&self, t: f64) -> Vec2 {
        match self {
            BoundarySegment::Line(seg) => seg.tangent_at(t),
            BoundarySegment::CircularArc(seg) => seg.tangent_at(t),
        }
    }
}

#[cfg(test)]
mod arc_tests {
    use super::{BoundarySegment, CircularArcSegment};
    use crate::geometry::primitives::Vec2;

    #[test]
    fn circular_arc_length_and_points() {
        // Quarter-circle arc from angle 0 to π/2, CCW, radius 1, centered at origin.
        let arc = CircularArcSegment::new(
            Vec2::new(0.0, 0.0),
            1.0,
            0.0,
            std::f64::consts::FRAC_PI_2,
            true,
        );

        let seg = BoundarySegment::CircularArc(arc);

        // Length should be π/2 (radius 1, 90 degrees).
        assert!((seg.length() - std::f64::consts::FRAC_PI_2).abs() < 1e-12);

        // At t = 0: point should be (1, 0)
        let (p0, _) = {
            let t0 = 0.0;
            (seg.point_at(t0), seg.tangent_at(t0))
        };
        assert!((p0.x - 1.0).abs() < 1e-12);
        assert!((p0.y - 0.0).abs() < 1e-12);

        // At t = length(): point should be (0, 1)
        let t_end = seg.length();
        let p1 = seg.point_at(t_end);
        assert!((p1.x - 0.0).abs() < 1e-12);
        assert!((p1.y - 1.0).abs() < 1e-12);
    }
}

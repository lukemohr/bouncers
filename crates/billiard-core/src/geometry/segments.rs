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

/// A boundary segment of any supported kind.
///
/// For now we only support line segments. Later we can add circular arcs
/// or other parametric curves.
#[derive(Clone, Copy, Debug)]
pub enum BoundarySegment {
    Line(LineSegment),
    // CircularArc(CircularArcSegment),
    // ...
}

impl BoundarySegment {
    /// Returns the total arc length of this segment.
    pub fn length(&self) -> f64 {
        match self {
            BoundarySegment::Line(seg) => seg.length(),
        }
    }

    /// Returns the world-space point at local arc-length parameter `t`.
    ///
    /// Precondition: 0.0 <= t <= self.length().
    pub fn point_at(&self, t: f64) -> Vec2 {
        match self {
            BoundarySegment::Line(seg) => seg.point_at(t),
        }
    }

    /// Returns the unit tangent vector at local parameter `t`.
    pub fn tangent_at(&self, t: f64) -> Vec2 {
        match self {
            BoundarySegment::Line(seg) => seg.tangent_at(t),
        }
    }
}

//! Boundary representations for billiard tables.
//
//! For now this is a very simple placeholder. Later it will:
//! - store piecewise segments,
//! - support arc-length parametrization,
//! - distinguish outer boundary vs internal obstacles (Sinai billiards).

use super::primitives::Vec2;
use super::segments::BoundarySegment;
use std::iter;

/// A closed boundary component built from an ordered list of segments.
///
/// For now, this represents the **outer boundary** only.
///
/// Assumptions at this stage:
/// - Segments are provided in order and their endpoints match up,
///   forming a closed loop (not yet validated).
/// - Orientation is counterclockwise (CCW).
pub struct BoundaryComponent {
    /// Human-readable label for this component.
    pub name: String,

    /// Ordered list of boundary segments.
    pub segments: Vec<BoundarySegment>,

    /// cumulative_lengths[i] = total length of segments[0..=i]
    cumulative_lengths: Vec<f64>,

    /// Total arc length of the component (sum of segment lengths).
    total_length: f64,
}

impl BoundaryComponent {
    /// Construct a new boundary component from a name and segment list.
    ///
    /// This constructor:
    /// - takes ownership of the segments,
    /// - precomputes cumulative arc-lengths,
    /// - stores the total length.
    ///
    /// It does NOT yet:
    /// - verify that the contour is closed,
    /// - check orientation,
    /// - detect self-intersections.
    pub fn new(name: impl Into<String>, segments: Vec<BoundarySegment>) -> Self {
        assert!(
            !segments.is_empty(),
            "BoundaryComponent must have at least one segment"
        );

        let mut cumulative_lengths = Vec::with_capacity(segments.len());
        let mut running = 0.0;

        for &segment in &segments {
            let len = segment.length();
            assert!(len > 0.0, "Boundary segments must have positive length");
            running += len;
            cumulative_lengths.push(running);
        }

        let total_length = running;

        Self {
            name: name.into(),
            segments,
            cumulative_lengths,
            total_length,
        }
    }

    /// Returns the total arc length of this boundary component.
    pub fn length(&self) -> f64 {
        self.total_length
    }

    /// Maps a global arc-length parameter `s` to a segment index and local `t`.
    ///
    /// - `s` may be outside [0, length); it will be wrapped using Euclidean
    ///   modulo, so negative values are allowed.
    /// - Returns:
    ///   - `seg_idx`: index into `self.segments`,
    ///   - `local_t`: arc-length along that segment in [0, segment.length()].
    ///
    /// # Panics
    /// Panics if there are no segments in this component.
    pub fn locate(&self, s: f64) -> (usize, f64) {
        assert!(!self.segments.is_empty(), "There are no boundary segments.");
        assert!(
            self.total_length > 0.0,
            "BoundaryComponent has zero total length."
        );

        let s_wrapped = s.rem_euclid(self.total_length);

        if let Some(seg_idx) = self
            .cumulative_lengths
            .iter()
            .position(|&len| len > s_wrapped)
        {
            let local_t = if seg_idx > 0 {
                s_wrapped - self.cumulative_lengths[seg_idx - 1]
            } else {
                s_wrapped
            };
            (seg_idx, local_t)
        } else {
            // Fallback to last segment in case of rounding noise
            let seg_idx = self.segments.len() - 1;
            let prev = if seg_idx > 0 {
                self.cumulative_lengths[seg_idx - 1]
            } else {
                0.0
            };
            (seg_idx, s_wrapped - prev)
        }
    }

    /// Returns the world-space point and unit tangent at global arc-length `s`.
    ///
    /// - `s` may be outside [0, length); it will be wrapped using Euclidean
    ///   modulo (so negative values are allowed).
    /// - The tangent direction follows the segment orientation (i.e., in the
    ///   direction of increasing arc-length along the boundary).
    pub fn point_and_tangent_at(&self, s: f64) -> (Vec2, Vec2) {
        let (seg_idx, local_t) = self.locate(s);
        let seg = &self.segments[seg_idx];
        (seg.point_at(local_t), seg.tangent_at(local_t))
    }

    /// Returns the world-space point and inward-pointing unit normal
    /// at global arc-length `s`.
    ///
    /// Assumes:
    /// - This component is the **outer boundary** of the billiard table.
    /// - The boundary is oriented **counterclockwise (CCW)**.
    ///
    /// Under these assumptions, the inward normal is obtained by rotating
    /// the unit tangent +90 degrees ("left turn").
    pub fn point_and_inward_normal_at(&self, s: f64) -> (Vec2, Vec2) {
        let (point, tangent) = self.point_and_tangent_at(s);
        let normal = tangent.perp();
        let inward = normal
            .try_normalized()
            .expect("Tangent should not be near-zero in a valid boundary.");
        (point, inward)
    }

    /// Convert a local parameter on a given segment into the global arc-length `s`.
    ///
    /// - `segment_index` must be a valid index into `self.segments`.
    /// - `local_t` should be in [0, segment_length].
    pub fn global_s_from_segment_local(&self, segment_index: usize, local_t: f64) -> f64 {
        assert!(
            segment_index < self.segments.len(),
            "segment_index out of bounds"
        );

        if segment_index == 0 {
            local_t
        } else {
            self.cumulative_lengths[segment_index - 1] + local_t
        }
    }
}

/// A full billiard table: an outer boundary plus zero or more internal obstacles.
///
/// At this stage:
/// - Only the outer boundary is actually used.
/// - Obstacles are present for future Sinai-style tables and may be empty.
pub struct BilliardTable {
    pub outer: BoundaryComponent,
    pub obstacles: Vec<BoundaryComponent>,
}

impl BilliardTable {
    /// Returns the total number of boundary components (outer + obstacles).
    pub fn component_count(&self) -> usize {
        // Concept: 1 (outer) + number of obstacles.
        1 + self.obstacles.len()
    }

    pub fn component(&self, index: usize) -> &BoundaryComponent {
        if index == 0 {
            &self.outer
        } else {
            &self.obstacles[index - 1]
        }
    }

    /// Returns an iterator over all boundary components, starting with the outer one.
    pub fn components(&self) -> impl Iterator<Item = &BoundaryComponent> {
        // Concept: chain a single-element iterator over `outer` with an iterator over `obstacles`.
        iter::once(&self.outer).chain(&self.obstacles)
    }
}

#[cfg(test)]
mod tests {
    use super::BoundaryComponent;
    use crate::geometry::primitives::Vec2;
    use crate::geometry::segments::{BoundarySegment, LineSegment};

    #[test]
    fn locate_maps_s_to_correct_segment_and_local_t() {
        // Build a polyline: segment 0 length 1, segment 1 length 2
        let s0 = BoundarySegment::Line(LineSegment::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0)));
        let s1 = BoundarySegment::Line(LineSegment::new(Vec2::new(1.0, 0.0), Vec2::new(3.0, 0.0)));

        let bc = BoundaryComponent::new("test", vec![s0, s1]);

        // total length should be 3
        assert!((bc.length() - 3.0).abs() < 1e-12);

        // s in [0,1) -> segment 0
        let (idx0, t0) = bc.locate(0.5);
        assert_eq!(idx0, 0);
        assert!((t0 - 0.5).abs() < 1e-12);

        // s in [1,3) -> segment 1
        let (idx1, t1) = bc.locate(2.0);
        assert_eq!(idx1, 1);
        assert!((t1 - 1.0).abs() < 1e-12); // since we are 1 unit into segment 1
    }

    #[test]
    fn point_tangent_and_normal_have_expected_directions() {
        use crate::geometry::primitives::Vec2;
        use crate::geometry::segments::{BoundarySegment, LineSegment};

        // Simple horizontal segment from (0,0) to (1,0), CCW orientation.
        let seg = BoundarySegment::Line(LineSegment::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0)));

        // Single-segment "boundary" (not closed in space yet, but OK for direction checks).
        let bc = BoundaryComponent::new("test", vec![seg]);

        // At s = 0.5 (halfway along)
        let (p, t) = bc.point_and_tangent_at(0.5);
        let (p2, n) = bc.point_and_inward_normal_at(0.5);

        // Point should be in the middle of the segment.
        assert!((p.x - 0.5).abs() < 1e-12);
        assert!((p.y - 0.0).abs() < 1e-12);
        assert!((p2.x - p.x).abs() < 1e-12);
        assert!((p2.y - p.y).abs() < 1e-12);

        // Tangent should point along +x.
        assert!(t.length() - 1.0 < 1e-12);
        assert!((t.x - 1.0).abs() < 1e-12);
        assert!((t.y - 0.0).abs() < 1e-12);

        // Inward normal (for CCW outer boundary, interior "above" the segment)
        // should be (0, 1): i.e., pointing into +y.
        assert!(n.length() - 1.0 < 1e-12);
        assert!((n.x - 0.0).abs() < 1e-12);
        assert!((n.y - 1.0).abs() < 1e-12);
    }

    #[test]
    fn tangent_and_inward_normal_on_horizontal_segment() {
        let seg = BoundarySegment::Line(LineSegment::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0)));
        let bc = BoundaryComponent::new("outer", vec![seg]);

        let s = 0.5;
        let (p, t) = bc.point_and_tangent_at(s);
        let (p2, n) = bc.point_and_inward_normal_at(s);

        // Point
        assert!((p.x - 0.5).abs() < 1e-12);
        assert!((p.y - 0.0).abs() < 1e-12);
        assert!((p2.x - p.x).abs() < 1e-12);
        assert!((p2.y - p.y).abs() < 1e-12);

        // Tangent: +x
        assert!((t.length() - 1.0).abs() < 1e-12);
        assert!((t.x - 1.0).abs() < 1e-12);
        assert!((t.y - 0.0).abs() < 1e-12);

        // Inward normal: +y (for CCW outer boundary)
        assert!((n.length() - 1.0).abs() < 1e-12);
        assert!((n.x - 0.0).abs() < 1e-12);
        assert!((n.y - 1.0).abs() < 1e-12);
    }
}

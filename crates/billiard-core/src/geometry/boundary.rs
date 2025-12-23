//! Boundary representations for billiard tables.
//
//! For now this is a very simple placeholder. Later it will:
//! - store piecewise segments,
//! - support arc-length parametrization,
//! - distinguish outer boundary vs internal obstacles (Sinai billiards).

use super::segments::BoundarySegment;

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
        let cumulative_lengths: Vec<f64> = segments
            .iter()
            .map(|&segment| segment.length())
            .scan(0f64, |acc, len| {
                *acc += len;
                Some(*acc)
            })
            .collect();
        let total_length = if cumulative_lengths.is_empty() {
            0f64
        } else {
            *cumulative_lengths.last().unwrap()
        };
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
        if self.segments.is_empty() {
            panic!("There are no boundary segments.")
        }
        let s_wrapped = s.rem_euclid(self.total_length);
        let seg_idx = self
            .cumulative_lengths
            .iter()
            .position(|&len| len > s_wrapped)
            .unwrap();
        let local_t = if seg_idx > 0 {
            s_wrapped - self.cumulative_lengths[seg_idx - 1]
        } else {
            s_wrapped
        };
        (seg_idx, local_t)
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
}

use crate::geometry::boundary::{BilliardTable, BoundaryComponent};
use crate::geometry::primitives::Vec2;
use crate::geometry::segments::{BoundarySegment, LineSegment};

/// A half-line (ray) in ℝ² originating at `origin` and extending in direction `direction`.
pub struct Ray {
    /// Origin point of the ray.
    pub origin: Vec2,

    /// Direction of the ray (ideally unit-length).
    ///
    /// The parametric form of the ray is:
    ///   origin + t * direction, for t >= 0.
    pub direction: Vec2,
}

/// Result of intersecting a ray with the billiard boundary.
///
/// This describes:
/// - which component and segment were hit,
/// - where along the segment (local t),
/// - and how far along the ray the hit occurs.
pub struct Intersection {
    /// Index of the boundary component: 0 = outer, 1.. = obstacles.
    pub component_index: usize,

    /// Index of the segment within that component.
    pub segment_index: usize,

    /// Local arc-length parameter along the hit segment.
    pub local_t: f64,

    /// Distance along the ray from the origin to the intersection.
    ///
    /// If `direction` is unit-length, this is also the Euclidean distance.
    pub ray_parameter: f64,
}

impl Ray {
    /// Intersect this ray with a single line segment.
    ///
    /// Returns the intersection that is:
    /// - in front of the ray origin (t > epsilon),
    /// - between the segment endpoints (segment parameter between 0 and segment length),
    ///   or `None` if there is no such point.
    pub fn intersect_line_segment(
        &self,
        segment: &LineSegment,
        epsilon: f64,
    ) -> Option<(f64, f64)> {
        let p = self.origin;
        let mut r = self.direction;

        // Normalize ray direction so that ray_parameter ≈ Euclidean distance.
        if let Some(r_hat) = r.try_normalized() {
            r = r_hat;
        } else {
            // Degenerate direction; treat as no intersection.
            return None;
        }

        let q = segment.start;
        let s_vec = segment.end - segment.start;
        let seg_len = s_vec.length();

        if seg_len <= epsilon {
            // Degenerate segment
            return None;
        }

        let s = s_vec / seg_len;

        // 2D cross product helper
        let cross = |a: Vec2, b: Vec2| a.x * b.y - a.y * b.x;

        let denom = cross(r, s);

        // Parallel or nearly parallel → no reliable intersection.
        let parallel_eps = 1e-12;
        if denom.abs() < parallel_eps {
            return None;
        }

        let q_minus_p = q - p;

        let t = cross(q_minus_p, s) / denom;
        let u = cross(q_minus_p, r) / denom;

        // u is the fraction along the segment direction; local arc-length is u * seg_len.
        let local_t = u * seg_len;

        if t > epsilon && (0.0..=1.0).contains(&u) {
            Some((t, local_t))
        } else {
            None
        }
    }

    /// Intersect this ray with a single boundary component.
    ///
    /// Returns the closest valid intersection along the ray, or `None` if:
    /// - there are no segments,
    /// - or no segment is hit with a positive ray parameter.
    ///
    /// `epsilon` is used to ignore intersections that are "too close" to the origin,
    /// which helps avoid hitting the current bounce point again.
    pub fn intersect_component(
        &self,
        component: &BoundaryComponent,
        epsilon: f64,
    ) -> Option<(usize, f64, f64)> {
        if component.segments.is_empty() {
            return None;
        }

        component
            .segments
            .iter()
            .enumerate()
            .filter_map(|(i, &seg)| match seg {
                BoundarySegment::Line(line_seg) => self
                    .intersect_line_segment(&line_seg, epsilon)
                    .map(|(ray_t, local_t)| (i, ray_t, local_t)),
            })
            // Choose the smallest ray_t (closest intersection)
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
    }

    /// Intersect this ray with the full billiard table (outer boundary and obstacles).
    ///
    /// Returns the closest valid intersection along the ray, with all indices and
    /// parameters filled in, or `None` if no intersection is found.
    pub fn intersect_table(&self, table: &BilliardTable, epsilon: f64) -> Option<Intersection> {
        table
            .components()
            .enumerate()
            .filter_map(|(comp_idx, comp)| {
                self.intersect_component(comp, epsilon)
                    .map(|(seg_idx, ray_t, local_t)| (comp_idx, seg_idx, ray_t, local_t))
            })
            .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap()) // compare on ray_t
            .map(
                |(component_index, segment_index, ray_parameter, local_t)| Intersection {
                    component_index,
                    segment_index,
                    local_t,
                    ray_parameter,
                },
            )
    }
}

#[cfg(test)]
mod tests {
    use super::Ray;
    use crate::geometry::boundary::{BilliardTable, BoundaryComponent};
    use crate::geometry::primitives::Vec2;
    use crate::geometry::segments::{BoundarySegment, LineSegment};

    fn simple_horizontal_table() -> BilliardTable {
        let seg = BoundarySegment::Line(LineSegment::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0)));
        let outer = BoundaryComponent::new("outer", vec![seg]);
        BilliardTable {
            outer,
            obstacles: Vec::new(),
        }
    }

    #[test]
    fn ray_hits_horizontal_segment_from_below() {
        let table = simple_horizontal_table();

        // Segment: from (0,0) to (1,0)
        // Ray: origin at (0.5, -1), pointing straight up.
        let ray = Ray {
            origin: Vec2::new(0.5, -1.0),
            direction: Vec2::new(0.0, 1.0),
        };

        let epsilon = 1e-8;
        let hit = ray.intersect_table(&table, epsilon);

        assert!(hit.is_some(), "Expected intersection but got None");
        let hit = hit.unwrap();

        // We expect:
        // - component_index = 0
        // - segment_index = 0
        // - intersection point at (0.5, 0)
        //   -> along the ray: t ≈ 1.0 (from y = -1 to y = 0)
        //   -> along the segment: local_t ≈ 0.5 * length = 0.5 * 1.0 = 0.5
        assert_eq!(hit.component_index, 0);
        assert_eq!(hit.segment_index, 0);

        // We know the segment length is 1.0 in this setup.
        let expected_ray_t = 1.0;
        let expected_local_t = 0.5;

        let ray_t_diff = (hit.ray_parameter - expected_ray_t).abs();
        let local_t_diff = (hit.local_t - expected_local_t).abs();

        assert!(
            ray_t_diff < 1e-10,
            "Unexpected ray_parameter: got {}, expected {}, diff = {}",
            hit.ray_parameter,
            expected_ray_t,
            ray_t_diff
        );

        assert!(
            local_t_diff < 1e-10,
            "Unexpected local_t: got {}, expected {}, diff = {}",
            hit.local_t,
            expected_local_t,
            local_t_diff
        );
    }

    #[test]
    fn ray_misses_segment_when_pointed_away() {
        let table = simple_horizontal_table();

        // Same origin, but ray pointing downward.
        let ray = Ray {
            origin: Vec2::new(0.5, -1.0),
            direction: Vec2::new(0.0, -1.0),
        };

        let epsilon = 1e-8;
        let hit = ray.intersect_table(&table, epsilon);

        assert!(
            hit.is_none(),
            "Expected no intersection when ray points away from the segment"
        );
    }
}

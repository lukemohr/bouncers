use crate::geometry::boundary::{BilliardTable, BoundaryComponent};
use crate::geometry::primitives::Vec2;
use crate::geometry::segments::{BoundarySegment, CircularArcSegment, LineSegment};

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

    /// Intersect this ray with a circle defined by `center` and `radius`.
    ///
    /// Returns all positive ray parameters `t` such that:
    ///   origin + t * direction lies on the circle,
    /// with `t > epsilon`, sorted ascending.
    ///
    /// This does *not* restrict to a specific arc; it is a full-circle test.
    fn intersect_circle(&self, center: Vec2, radius: f64, epsilon: f64) -> Vec<f64> {
        let p = self.origin;
        let mut d = self.direction;

        // Normalize ray direction so that ray_parameter ≈ Euclidean distance.
        if let Some(d_hat) = d.try_normalized() {
            d = d_hat;
        } else {
            // Degenerate direction; treat as no intersection.
            return Vec::new();
        }

        let m = p - center;

        // Solve |m + t d|^2 = r^2
        // => t^2 + 2 (m·d) t + (m·m - r^2) = 0
        let b = m.dot(d); // m·d
        let c = m.dot(m) - radius * radius;

        let discriminant = b * b - c;
        if discriminant < 0.0 {
            return Vec::new();
        }

        let sqrt_disc = discriminant.sqrt();

        let t1 = -b - sqrt_disc;
        let t2 = -b + sqrt_disc;

        let mut ts = Vec::new();

        if t1 > epsilon {
            ts.push(t1);
        }
        if t2 > epsilon && (t2 - t1).abs() > 1e-14 {
            ts.push(t2);
        }

        ts.sort_by(|a, b| a.partial_cmp(b).unwrap());
        ts
    }

    /// Intersect this ray with a circular arc segment.
    ///
    /// Returns (ray_t, arc_local_t) where:
    /// - ray_t is the distance along the ray (>= epsilon),
    /// - arc_local_t is the arc-length parameter in [0, arc.length()].
    ///
    /// Returns `None` if the ray misses the arc or hits the circle outside
    /// the arc's angular span.
    pub fn intersect_circular_arc(
        &self,
        arc: &CircularArcSegment,
        epsilon: f64,
    ) -> Option<(f64, f64)> {
        let d = self.direction.try_normalized()?;

        let t_candidates = self.intersect_circle(arc.center, arc.radius, epsilon);
        if t_candidates.is_empty() {
            return None;
        }

        let arc_len = arc.length();
        let tol = 1e-9;

        let mut best: Option<(f64, f64)> = None;

        for t in t_candidates {
            let p = self.origin + d * t;
            let rel = p - arc.center;
            let theta = rel.y.atan2(rel.x);

            // Compute angular span of the arc in the direction of its parameterization.
            let span = if arc.ccw {
                // CCW sweep from start_angle to end_angle
                let s = arc.start_angle;
                let mut e = arc.end_angle;
                // Normalize angles so that e is ahead of s in CCW direction
                let two_pi = 2.0 * std::f64::consts::PI;
                while e < s {
                    e += two_pi;
                }
                // Bring theta into the same "band"
                let mut th = theta;
                while th < s {
                    th += two_pi;
                }
                while th > e {
                    th -= two_pi;
                }
                // If theta is now outside [s, e] by more than tol, it's not on the arc.
                if th < s - tol || th > e + tol {
                    continue;
                }
                // local_t = radius * (th - s)
                let local_t = arc.radius * (th - s);
                (local_t, e - s)
            } else {
                // CW sweep from start_angle to end_angle
                let mut s = arc.start_angle;
                let e = arc.end_angle;
                let two_pi = 2.0 * std::f64::consts::PI;
                // For CW, we want s to be "ahead" of e when going CW,
                // which is equivalent to e being ahead of s in CCW if we swap.
                while s < e {
                    s += two_pi;
                }
                let mut th = theta;
                while th > s {
                    th -= two_pi;
                }
                while th < e {
                    th += two_pi;
                }
                if th > s + tol || th < e - tol {
                    continue;
                }
                // In CW direction, local_t = radius * (s - th)
                let local_t = arc.radius * (s - th);
                (local_t, s - e)
            };

            let (mut local_t, _) = span;

            if local_t < -tol || local_t > arc_len + tol {
                continue;
            }

            if local_t < 0.0 {
                local_t = 0.0;
            } else if local_t > arc_len {
                local_t = arc_len;
            }

            match best {
                None => best = Some((t, local_t)),
                Some((best_t, _)) => {
                    if t < best_t {
                        best = Some((t, local_t));
                    }
                }
            }
        }

        best
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
                BoundarySegment::CircularArc(arc_seg) => self
                    .intersect_circular_arc(&arc_seg, epsilon)
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

#[cfg(test)]
mod arc_intersection_tests {
    use super::Ray;
    use crate::geometry::boundary::{BilliardTable, BoundaryComponent};
    use crate::geometry::primitives::Vec2;
    use crate::geometry::segments::{BoundarySegment, CircularArcSegment};

    fn quarter_circle_table() -> BilliardTable {
        // Single boundary: quarter-circle from (1,0) to (0,1), CCW.
        let arc = CircularArcSegment::new(
            Vec2::new(0.0, 0.0),
            1.0,
            0.0,
            std::f64::consts::FRAC_PI_2,
            true,
        );
        let outer = BoundaryComponent::new("outer", vec![BoundarySegment::CircularArc(arc)]);
        BilliardTable {
            outer,
            obstacles: Vec::new(),
        }
    }

    #[test]
    fn ray_hits_quarter_circle_arc() {
        let table = quarter_circle_table();

        // Ray from x = 2, y = 0.5 pointing toward the center.
        let ray = Ray {
            origin: Vec2::new(2.0, 0.5),
            direction: Vec2::new(-1.0, 0.0),
        };

        let epsilon = 1e-8;
        let hit = ray.intersect_table(&table, epsilon);

        assert!(hit.is_some(), "Expected intersection with quarter-circle");
        let hit = hit.unwrap();

        // Should be outer component, segment 0
        assert_eq!(hit.component_index, 0);
        assert_eq!(hit.segment_index, 0);

        // Hit point should lie on the circle x^2 + y^2 = 1 and between angles 0 and π/2.
        // Compute the actual point from ray parameter:
        let dir = ray.direction.try_normalized().unwrap();
        let p = ray.origin + dir * hit.ray_parameter;

        let r2 = p.x * p.x + p.y * p.y;
        assert!(
            (r2 - 1.0).abs() < 1e-10,
            "Hit point not on circle: r^2 = {}",
            r2
        );

        let angle = p.y.atan2(p.x);
        assert!(
            (-1e-8..=std::f64::consts::FRAC_PI_2 + 1e-8).contains(&angle),
            "Angle {} outside expected [0, π/2]",
            angle
        );
    }
}

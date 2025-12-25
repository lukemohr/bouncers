use serde::{Deserialize, Serialize};
use std::ops::{Add, Div, Mul, Sub};

const LENGTH_LOWER_BOUND: f64 = 1e-10;

/// A simple 2D vector for geometric computations.
///
/// This is intentionally minimal to start. You can later:
/// - derive more traits (e.g., `Eq`, `PartialOrd`),
/// - add more methods (e.g., `distance_to`, `angle`, etc.),
/// - or swap this out for a library type (like `glam::DVec2`).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Add for Vec2 {
    type Output = Self;

    /// Component-wise addition.
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Vec2 {
    type Output = Self;

    /// Component-wise subtraction.
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul<f64> for Vec2 {
    type Output = Self;

    /// Scalar multiplication.
    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

// Optional but handy: scalar * Vec2
impl Mul<Vec2> for f64 {
    type Output = Vec2;

    /// Scalar multiplication (commuted).
    fn mul(self, rhs: Vec2) -> Self::Output {
        Vec2 {
            x: self * rhs.x,
            y: self * rhs.y,
        }
    }
}

impl Div<f64> for Vec2 {
    type Output = Self;

    /// Scalar multiplication.
    fn div(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl Vec2 {
    /// Construct a new vector from components.
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Euclidean length (magnitude) of the vector.
    pub fn length(&self) -> f64 {
        self.length_squared().sqrt()
    }

    /// Squared length, useful when you want to avoid a square root.
    pub fn length_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y
    }

    /// Returns a normalized (unit-length) copy of this vector.
    ///
    /// # Panics
    /// May panic or behave badly if the length is extremely small.
    /// For robust code, prefer `try_normalized`.
    pub fn normalized(&self) -> Self {
        let length = self.length();
        debug_assert!(
            length > LENGTH_LOWER_BOUND,
            "Vec2::normalized on near-zero vector"
        );
        Self {
            x: self.x / length,
            y: self.y / length,
        }
    }

    /// Attempts to return a normalized copy of this vector.
    ///
    /// Returns `None` if the vector is too close to zero-length.
    pub fn try_normalized(&self) -> Option<Self> {
        if self.length_squared() < LENGTH_LOWER_BOUND * LENGTH_LOWER_BOUND {
            None
        } else {
            Some(self.normalized())
        }
    }

    /// Dot product of two vectors.
    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y
    }

    /// Returns a vector perpendicular to this one, rotated +90 degrees.
    ///
    /// Mathematically, this is (-y, x). This is the "left turn" of the vector,
    /// and we will use that for defining inward normals on CCW boundaries.
    pub fn perp(self) -> Self {
        Self {
            x: -self.y,
            y: self.x,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Vec2;

    #[test]
    fn length_and_normalization_work() {
        let v = Vec2::new(3.0, 4.0);
        assert!((v.length() - 5.0).abs() < 1e-12);

        let n = v.normalized();
        assert!((n.length() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn scalar_multiplication_works() {
        let v = Vec2::new(1.0, -2.0);
        let a = v * 3.0;
        let b = 3.0 * v;

        assert_eq!(a, Vec2::new(3.0, -6.0));
        assert_eq!(b, Vec2::new(3.0, -6.0));
    }

    #[test]
    fn perp_rotates_left() {
        let v = Vec2::new(1.0, 0.0);
        let p = v.perp();
        // Expected left rotation: (0, 1)
        assert!((p.x - 0.0).abs() < 1e-12);
        assert!((p.y - 1.0).abs() < 1e-12);
    }
}

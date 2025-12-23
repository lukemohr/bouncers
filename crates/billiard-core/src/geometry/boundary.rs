//! Boundary representations for billiard tables.
//
//! For now this is a very simple placeholder. Later it will:
//! - store piecewise segments,
//! - support arc-length parametrization,
//! - distinguish outer boundary vs internal obstacles (Sinai billiards).

/// Placeholder type representing a closed boundary component.
///
/// In the next steps, this will be extended to hold segments and
/// arc-length information.
pub struct BoundaryComponent {
    /// Human-readable name or label for this component.
    pub name: String,
    // Later:
    // pub segments: Vec<BoundarySegment>,
    // cumulative_lengths: Vec<f64>,
    // total_length: f64,
}

impl BoundaryComponent {
    /// Create a new boundary component with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        BoundaryComponent { name: name.into() }
    }
}

#[cfg(test)]
mod tests {
    use super::BoundaryComponent;

    #[test]
    fn can_create_boundary_component() {
        let c = BoundaryComponent::new("outer");
        assert_eq!(c.name, "outer");
    }
}

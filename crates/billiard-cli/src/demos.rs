use billiard_core::dynamics::simulation::run_trajectory;
use billiard_core::dynamics::state::BoundaryState;
use billiard_core::geometry::boundary::BilliardTable;

use crate::demo_tables::sinai_table;

/// Run a demonstration trajectory on a Sinai-style table and print collisions.
pub fn run_sinai_demo() -> Result<(), Box<dyn std::error::Error>> {
    // TODO:
    // 1. Build table with sinai_table()
    // 2. Choose an initial BoundaryState
    // 3. Call run_trajectory
    // 4. Print the collisions
    let table: BilliardTable = sinai_table();

    let initial = BoundaryState {
        component_index: 0,                 // outer boundary
        s: 0.3, // 30% along bottom edge, if that’s how you set up segments
        theta: std::f64::consts::FRAC_PI_3, // 60 degrees inward
    };

    let epsilon = 1e-8;
    let max_steps = 50;

    let collisions = run_trajectory(&table, &initial, max_steps, epsilon);

    // ---- CSV HEADER ----
    println!(
        "{:<6} {:<6} {:<8} {:>10} {:>12} {:>12} {:>12}",
        "step", "comp", "seg", "s", "theta", "x", "y"
    );

    // ---- Print each collision with color ----
    for (step, c) in collisions.iter().enumerate() {
        // Color coding:
        // - Outer boundary (component 0) = bright white (default)
        // - Internal obstacles           = bright cyan
        let color = if c.component_index == 0 {
            "\x1b[97m" // bright white
        } else {
            "\x1b[96m" // bright cyan
        };
        let reset = "\x1b[0m";

        println!(
            "{}{:<6} {:<6} {:<8} {:>10.6} {:>12.6} {:>12.6} {:>12.6}{}",
            color,
            step,
            c.component_index,
            c.segment_index,
            c.s,
            c.theta,
            c.hit_point.x,
            c.hit_point.y,
            reset
        );
    }

    Ok(())
}

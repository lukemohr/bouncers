mod demo_tables;
mod demos;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // For now, just run a hard-coded demo.
    demos::run_sinai_demo()?;
    Ok(())
}

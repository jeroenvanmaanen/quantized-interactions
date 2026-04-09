use anyhow::Result;

fn main() -> Result<()> {
    env_logger::init();
    if let Err(error) = quantized_interactions::main() {
        eprintln!("Error: {error:?}");
        return Err(error);
    }
    Ok(())
}

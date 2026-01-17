use anyhow::Result;

fn main() -> Result<()> {
    env_logger::init();
    quantized_interactions::main()
}

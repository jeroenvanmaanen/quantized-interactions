mod cell;
mod conway;
mod experiment;
mod torus;
mod wave;

use anyhow::Result;
use clap::{Parser, Subcommand, command};
use log::{debug, info};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    cli_command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "Conway's game of life")]
    Conway,

    #[command(about = "Experiment with a real-valued state value")]
    Experiment,

    #[command(about = "simulate a wave")]
    Wave {
        #[arg(help = "size of torus (must be even)")]
        size: usize,

        #[arg(help = "execute debug function", required = false, long)]
        debug: bool,
    },
}

pub fn main() -> Result<()> {
    info!("Quantized interactions");

    let cli = Cli::parse();
    match cli.cli_command {
        Some(Commands::Wave { size, debug }) => {
            if debug {
                wave::debug(size)?
            } else {
                wave::example(size)?
            }
        }
        Some(Commands::Conway) => conway::example()?,
        Some(Commands::Experiment) => experiment::example()?,
        None => help()?,
    }

    Ok(())
}

fn help() -> Result<()> {
    info!("Help!");
    let executable = if let Some(exe) = std::env::args().next() {
        exe
    } else {
        "?".to_string()
    };
    let cli = Cli::parse_from([&executable, "--help"].iter());
    debug!("CLI command: [{:?}]", cli.cli_command);
    Ok(())
}

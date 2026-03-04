//! MathCore CLI entry point

use std::str::FromStr;

use clap::Parser;
use mathcore_cli::{run_compute, run_diff, run_integrate, run_simplify, Cli, Commands};

fn main() {
    // Set up logging based on verbosity
    let cli = Cli::parse();

    let log_level = tracing::Level::from_str(&cli.verbose).unwrap_or(tracing::Level::WARN);
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .init();

    // Execute the appropriate command
    let result = match &cli.command {
        Commands::Compute {
            expression,
            variables,
        } => run_compute(expression, variables),
        Commands::Simplify { expression } => run_simplify(expression),
        Commands::Diff { expression, var } => run_diff(expression, var),
        Commands::Integrate {
            expression,
            var,
            from,
            to,
            n,
        } => run_integrate(expression, var, *from, *to, *n),
        Commands::Version => {
            println!("{}", mathcore_cli::get_version());
            return;
        }
    };

    match result {
        Ok(output) => {
            println!("{}", output);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

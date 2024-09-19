use std::env;

use clap::Parser;

mod error;
mod logger;
mod service;
mod types;
mod util;

use error::*;
use logger::*;
use types::Cli;
use util::load_env;

fn main() -> Result<(), AppError> {
    let Cli {
        operation,
        verbose,
        debug,
    } = Cli::parse();
    let _ = logger::init(LoggerFlags { verbose, debug });

    if debug {
        env::set_var("HABITUI_DEBUG", "true");
    }

    load_env()?;

    service::run_operation(operation)?;

    Ok(())
}

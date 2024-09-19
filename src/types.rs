use clap::{Parser, Subcommand};

#[derive(Subcommand)]
pub enum Operation {
    /// List all TODOs
    List {
        /// Save the list of tasks as a JSON file
        #[arg(long, default_value_t = false)]
        save_json: bool,
    },

    /// List completed TODOs
    History,

    /// Create a new TODO item
    Task {
        /// Optionally define TODO item with a descriptor. Format:
        /// <name>,<difficulty>,<notes>,<due>,<checklist1>;<checklist2>;...
        descriptor: Option<String>,
    },

    /// Reorder tasks by descending priority
    Reorder,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(next_line_help = true)]
pub struct Cli {
    #[command(subcommand)]
    pub operation: Option<Operation>,

    /// Run command verbosely
    #[arg(long, default_value_t = false)]
    pub verbose: bool,

    /// Turn debugging information on
    #[arg(short, long, default_value_t = false)]
    pub debug: bool,
}

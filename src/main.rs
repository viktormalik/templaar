mod errors;
mod new;
mod take;
mod utils;

use clap::{Parser, Subcommand};
use new::new;
use std::{path::PathBuf, process};
use take::take;

#[derive(Debug, Parser)]
#[clap(
    name = "templaar",
    about = "A simple tool for creating text files from templates"
)]
struct Templaar {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Create a template
    New {
        /// Name of the template
        name: Option<String>,
        /// Make the template global
        #[clap(long, short)]
        global: bool,
        /// Create the template from file(s).
        /// In case of multiple files, the template will be a directory.
        #[clap(long, short, verbatim_doc_comment, num_args(0..))]
        files: Vec<PathBuf>,
    },
    /// Create a file from a template
    Take {
        /// Name of the created file.
        /// Path in the case of a directory template.
        #[clap(verbatim_doc_comment)]
        name: Option<String>,
        /// Use specific template
        #[clap(long, short = 't')]
        template: Option<String>,
    },
}

fn main() {
    let templaar = Templaar::parse();

    if let Err(e) = match templaar.command {
        Command::New {
            name,
            global,
            files,
        } => new(&name, global, &files),
        Command::Take { name, template } => take(&name, &template),
    } {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}

use clap::{Parser, Subcommand};
use std::{
    env, error,
    io::{self, Write},
    process,
};

#[derive(Debug, Parser)]
#[clap(name = "templaar")]
struct Templaar {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    New,
}

fn new() -> Result<(), Box<dyn error::Error>> {
    let mut templ_name = String::new();
    print!("Enter template name (default 'templ'): ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut templ_name)?;

    if templ_name.trim().is_empty() {
        templ_name = "templ".to_string();
    }

    let editor = env::var("EDITOR")?;
    let templ_file = env::current_dir()?.join(format!(".{templ_name}.aar"));
    process::Command::new(editor).arg(&templ_file).status()?;

    Ok(())
}

fn main() {
    let templaar = Templaar::parse();

    if let Err(e) = match templaar.command {
        Command::New => new(),
    } {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}

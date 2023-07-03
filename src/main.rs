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
    New { name: Option<String> },
}

fn new(name: &Option<String>) -> Result<(), Box<dyn error::Error>> {
    let templ_name = match name {
        Some(n) => n.clone(),
        None => {
            // Read template name from stdin
            let mut buf = String::new();
            print!("Enter template name (default 'templ'): ");
            io::stdout().flush()?;
            io::stdin().read_line(&mut buf)?;

            match buf.trim() {
                "" => "templ".to_string(),
                b => b.to_string(),
            }
        }
    };

    let editor = env::var("EDITOR")?;
    let templ_file = env::current_dir()?.join(format!(".{templ_name}.aar"));
    process::Command::new(editor).arg(&templ_file).status()?;

    Ok(())
}

fn main() {
    let templaar = Templaar::parse();

    if let Err(e) = match templaar.command {
        Command::New { name } => new(&name),
    } {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}

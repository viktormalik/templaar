use clap::{Parser, Subcommand};
use std::{
    env, error,
    ffi::OsStr,
    fmt, fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
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
    Take,
}

/// Templaar errors
#[derive(Debug, Clone)]
struct NoTemplateFound;

impl error::Error for NoTemplateFound {}

impl fmt::Display for NoTemplateFound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "No template found in the current or parent directories")
    }
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

/// Searches for a file with the .aar extension in `dir`
fn find_templ_in_dir(dir: &Path) -> Result<Option<PathBuf>, io::Error> {
    match fs::read_dir(dir)?.find(|f| match f {
        Ok(file) => file.path().extension() == Some(OsStr::new("aar")),
        Err(_) => false,
    }) {
        Some(f) => f.map(|file| Some(file.path())),
        None => Ok(None),
    }
}

/// Searches for a template, starting from the current directory and recursively descending into
/// its parents, until any template is found.
fn find_templ() -> Result<Option<PathBuf>, io::Error> {
    let mut dir = env::current_dir()?;
    loop {
        match find_templ_in_dir(&dir)? {
            Some(file) => return Ok(Some(dir.join(&file))),
            None => match dir.parent() {
                Some(parent) => dir = parent.to_path_buf(),
                None => return Ok(None),
            },
        }
    }
}

fn take() -> Result<(), Box<dyn error::Error>> {
    // Find and read the template
    let templ_file = find_templ()?.ok_or(NoTemplateFound)?;
    let mut templ = String::new();
    fs::File::open(&templ_file)?.read_to_string(&mut templ)?;

    // default filename is the template name without the leading '.' and file extension
    let filename: String = templ_file.file_stem().unwrap().to_str().unwrap()[1..].to_string();
    let file = env::current_dir()?.join(filename);

    // Write template contents to file and open it in EDITOR
    fs::File::create(&file)?.write_all(templ.as_bytes())?;

    let editor = env::var("EDITOR")?;
    process::Command::new(editor).arg(file).status()?;

    Ok(())
}

fn main() {
    let templaar = Templaar::parse();

    if let Err(e) = match templaar.command {
        Command::New { name } => new(&name),
        Command::Take => take(),
    } {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}

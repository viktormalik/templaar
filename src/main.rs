use clap::{Parser, Subcommand};
use std::{
    env, error,
    ffi::OsStr,
    fmt, fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process,
    str::FromStr,
};

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
        /// Create the template from file
        #[clap(long, short)]
        file: Option<PathBuf>,
    },
    /// Create a file from a template
    Take {
        /// Name of the created file
        name: Option<String>,
        /// Use specific template
        #[clap(long, short = 't')]
        template: Option<String>,
    },
}

/// Templaar errors
#[derive(Debug, Clone)]
struct NoTemplateFound;

impl error::Error for NoTemplateFound {}

impl fmt::Display for NoTemplateFound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = "No template found in the current or parent directories.\n\
                   For global templates, specify the template name using the -t option.";
        write!(f, "{}", msg)
    }
}

#[derive(Debug, Clone)]
struct TemplExists {
    path: PathBuf,
}

impl error::Error for TemplExists {}

impl fmt::Display for TemplExists {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Template {} already exists. Please edit it manually.",
            self.path.to_str().ok_or(fmt::Error)?
        )
    }
}

#[derive(Debug, Clone)]
struct PathExists {
    path: PathBuf,
}

impl error::Error for PathExists {}

impl fmt::Display for PathExists {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Cannot create {} from template, path already exists.",
            self.path.to_str().ok_or(fmt::Error)?
        )
    }
}

#[derive(Debug, Clone)]
struct AmbiguousTemplate {
    names: Vec<String>,
    dir: PathBuf,
}

impl error::Error for AmbiguousTemplate {}

impl fmt::Display for AmbiguousTemplate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Ambiguous template: found {:?} in {:?}. Use -t to select the template.",
            self.names, self.dir,
        )
    }
}

/// Encode `templ` into the corresponding filename:
///   .`templ`.aar
fn templ_to_path(templ: &str, global: bool) -> PathBuf {
    let prefix = if global { "" } else { "." };
    PathBuf::from_str(&format!("{prefix}{templ}.aar")).unwrap()
}

/// Decode template name from `path`, inverse to `templ_to_path`.
fn path_to_templ(path: &PathBuf) -> String {
    let mut templ = path.file_stem().unwrap().to_str().unwrap_or("<invalid>");
    if templ.starts_with(".") {
        templ = &templ[1..];
    }
    templ.to_string()
}

/// Get global templates directory (~/.config/templaar).
/// Creates the directory if it doesn't exist.
fn global_dir() -> Result<PathBuf, Box<dyn error::Error>> {
    let dir: PathBuf = [env::var("HOME")?.as_str(), ".config", "templaar"]
        .iter()
        .collect();
    if !dir.exists() {
        fs::create_dir(&dir)?;
    }
    return Ok(dir);
}

fn new(
    name: &Option<String>,
    global: bool,
    file: &Option<PathBuf>,
) -> Result<(), Box<dyn error::Error>> {
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

    let templ_dir = match global {
        true => global_dir()?,
        false => env::current_dir()?,
    };
    let templ_file = templ_dir.join(templ_to_path(&templ_name, global));

    if templ_file.exists() {
        return Err(Box::new(TemplExists { path: templ_file }));
    }

    if file.is_some() {
        fs::copy(file.as_ref().unwrap(), &templ_file)?;
    }
    let editor = env::var("EDITOR")?;
    process::Command::new(editor).arg(&templ_file).status()?;

    Ok(())
}

/// Searches for a template file in `dir`.
/// If `name` is given, looks for the corresponding file,
/// otherwise looks for any file the the ".aar" extension.
fn find_templ_in_dir(
    dir: &Path,
    name: &Option<String>,
) -> Result<Option<PathBuf>, Box<dyn error::Error>> {
    let templates: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(|f| match f {
            Ok(file) => (match name {
                Some(n) => path_to_templ(&file.path()) == *n,
                None => file.path().extension() == Some(OsStr::new("aar")),
            })
            .then_some(file.path()),
            Err(_) => None,
        })
        .collect();

    match &templates[..] {
        [] => Ok(None),
        [f] => Ok(Some(f.clone())),
        _ => Err(Box::new(AmbiguousTemplate {
            names: templates.iter().map(path_to_templ).collect(),
            dir: dir.to_path_buf(),
        })),
    }
}

/// Searches for a template.
///
/// The search starts from the current directory and recursively descends into the parents.
/// If no template is found, the global templates directory is searched.
fn find_templ(name: &Option<String>) -> Result<Option<PathBuf>, Box<dyn error::Error>> {
    let mut dir = env::current_dir()?;
    loop {
        match find_templ_in_dir(&dir, name)? {
            Some(file) => return Ok(Some(dir.join(&file))),
            None => match dir.parent() {
                Some(parent) => dir = parent.to_path_buf(),
                None => break,
            },
        }
    }

    // Search global directory -> name must be specified
    if name.is_none() {
        return Ok(None);
    }
    return find_templ_in_dir(&global_dir()?, name);
}

fn take(name: &Option<String>, template: &Option<String>) -> Result<(), Box<dyn error::Error>> {
    // Find and read the template
    let templ_file = find_templ(template)?.ok_or(NoTemplateFound)?;
    let mut templ = String::new();
    fs::File::open(&templ_file)?.read_to_string(&mut templ)?;

    let filename = match name {
        Some(n) => n.clone(),
        None => path_to_templ(&templ_file),
    };
    let file = env::current_dir()?.join(filename);

    if file.exists() {
        return Err(Box::new(PathExists { path: file }));
    }

    // Write template contents to file and open it in EDITOR
    fs::File::create(&file)?.write_all(templ.as_bytes())?;

    let editor = env::var("EDITOR")?;
    process::Command::new(editor).arg(&file).status()?;

    let mut file_contents = String::new();
    fs::File::open(&file)?.read_to_string(&mut file_contents)?;
    if file_contents == templ {
        let mut buf = String::new();
        print!("The file contains no change from the template. Save it anyways? [Y/n]: ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut buf)?;

        if buf.trim().to_lowercase() == "n" {
            std::fs::remove_file(file)?;
        }
    }

    Ok(())
}

fn main() {
    let templaar = Templaar::parse();

    if let Err(e) = match templaar.command {
        Command::New { name, global, file } => new(&name, global, &file),
        Command::Take { name, template } => take(&name, &template),
    } {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}

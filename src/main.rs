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
#[clap(name = "templaar")]
struct Templaar {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    New {
        name: Option<String>,
    },
    Take {
        name: Option<String>,
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
        write!(f, "No template found in the current or parent directories")
    }
}

#[derive(Debug, Clone)]
struct FileExists {
    path: PathBuf,
}

impl error::Error for FileExists {}

impl fmt::Display for FileExists {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "File {} already exists. Please edit it manually.",
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
fn templ_to_path(templ: &str) -> PathBuf {
    PathBuf::from_str(&format!(".{templ}.aar")).unwrap()
}

/// Decode template name from `path`, inverse to `templ_to_path`.
fn path_to_templ(path: &PathBuf) -> String {
    path.file_stem().unwrap().to_str().unwrap_or("<invalid>")[1..].to_string()
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

    let templ_file = env::current_dir()?.join(templ_to_path(&templ_name));
    if templ_file.exists() {
        return Err(Box::new(FileExists { path: templ_file }));
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

/// Searches for a template, starting from the current directory and recursively descending into
/// its parents, until any template is found.
fn find_templ(name: &Option<String>) -> Result<Option<PathBuf>, Box<dyn error::Error>> {
    let mut dir = env::current_dir()?;
    loop {
        match find_templ_in_dir(&dir, name)? {
            Some(file) => return Ok(Some(dir.join(&file))),
            None => match dir.parent() {
                Some(parent) => dir = parent.to_path_buf(),
                None => return Ok(None),
            },
        }
    }
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
        return Err(Box::new(FileExists { path: file }));
    }

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
        Command::Take { name, template } => take(&name, &template),
    } {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}

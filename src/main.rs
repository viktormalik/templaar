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

#[derive(Debug, Clone)]
struct InvalidTemplate {
    templ_path: PathBuf,
    reason: String,
}

impl error::Error for InvalidTemplate {}

impl fmt::Display for InvalidTemplate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Invalid template {}: {}",
            self.templ_path.to_str().ok_or(fmt::Error)?,
            self.reason
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
    files: &Vec<PathBuf>,
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

    match files.len() {
        0 => {}
        1 => {
            // Single file -> copy it to template
            fs::copy(&files[0], &templ_file)?;
        }
        _ => {
            // Multiple files -> make template a directory containing all files
            // under their original names
            fs::create_dir(&templ_file)?;
            for f in files {
                fs::copy(f, templ_file.join(f.file_name().unwrap()))?;
            }
        }
    };

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

fn user_prompt_bool(prompt: &str) -> Result<bool, Box<dyn error::Error>> {
    let mut buf = String::new();
    print!("{} [Y/n]: ", prompt);

    io::stdout().flush()?;
    io::stdin().read_line(&mut buf)?;

    Ok(buf.trim().to_lowercase() != "n")
}

fn take(name: &Option<String>, template: &Option<String>) -> Result<(), Box<dyn error::Error>> {
    let templ = find_templ(template)?.ok_or(NoTemplateFound)?;

    let target_name = match name {
        Some(n) => n.clone(),
        None => path_to_templ(&templ),
    };
    let target = env::current_dir()?.join(target_name);

    if templ.is_dir() {
        // Directory template

        let templ_files = templ
            .read_dir()?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, io::Error>>()?;

        // Error if the template contains sub-directories
        if templ_files.iter().any(|f| f.is_dir()) {
            return Err(Box::new(InvalidTemplate {
                templ_path: templ,
                reason: "directory template contains sub-directories".to_string(),
            }));
        }

        // Create the target directory, if it doesn't exist
        if !target.exists() {
            fs::create_dir(&target)?;
        }

        let target_files = target
            .read_dir()?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, io::Error>>()?;

        // Warn if the target directory is non-empty
        if !target_files.is_empty() {
            let prompt = format!(
                "Directory {} is not empty, do you wish to continue?",
                target.to_str().ok_or(fmt::Error)?
            );
            if !user_prompt_bool(&prompt)? {
                return Ok(());
            }
        }

        // Error if the target directory contains any of the template files
        match target_files.iter().find(|file| {
            templ_files
                .iter()
                .any(|f| file.file_name() == f.file_name())
        }) {
            Some(file) => return Err(Box::new(PathExists { path: file.clone() })),
            None => {}
        }

        // Copy files from the template to the target directory
        for file in templ_files {
            fs::copy(&file, target.join(file.file_name().unwrap()))?;
        }
    } else {
        // File template

        // Error if the target already exists
        if target.exists() {
            return Err(Box::new(PathExists { path: target }));
        }

        // Copy the template into the target file
        fs::copy(&templ, &target)?;
    }

    // Open the target file/directory in the default editor
    let editor = env::var("EDITOR")?;
    process::Command::new(editor).arg(&target).status()?;

    // For normal file templates, check if the target file contents is different
    // from the template and if not, warn and offer user not to save the target.
    if templ.is_file() {
        let mut target_contents = String::new();
        let mut templ_contents = String::new();
        fs::File::open(&target)?.read_to_string(&mut target_contents)?;
        fs::File::open(&templ)?.read_to_string(&mut templ_contents)?;
        if target_contents == templ_contents {
            let prompt = "The file contains no change from the template. Save it anyways?";
            if !user_prompt_bool(&prompt)? {
                std::fs::remove_file(target)?;
            }
        }
    }

    Ok(())
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

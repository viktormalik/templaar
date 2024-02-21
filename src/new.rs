use std::{
    env, error, fs,
    io::{self, Write},
    path::PathBuf,
    process,
};

use crate::{
    errors::TemplExists,
    utils::{global_dir, templ_to_path},
};

/// The handler of the `new` sub-command
///
/// # Arguments
///
/// * `name` - Optional name of the template. If not specified, the user is
///            queried for the name.
/// * `global` - Boolean flag whether the template should be created as global
/// * `files` - List of files to create the template from.
pub fn new(
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

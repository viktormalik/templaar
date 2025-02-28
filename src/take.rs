use std::{
    env, error, fmt, fs,
    io::{self, Read},
    path::{Path, PathBuf},
    process,
};

use crate::{
    errors::{AmbiguousTemplate, InvalidTemplate, NoTemplateFound, PathExists},
    utils::{global_dir, path_to_templ, templs_in_dir, user_prompt_bool},
};

/// Searches for a template file in `dir`.
/// If `name` is given, looks for the corresponding file,
/// otherwise looks for any file the the ".aar" extension.
fn find_templ_in_dir(
    dir: &Path,
    name: &Option<String>,
) -> Result<Option<PathBuf>, Box<dyn error::Error>> {
    let templates = templs_in_dir(dir)?;
    let matches = match name {
        Some(n) => templates
            .into_iter()
            .filter(|t| path_to_templ(t) == *n)
            .collect(),
        None => templates,
    };

    match &matches[..] {
        [] => Ok(None),
        [f] => Ok(Some(f.clone())),
        _ => Err(Box::new(AmbiguousTemplate {
            names: matches.iter().map(path_to_templ).collect(),
            dir: dir.to_path_buf(),
        })),
    }
}

/// Searches for a template.
///
/// The search starts from the current directory and recursively descends into
/// the parents. If no template is found, the global templates directory is searched.
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

/// The handler of the 'take' sub-command.
///
/// # Arguments
///
/// * `name` - Optional name of the target
/// * `template` - Optional name of the template to use
pub fn take(name: &Option<String>, template: &Option<String>) -> Result<(), Box<dyn error::Error>> {
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

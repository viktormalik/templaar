use std::{
    env, error, fs,
    io::{self, Write},
    path::PathBuf,
    str::FromStr,
};

/// Encode template name into the corresponding file name.
///
/// The returned filename is:
/// - .`templ`.aar for local templates
/// - `templ`.aar for global templates
pub fn templ_to_path(templ: &str, global: bool) -> PathBuf {
    let prefix = if global { "" } else { "." };
    PathBuf::from_str(&format!("{prefix}{templ}.aar")).unwrap()
}

/// Decode template name from a file name (inverse to `templ_to_path`).
pub fn path_to_templ(path: &PathBuf) -> String {
    let mut templ = path.file_stem().unwrap().to_str().unwrap_or("<invalid>");
    if templ.starts_with(".") {
        templ = &templ[1..];
    }
    templ.to_string()
}

/// Get global templates directory (~/.config/templaar).
/// Creates the directory if it doesn't exist.
pub fn global_dir() -> Result<PathBuf, Box<dyn error::Error>> {
    let dir: PathBuf = [env::var("HOME")?.as_str(), ".config", "templaar"]
        .iter()
        .collect();
    if !dir.exists() {
        fs::create_dir(&dir)?;
    }
    return Ok(dir);
}

/// Query user for a boolean (yes/no) input.
///
/// Returns true if the user selected "yes".
///
/// Default answer is "yes".
pub fn user_prompt_bool(prompt: &str) -> Result<bool, Box<dyn error::Error>> {
    let mut buf = String::new();
    print!("{} [Y/n]: ", prompt);

    io::stdout().flush()?;
    io::stdin().read_line(&mut buf)?;

    Ok(buf.trim().to_lowercase() != "n")
}

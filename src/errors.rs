use std::{error, fmt, path::PathBuf};

/// No template was found, print a hint.
#[derive(Debug, Clone)]
pub struct NoTemplateFound;

impl error::Error for NoTemplateFound {}

impl fmt::Display for NoTemplateFound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = "No template found in the current or parent directories.\n\
                   For global templates, specify the template name using the -t option.";
        write!(f, "{}", msg)
    }
}

/// Could not create a new template as it already exists.
///
/// # Arguments
///
/// * `path` - Path to the existing template
#[derive(Debug, Clone)]
pub struct TemplExists {
    pub path: PathBuf,
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

/// Could not create a new file from the template because a file with the same
/// name already exists.
///
/// # Arguments
///
/// * `path` - Path to the existing (conflicting) file
#[derive(Debug, Clone)]
pub struct PathExists {
    pub path: PathBuf,
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

/// Found multiple candidate templates to use. Print a hint to use -t to select
/// one template.
///
/// # Arguments
///
/// * `names` - List of candidate templates found
/// * `dir` - Directory in which the templates were found
#[derive(Debug, Clone)]
pub struct AmbiguousTemplate {
    pub names: Vec<String>,
    pub dir: PathBuf,
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

/// Invalid template format
///
/// # Arguments
///
/// * `templ_path` - Path to the template
/// * `reason` - Reason why the template is invalid (error message)
#[derive(Debug, Clone)]
pub struct InvalidTemplate {
    pub templ_path: PathBuf,
    pub reason: String,
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

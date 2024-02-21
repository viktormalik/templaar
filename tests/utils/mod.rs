use std::{
    collections::HashMap,
    env, fs,
    io::Write,
    ops,
    path::{Path, PathBuf},
};

pub fn set_editor(editor: &str) {
    env::set_var("EDITOR", editor);
}

/// Basic structure which should be used by each test.
/// Transparently sets up a new directory for testing data and cleans it up
/// after the test end.
pub struct Test {
    cwd: PathBuf,
    test_dir: PathBuf,
}

impl Test {
    /// Initialize a new test. See inline comments for individual steps.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the test
    /// * `init_dirs` - List of directory names to initialize
    /// * `init_files` - Map of file names and their contents to initialize
    /// * `editor` - Value to set the EDITOR env var to
    pub fn init(
        name: &str,
        init_dirs: Vec<PathBuf>,
        init_files: HashMap<PathBuf, String>,
        editor: &str,
    ) -> Result<Self, std::io::Error> {
        set_editor(editor);
        // Create test directory and change to it
        let test_dir = Path::new(name).to_path_buf();
        fs::create_dir(&test_dir)?;
        let cwd = env::current_dir()?;
        env::set_current_dir(&test_dir)?;

        // Create directories
        for dir in &init_dirs {
            fs::create_dir_all(dir)?;
        }
        // Create pre-initialized files
        for (file, contents) in init_files {
            fs::File::create(&file)?.write_all(contents.as_bytes())?;
        }

        Ok(Self { cwd, test_dir })
    }
}

impl ops::Drop for Test {
    fn drop(&mut self) {
        let _ = env::set_current_dir(&self.cwd);
        let _ = fs::remove_dir_all(&self.test_dir);
    }
}

use assert_cmd::Command;
use serial_test::serial;
use std::{
    collections::HashMap,
    env,
    error::Error,
    fs,
    io::{Read, Write},
    ops,
    path::{Path, PathBuf},
    str::FromStr,
};

fn set_editor(editor: &str) {
    env::set_var("EDITOR", editor);
}

struct Test {
    cwd: PathBuf,
    test_dir: PathBuf,
}

impl Test {
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

#[test]
#[serial]
fn test_new_default() -> Result<(), Box<dyn Error>> {
    let _t = Test::init("new_default", vec![], HashMap::new(), "touch");

    let mut cmd = Command::cargo_bin("templaar")?;
    cmd.arg("new");
    cmd.assert().success();

    assert!(Path::new(".templ.aar").exists());

    Ok(())
}

#[test]
#[serial]
fn test_new_name_from_arg() -> Result<(), Box<dyn Error>> {
    let _t = Test::init("new_name_from_arg", vec![], HashMap::new(), "touch");

    let mut cmd = Command::cargo_bin("templaar")?;
    cmd.arg("new").arg("arg_name");
    cmd.assert().success();

    let templ_path = Path::new(".arg_name.aar");
    assert!(templ_path.exists());

    Ok(())
}

#[test]
#[serial]
fn test_new_name_from_stdin() -> Result<(), Box<dyn Error>> {
    let _t = Test::init("new_name_from_stdin", vec![], HashMap::new(), "touch");

    let mut cmd = Command::cargo_bin("templaar")?;
    cmd.arg("new").write_stdin("stdin_name");
    cmd.assert().success();

    let templ_path = Path::new(".stdin_name.aar");
    assert!(templ_path.exists());

    Ok(())
}

#[test]
#[serial]
fn test_new_global() -> Result<(), Box<dyn Error>> {
    let home_dir = Path::new("home");
    let config_dir = home_dir.join(".config").join("templaar");
    let _t = Test::init(
        "new_global",
        vec![config_dir.to_path_buf()],
        HashMap::new(),
        "touch",
    );
    env::set_var("HOME", env::current_dir()?.join(home_dir));

    let mut cmd = Command::cargo_bin("templaar")?;
    cmd.arg("new").arg("--global");
    cmd.assert().success();

    let templ_path = config_dir.join("templ.aar");
    assert!(templ_path.exists());
    for path in fs::read_dir(env::current_dir()?)? {
        assert!(path?.path().is_dir());
    }

    Ok(())
}

#[test]
#[serial]
fn test_new_exists() -> Result<(), Box<dyn Error>> {
    let _t = Test::init(
        "new_exists",
        vec![PathBuf::from_str(".templ.aar")?],
        HashMap::new(),
        "touch",
    );

    let mut cmd = Command::cargo_bin("templaar")?;
    cmd.arg("new");
    cmd.assert().failure();

    Ok(())
}

#[test]
#[serial]
fn test_no_editor() -> Result<(), Box<dyn Error>> {
    env::remove_var("EDITOR");

    let mut cmd = Command::cargo_bin("templaar")?;
    cmd.arg("new").arg("note");
    cmd.assert().failure();

    Ok(())
}

#[test]
#[serial]
fn test_invalid_editor() -> Result<(), Box<dyn Error>> {
    set_editor("invalid");

    let mut cmd = Command::cargo_bin("templaar")?;
    cmd.arg("new").arg("note");
    cmd.assert().failure();

    Ok(())
}

#[test]
#[serial]
fn test_take_same_dir() -> Result<(), Box<dyn Error>> {
    let templ_content = "Template";
    let _t = Test::init(
        "take_same_dir",
        vec![],
        HashMap::from([(PathBuf::from_str(".templ.aar")?, templ_content.to_string())]),
        "touch",
    );

    let mut cmd = Command::cargo_bin("templaar")?;
    cmd.arg("take");
    cmd.assert().success();

    let file_path = Path::new("templ");
    let mut contents = String::new();
    assert!(file_path.exists());
    fs::File::open(&file_path)?.read_to_string(&mut contents)?;
    assert_eq!(contents, templ_content);

    Ok(())
}

#[test]
#[serial]
fn test_take_subdir() -> Result<(), Box<dyn Error>> {
    let subdir = Path::new("testdir");
    let templ_content = "Template";
    let _t = Test::init(
        "take_subdir",
        vec![subdir.to_path_buf()],
        HashMap::from([(subdir.join(".templ.aar"), templ_content.to_string())]),
        "touch",
    );

    env::set_current_dir("testdir")?;
    let mut cmd = Command::cargo_bin("templaar")?;
    cmd.arg("take");
    cmd.assert().success();

    let file_path = Path::new("templ");
    let mut contents = String::new();
    assert!(file_path.exists());
    fs::File::open(&file_path)?.read_to_string(&mut contents)?;
    assert_eq!(contents, templ_content);

    Ok(())
}

#[test]
#[serial]
fn test_take_named() -> Result<(), Box<dyn Error>> {
    let templ_content = "Template";
    let _t = Test::init(
        "take_named",
        vec![],
        HashMap::from([(PathBuf::from_str(".templ.aar")?, templ_content.to_string())]),
        "touch",
    );

    let mut cmd = Command::cargo_bin("templaar")?;
    cmd.arg("take").arg("name");
    cmd.assert().success();

    let file_path = Path::new("name");
    let mut contents = String::new();
    assert!(file_path.exists());
    fs::File::open(&file_path)?.read_to_string(&mut contents)?;
    assert_eq!(contents, templ_content);

    let default_file_path = Path::new("templ");
    assert!(!default_file_path.exists());

    Ok(())
}

#[test]
#[serial]
fn test_take_from_template() -> Result<(), Box<dyn Error>> {
    let templ_content = "Template";
    let other_content = "Template";
    let _t = Test::init(
        "take_from_template",
        vec![],
        HashMap::from([
            (PathBuf::from_str(".templ.aar")?, templ_content.to_string()),
            (PathBuf::from_str(".other.aar")?, other_content.to_string()),
        ]),
        "touch",
    );

    let mut cmd = Command::cargo_bin("templaar")?;
    cmd.arg("take").arg("name").arg("-t").arg("other");
    cmd.assert().success();

    let file_path = Path::new("name");
    let mut contents = String::new();
    assert!(file_path.exists());
    fs::File::open(&file_path)?.read_to_string(&mut contents)?;
    assert_eq!(contents, other_content);

    Ok(())
}

#[test]
#[serial]
fn test_take_global() -> Result<(), Box<dyn Error>> {
    let home_dir = Path::new("home");
    let config_dir = home_dir.join(".config").join("templaar");
    let templ_content = "Template";
    let _t = Test::init(
        "take_global",
        vec![config_dir.to_path_buf()],
        HashMap::from([(config_dir.join("templ.aar"), templ_content.to_string())]),
        "touch",
    );
    env::set_var("HOME", env::current_dir()?.join(home_dir));

    let mut cmd = Command::cargo_bin("templaar")?;
    cmd.arg("take").arg("-t").arg("templ");
    cmd.assert().success();

    let file_path = Path::new("templ");
    let mut contents = String::new();
    assert!(file_path.exists());
    fs::File::open(&file_path)?.read_to_string(&mut contents)?;
    assert_eq!(contents, templ_content);

    Ok(())
}

#[test]
#[serial]
fn test_take_global_precedence() -> Result<(), Box<dyn Error>> {
    let home_dir = Path::new("home");
    let config_dir = home_dir.join(".config").join("templaar");
    let templ_content = "Template";
    let other_content = "Other template";
    let _t = Test::init(
        "take_global",
        vec![config_dir.to_path_buf()],
        HashMap::from([
            (config_dir.join("templ.aar"), templ_content.to_string()),
            (PathBuf::from_str(".templ.aar")?, other_content.to_string()),
        ]),
        "touch",
    );
    env::set_var("HOME", env::current_dir()?.join(home_dir));

    let mut cmd = Command::cargo_bin("templaar")?;
    cmd.arg("take").arg("-t").arg("templ");
    cmd.assert().success();

    let file_path = Path::new("templ");
    let mut contents = String::new();
    assert!(file_path.exists());
    fs::File::open(&file_path)?.read_to_string(&mut contents)?;
    // Local template has precedence over the global one
    assert_eq!(contents, other_content);

    Ok(())
}

#[test]
#[serial]
fn test_take_exists() -> Result<(), Box<dyn Error>> {
    let _t = Test::init(
        "take_exists",
        vec![
            PathBuf::from_str(".templ.aar")?,
            PathBuf::from_str("templ")?,
        ],
        HashMap::new(),
        "touch",
    );

    let mut cmd = Command::cargo_bin("templaar")?;
    cmd.arg("take");
    cmd.assert().failure();

    Ok(())
}

#[test]
#[serial]
fn test_take_ambiguous() -> Result<(), Box<dyn Error>> {
    let _t = Test::init(
        "take_ambiguous",
        vec![
            PathBuf::from_str(".templ.aar")?,
            PathBuf::from_str(".note.aar")?,
        ],
        HashMap::new(),
        "touch",
    );

    let mut cmd = Command::cargo_bin("templaar")?;
    cmd.arg("take");
    cmd.assert().failure();

    Ok(())
}

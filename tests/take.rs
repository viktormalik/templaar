mod utils;

use assert_cmd::Command;
use serial_test::serial;
use std::{
    collections::HashMap,
    env,
    error::Error,
    fs,
    io::Read,
    path::{Path, PathBuf},
    str::FromStr,
};
use utils::Test;

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
    let other_content = "Other template";
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
fn test_take_no_change() -> Result<(), Box<dyn Error>> {
    let templ_content = "Template";
    let _t = Test::init(
        "take_no_change",
        vec![],
        HashMap::from([(PathBuf::from_str(".templ.aar")?, templ_content.to_string())]),
        "touch",
    );

    let mut cmd = Command::cargo_bin("templaar")?;
    cmd.arg("take").write_stdin("n");
    cmd.assert().success();

    let file_path = Path::new("templ");
    assert!(!file_path.exists());

    Ok(())
}

#[test]
#[serial]
fn test_take_exists() -> Result<(), Box<dyn Error>> {
    let _t = Test::init(
        "take_exists",
        vec![],
        HashMap::from([
            (PathBuf::from_str(".templ.aar")?, String::new()),
            (PathBuf::from_str("templ")?, String::new()),
        ]),
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

#[test]
#[serial]
fn test_take_from_dir() -> Result<(), Box<dyn Error>> {
    let templ_dir = PathBuf::from_str(".templ.aar")?;
    let file1_content = "Template";
    let file2_content = "Other template";

    let _t = Test::init(
        "take_from_dir",
        vec![templ_dir.clone()],
        HashMap::from([
            (templ_dir.join("file1"), file1_content.to_string()),
            (templ_dir.join("file2"), file2_content.to_string()),
        ]),
        "touch",
    );

    let mut cmd = Command::cargo_bin("templaar")?;
    cmd.arg("take");
    cmd.assert().success();

    let target_path = Path::new("templ");
    let file1_path = target_path.join("file1");
    let file2_path = target_path.join("file2");
    assert!(target_path.is_dir());
    assert!(file1_path.is_file());
    assert!(file2_path.is_file());

    let mut contents = String::new();
    fs::File::open(&file1_path)?.read_to_string(&mut contents)?;
    assert_eq!(contents, file1_content);

    contents.clear();
    fs::File::open(&file2_path)?.read_to_string(&mut contents)?;
    assert_eq!(contents, file2_content);

    Ok(())
}

#[test]
#[serial]
fn test_take_from_dir_into_nonempty_dir() -> Result<(), Box<dyn Error>> {
    let templ_dir = PathBuf::from_str(".templ.aar")?;
    let file1_content = "Template";
    let file2_content = "Other template";

    let _t = Test::init(
        "take_from_dir",
        vec![templ_dir.clone()],
        HashMap::from([
            (templ_dir.join("file1"), file1_content.to_string()),
            (templ_dir.join("file2"), file2_content.to_string()),
            (PathBuf::from_str("other_file")?, String::new()),
        ]),
        "touch",
    );

    let mut cmd = Command::cargo_bin("templaar")?;
    cmd.arg("take").arg(".");
    cmd.assert().success();

    let file1_path = Path::new("file1");
    let file2_path = Path::new("file2");
    assert!(file1_path.is_file());
    assert!(file2_path.is_file());

    let mut contents = String::new();
    fs::File::open(&file1_path)?.read_to_string(&mut contents)?;
    assert_eq!(contents, file1_content);

    contents.clear();
    fs::File::open(&file2_path)?.read_to_string(&mut contents)?;
    assert_eq!(contents, file2_content);

    Ok(())
}

#[test]
#[serial]
fn test_take_from_dir_conflict() -> Result<(), Box<dyn Error>> {
    let templ_dir = PathBuf::from_str(".templ.aar")?;
    let file1_content = "Template";
    let file2_content = "Other template";

    let _t = Test::init(
        "take_from_dir",
        vec![templ_dir.clone()],
        HashMap::from([
            (templ_dir.join("file1"), file1_content.to_string()),
            (templ_dir.join("file2"), file2_content.to_string()),
            (PathBuf::from_str("file1")?, String::new()),
        ]),
        "touch",
    );

    let mut cmd = Command::cargo_bin("templaar")?;
    cmd.arg("take").arg(".");
    cmd.assert().failure();

    assert!(!Path::new("file2").exists());

    Ok(())
}

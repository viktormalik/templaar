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
use utils::{set_editor, Test};

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
        vec![],
        HashMap::from([(PathBuf::from_str(".templ.aar")?, String::new())]),
        "touch",
    );

    let mut cmd = Command::cargo_bin("templaar")?;
    cmd.arg("new");
    cmd.assert().failure();

    Ok(())
}

#[test]
#[serial]
fn test_new_from_file() -> Result<(), Box<dyn Error>> {
    let templ_content = "Template";
    let _t = Test::init(
        "new_from_file",
        vec![],
        HashMap::from([(PathBuf::from_str("templ")?, templ_content.to_string())]),
        "touch",
    );

    let mut cmd = Command::cargo_bin("templaar")?;
    cmd.arg("new").arg("templ").arg("-f").arg("templ");
    cmd.assert().success();

    let templ_path = Path::new(".templ.aar");
    let mut contents = String::new();
    assert!(templ_path.exists());
    fs::File::open(&templ_path)?.read_to_string(&mut contents)?;
    assert_eq!(contents, templ_content);

    Ok(())
}

#[test]
#[serial]
fn test_new_from_multiple_files() -> Result<(), Box<dyn Error>> {
    let file1_content = "Template";
    let file2_content = "Other template";
    let _t = Test::init(
        "new_from_multiple_files",
        vec![],
        HashMap::from([
            (PathBuf::from_str("file1")?, file1_content.to_string()),
            (PathBuf::from_str("file2")?, file2_content.to_string()),
        ]),
        "touch",
    );

    let mut cmd = Command::cargo_bin("templaar")?;
    cmd.arg("new")
        .arg("templ")
        .arg("-f")
        .arg("file1")
        .arg("file2");
    cmd.assert().success();

    let templ_path = Path::new(".templ.aar");
    let file1_path = templ_path.join("file1");
    let file2_path = templ_path.join("file2");
    assert!(templ_path.exists());
    assert!(file1_path.exists());
    assert!(file2_path.exists());

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
fn test_new_from_file_exists() -> Result<(), Box<dyn Error>> {
    let _t = Test::init("new_from_file_missing", vec![], HashMap::new(), "touch");

    let mut cmd = Command::cargo_bin("templaar")?;
    cmd.arg("new").arg("-f").arg("templ").arg("templ");
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

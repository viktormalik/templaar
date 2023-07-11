use assert_cmd::Command;
use serial_test::serial;
use std::{
    env,
    error::Error,
    fs,
    io::{Read, Write},
    path::Path,
};

fn set_editor(editor: &str) {
    env::set_var("EDITOR", editor);
}

fn set_default_editor() {
    set_editor("touch");
}

#[test]
#[serial]
fn test_new_default() -> Result<(), Box<dyn Error>> {
    set_default_editor();

    let mut cmd = Command::cargo_bin("templaar")?;
    cmd.arg("new");
    cmd.assert().success();

    let templ_path = Path::new(".templ.aar");
    assert!(templ_path.exists());
    fs::remove_file(templ_path)?;

    Ok(())
}

#[test]
#[serial]
fn test_new_name_from_arg() -> Result<(), Box<dyn Error>> {
    set_default_editor();

    let mut cmd = Command::cargo_bin("templaar")?;
    cmd.arg("new").arg("arg_name");
    cmd.assert().success();

    let templ_path = Path::new(".arg_name.aar");
    assert!(templ_path.exists());
    fs::remove_file(templ_path)?;

    Ok(())
}

#[test]
#[serial]
fn test_new_name_from_stdin() -> Result<(), Box<dyn Error>> {
    set_default_editor();

    let mut cmd = Command::cargo_bin("templaar")?;
    cmd.arg("new").write_stdin("stdin_name");
    cmd.assert().success();

    let templ_path = Path::new(".stdin_name.aar");
    assert!(templ_path.exists());
    fs::remove_file(templ_path)?;

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
    set_default_editor();

    let dir = Path::new("testdir");
    fs::create_dir(dir)?;
    let templ_path = dir.join(".templ.aar");
    fs::File::create(&templ_path)?.write_all(b"Template")?;

    let cwd = env::current_dir()?;
    env::set_current_dir(dir)?;

    let mut cmd = Command::cargo_bin("templaar")?;
    cmd.arg("take");
    cmd.assert().success();

    let file_path = Path::new("templ");
    let mut contents = String::new();
    assert!(file_path.exists());
    fs::File::open(&file_path)?.read_to_string(&mut contents)?;
    assert_eq!(contents, "Template");

    env::set_current_dir(cwd)?;
    fs::remove_dir_all(&dir)?;

    Ok(())
}

#[test]
#[serial]
fn test_take_subdir() -> Result<(), Box<dyn Error>> {
    set_default_editor();

    let templ_dir = Path::new("testdir");
    let file_dir = templ_dir.join("subdir");
    fs::create_dir_all(&file_dir)?;

    let templ_path = templ_dir.join(".templ.aar");
    fs::File::create(&templ_path)?.write_all(b"Template")?;

    let cwd = env::current_dir()?;
    env::set_current_dir(file_dir)?;

    let mut cmd = Command::cargo_bin("templaar")?;
    cmd.arg("take");
    cmd.assert().success();

    let file_path = Path::new("templ");
    let mut contents = String::new();
    assert!(file_path.exists());
    fs::File::open(&file_path)?.read_to_string(&mut contents)?;
    assert_eq!(contents, "Template");

    env::set_current_dir(cwd)?;
    fs::remove_dir_all(&templ_dir)?;

    Ok(())
}

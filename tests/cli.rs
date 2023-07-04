use assert_cmd::Command;
use serial_test::serial;
use std::{env, error::Error, fs, path::Path};

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

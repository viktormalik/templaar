use std::{cmp, env, error};

use crate::utils::{global_dir, path_to_templ, templs_in_dir};

fn longest(templs: &Vec<String>) -> usize {
    templs.iter().map(|s| s.len()).max().unwrap_or(0)
}

fn print(templs: &Vec<String>, suffix: &str, align: usize) {
    for templ in templs {
        println!("{templ:align$} [{suffix}]");
    }
}

pub fn list(only_local: bool, only_global: bool) -> Result<(), Box<dyn error::Error>> {
    let local: Vec<String> = templs_in_dir(&env::current_dir()?)?
        .iter()
        .map(path_to_templ)
        .collect();
    let global: Vec<String> = templs_in_dir(&global_dir()?)?
        .iter()
        .map(path_to_templ)
        .collect();

    let align = cmp::max(
        if !only_local { longest(&global) } else { 0 },
        if !only_global { longest(&local) } else { 0 },
    ) + 1;

    if !only_global {
        print(&local, "local", align);
    }
    if !only_local {
        print(&global, "global", align);
    }
    Ok(())
}

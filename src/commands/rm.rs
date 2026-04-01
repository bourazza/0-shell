use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn current_dir_path() -> Option<PathBuf> {
    env::current_dir().ok().or_else(|| env::var("PWD").ok().map(PathBuf::from))
}

fn targets_current_directory(path: &Path, current_dir: &Path) -> bool {
    fs::canonicalize(path)
        .map(|resolved| current_dir.starts_with(&resolved))
        .unwrap_or(false)
}

pub fn run(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("rm: missing operand".to_string());
    }

    let mut recursive = false;
    let mut force = false;
    let mut targets: Vec<&str> = Vec::new();

    for arg in args {
        if arg.starts_with('-') {
            for ch in arg.chars().skip(1) {
                match ch {
                    'r' | 'R' => recursive = true,
                    'f' => force = true,
                    _ => return Err(format!("rm: invalid option: -{}", ch)),
                }
            }
        } else {
            targets.push(arg.as_str());
        }
    }

    if targets.is_empty() {
        return Err("rm: missing operand".to_string());
    }

    let mut errors = Vec::new();
    let current_dir = current_dir_path();

    for target in targets {
        let path = Path::new(target);

        if !path.exists() {
            if force {
                continue;
            }
            errors.push(format!("rm: {}: No such file or directory", target));
            continue;
        }

        if path.is_dir() {
            if !recursive {
                errors.push(format!("rm: {}: is a directory (use -r to remove)", target));
                continue;
            }
            if current_dir
                .as_ref()
                .is_some_and(|dir| targets_current_directory(path, dir))
            {
                errors.push(format!(
                    "rm: cannot remove '{}': current directory or parent",
                    target
                ));
                continue;
            }
            if let Err(e) = fs::remove_dir_all(path) {
                errors.push(format!("rm: {}: {}", target, e));
            }
        } else {
            if let Err(e) = fs::remove_file(path) {
                errors.push(format!("rm: {}: {}", target, e));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

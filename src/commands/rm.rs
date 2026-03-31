use std::fs;
use std::path::Path;

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

    for target in targets {
        let path = Path::new(target);

        if !path.exists() {
            if force {
                continue;
            }
            return Err(format!("rm: {}: No such file or directory", target));
        }

        if path.is_dir() {
            if !recursive {
                return Err(format!("rm: {}: is a directory (use -r to remove)", target));
            }
            fs::remove_dir_all(path).map_err(|e| format!("rm: {}: {}", target, e))?;
        } else {
            fs::remove_file(path).map_err(|e| format!("rm: {}: {}", target, e))?;
        }
    }

    Ok(())
}

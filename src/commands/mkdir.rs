use std::fs;
use std::path::Path;

pub fn run(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("mkdir: missing operand".to_string());
    }

    let mut parents = false;
    let mut dirs: Vec<&str> = Vec::new();

    for arg in args {
        match arg.as_str() {
            "-p" => parents = true,
            other => dirs.push(other),
        }
    }

    if dirs.is_empty() {
        return Err("mkdir: missing operand".to_string());
    }

    let mut errors = Vec::new();

    for dir in dirs {
        let path = Path::new(dir);
        let result = if parents {
            fs::create_dir_all(path)
        } else {
            fs::create_dir(path)
        };
        if let Err(e) = result {
            errors.push(format!("mkdir: {}: {}", dir, e));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

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

    for dir in dirs {
        let path = Path::new(dir);
        let result = if parents {
            fs::create_dir_all(path)
        } else {
            fs::create_dir(path)
        };
        result.map_err(|e| format!("mkdir: {}: {}", dir, e))?;
    }

    Ok(())
}

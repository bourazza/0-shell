use std::fs;
use std::path::Path;

pub fn run(args: &[String]) -> Result<(), String> {
    if args.len() < 2 {
        return Err("mv: missing destination operand".to_string());
    }

    let dest = Path::new(args.last().unwrap());
    let sources = &args[..args.len() - 1];

    // Moving multiple sources: destination must be a directory
    if sources.len() > 1 && !dest.is_dir() {
        return Err(format!(
            "mv: target '{}' is not a directory",
            dest.display()
        ));
    }

    let mut errors = Vec::new();

    for src_str in sources {
        let src = Path::new(src_str.as_str());

        if !src.exists() {
            errors.push(format!("mv: {}: No such file or directory", src_str));
            continue;
        }

        let actual_dest = if dest.is_dir() {
            let Some(name) = src.file_name() else {
                errors.push(format!("mv: {}: invalid path", src_str));
                continue;
            };
            dest.join(name)
        } else {
            dest.to_path_buf()
        };

        // Try rename first (same filesystem), fallback to copy+delete
        if fs::rename(src, &actual_dest).is_err() {
            if src.is_dir() {
                if let Err(err) = copy_dir_all(src, &actual_dest) {
                    errors.push(err);
                    continue;
                }
                if let Err(e) = fs::remove_dir_all(src) {
                    errors.push(format!("mv: {}: {}", src_str, e));
                }
            } else {
                if let Err(e) = fs::copy(src, &actual_dest) {
                    errors.push(format!("mv: {}: {}", src_str, e));
                    continue;
                }
                if let Err(e) = fs::remove_file(src) {
                    errors.push(format!("mv: {}: {}", src_str, e));
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| format!("mv: {}: {}", dst.display(), e))?;
    for entry in fs::read_dir(src).map_err(|e| format!("mv: {}: {}", src.display(), e))? {
        let entry = entry.map_err(|e| format!("mv: {}", e))?;
        let ty = entry.file_type().map_err(|e| format!("mv: {}", e))?;
        let dest_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dest_path)?;
        } else {
            fs::copy(entry.path(), &dest_path).map_err(|e| format!("mv: {}", e))?;
        }
    }
    Ok(())
}

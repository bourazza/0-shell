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

    for src_str in sources {
        let src = Path::new(src_str.as_str());

        if !src.exists() {
            return Err(format!("mv: {}: No such file or directory", src_str));
        }

        let actual_dest = if dest.is_dir() {
            let name = src
                .file_name()
                .ok_or_else(|| format!("mv: {}: invalid path", src_str))?;
            dest.join(name)
        } else {
            dest.to_path_buf()
        };

        // Try rename first (same filesystem), fallback to copy+delete
        if fs::rename(src, &actual_dest).is_err() {
            if src.is_dir() {
                copy_dir_all(src, &actual_dest)?;
                fs::remove_dir_all(src).map_err(|e| format!("mv: {}: {}", src_str, e))?;
            } else {
                fs::copy(src, &actual_dest).map_err(|e| format!("mv: {}: {}", src_str, e))?;
                fs::remove_file(src).map_err(|e| format!("mv: {}: {}", src_str, e))?;
            }
        }
    }

    Ok(())
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

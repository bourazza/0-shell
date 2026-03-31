use std::fs;
use std::path::Path;

pub fn run(args: &[String]) -> Result<(), String> {
    if args.len() < 2 {
        return Err("cp: missing destination operand".to_string());
    }

    let mut recursive = false;
    let mut file_args: Vec<&str> = Vec::new();

    for arg in args {
        if arg.starts_with('-') {
            for ch in arg.chars().skip(1) {
                match ch {
                    'r' | 'R' => recursive = true,
                    _ => return Err(format!("cp: invalid option: -{}", ch)),
                }
            }
        } else {
            file_args.push(arg.as_str());
        }
    }

    if file_args.len() < 2 {
        return Err("cp: missing destination operand".to_string());
    }

    let dest = Path::new(file_args.last().unwrap());
    let sources = &file_args[..file_args.len() - 1];

    if sources.len() > 1 && !dest.is_dir() {
        return Err(format!(
            "cp: target '{}' is not a directory",
            dest.display()
        ));
    }

    for src_str in sources {
        let src = Path::new(src_str);

        if !src.exists() {
            return Err(format!("cp: {}: No such file or directory", src_str));
        }

        let actual_dest = if dest.is_dir() {
            let name = src
                .file_name()
                .ok_or_else(|| format!("cp: {}: invalid path", src_str))?;
            dest.join(name)
        } else {
            dest.to_path_buf()
        };

        if src.is_dir() {
            if !recursive {
                return Err(format!("cp: {}: is a directory (use -r to copy)", src_str));
            }
            copy_dir_all(src, &actual_dest)?;
        } else {
            fs::copy(src, &actual_dest).map_err(|e| format!("cp: {}: {}", src_str, e))?;
        }
    }

    Ok(())
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| format!("cp: {}: {}", dst.display(), e))?;
    for entry in fs::read_dir(src).map_err(|e| format!("cp: {}: {}", src.display(), e))? {
        let entry = entry.map_err(|e| format!("cp: {}", e))?;
        let ty = entry.file_type().map_err(|e| format!("cp: {}", e))?;
        let dest_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dest_path)?;
        } else {
            fs::copy(entry.path(), &dest_path).map_err(|e| format!("cp: {}", e))?;
        }
    }
    Ok(())
}

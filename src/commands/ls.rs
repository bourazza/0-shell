use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

struct LsOptions {
    long: bool,
    all: bool,
    classify: bool,
}

fn parse_flags(args: &[String]) -> (LsOptions, Vec<PathBuf>) {
    let mut opts = LsOptions { long: false, all: false, classify: false };
    let mut paths = Vec::new();

    for arg in args {
        if arg.starts_with('-') {
            for ch in arg.chars().skip(1) {
                match ch {
                    'l' => opts.long = true,
                    'a' => opts.all = true,
                    'F' => opts.classify = true,
                    _ => {}
                }
            }
        } else {
            paths.push(PathBuf::from(arg));
        }
    }

    if paths.is_empty() {
        paths.push(PathBuf::from("."));
    }

    (opts, paths)
}

fn format_permissions(mode: u32) -> String {
    let file_type = match mode & 0o170000 {
        0o040000 => 'd',
        0o120000 => 'l',
        _ => '-',
    };
    let bits = [
        (0o400, 'r'), (0o200, 'w'), (0o100, 'x'),
        (0o040, 'r'), (0o020, 'w'), (0o010, 'x'),
        (0o004, 'r'), (0o002, 'w'), (0o001, 'x'),
    ];
    let perms: String = bits.iter().map(|(mask, ch)| {
        if mode & mask != 0 { *ch } else { '-' }
    }).collect();
    format!("{}{}", file_type, perms)
}

fn colorize_entry(name: &str, meta: &fs::Metadata, classify: bool) -> String {
    let is_dir = meta.is_dir();
    let is_symlink = meta.file_type().is_symlink();
    let is_exec = meta.permissions().mode() & 0o111 != 0;

    let colored = if is_symlink {
        format!("\x1b[36m{}\x1b[0m", name)       // cyan for symlinks
    } else if is_dir {
        format!("\x1b[1;34m{}\x1b[0m", name)     // bold blue for dirs
    } else if is_exec {
        format!("\x1b[1;32m{}\x1b[0m", name)     // bold green for executables
    } else {
        name.to_string()
    };

    if classify {
        let suffix = if is_dir { "/" } else if is_symlink { "@" } else if is_exec { "*" } else { "" };
        format!("{}{}", colored, suffix)
    } else {
        colored
    }
}

fn list_dir(path: &Path, opts: &LsOptions, show_header: bool) -> Result<(), String> {
    if show_header {
        println!("{}:", path.display());
    }

    let mut entries: Vec<_> = fs::read_dir(path)
        .map_err(|e| format!("ls: {}: {}", path.display(), e))?
        .filter_map(|e| e.ok())
        .collect();

    entries.sort_by_key(|e| e.file_name());

    let mut all_entries: Vec<(String, fs::Metadata)> = Vec::new();

    // Add . and .. if -a
    if opts.all {
        let dot_meta = fs::metadata(path).ok();
        let dotdot_meta = fs::metadata(path.parent().unwrap_or(path)).ok();
        if let Some(m) = dot_meta {
            all_entries.push((".".to_string(), m));
        }
        if let Some(m) = dotdot_meta {
            all_entries.push(("..".to_string(), m));
        }
    }

    for entry in &entries {
        let name = entry.file_name().to_string_lossy().to_string();
        if !opts.all && name.starts_with('.') {
            continue;
        }
        let meta = entry.metadata().map_err(|e| format!("ls: {}", e))?;
        all_entries.push((name, meta));
    }

    if opts.long {
        // Calculate total blocks
        let total: u64 = all_entries.iter().map(|(_, m)| m.blocks()).sum::<u64>() / 2;
        println!("total {}", total);

        for (name, meta) in &all_entries {
            let mode = meta.permissions().mode();
            let nlink = meta.nlink();
            let uid = meta.uid();
            let gid = meta.gid();
            let size = meta.len();
            let mtime = meta.modified().ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);

            // Format time
            let time_str = format_mtime(mtime);

            let perm_str = format_permissions(mode);
            let colored = colorize_entry(name, meta, opts.classify);

            println!(
                "{} {:>3} {:>5} {:>5} {:>8} {} {}",
                perm_str, nlink, uid, gid, size, time_str, colored
            );
        }
    } else {
        // Short listing: names separated by spaces
        let names: Vec<String> = all_entries.iter()
            .map(|(name, meta)| colorize_entry(name, meta, opts.classify))
            .collect();
        println!("{}", names.join("  "));
    }

    Ok(())
}

fn format_mtime(secs: u64) -> String {
    // Simple Unix timestamp to "Mon DD HH:MM" format
    // Using a simple calculation without external crates
    let months = ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"];
    
    let secs_per_min = 60u64;
    let secs_per_hour = 3600u64;
    let secs_per_day = 86400u64;
    let secs_per_year = 365 * secs_per_day;

    let mut remaining = secs;
    let years_since_1970 = remaining / secs_per_year;
    remaining %= secs_per_year;

    let month_days = [31u64,28,31,30,31,30,31,31,30,31,30,31];
    let mut month = 0usize;
    for (i, &days) in month_days.iter().enumerate() {
        if remaining < days * secs_per_day {
            month = i;
            break;
        }
        remaining -= days * secs_per_day;
    }

    let day = remaining / secs_per_day + 1;
    remaining %= secs_per_day;
    let hour = remaining / secs_per_hour;
    remaining %= secs_per_hour;
    let min = remaining / secs_per_min;

    let _ = years_since_1970; // suppress warning
    format!("{} {:>2} {:02}:{:02}", months[month], day, hour, min)
}

pub fn run(args: &[String]) -> Result<(), String> {
    let (opts, paths) = parse_flags(args);
    let multiple = paths.len() > 1;

    for (i, path) in paths.iter().enumerate() {
        if i > 0 {
            println!();
        }
        let meta = fs::metadata(path).map_err(|e| format!("ls: {}: {}", path.display(), e))?;
        if meta.is_dir() {
            list_dir(path, &opts, multiple)?;
        } else {
            // Single file: just print its name
            let name = path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.display().to_string());
            let colored = colorize_entry(&name, &meta, opts.classify);
            if opts.long {
                let mode = meta.permissions().mode();
                let nlink = meta.nlink();
                let uid = meta.uid();
                let gid = meta.gid();
                let size = meta.len();
                let mtime = meta.modified().ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let time_str = format_mtime(mtime);
                println!("{} {:>3} {:>5} {:>5} {:>8} {} {}",
                    format_permissions(mode), nlink, uid, gid, size, time_str, colored);
            } else {
                println!("{}", colored);
            }
        }
    }

    Ok(())
}
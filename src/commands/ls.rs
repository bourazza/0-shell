use chrono::{DateTime, Local};
use nix::unistd::{Gid, Group, Uid, User};
use std::fs;
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

struct LsOptions {
    long: bool,
    all: bool,
    classify: bool,
}

fn parse_flags(args: &[String]) -> Result<(LsOptions, Vec<PathBuf>), String> {
    let mut opts = LsOptions {
        long: false,
        all: false,
        classify: false,
    };
    let mut paths = Vec::new();

    for arg in args {
        if arg.starts_with('-') {
            for ch in arg.chars().skip(1) {
                match ch {
                    'l' => opts.long = true,
                    'a' => opts.all = true,
                    'F' => opts.classify = true,
                    _ => return Err(format!("ls: invalid option -- {}", ch)),
                }
            }
        } else {
            paths.push(PathBuf::from(arg));
        }
    }

    if paths.is_empty() {
        paths.push(PathBuf::from("."));
    }

    Ok((opts, paths))
}

fn format_permissions(meta: &fs::Metadata) -> String {
    let ft = meta.file_type();
    let file_type_char = if ft.is_symlink() {
        'l'
    } else if ft.is_dir() {
        'd'
    } else if ft.is_fifo() {
        'p'
    } else if ft.is_socket() {
        's'
    } else if ft.is_block_device() {
        'b'
    } else if ft.is_char_device() {
        'c'
    } else {
        '-'
    };
    let mode = meta.permissions().mode();

    // rwx with setuid/setgid/sticky handling
    let mut chars = Vec::with_capacity(9);

    // user
    chars.push(if mode & 0o400 != 0 { 'r' } else { '-' });
    chars.push(if mode & 0o200 != 0 { 'w' } else { '-' });
    let ux = mode & 0o100 != 0;
    if mode & 0o4000 != 0 {
        chars.push(if ux { 's' } else { 'S' });
    } else {
        chars.push(if ux { 'x' } else { '-' });
    }

    // group
    chars.push(if mode & 0o040 != 0 { 'r' } else { '-' });
    chars.push(if mode & 0o020 != 0 { 'w' } else { '-' });
    let gx = mode & 0o010 != 0;
    if mode & 0o2000 != 0 {
        chars.push(if gx { 's' } else { 'S' });
    } else {
        chars.push(if gx { 'x' } else { '-' });
    }

    // others
    chars.push(if mode & 0o004 != 0 { 'r' } else { '-' });
    chars.push(if mode & 0o002 != 0 { 'w' } else { '-' });
    let ox = mode & 0o001 != 0;
    if mode & 0o1000 != 0 {
        chars.push(if ox { 't' } else { 'T' });
    } else {
        chars.push(if ox { 'x' } else { '-' });
    }

    format!(
        "{}{}",
        file_type_char,
        chars.into_iter().collect::<String>()
    )
}

fn format_size_or_dev(meta: &fs::Metadata) -> String {
    let ft = meta.file_type();
    if ft.is_char_device() || ft.is_block_device() {
        let rdev = meta.rdev();
        let major = (rdev >> 8) & 0xfff;
        let minor = (rdev & 0xff) | ((rdev >> 12) & 0xfff00);
        format!("{:>3},{:>4}", major, minor)
    } else {
        format!("{:>8}", meta.len())
    }
}

fn decorate_entry(name: &str, meta: &fs::Metadata, classify: bool) -> String {
    if !classify {
        return name.to_string();
    }
    let is_dir = meta.is_dir();
    let is_exec = meta.permissions().mode() & 0o111 != 0;
    let suffix = if is_dir {
        "/"
    } else if is_exec {
        "*"
    } else {
        ""
    };
    format!("{}{}", name, suffix)
}

fn escape_leading_special(name: &str) -> String {
    if let Some(first) = name.chars().next() {
        if !first.is_alphanumeric() && first != '.' {
            return format!("\\{}", name);
        }
    }
    name.to_string()
}

fn lookup_user(uid: u32) -> String {
    let uid = Uid::from_raw(uid);
    User::from_uid(uid)
        .ok()
        .flatten()
        .map(|u| u.name)
        .unwrap_or_else(|| uid.as_raw().to_string())
}

fn lookup_group(gid: u32) -> String {
    let gid = Gid::from_raw(gid);
    Group::from_gid(gid)
        .ok()
        .flatten()
        .map(|g| g.name)
        .unwrap_or_else(|| gid.as_raw().to_string())
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

    let mut all_entries: Vec<(String, fs::Metadata, Option<PathBuf>)> = Vec::new();

    // Add . and .. if -a
    if opts.all {
        let dot_meta = fs::metadata(path).ok();
        // Try parent via ".."; if that fails, fall back to current dir metadata so ".." still shows.
        let dotdot_meta = fs::metadata(path.join(".."))
            .ok()
            .or_else(|| dot_meta.clone());
        if let Some(m) = dot_meta {
            all_entries.push((".".to_string(), m, None));
        }
        if let Some(m) = dotdot_meta {
            all_entries.push(("..".to_string(), m, None));
        }
    }

    for entry in &entries {
        let name = entry.file_name().to_string_lossy().to_string();
        if !opts.all && name.starts_with('.') {
            continue;
        }
        let meta = fs::symlink_metadata(entry.path()).map_err(|e| format!("ls: {}", e))?;
        let target = if meta.file_type().is_symlink() {
            fs::read_link(entry.path()).ok()
        } else {
            None
        };
        all_entries.push((name, meta, target));
    }

    if opts.long {
        // Calculate total blocks
        let total: u64 = all_entries.iter().map(|(_, m, _)| m.blocks()).sum::<u64>() / 2;
        println!("total {}", total);

        for (name, meta, target) in &all_entries {
            let nlink = meta.nlink();
            let user = lookup_user(meta.uid());
            let group = lookup_group(meta.gid());
            let size_or_dev = format_size_or_dev(meta);
            let mtime = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);

            // Format time
            let time_str = format_mtime(mtime);

            let perm_str = format_permissions(meta);
            let base_name = escape_leading_special(name);
            let display_name = if meta.file_type().is_symlink() {
                if let Some(t) = target {
                    format!("{} -> {}", base_name, t.display())
                } else {
                    format!("{} -> ?", base_name)
                }
            } else {
                base_name
            };
            let decorated = decorate_entry(&display_name, meta, opts.classify);

            println!(
                "{} {:>3} {:<8} {:<8} {} {} {}",
                perm_str, nlink, user, group, size_or_dev, time_str, decorated
            );
        }
    } else {
        // Short listing: names separated by spaces
        let names: Vec<String> = all_entries
            .iter()
            .map(|(name, meta, target)| {
                let base_name = escape_leading_special(name);
                let display_name = if meta.file_type().is_symlink() && opts.classify {
                    if let Some(t) = target {
                        format!("{} -> {}", base_name, t.display())
                    } else {
                        format!("{} -> ?", base_name)
                    }
                } else {
                    base_name
                };
                decorate_entry(&display_name, meta, opts.classify)
            })
            .collect();
        println!("{}", names.join("  "));
    }

    Ok(())
}

fn format_mtime(time: SystemTime) -> String {
    let dt: DateTime<Local> = time.into();
    dt.format("%b %e %H:%M").to_string()
}

pub fn run(args: &[String]) -> Result<(), String> {
    let (opts, paths) = parse_flags(args)?;
    let multiple = paths.len() > 1;

    for (i, path) in paths.iter().enumerate() {
        if i > 0 {
            println!();
        }
        let meta =
            fs::symlink_metadata(path).map_err(|e| format!("ls: {}: {}", path.display(), e))?;
        let target = if meta.file_type().is_symlink() {
            fs::read_link(path).ok()
        } else {
            None
        };
        if meta.is_dir() {
            list_dir(path, &opts, multiple)?;
        } else {
            // Single file: just print its name
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.display().to_string());
            let base_name = escape_leading_special(&name);
            let display_name = if meta.file_type().is_symlink() {
                if let Some(t) = target {
                    format!("{} -> {}", base_name, t.display())
                } else {
                    format!("{} -> ?", base_name)
                }
            } else {
                base_name
            };
            let decorated = decorate_entry(&display_name, &meta, opts.classify);
            if opts.long {
                let nlink = meta.nlink();
                let user = lookup_user(meta.uid());
                let group = lookup_group(meta.gid());
                let size_or_dev = format_size_or_dev(&meta);
                let mtime = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                let time_str = format_mtime(mtime);
                println!(
                    "{} {:>3} {:<8} {:<8} {} {} {}",
                    format_permissions(&meta),
                    nlink,
                    user,
                    group,
                    size_or_dev,
                    time_str,
                    decorated
                );
            } else {
                println!("{}", decorated);
            }
        }
    }

    Ok(())
}

use std::env;
use std::path::Path;

pub fn run(args: &[String]) -> Result<(), String> {
    let current_dir = env::current_dir().map_err(|e| format!("cd: {}", e))?;

    let target = match args.first() {
        Some(dir) if dir == "-" => {
            env::var("OLDPWD").map_err(|_| "cd: OLDPWD not set".to_string())?
        }
        Some(dir) => {
            // Support ~ expansion
            if dir == "~" || dir.starts_with("~/") {
                let home = env::var("HOME").unwrap_or_else(|_| "/".to_string());
                if dir == "~" {
                    home
                } else {
                    format!("{}{}", home, &dir[1..])
                }
            } else {
                dir.clone()
            }
        }
        None => {
            // cd with no args goes to HOME
            env::var("HOME").unwrap_or_else(|_| "/".to_string())
        }
    };

    let path = Path::new(&target);
    env::set_current_dir(path).map_err(|e| format!("cd: {}: {}", target, e))?;

    // update OLDPWD to previous directory
    env::set_var("OLDPWD", current_dir);

    // When using "cd -", print the new directory like POSIX shells
    if args.first().map(|s| s.as_str()) == Some("-") {
        if let Ok(new_dir) = env::current_dir() {
            println!("{}", new_dir.display());
        }
    }

    Ok(())
}

use std::env;
use std::path::Path;

pub fn run(args: &[String]) -> Result<(), String> {
    let current_dir = env::var("PWD").or_else(|_| {
        env::current_dir()
            .map(|path| path.display().to_string())
            .map_err(|_| env::VarError::NotPresent)
    });
    let current_dir = current_dir.map_err(|e| format!("cd: {}", e))?;

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
    env::set_var(
        "PWD",
        env::current_dir()
            .map(|path| path.display().to_string())
            .unwrap_or(target.clone()),
    );

    // When using "cd -", print the new directory like POSIX shells
    if args.first().map(|s| s.as_str()) == Some("-") {
        if let Ok(new_dir) = env::var("PWD") {
            println!("{}", new_dir);
        }
    }

    Ok(())
}

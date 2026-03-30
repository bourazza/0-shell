use std::env;
use std::path::Path;

pub fn run(args: &[String]) -> Result<(), String> {
    let target = match args.first() {
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
    env::set_current_dir(path).map_err(|e| format!("cd: {}: {}", target, e))
}
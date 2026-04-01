use std::env;

pub fn run() -> Result<(), String> {
    let path = env::var("PWD")
        .or_else(|_| {
            env::current_dir()
                .map(|path| path.display().to_string())
                .map_err(|_| env::VarError::NotPresent)
        })
        .map_err(|e| format!("pwd: {}", e))?;
    println!("{}", path);
    Ok(())
}

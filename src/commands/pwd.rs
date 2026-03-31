use std::env;

pub fn run() -> Result<(), String> {
    let path = env::current_dir().map_err(|e| format!("pwd: {}", e))?;
    println!("{}", path.display());
    Ok(())
}

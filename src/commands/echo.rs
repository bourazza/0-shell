pub fn run(args: &[String]) -> Result<(), String> {
    // Handle -n flag (no trailing newline)
    if args.first().map(|s| s.as_str()) == Some("-n") {
        print!("{}", args[1..].join(" "));
        use std::io::Write;
        std::io::stdout().flush().ok();
    } else {
        println!("{}", args.join(" "));
    }
    Ok(())
}

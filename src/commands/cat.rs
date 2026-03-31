use std::fs;
use std::io::{self, Read, Write};

pub fn run(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        // Read from stdin
        let mut buf = String::new();
        io::stdin()
            .read_to_string(&mut buf)
            .map_err(|e| format!("cat: {}", e))?;
        print!("{}", buf);
        io::stdout().flush().ok();
        return Ok(());
    }

    let mut number_lines = false;
    let mut files: Vec<&str> = Vec::new();

    for arg in args {
        match arg.as_str() {
            "-n" => number_lines = true,
            other => files.push(other),
        }
    }

    for filename in files {
        if filename == "-" {
            let mut buf = String::new();
            io::stdin()
                .read_to_string(&mut buf)
                .map_err(|e| format!("cat: {}", e))?;
            if number_lines {
                for (i, line) in buf.lines().enumerate() {
                    println!("{:>6}\t{}", i + 1, line);
                }
            } else {
                print!("{}", buf);
            }
            continue;
        }

        let contents =
            fs::read_to_string(filename).map_err(|e| format!("cat: {}: {}", filename, e))?;

        if number_lines {
            for (i, line) in contents.lines().enumerate() {
                println!("{:>6}\t{}", i + 1, line);
            }
        } else {
            print!("{}", contents);
        }
    }

    io::stdout().flush().ok();
    Ok(())
}

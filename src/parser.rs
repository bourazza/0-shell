#[derive(Debug)]
pub enum Command {
    Cat,
    Cd,
    Cp,
    Echo,
    Exit,
    Help,
    Ls,
    Mkdir,
    Mv,
    Pwd,
    Rm,
    Unknown(String),
}

pub fn parsing(input: &str) -> Result<(Command, Vec<String>), String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_double_quotes = false;
    let mut in_single_quotes = false;
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '"' if !in_single_quotes => {
                in_double_quotes = !in_double_quotes;
            }
            '\'' if !in_double_quotes => {
                in_single_quotes = !in_single_quotes;
            }
            '\\' if in_double_quotes => {
                // Handle escape sequences inside double quotes
                if let Some(next) = chars.next() {
                    match next {
                        '"' | '\\' | '$' | '\n' => current.push(next),
                        _ => {
                            current.push('\\');
                            current.push(next);
                        }
                    }
                }
            }
            ' ' | '\t' if !in_double_quotes && !in_single_quotes => {
                if !current.is_empty() {
                    args.push(current.clone());
                    current.clear();
                }
            }
            _ => current.push(c),
        }
    }

    if !current.is_empty() {
        args.push(current);
    }

    if in_double_quotes {
        return Err("Unclosed double quote".to_string());
    }
    if in_single_quotes {
        return Err("Unclosed single quote".to_string());
    }

    if args.is_empty() {
        return Err("Empty input".to_string());
    }

    let command_str = args.remove(0);
    let command_args = args;

    let command = match command_str.as_str() {
        "cat" => Command::Cat,
        "cd" => Command::Cd,
        "cp" => Command::Cp,
        "echo" => Command::Echo,
        "exit" => Command::Exit,
        "help" => Command::Help,
        "ls" => Command::Ls,
        "mkdir" => Command::Mkdir,
        "mv" => Command::Mv,
        "pwd" => Command::Pwd,
        "rm" => Command::Rm,
        other => Command::Unknown(other.to_string()),
    };

    Ok((command, command_args))
}

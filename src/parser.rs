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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContinuationState {
    DoubleQuote,
    SingleQuote,
    Backslash,
}

pub fn continuation_state(input: &str) -> Option<ContinuationState> {
    let mut in_double_quotes = false;
    let mut in_single_quotes = false;
    let mut chars = input.chars().peekable();
    let mut trailing_backslash = false;

    while let Some(c) = chars.next() {
        match c {
            '"' if !in_single_quotes => {
                in_double_quotes = !in_double_quotes;
                trailing_backslash = false;
            }
            '\'' if !in_double_quotes => {
                in_single_quotes = !in_single_quotes;
                trailing_backslash = false;
            }
            '\\' if !in_single_quotes => {
                if in_double_quotes {
                    match chars.peek().copied() {
                        Some('"') | Some('\\') | Some('$') | Some('\n') => {
                            trailing_backslash = false;
                            chars.next();
                        }
                        Some(_) => trailing_backslash = false,
                        None => trailing_backslash = true,
                    }
                } else {
                    if chars.peek().is_some() {
                        trailing_backslash = false;
                        chars.next();
                    } else {
                        trailing_backslash = true;
                    }
                }
            }
            _ => trailing_backslash = false,
        }
    }

    if in_double_quotes {
        Some(ContinuationState::DoubleQuote)
    } else if in_single_quotes {
        Some(ContinuationState::SingleQuote)
    } else if trailing_backslash {
        Some(ContinuationState::Backslash)
    } else {
        None
    }
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
            '\\' if !in_single_quotes => {
                if let Some(next) = chars.next() {
                    if in_double_quotes {
                        match next {
                            '"' | '\\' | '$' | '\n' => current.push(next),
                            _ => {
                                current.push('\\');
                                current.push(next);
                            }
                        }
                    } else {
                        current.push(next);
                    }
                } else {
                    current.push('\\');
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

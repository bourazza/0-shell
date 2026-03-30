pub fn run() -> Result<(), String> {
    println!("\x1b[1;33m0-shell\x1b[0m — Built-in Commands\n");

    let commands = [
        ("echo [-n] [args...]",   "Print text to stdout. -n suppresses trailing newline."),
        ("cd [dir]",              "Change directory. No arg = $HOME. Supports ~."),
        ("ls [-l] [-a] [-F] [path...]", "List directory contents. -l long, -a all, -F classify."),
        ("pwd",                   "Print current working directory."),
        ("cat [-n] [file...]",    "Concatenate and print files. -n shows line numbers."),
        ("cp [-r] src... dest",   "Copy files or directories. -r for recursive."),
        ("mv src... dest",        "Move/rename files or directories."),
        ("rm [-r] [-f] file...",  "Remove files or directories. -r recursive, -f force."),
        ("mkdir [-p] dir...",     "Create directories. -p creates parent dirs as needed."),
        ("exit",                  "Exit the shell."),
        ("help",                  "Show this help message."),
    ];

    for (cmd, desc) in &commands {
        println!("  \x1b[1;32m{:<38}\x1b[0m {}", cmd, desc);
    }

    println!("\n\x1b[1mBonus features:\x1b[0m");
    println!("  Command chaining with \x1b[1m;\x1b[0m  (e.g. \x1b[36mmkdir foo; cd foo; pwd\x1b[0m)");
    println!("  Prompt shows current directory");
    println!("  Colorized output for directories, executables, symlinks");
    println!("  ~ expansion in cd");

    Ok(())
}
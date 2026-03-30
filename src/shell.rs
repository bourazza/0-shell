use crate::parser::Command;
use crate::commands;

pub struct Shell {
    // Future: history, env vars, etc.
}

impl Shell {
    pub fn new() -> Self {
        Shell {}
    }

    pub fn execute(&mut self, cmd: Command, args: Vec<String>) -> Result<(), String> {
        match cmd {
            Command::Echo  => commands::echo::run(&args),
            Command::Cd    => commands::cd::run(&args),
            Command::Ls    => commands::ls::run(&args),
            Command::Pwd   => commands::pwd::run(),
            Command::Cat   => commands::cat::run(&args),
            Command::Cp    => commands::cp::run(&args),
            Command::Rm    => commands::rm::run(&args),
            Command::Mv    => commands::mv::run(&args),
            Command::Mkdir => commands::mkdir::run(&args),
            Command::Help  => commands::help::run(),
            Command::Exit | Command::Unknown(_) => Ok(()), // handled in main
        }
    }
}
pub enum Command {
    Cat,
    Cd,
    Echo,
    Exit,
    Ls,
    Mkdir,
    Mv,
    Pwd,
    Rm,
    Unknown,
}



pub fn parsing(ioinput : &str) -> (String, Vec<String>){
    let prs: Vec<String> =ioinput.split_whitespace().map(|s| s.to_string()).collect();
    let command = prs[0].clone();
    let argument = prs[1..].to_vec();
    let cmd = match command.as_str() {
    "cat" => Command::Cat,
    "cd" => Command::Cd,
    "echo" => Command::Echo,
    "exit" => Command::Exit,
    "ls" => Command::Ls,
    "mkdir" => Command::Mkdir,
    "mv" => Command::Mv,
    "pwd" => Command::Pwd,
    "rm" => Command::Rm,
    _ => Command::Unknown,
};
    (command,argument) 


}
mod utils;
mod parser;
use utils::*;
use parser::*;
use std::env;
use std::io::{self, Write};
fn main() {
    welcom::welcom();
   loop {
    // get current path
    let path = env::current_dir().unwrap();
    print!("{} $ ", path.display());
    io::stdout().flush().unwrap();

    let mut input = String::new();
    let bytes_read = io::stdin().read_line(&mut input).unwrap();

    if bytes_read == 0 {
        println!("\nExiting shell. Bye!");
        break;
    }

    let input = input.trim();
    if input.is_empty() {
        continue;
    }

    let (o,g) = parser::parsing(&input);
    println!("{},{:?}", o,g);
}
}
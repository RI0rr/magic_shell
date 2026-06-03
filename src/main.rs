use std::{env, io::{self, Write}};

use nix::unistd::{ROOT, Uid, gethostname};

use crate::{builtin::{execute, is_builtin}, process::execute_line};

pub mod builtin;
pub mod parser;
pub mod process;



fn main() {
    // User
    let Ok(user) = env::var("USER") else { return; };

    // Host
    let Ok(host) = gethostname() else { return; };
    let Ok(host) = host.into_string() else { return; };

    

    // uid
    let euid = Uid::effective();
    let shell_char = if euid == ROOT { '#' } else { '$' };

    loop {
        let curr_dir = env::current_dir().unwrap();
        print!("{user}@{host}:{}{shell_char} ", curr_dir.display());
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let parsed_input = match parser::parse(&input) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("{e}");
                continue;
            } 
        };

        match execute_line(parsed_input) {
            Ok(_) => {},
            Err(e) => eprintln!("{e}"),
        }
    }
}

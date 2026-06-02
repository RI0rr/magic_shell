use std::{env, io::{self, Write}, path::PathBuf};

use crate::builtin::{execute, is_builtin};

pub mod builtin;
pub mod parser;
pub mod process;



fn main() {
    // User
    let Ok(user) = env::var("USER") else { return; };

    // Host
    let Ok(host) = hostname::get() else { return; };
    let Ok(host) = host.into_string() else { return; };

    

    // uid
    let euid = unsafe { libc::geteuid() };
    let shell_char = if euid == 0 { '#' } else { '$' };

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
        
        let cmd = parsed_input.cmd();
        let args = parsed_input.args();
        if is_builtin(cmd) {
            match execute(cmd, args) {
                Ok(_) => {},
                Err(e) => eprintln!("{e}"),
            }
        } else {
            // Spawn child process 
            match process::spawn(parsed_input) {
                Ok(_) => {},
                Err(e) => eprintln!("{e}"),
            }
        }

        

    }
}

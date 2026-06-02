use std::{env, path::PathBuf};
use crate::builtin::Error::NoHomeDir;
use thiserror::Error;

static BUILTINS: &[&str] = &["cd", "exit", "export", "source", "echo"];

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error")]
    IOError,
    #[error("Unknown command: {0}")]
    UnknownCommand(String),
    #[error("No home dir")]
    NoHomeDir,
    #[error("Too many arguments")]
    TooManyArguments,
    #[error("Invalid exit code: {0}")]
    InvalidExitCode(String)
}

pub fn is_builtin(cmd: &str) -> bool {
    BUILTINS.contains(&cmd)
}

pub fn execute(cmd: &str, args: &[&str]) -> Result<(), Error> {
    match cmd {
        "cd" => builtin_cd(&args[1..]),
        "exit" => builtin_exit(&args[1..]),
        _ => Err(Error::UnknownCommand(cmd.to_string())),
    }
}

fn builtin_cd(args: &[&str]) -> Result<(), Error> {
    match args.len() {
        0 => {
            let home = env::var("HOME").map_err(|_| Error::NoHomeDir)?;
            env::set_current_dir(home).map_err(|_| Error::IOError)
        }
        1 => env::set_current_dir(args[0]).map_err(|_| Error::IOError),
        _ => Err(Error::TooManyArguments),
    }
}

fn builtin_exit(args: &[&str]) -> Result<(), Error> {
    match args.len() {
        0 => std::process::exit(0),
        1 => {
            let code = args[0]
                .parse()
                .map_err(|_| Error::InvalidExitCode(args[0].to_owned()))?;
            std::process::exit(code)

        }
        _ => Err(Error::TooManyArguments)
    }
}

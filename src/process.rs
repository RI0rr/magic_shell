use std::ffi::CString;

use libc::_exit;
use nix::{errno::Errno, sys::wait::waitpid, unistd::{ForkResult, execvp, fork}};
use thiserror::Error;

use crate::parser::ParsedCommand;


#[derive(Debug, Error)]
pub enum Error {
    #[error("Fork failed: {0}")]
    ForkFailed(Errno),
    #[error("Command {0} does not exist")]
    CommandDoesNotExist(String),
    #[error("Unknown error: {0}")]
    Unknown(Errno),
    #[error("Child error")]
    ChildError,
}

pub fn spawn(parsed_command: ParsedCommand) -> Result<(), Error> {
    match unsafe{fork()} {
        Ok(ForkResult::Parent { child, .. }) => {
            
            match waitpid(child, None) {
                Ok(status) => Ok(()),
                Err(e) => Err(Error::Unknown(e)),
            }
        }
        Ok(ForkResult::Child) => {
            let cmd = CString::new(parsed_command.cmd()).unwrap();
            let args: Vec<CString> = parsed_command.args()
                .iter()
                .map(|&s| CString::new(s).unwrap())
                .collect();
            let _ = execvp(&cmd, &args);
            unsafe { _exit(1); }
        }
        Err(e) => Err(Error::ForkFailed(e)),
    }
}
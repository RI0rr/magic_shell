use std::{ffi::CString, os::fd::{AsFd, AsRawFd, OwnedFd, RawFd}};

use libc::{_exit, putchar};
use nix::{errno::Errno, sys::wait::waitpid, unistd::{ForkResult, dup2_stdin, dup2_stdout, execvp, fork, pipe}};
use thiserror::Error;

use crate::parser::{ParsedCommand, ParsedInput, PipelineItem};


#[derive(Debug, Error)]
pub enum Error {
    #[error("Fork failed: {0}")]
    ForkFailed(Errno),
    #[error("Command {0} does not exist")]
    CommandDoesNotExist(String),
    #[error("Unknown error: {0}")]
    Unknown(Errno),
    #[error("dup2 error: {0}")]
    Dup2(Errno),
    #[error("Child error")]
    ChildError,
}

struct Pipe {
    read: Option<OwnedFd>,
    write: Option<OwnedFd>,
}

impl Pipe {
    fn new(pipe: (OwnedFd, OwnedFd)) -> Self {
        Self { read: Some(pipe.0), write: Some(pipe.1) }
    }
}

pub fn execute_line(input: ParsedInput<'_>) -> Result<(), Box<dyn std::error::Error>>{
    // First create and connect pipes then call processes
    let pipes= input
        .iter()
        .filter(|&item| *item == PipelineItem::Separator(crate::parser::ProcessSeparator::Pipe))
        .map(|_| pipe())
        .collect::<Result<Vec<_>, _>>()?;

    let mut pipes: Vec<_> = pipes
        .into_iter()
        .map(Pipe::new)
        .collect();

    let commands: Vec<ParsedCommand> = input
        .into_iter()
        .filter_map(|item| match item {
            PipelineItem::Command(cmd) => Some(cmd),
            _ => None,
        })
        .collect();

    for (i, cmd) in commands.iter().enumerate() {
        let pipein = if i == 0 { None } else { pipes[i-1].read.take() };
        let pipeout = if i == commands.len()-1 { None } else { pipes[i].write.take() };
        spawn(cmd, pipein, pipeout)?;
    }

    Ok(())
}

pub fn spawn(parsed_command: &ParsedCommand, pipein: Option<OwnedFd>, pipeout: Option<OwnedFd>) -> Result<(), Error> {
    match unsafe{fork()} {
        Ok(ForkResult::Parent { child, .. }) => { 
            match waitpid(child, None) {
                Ok(status) => Ok(()),
                Err(e) => Err(Error::Unknown(e)),
            }
        }
        
        Ok(ForkResult::Child) => {
            if let Some(fd) = pipein {
                dup2_stdin(fd).map_err(Error::Dup2)?;
            }
            if let Some(fd) = pipeout {
                dup2_stdout(fd).map_err(Error::Dup2)?;
            }
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
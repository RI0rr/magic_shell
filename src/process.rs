use std::{ffi::CString, os::fd::{OwnedFd}};

use libc::{_exit,};
use nix::{errno::Errno, sys::wait::waitpid, unistd::{ForkResult, Pid, dup2_stdin, dup2_stdout, execvp, fork, pipe}};
use thiserror::Error;

use crate::{builtin::{execute, is_builtin}, parser::{ParsedCommand, ParsedInput, PipelineItem}};


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

pub fn execute_line(input: ParsedInput<'_>) -> Result<(), Box<dyn std::error::Error>> {
    match input.len() {
        0 => {},
        1 => {
            let Some(PipelineItem::Command(c)) = input.into_iter().next() else { panic!(); };  
            if is_builtin(c.cmd()) {
                execute(c.cmd(), c.args())?;
            } else {
                let _ = waitpid(spawn(&c, None, None)?, None);
            }
        }
        _ => {
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

            let mut pids = Vec::new();

            for (i, cmd) in commands.iter().enumerate() {
                let pipein = if i == 0 { None } else { pipes[i-1].read.take() };
                let pipeout = if i == commands.len()-1 { None } else { pipes[i].write.take() };
                pids.push(spawn(cmd, pipein, pipeout)?);
            }

            for pid in pids {
                waitpid(pid, None)?;
            }
        } // Avoid indentation
    }

    Ok(())
}

pub fn spawn(parsed_command: &ParsedCommand, pipein: Option<OwnedFd>, pipeout: Option<OwnedFd>) -> Result<Pid, Error> {
    match unsafe{fork()} {
        Ok(ForkResult::Parent { child, .. }) => { 
            Ok(child)
        }
        Ok(ForkResult::Child) => {
            #[allow(clippy::collapsible_if)]
            if let Some(fd) = pipein {
                if dup2_stdin(fd).is_err() {
                    unsafe { _exit(1); }
                } 
            }

            #[allow(clippy::collapsible_if)]
            if let Some(fd) = pipeout {
                if dup2_stdout(fd).is_err() {
                    unsafe { _exit(1); }
                }
            }

            let cmd = CString::new(parsed_command.cmd()).unwrap();
            let args: Vec<CString> = parsed_command.args()
                .iter()
                .map(|&s| CString::new(s).unwrap())
                .collect();
            let _ = execvp(&cmd, &args);
            eprintln!("magic-shell: command not found: {}", parsed_command.cmd());
            unsafe { _exit(127); } // 127 is the conventional "command not found" exit code
        }
        Err(e) => Err(Error::ForkFailed(e)),
    }
}
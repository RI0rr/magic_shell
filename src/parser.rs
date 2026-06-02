use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Emtpy input")]
    EmptyInput,
    #[error("Unknown error")]
    Unknown,
}
pub struct ParsedCommand<'a> {
    cmd: &'a str,
    args: Vec<&'a str>,
    pipes: Vec<usize>,
    redirs: Vec<usize>,
}

impl<'a> ParsedCommand<'a> {
    pub fn cmd(&self) -> &str {
        self.cmd
    }

    pub fn args(&self) -> &[&str] {
        &self.args
    }
}

pub fn parse(s: &str) -> Result<ParsedCommand<'_>, Error> {
    let tokens = s.split_whitespace();
    let args: Vec<&str> = tokens.collect();
    let cmd = args[0];
    let pipes = Vec::new();
    let redirs = Vec::new();

    Ok(ParsedCommand {
        cmd,
        args,
        pipes,
        redirs,
    })
}


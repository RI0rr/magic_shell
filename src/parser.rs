use thiserror::Error;

use crate::parser::PipelineItem::{Command, Separator};

static SEPARATORS: &[&str] = &["|", "<<", ">>", "&&", "||"];
fn is_separator(s: &str) -> bool {
    SEPARATORS.contains(&s)
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Emtpy input")]
    EmptyInput,
    #[error("Unknown error")]
    Unknown,
}

#[derive(PartialEq, Eq)]
pub enum ProcessSeparator {
    Pipe,
    And,
    Or,
}

#[derive(PartialEq, Eq)]
pub enum PipelineItem<'a> {
    Command(ParsedCommand<'a>),
    Separator(ProcessSeparator),
}

pub struct ParsedInput<'a> {
    items: Vec<PipelineItem<'a>>,
}

impl<'a> ParsedInput<'a> {
    fn new<I>(input: I) -> Self 
    where
        I: IntoIterator<Item = PipelineItem<'a>>
    {
        let items = input.into_iter().collect();
        Self { items }
    }

    pub fn iter(&self) -> impl Iterator<Item = &PipelineItem<'a>> {
        self.items.iter()
    }
}

impl<'a> IntoIterator for ParsedInput<'a> {
    type Item = PipelineItem<'a>;
    type IntoIter = std::vec::IntoIter<PipelineItem<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

#[derive(PartialEq, Eq)]

pub struct ParsedCommand<'a> {
    args: Vec<&'a str>,
    redirection: Option<&'a str>,
}

impl<'a> ParsedCommand<'a> {
    pub fn cmd(&self) -> &str {
        self.args[0]
    }

    pub fn args(&self) -> &[&str] {
        &self.args
    }
}

// Returns a vec of parsed processes to execute
// I've never parsed before so I'm making this up, feels logical
// Parsing strategy:
// 1. Separate processes: by ["|", "||", "&&"]
// 2. See if processes have redirections, in each &str look for ["<<", " >>"]
// 3. Separate args by whitespace
// 4. Join whitespaces if "" is found
pub fn parse(s: &str) -> Result<ParsedInput<'_>, Error> {
    // 1. 
    let commands = s.split('|');
    // 2, 
    let commands: Vec<ParsedCommand> = commands
        .map(|s| {
            if let Some((args, redir)) = s.split_once(">>") {
                return (args, Some(redir));
            }
            (s, None)
        })
        // 3. Separate by whitespace
        .map(|(args, redir)| {
            (args.split_whitespace(), redir)
        })
        // 5. Build collection
        .map(|(args, redirection)| {
            let args = args.collect();
            ParsedCommand { args, redirection }
        })
        .collect();

    
    // TODO: Add more separators, more sophisticated parsing
    let mut parsed_input = Vec::new();
    let n = commands.len();
    for (i, command) in commands.into_iter().enumerate() {
        parsed_input.push(Command(command));
        if i < n-1 {
            parsed_input.push(Separator(ProcessSeparator::Pipe));
        }
    }
    let p = ParsedInput::new(parsed_input);
    Ok(p)
}


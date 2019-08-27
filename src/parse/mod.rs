use std::{iter::{Iterator, Peekable, Enumerate}, slice::Iter};
use crate::{Command, IconType, ResamplingFilter, Entries, Output, syntax, error::{Error, SyntaxError}};
use token::{Flag};
pub use token::Token;

mod token;
type TokenStream<'a> = Peekable<Enumerate<Iter<'a, Token>>>;

macro_rules! icon {
    ($entries:expr, $type:expr) => {
        Command::Icon($entries, $type, Output::Stdout)
    };
    ($entries:expr, $type:expr, $path:expr) => {
        Command::Icon($entries, $type, Output::Path($path))
    };
}

macro_rules! next {
    ($it:expr, $p:pat, $e:expr) => {
        $it.next();
        match $it.peek() {
            Some(&(_, $p)) => $e,
            Some(&(c, _))  => return syntax!(SyntaxError::UnexpectedToken(c)),
            None           => return syntax!(SyntaxError::UnexpectedEnd)
        }
    };
}

pub fn args() -> Result<Command, Error> {
    let args = crate::args();

    if args.is_empty() { return Ok(Command::Help); }

    let n_entries = args.iter().fold(0, |sum, arg| if arg == "-e" { sum + 1 } else { sum });
    let mut entries = Vec::with_capacity(n_entries);

    let tokens = tokens(args);
    let mut it = tokens.iter().enumerate().peekable();

    while let Some(&(c, token)) = it.peek() {
        match token {
            Token::Flag(Flag::Entry) => entry(&mut it, &mut entries)?,
            Token::Flag(Flag::Ico) | Token::Flag(Flag::Icns) | Token::Flag(Flag::Png) => return command(&mut it, entries),
            Token::Flag(Flag::Help)    => return expect_end(&mut it, Command::Help),
            Token::Flag(Flag::Version) => return expect_end(&mut it, Command::Version),
            _                          => return syntax!(SyntaxError::UnexpectedToken(c))
        }
    }

    syntax!(SyntaxError::UnexpectedEnd)
}

fn tokens<'a>(args: Vec<String>) -> Vec<Token> {
    let mut output = Vec::with_capacity(args.len());

    for arg in args {
        output.push(Token::from(arg.as_str()));
    }

    // Remove the first token if it is a path. This accounts for the fact that the first element of
    // env.os_args() may be the path to this executable, removing it from the output if necessary.
    if output.len() > 0 {
        if let Token::Path(_) = output[0] {
            output.remove(0);
        }
    }

    output
}

fn entry(it: &mut TokenStream, entries: &mut Entries) -> Result<(), Error> {
    it.next();

    match it.peek() {
        Some(&(_, Token::Path(path))) => {
            // TODO Preallocate this Vec
            let mut sizes_acc: Vec<u32> = Vec::with_capacity(0);
            next!(it, Token::Size(_), sizes(it, &mut sizes_acc));

            let filter = filter(it)?;
    
            for size in sizes_acc {
                entries.push((size, path.clone(), filter));
            }
    
            Ok(())
        },
        Some(&(c, _)) => syntax!(SyntaxError::UnexpectedToken(c)),
        None          => syntax!(SyntaxError::UnexpectedEnd)
    }
}

fn sizes(it: &mut TokenStream, acc: &mut Vec<u32>) {
    while let Some(&(_, Token::Size(size))) = it.peek() {
        it.next();
        acc.push(*size);
    }
}

fn filter(it: &mut TokenStream) -> Result<ResamplingFilter, Error> {
    if let Some((_, Token::Flag(Flag::Resample))) = it.peek() {
        next!(it, Token::Filter(filter), { it.next(); return Ok(*filter); });
    }

    Ok(ResamplingFilter::Nearest)
}

fn command(it: &mut TokenStream, entries: Entries) -> Result<Command, Error> {
    let icon_type = icon_type(it)?;
    it.next();

    match it.peek() {
        Some(&(_, Token::Path(path))) => expect_end(it, icon!(entries, icon_type, path.clone())),
        Some(&(c, _)) => syntax!(SyntaxError::UnexpectedToken(c)),
        None          => expect_end(it, icon!(entries, icon_type))
    }
}

fn icon_type(it: &mut TokenStream) -> Result<IconType, Error> {
    match it.peek() {
        Some(&(_, Token::Flag(Flag::Ico)))  => Ok(IconType::Ico),
        Some(&(_, Token::Flag(Flag::Icns))) => Ok(IconType::Icns),
        Some(&(_, Token::Flag(Flag::Png)))  => Ok(IconType::PngSequence),
        Some(&(c, _))                       => syntax!(SyntaxError::UnexpectedToken(c)),
        None                                => syntax!(SyntaxError::UnexpectedEnd)
    }
}

fn expect_end(it: &mut TokenStream, command: Command) -> Result<Command, Error> {
    it.next();
    match it.peek() {
        Some(&(c, _)) => syntax!(SyntaxError::UnexpectedToken(c)),
        None => Ok(command)
    }
}
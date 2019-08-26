use std::{iter::{Iterator, Peekable, Enumerate}, slice::Iter};
use crate::{Command, IconType, ResamplingFilter, Entries, Output, syntax, error::{Error, SyntaxError}};
use token::{Token, Flag};

mod token;
type TokenStream<'a> = Peekable<Enumerate<Iter<'a, Token>>>;

pub fn args(args: Vec<String>) -> Result<Command, Error> {
    if args.is_empty() { return Ok(Command::Help); }

    let n_entries = args.iter().fold(0, |sum, arg| if arg == "-e" { sum + 1 } else { sum });
    let mut entries = Vec::with_capacity(n_entries);

    let tokens = tokens(args);
    let mut it = tokens.iter().enumerate().peekable();

    while let Some((c, token)) = it.peek() {
        match token {
            Token::Flag(Flag::Entry) => if let Err(err) = entry(&mut it, &mut entries) { return Err(err); },
            Token::Flag(Flag::Ico) | Token::Flag(Flag::Icns) | Token::Flag(Flag::Png) => return command(&mut it, entries),
            Token::Flag(Flag::Help)    => return expect_end(&mut it, Command::Help),
            Token::Flag(Flag::Version) => return expect_end(&mut it, Command::Version),
            _                          => return syntax!(SyntaxError::UnexpectedToken(*c))
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
    let &(c, _) = it.peek().expect("Variable 'it' should not be over.");
    it.next();

    if let Some(&(_, Token::Path(path))) = it.peek() {
        // TODO Preallocate this Vec
        let mut sizes_acc: Vec<u32> = Vec::with_capacity(0);
        it.next();

        match it.peek() {
            Some(&(_, Token::Size(_))) => sizes(it, &mut sizes_acc),
            Some(&(c, _)) => return syntax!(SyntaxError::UnexpectedToken(c)),
            None          => return syntax!(SyntaxError::UnexpectedEnd)
        }

        let filter = filter(it)?;

        for size in sizes_acc {
            entries.push((size, path.clone(), filter));
        }

        Ok(())
    } else {
        syntax!(SyntaxError::UnexpectedToken(c))
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
        it.next();
        match it.peek() {
            Some((_, Token::Filter(filter))) => return Ok(*filter),
            Some(&(c, _)) => return syntax!(SyntaxError::UnexpectedToken(c)),
            None          => return syntax!(SyntaxError::UnexpectedEnd)
        }
    }

    Ok(ResamplingFilter::Nearest)
}

fn command(it: &mut TokenStream, entries: Entries) -> Result<Command, Error> {
    let (c, token) = *it.peek().expect("Variable 'it' should not be over.");
    it.next();

    macro_rules! end {
        ($out:expr, $c:expr) => {
            match token {
                Token::Flag(Flag::Ico)  => expect_end(it, Command::Icon(entries, IconType::Ico,         $out)),
                Token::Flag(Flag::Icns) => expect_end(it, Command::Icon(entries, IconType::Icns,        $out)),
                Token::Flag(Flag::Png)  => expect_end(it, Command::Icon(entries, IconType::PngSequence, $out)),
                _                       => syntax!(SyntaxError::UnexpectedToken($c))
            }
        };
    }

    match it.peek() {
        Some(&(c, Token::Path(path))) => end!(Output::Path(path.clone()), c),
        Some(&(c, _)) => syntax!(SyntaxError::UnexpectedToken(c)),
        None          => end!(Output::Stdout, c + 1)
    }
}

fn expect_end(it: &mut TokenStream, command: Command) -> Result<Command, Error> {
    it.next();
    match it.peek() {
        Some(&(c, _)) => syntax!(SyntaxError::UnexpectedToken(c)),
        None => Ok(command)
    }
}
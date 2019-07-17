extern crate regex;

use std::{iter::{Iterator, Peekable, Enumerate}, slice::Iter, path::PathBuf, ffi::OsString, collections::HashMap};
use super::{Command, Error, error::SyntaxError};
use tokens::{Token, Flag, Opt};
use icon_baker::{Entry, ResamplingFilter, Crop};

type TokenStream<'a> = Peekable<Enumerate<Iter<'a, Token>>>;

macro_rules! syntax {
    ($err:expr) => { Err(Error::Syntax($err)) };
}

pub fn args(args: Vec<OsString>) -> Result<Command, Error> {
    if args.is_empty() { return Ok(Command::Help); }

    let n_entries = args.iter().fold(0, |sum, arg| if arg == "-e" { sum + 1 } else { sum });
    let mut entries = HashMap::with_capacity(n_entries);

    let tokens = tokens::parse(args)?;
    let mut it = tokens.iter().enumerate().peekable();

    while let Some((c, token)) = it.peek() {
        match token {
            Token::Flag(Flag::Entry) => if let Err(err) = entry(&mut it, &mut entries) {
                return Err(err);
            },
            Token::Flag(Flag::Output) | Token::Flag(Flag::Png) => return command::parse(&mut it, entries),
            Token::Flag(Flag::Help) => return help(&mut it),
            _ => return syntax!(SyntaxError::UnexpectedToken(*c))
        }
    }

     syntax!(SyntaxError::MissingOutputFlag)
}

mod tokens {
    use regex::Regex;
    use std::{iter::Iterator, path::PathBuf, ffi::OsString};
    use crate::{Error};
    use icon_baker::Size;

    #[derive(Clone, Debug, PartialEq)]
    pub enum Token {
        Flag(Flag),
        Opt(Opt),
        Path(PathBuf),
        Size(Size)
    }
    
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum Opt {
        Proportional,
        Interpolate
    }
    
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum Flag {
        Entry,
        Output,
        Png,
        Help,
        Version,
    }

    pub fn parse<'a>(args: Vec<OsString>) -> Result<Vec<Token>, Error> {
        let mut output = Vec::with_capacity(args.len());
    
        for arg in args {
            if let Some(arg_str) = arg.to_str() {
                output.push(token_from_str(arg_str));
            }
        }
    
        // Remove the first token if it is a path. This accounts for the fact that the first element of
        // env.os_args() may be the path to this executable, removing it from the output if necessary.
        if output.len() > 0 {
            if let Token::Path(_) = output[0] {
                output.remove(0);
            }
        }
    
        Ok(output)
    }

    fn token_from_str(string: &str) -> Token {
        let size_regex: Regex = Regex::new(r"^\d+x\d+$").unwrap();

        match string {
            "-e"   => Token::Flag(Flag::Entry),
            "-o"   => Token::Flag(Flag::Output),
            "-png" => Token::Flag(Flag::Png),
            "-h"   => Token::Flag(Flag::Help),
            "-v"   => Token::Flag(Flag::Version),
            "-p" | "--proportional" => Token::Opt(Opt::Proportional),
            "-i" | "--interpolate"  => Token::Opt(Opt::Interpolate),
            _ => if let Ok(size) = string.parse::<u16>() /* Parse a numeric value */ {
                Token::Size((size, size))
            } else if size_regex.is_match(string) /* Parse a tuple of numeric values */ {
                let sizes: Vec<&str> = string.split("x").collect();
                let w: u16 = sizes[0].parse().unwrap();
                let h: u16 = sizes[1].parse().unwrap();

                Token::Size((w, h))
            } else /* Parse a path */ {
                let mut p = PathBuf::new();
                p.push(string);

                Token::Path(p)
            } 
        }
    }

}

fn entry(it: &mut TokenStream, entries: &mut HashMap<Entry, PathBuf>) -> Result<(), Error> {
    let &(c, _) = it.peek().expect("Variable 'it' should not be over.");
    it.next();

    if let Some((_, Token::Path(path))) = it.peek() {
        it.next();
        // TODO Determine the number of sizes in this entry so that 'sizes' can be pre-allocated
        let mut sizes = Vec::new();

        match it.peek() {
            Some(&(_, Token::Size(_))) => while let Some((_, Token::Size((w, h)))) = it.peek() {
                it.next();
                sizes.push((*w, *h));
            },
            Some(&(c, _)) => return syntax!(SyntaxError::UnexpectedToken(c)),
            None => unreachable!()
        }

        let mut opts = Entry::new(sizes, ResamplingFilter::Neareast, Crop::Square);

        while let Some((_, Token::Opt(atrr))) = it.peek() {
            it.next();

            match atrr {
                Opt::Proportional => opts.crop = Crop::Proportional,
                Opt::Interpolate  => opts.filter = ResamplingFilter::Linear
            }
        }

        entries.insert(opts, path.clone());
        Ok(())
    } else {
        syntax!(SyntaxError::UnexpectedToken(c))
    }
}

mod command {
    use std::{convert::From, path::PathBuf, collections::HashMap};
    use crate::{Command, Error, error::SyntaxError};
    use super::{Token, TokenStream, Flag};
    use icon_baker::{Entry, IconType};
    
    pub fn parse(it: &mut TokenStream, entries: HashMap<Entry, PathBuf>) -> Result<Command, Error> {
        let (_, token) = *it.peek().expect("Variable 'it' should not be over.");
        it.next();
        
        if let Some(&(c, Token::Path(path))) = it.peek() {
            let ext = path.extension().unwrap_or_default().to_str().unwrap_or_default();
    
            match ext {
                "ico"  => match token {
                    Token::Flag(Flag::Output) => expect_end(it, Command::Icon(entries, IconType::Ico, path.clone())),
                    Token::Flag(Flag::Png) => syntax!(SyntaxError::UnsupportedPngOutput(String::from(ext))),
                    _ => syntax!(SyntaxError::UnexpectedToken(c))
                },
                "icns" => match token {
                    Token::Flag(Flag::Output) => expect_end(it, Command::Icon(entries, IconType::Icns, path.clone())),
                    Token::Flag(Flag::Png) => syntax!(SyntaxError::UnsupportedPngOutput(String::from(ext))),
                    _ => syntax!(SyntaxError::UnexpectedToken(c))
                },
                "zip"  => match token {
                    Token::Flag(Flag::Output) => syntax!(SyntaxError::UnsupportedOutputType(String::from(ext), c)),
                    Token::Flag(Flag::Png) => expect_end(it, Command::Icon(entries, IconType::PngSequence, path.clone())),
                    _ => syntax!(SyntaxError::UnexpectedToken(c))
                },
                _     => unreachable!()
            }
        } else {
            syntax!(SyntaxError::MissingOutputPath)
        }
    }
    
    fn expect_end(it: &mut TokenStream, command: Command) -> Result<Command, Error> {
        it.next();
        match it.peek() {
            Some(&(c, _)) => syntax!(SyntaxError::UnexpectedToken(c)),
            None => Ok(command)
        }
    }
}

fn help(it: &mut TokenStream) -> Result<Command, Error> {
    it.next();
    if let Some(&(c, _)) = it.peek() {
        syntax!(SyntaxError::UnexpectedToken(c))
    } else {
        Ok(Command::Help)
    }
}
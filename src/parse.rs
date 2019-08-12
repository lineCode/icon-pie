extern crate regex;

use std::{iter::{Iterator, Peekable, Enumerate}, slice::Iter, path::PathBuf, ffi::OsString, collections::HashMap};
use crate::{Command, syntax, error::{Error, SyntaxError}};
use tokens::{Token, Flag};
use icon_baker::{Size};

type TokenStream<'a> = Peekable<Enumerate<Iter<'a, Token>>>;

pub fn args(args: Vec<OsString>) -> Result<Command, Error> {
    if args.is_empty() { return Ok(Command::Help); }

    let n_entries = args.iter().fold(0, |sum, arg| if arg == "-e" { sum + 1 } else { sum });
    let mut entries = HashMap::with_capacity(n_entries);

    let tokens = tokens::parse(args)?;
    let mut it = tokens.iter().enumerate().peekable();

    while let Some((c, token)) = it.peek() {
        match token {
            Token::Flag(Flag::Size) => if let Err(err) = entry(&mut it, &mut entries) {
                return Err(err);
            },
            Token::Flag(Flag::Ico) | Token::Flag(Flag::Icns) | Token::Flag(Flag::Png) => return command::parse(&mut it, entries),
            Token::Flag(Flag::Help) => return help(&mut it),
            _ => return syntax!(SyntaxError::UnexpectedToken(*c))
        }
    }

     syntax!(SyntaxError::MissingOutputFlag)
}

mod tokens {
    use regex::Regex;
    use std::{path::PathBuf, ffi::OsString};
    use crate::{Error};
    use icon_baker::Size;

    #[derive(Clone, Debug, PartialEq)]
    pub enum Token {
        Flag(Flag),
        Path(PathBuf),
        Size(Size)
    }
    
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum Flag {
        Size,
        Ico,
        Icns,
        Png,
        Help,
        Version,
        Interpolate
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
        lazy_static! {
            static ref SIZE_REGEX: Regex = Regex::new(r"^\d+x\d+$").unwrap();
        }

        match string {
            "-e"    => Token::Flag(Flag::Size),
            "-ico"  => Token::Flag(Flag::Ico),
            "-icns" => Token::Flag(Flag::Icns),
            "-png"  => Token::Flag(Flag::Png),
            "-h" | "--help"        => Token::Flag(Flag::Help),
            "-v" | "--version"     => Token::Flag(Flag::Version),
            "-i" | "--interpolate" => Token::Flag(Flag::Interpolate),
            _ => if let Ok(size) = string.parse::<u32>() /* Parse a numeric value */ {
                Token::Size(size)
            } else /* Parse a path */ {
                path_from_str(string)
            }
        }
    }

    fn path_from_str(string: &str) -> Token {
        let mut p = PathBuf::new();
        p.push(string);

        Token::Path(p)
    }

}

fn entry(it: &mut TokenStream, entries: &mut HashMap<Size, (PathBuf, bool)>) -> Result<(), Error> {
    let &(c, _) = it.peek().expect("Variable 'it' should not be over.");
    it.next();

    if let Some(&(c, Token::Path(path))) = it.peek() {
        // TODO Preallocate this Vec and this HashMap
        let mut sizes = Vec::with_capacity(0);
        let mut sizes_index = HashMap::with_capacity(0);
        it.next();

        match it.peek() {
            Some(&(_, Token::Size(_))) => while let Some(&(c, Token::Size(size))) = it.peek() {
                it.next();
                sizes.push(size);

                if !sizes_index.contains_key(&size) {
                    sizes_index.insert(size, c);
                }
            },
            Some(&(c, _)) => return syntax!(SyntaxError::UnexpectedToken(c)),
            None => unreachable!()
        }

        let mut interpolate = false;
        if let Some((_, Token::Flag(Flag::Interpolate))) = it.peek() {
            interpolate = true;
            it.next();
        }

        for &size in sizes {
            if let Some(_) = entries.insert(size, (path.clone(), interpolate)) {
                return syntax!(SyntaxError::SizeAlreadyIncluded(
                    c, *sizes_index.get(&size)
                        .expect("The variable 'sizes_index' should contain a key for 'size'")
                ));
            }
        }

        it.next();
        Ok(())
    } else {
        syntax!(SyntaxError::UnexpectedToken(c))
    }
}

mod command {
    use std::{path::PathBuf, collections::HashMap};
    use crate::{Command, Output, error::{Error, SyntaxError}, syntax};
    use super::{Token, TokenStream, Flag};
    use icon_baker::{Size, IconType};
    
    pub fn parse(it: &mut TokenStream, entries: HashMap<Size, (PathBuf, bool)>) -> Result<Command, Error> {
        let (_, token) = *it.peek().expect("Variable 'it' should not be over.");
        it.next();

        match it.peek() {
            Some(&(c, Token::Path(path))) => match token {
                Token::Flag(Flag::Ico)  => expect_end(it, Command::Icon(entries, IconType::Ico,         Output::Path(path.clone()))),
                Token::Flag(Flag::Icns) => expect_end(it, Command::Icon(entries, IconType::Icns,        Output::Path(path.clone()))),
                Token::Flag(Flag::Png)  => expect_end(it, Command::Icon(entries, IconType::PngSequence, Output::Path(path.clone()))),
                _ => syntax!(SyntaxError::UnexpectedToken(c))
            },
            Some(&(c, _)) => syntax!(SyntaxError::UnexpectedToken(c)),
            None => syntax!(SyntaxError::MissingOutputPath)
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
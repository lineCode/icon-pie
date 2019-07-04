extern crate regex;

use regex::Regex;
use std::{convert::From, iter::{Iterator, Peekable, Enumerate}, slice::Iter, path::PathBuf, ffi::OsString, collections::HashMap};
use super::{Command, Error, error::SyntaxError};
use icon_baker::{Entry, IconType, ResamplingFilter, Crop, Size};

#[derive(Clone, Debug, PartialEq)]
enum Token {
    EntryFlag,
    OutputFlag,
    PngFlag,
    HelpFlag,
    VersionFlag,
    Opt(Opt),
    Path(PathBuf),
    Size(Size)
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Opt {
    Proportional,
    Interpolate
}

type TokenStream<'a> = Peekable<Enumerate<Iter<'a, Token>>>;

macro_rules! syntax {
    ($err:expr) => { Err(Error::Syntax($err)) };
}

pub fn args(args: Vec<OsString>) -> Result<Command, Error> {
    if args.is_empty() { return Ok(Command::Help); }

    let n_entries = args.iter().fold(0, |sum, arg| if arg == "-e" { sum + 1 } else { sum });
    let mut entries = HashMap::with_capacity(n_entries);

    let tokens = parse_tokens(args)?;
    let mut it = tokens.iter().enumerate().peekable();

    while let Some((c, token)) = it.peek() {
        match token {
            Token::EntryFlag => if let Err(err) = parse_entry(&mut it, &mut entries) {
                return Err(err);
            },
            Token::OutputFlag | Token::PngFlag => return parse_command(&mut it, entries),
            Token::HelpFlag => return parse_help(&mut it),
            _ => return syntax!(SyntaxError::UnexpectedToken(*c))
        }
    }

     syntax!(SyntaxError::MissingOutputFlag)
}

fn parse_tokens<'a>(args: Vec<OsString>) -> Result<Vec<Token>, Error> {
    let size_regex: Regex = Regex::new(r"^\d+x\d+$").unwrap();
    let mut output = Vec::with_capacity(args.len());

    for arg in args {
        let arg_str = arg.to_str().unwrap_or_default();

        match arg_str {
            "-e"   => output.push(Token::EntryFlag),
            "-o"   => output.push(Token::OutputFlag),
            "-png" => output.push(Token::PngFlag),
            "-h"   => output.push(Token::HelpFlag),
            "-v"   => output.push(Token::VersionFlag),
            "-p" | "--proportional" => output.push(Token::Opt(Opt::Proportional)),
            "-i" | "--interpolate"  => output.push(Token::Opt(Opt::Interpolate)),
            _ => if let Ok(size) = arg_str.parse::<u16>() /* Parse a numeric value */ {
                output.push(Token::Size((size, size)));
            } else if size_regex.is_match(arg_str) /* Parse a tuple of numeric values */ {
                let sizes: Vec<&str> = arg_str.split("x").collect();
                let w: u16 = sizes[0].parse().unwrap();
                let h: u16 = sizes[1].parse().unwrap();

                output.push(Token::Size((w, h)));
            } else /* Parse a path */ {
                let mut p = PathBuf::new();
                p.push(arg);

                output.push(Token::Path(p));
            } 
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

fn parse_entry(it: &mut TokenStream, entries: &mut HashMap<Entry, PathBuf>) -> Result<(), Error> {
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

fn parse_command(it: &mut TokenStream, entries: HashMap<Entry, PathBuf>) -> Result<Command, Error> {
    let (_, token) = *it.peek().expect("Variable 'it' should not be over.");
    it.next();
    
    if let Some(&(c, Token::Path(path))) = it.peek() {
        let ext = path.extension().unwrap_or_default().to_str().unwrap_or_default();

        match (token, ext) {
            (Token::PngFlag, "zip") | (Token::OutputFlag, "ico") | (Token::OutputFlag, "icns") => {
                it.next();
                match it.peek() {
                    Some(_) => syntax!(SyntaxError::UnexpectedToken(c)),
                    None => match ext {
                        "ico"  => Ok(Command::Icon(entries, IconType::Ico, path.clone())),
                        "icns" => Ok(Command::Icon(entries, IconType::Icns, path.clone())),
                        "zip"  => Ok(Command::Icon(entries, IconType::PngSequence, path.clone())),
                        _      => unreachable!()
                    }
                }
            },
            (_, ext) => match token {
                Token::OutputFlag => syntax!(SyntaxError::UnsupportedOutputType(String::from(ext), c)),
                Token::PngFlag => syntax!(SyntaxError::UnsupportedPngOutput(String::from(ext))),
                _ => syntax!(SyntaxError::UnexpectedToken(c))
            }
        }
    } else {
        syntax!(SyntaxError::MissingOutputPath)
    }
}

fn parse_help(it: &mut TokenStream) -> Result<Command, Error> {
    it.next();
    if let Some(&(c, _)) = it.peek() {
        syntax!(SyntaxError::UnexpectedToken(c))
    } else {
        Ok(Command::Help)
    }
}
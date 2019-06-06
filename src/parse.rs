extern crate regex;

use regex::Regex;
use std::{convert::From, iter::{Iterator, Peekable}, slice::Iter, path::PathBuf, ffi::OsString, collections::HashMap};
use super::{Command, Error, error::SyntaxError};
use icon_baker::{IconOptions, IconType, ResamplingFilter, Crop, Size};

#[derive(Clone, Debug, PartialEq)]
enum Token {
    EntryFlag,
    OutputFlag,
    PngFlag,
    HelpFlag,
    Attribute(Attribute),
    Path(PathBuf),
    Size(Size)
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Attribute {
    Proportional,
    Interpolate
}

impl From<&Token> for String {
    fn from(token: &Token) -> String {
        match token {
            Token::EntryFlag   => String::from("-e"),
            Token::OutputFlag => String::from("-o"),
            Token::PngFlag    => String::from("-png"),
            Token::HelpFlag   => String::from("-h"),
            Token::Path(path) => format!("{}", path.as_path().display()),
            Token::Attribute(atrr) => match atrr {
                Attribute::Proportional => String::from("-p"),
                Attribute::Interpolate  => String::from("-i")
            },
            Token::Size((w, h)) => if w == h { format!("{}", w) } else { format!("{}x{}", w, h) }
        }
    }
}

macro_rules! syntax {
    ($err:expr) => {Err(Error::Syntax($err))};
}

pub fn args(args: Vec<OsString>) -> Result<Command, Error> {
    let n_entries = args.iter().fold(0, |sum, arg| if arg == "-f" { sum + 1 } else { sum });
    let mut entries = HashMap::with_capacity(n_entries);
    let mut first_arg = true;

    let tokens = parse_tokens(args)?;
    let mut it = tokens.iter().peekable();

    while let Some(&token) = it.peek() {
        match token {
            &Token::EntryFlag => if let Err(err) = parse_entry(&mut it, &mut entries) {
                return Err(err);
            },
            &Token::OutputFlag | &Token::PngFlag => return parse_command(&mut it, entries),
            &Token::HelpFlag => return parse_help(&mut it),
            _ => if first_arg {
                it.next();
                first_arg = false;
            } else {
                return syntax!(SyntaxError::UnexpectedToken(String::from(token)));
            }
        }
    }

     syntax!(SyntaxError::MissingOutputFlag)
}

fn parse_tokens(args: Vec<OsString>) -> Result<Vec<Token>, Error> {
    let size_regex: Regex = Regex::new(r"^\d+x\d+$").unwrap();
    let mut output = Vec::with_capacity(args.len());

    for arg in args {
        let arg_str = arg.to_str().unwrap_or_default();

        match arg_str {
            "-e"   => output.push(Token::EntryFlag),
            "-o"   => output.push(Token::OutputFlag),
            "-png" => output.push(Token::PngFlag),
            "-h"   => output.push(Token::HelpFlag),
            "-p" | "--proportional" => output.push(Token::Attribute(Attribute::Proportional)),
            "-i" | "--interpolate"  => output.push(Token::Attribute(Attribute::Interpolate)),
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

    Ok(output)
}

fn parse_entry(it: &mut Peekable<Iter<'_, Token>>, entries: &mut HashMap<IconOptions, PathBuf>) -> Result<(), Error> {
    it.next();
    if let Some(Token::Path(path)) = it.peek() {
        it.next();
        // TODO Determine the number of sizes in this File struct so that File::sizes can be pre-allocated
        let mut sizes = Vec::new();

        while let Some(Token::Size((w, h))) = it.peek() {
            it.next();
            sizes.push((*w, *h));
        }

        let mut opts = IconOptions::new(sizes, ResamplingFilter::Neareast, Crop::Square);

        while let Some(Token::Attribute(atrr)) = it.peek() {
            it.next();

            match atrr {
                Attribute::Proportional => opts.crop = Crop::Proportional,
                Attribute::Interpolate  => opts.filter = ResamplingFilter::Linear
            }
        }

        entries.insert(opts, path.clone());
        Ok(())
    } else {
        syntax!(SyntaxError::UnexpectedToken(String::from(&Token::EntryFlag)))
    }
}

fn parse_command(it: &mut Peekable<Iter<'_, Token>>, entries: HashMap<IconOptions, PathBuf>) -> Result<Command, Error> {
    let token = *it.peek().expect("Variable 'it' should not be over.");

    it.next();
    if let Some(Token::Path(path)) = it.peek() {
        let ext = path.extension().unwrap_or_default().to_str().unwrap_or_default();

        match (token, ext) {
            (Token::PngFlag, "zip") | (Token::OutputFlag, "ico") | (Token::OutputFlag, "icns") => {
                it.next();
                match it.peek() {
                    Some(token) => syntax!(SyntaxError::UnexpectedToken(String::from(*token))),
                    None => match ext {
                        "ico"  => Ok(Command::Icon(entries, IconType::Ico, path.clone())),
                        "icns" => Ok(Command::Icon(entries, IconType::Icns, path.clone())),
                        "zip"  => Ok(Command::Icon(entries, IconType::PngSequence, path.clone())),
                        _      => unreachable!()
                    }
                }
            },
            (_, ext) => match token {
                Token::OutputFlag => syntax!(SyntaxError::UnsupportedOutputType(String::from(ext))),
                Token::PngFlag => syntax!(SyntaxError::UnsupportedPngOutput(String::from(ext))),
                _ => syntax!(SyntaxError::UnexpectedToken(String::from(token)))
            }
        }
    } else {
        syntax!(SyntaxError::MissingOutputPath)
    }
}

fn parse_help(it: &mut Peekable<Iter<'_, Token>>) -> Result<Command, Error> {
    it.next();
    if let Some(&token) = it.peek() {
        syntax!(SyntaxError::UnexpectedToken(String::from(token)))
    } else {
        Ok(Command::Help)
    }
}
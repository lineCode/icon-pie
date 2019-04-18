extern crate regex;

use regex::Regex;
use std::{convert::From, iter::Iterator};
use super::{eval::{Command, FitType, OutputType}, Size, Error, SyntaxError, EvalError};
use nsvg::image::FilterType;

#[derive(Clone)]
pub struct File {
    path: String,
    sizes: Vec<Size>,
    pub fit_type: FitType,
    pub filter_type: FilterType
}

#[derive(Clone, Debug, PartialEq)]
enum Token {
    FileFlag,
    OutputFlag,
    PngFlag,
    HelpFlag,
    Attribute(Attribute),
    Path(String),
    Size(Size)
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Attribute {
    Proportional,
    Interpolate
}

impl File {
    fn new(path: &String) -> File {
        File {path: path.clone(), sizes: Vec::new(), fit_type: FitType::Strict, filter_type: FilterType::Nearest}
    }

    fn add_size(&mut self, size: Size) {
        let (w, h) = size;

        if !self.sizes.iter().map(|(w, h)| (*w, *h)).collect::<Vec<Size>>().contains(&(w, h)) {
            self.sizes.push(size);
        }
    }

    pub fn path(&self) -> &String {
        &self.path
    }

    pub fn sizes(&self) -> Vec<Size> {
        self.sizes.clone()
    }

    pub fn n_sizes(&self) -> usize {
        self.sizes.len()
    }
}

impl From<&Token> for String {
    fn from(token: &Token) -> String {
        match token {
            Token::FileFlag => String::from("-f"),
            Token::OutputFlag => String::from("-o"),
            Token::PngFlag => String::from("-png"),
            Token::HelpFlag => String::from("-h"),
            Token::Path(path) => path.clone(),
            Token::Attribute(atrr) => match atrr {
                Attribute::Proportional => String::from("--proportional"),
                Attribute::Interpolate => String::from("--interpolate")
            },
            Token::Size((w, h)) => if w == h { format!("{}", w) } else { format!("{}x{}", w, h) }
        }
    }
}

macro_rules! syntax {
    ($err:expr) => {Err(Error::Syntax($err))};
}

macro_rules! eval {
    ($err:expr) => {Err(Error::Eval($err))};
}

pub fn args(args: Vec<String>) -> Result<Command, Error> {
    let n_files = args.iter().fold(0, |sum, arg| if arg == "-f" { sum + 1 } else { sum });
    let mut files = Vec::with_capacity(n_files);
    let mut first_arg = true;

    let tokens = tokens(args)?;
    let mut it = tokens.iter().peekable();

    while let Some(&token) = it.peek() {
        match token {
            &Token::FileFlag => {
                it.next();
                // TODO Determine the number of sizes in this File struct so that File::sizes can be pre-allocated
                if let Some(Token::Path(path)) = it.peek() {
                    it.next();
                    let mut file = File::new(&path);

                    while let Some(Token::Size((w, h))) = it.peek() {
                        it.next();
                        file.add_size((*w, *h));
                    }

                    while let Some(Token::Attribute(atrr)) = it.peek() {
                        it.next();

                        match atrr {
                            Attribute::Proportional => file.fit_type = FitType::Proportional,
                            Attribute::Interpolate => file.filter_type = FilterType::Triangle
                        }
                    }

                    files.push(file);
                } else {
                    return syntax!(SyntaxError::UnexpectedToken(String::from(token)));
                }
            },
            &Token::OutputFlag | &Token::PngFlag => {
                it.next();
                if let Some(Token::Path(path)) = it.peek() {
                    let ext = ext(path).unwrap_or_default();

                    match (token, ext.as_ref()) {
                        (Token::PngFlag, "zip") | (Token::OutputFlag, "ico") | (Token::OutputFlag, "icns") => {
                            it.next();
                            match it.peek() {
                                Some(token) => return syntax!(SyntaxError::UnexpectedToken(String::from(*token))),
                                None => match ext.as_ref() {
                                    "ico"  => return Ok(Command::encode(files, path.clone(), OutputType::Ico)),
                                    "icns" => return Ok(Command::encode(files, path.clone(), OutputType::Icns)),
                                    "zip"  => return Ok(Command::encode(files, path.clone(), OutputType::PngSequence)),
                                    _      => unreachable!()
                                }
                            }
                        },
                        (_, ext) => match token {
                            Token::OutputFlag => return eval!(EvalError::UnsupportedOutputType(String::from(ext))),
                            Token::PngFlag => return eval!(EvalError::UnsupportedPngOutput(String::from(ext))),
                            _ => return syntax!(SyntaxError::UnexpectedToken(String::from(token)))
                        }
                    }
                } else {
                    return syntax!(SyntaxError::MissingOutputPath);
                }
            },
            &Token::HelpFlag => {
                it.next();
                if let Some(&token) = it.peek() {
                    return syntax!(SyntaxError::UnexpectedToken(String::from(token)));
                } else {
                    return Ok(Command::Help);
                }
            },
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

pub fn ext(path: &String) -> Option<String> {
    let mut it = path.chars().rev();
    let mut ext = String::with_capacity(3);

    while let Some(c) = it.next() {
        if c == '.' {
            break;
        } else {
            if ext.len() > 0 {
                ext.insert(0, c);
            } else {
                ext.push(c);
            }
        }
    }

    if ext.len() > 0 {
        Some(ext)
    } else {
        None
    }
}

fn tokens(args: Vec<String>) -> Result<Vec<Token>, Error> {
    let size_regex: Regex = Regex::new(r"^\d+x\d+$").unwrap();
    let mut output = Vec::with_capacity(args.len());

    for arg in args {
        match arg.clone().into_boxed_str().as_ref() {
            "-f"   => output.push(Token::FileFlag),
            "-o"   => output.push(Token::OutputFlag),
            "-png" => output.push(Token::PngFlag),
            "-h"   => output.push(Token::HelpFlag),
            "-p" | "--proportional" => output.push(Token::Attribute(Attribute::Proportional)),
            "-i" | "--interpolate"  => output.push(Token::Attribute(Attribute::Interpolate)),
            _ => if let Ok(size) = arg.parse::<u16>() /* Parse a numeric value */ {
                output.push(Token::Size((size, size)));
            } else if size_regex.is_match(&arg) /* Parse a tuple of numeric values */ {
                let sizes: Vec<&str> = arg.split("x").collect();
                let w: u16 = sizes[0].parse().unwrap();
                let h: u16 = sizes[1].parse().unwrap();

                output.push(Token::Size((w, h)));
            } else /* Parse a path */ {
                output.push(Token::Path(arg.clone()));
            } 
        }
    }

    Ok(output)
}
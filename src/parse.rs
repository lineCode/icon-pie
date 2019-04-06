extern crate regex;

use regex::Regex;
use std::{path::Path, convert::From, iter::Iterator};
use super::{img::{Command, FitType}, Error, SyntaxError};
use image::FilterType;

#[derive(Clone)]
pub struct File {
    path: String,
    sizes: Vec<(u16, u16)>,
    pub fit_type: FitType,
    pub filter_type: FilterType
}

#[derive(Clone, Debug, PartialEq)]
enum Token {
    FileFlag,
    OutputFlag,
    PngFlag,
    Attribute(Attribute),
    Path(String),
    Size((u16, u16))
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Attribute {
    Proportional,
    Interpolate
}

impl File {
    fn new(path: &String) -> File {
        File {path: path.clone(), sizes: Vec::new(), fit_type: FitType::Exact, filter_type: FilterType::Nearest}
    }

    fn add_size(&mut self, size: (u16, u16)) {
        let (w, h) = size;

        if !self.sizes.iter().map(|(w, h)| (*w, *h)).collect::<Vec<(u16, u16)>>().contains(&(w, h)) {
            self.sizes.push(size);
        }
    }

    pub fn path(&self) -> &String {
        &self.path
    }

    pub fn sizes(&self) -> Vec<(u16, u16)> {
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
            Token::Path(path) => path.clone(),
            Token::Attribute(atrr) => {
                match atrr {
                    Attribute::Proportional => String::from("--proportional"),
                    Attribute::Interpolate => String::from("--interpolate")
                }
            },
            Token::Size((w, h)) => {
                if w == h {
                    format!("{}", w)
                } else {
                    format!("{}x{}", w, h)
                }
             }
        }
    }
}

macro_rules! error {
    ($err:expr) => {Err(Error::Syntax($err))};
}

pub fn args(args: &Vec<String>) -> Result<Command, Error> {

    // Remove the first element of the args Vec if it is a path to this executable
    let mut args: Vec<String> = args.clone();
    if let Some(ext) = ext(args.first().unwrap_or(&String::new())) {
        if ext == String::from("exe") {
            args = args[1..].to_vec();
        }
    }

    let tokens = tokens(&args)?;
    let mut it = tokens.iter().peekable();

    let n_files = args.iter().map(|arg| if arg == "-f" { 1 } else { 0 }).fold(0, |sum, x| sum + x);
    let mut files = Vec::with_capacity(n_files);

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
                    return error!(SyntaxError::UnexpectedToken(String::from(token)));
                }
            },
            &Token::OutputFlag | &Token::PngFlag => {
                it.next();
                if let Some(Token::Path(path)) = it.peek() {
                    match (token, ext(path).unwrap_or_default().as_ref()) {
                        (Token::PngFlag, "zip") | (Token::OutputFlag, "ico") | (Token::OutputFlag, "icns") => {
                            it.next();
                            match it.peek() {
                                Some(token) => return error!(SyntaxError::UnexpectedToken(String::from(*token))),
                                None => return Ok(Command::new(files, path.clone()))
                            }
                        },
                        _ => unimplemented!("TODO Return an error saying 'Wrong combination of output options'")
                    }
                } else {
                    return error!(SyntaxError::MissingOutputPath);
                }
            },
            _ => return error!(SyntaxError::UnexpectedToken(String::from(token)))
        }
    }

     error!(SyntaxError::MissingOutputFlag)
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

fn tokens(args: &Vec<String>) -> Result<Vec<Token>, Error> {
    let size_regex: Regex = Regex::new(r"^\d+x\d+$").unwrap();
    let mut output = Vec::with_capacity(args.len());

    for arg in args {
        match arg.clone().into_boxed_str().as_ref() {
            "-f" => output.push(Token::FileFlag),
            "-o" => output.push(Token::OutputFlag),
            "-png" => output.push(Token::PngFlag),
            "--proportional" => output.push(Token::Attribute(Attribute::Proportional)),
            "--interpolate" => output.push(Token::Attribute(Attribute::Interpolate)),
            _ => if let Ok(size) = arg.parse::<u16>() /* Parse a numeric value */ {
                output.push(Token::Size((size, size)));
            } else if size_regex.is_match(&arg) /* Parse a tuple of numeric values */ {
                let sizes: Vec<&str> = arg.split("x").collect();
                let w: u16 = sizes[0].parse().unwrap();
                let h: u16 = sizes[1].parse().unwrap();

                output.push(Token::Size((w, h)));
            } else if let Some(_) = Path::new(&arg).extension() /* Parse a path */ {
                output.push(Token::Path(arg.clone()));
            } else {
                return error!(SyntaxError::UnexpectedToken(arg.clone()));
            }
        }
    }

    Ok(output)
}
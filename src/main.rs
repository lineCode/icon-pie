extern crate image;
extern crate crossterm;
extern crate regex;

mod img;
mod parse;

use std::{env, io, path::Path};
use image::ImageError;
use crossterm::{style, Color};
use regex::Regex;

#[derive(Debug)]
pub enum Error {
    Syntax(SyntaxError),
    Image(ImageError),
    Io((io::Error, Option<String>)),
}

#[derive(Clone, Debug, PartialEq)]
pub enum SyntaxError {
    UnexpectedToken(String),
    MissingOutputFlag,
    MissingOutputPath,
    UnknownOutputType,
    UnsupportedOutputType(String),
    SizeReassignment((u16, u16, String, String)),
    InvalidProportionalFlag((u16, u16)),
    InvalidDownsizingOpts((u16, u16))
}

impl Error {
    // TODO Implement this function
    fn show(&self) {
        match &self {
            Error::Syntax(err) => match err {
                // TODO Find a way to show where the error is located in a original command
                SyntaxError::UnexpectedToken(token) => println!(
                    "{} Unexpected token: {}.",
                    style("[Syntax Error]").with(Color::Red),
                    style(token).with(Color::Red)
                ),
                SyntaxError::MissingOutputFlag => println!(
                    "{} Missing output details: TODO. Type {} for more details on IconBaker's usage.",
                    style("[Syntax Error]").with(Color::Red),
                    style("icon-baker -h").with(Color::Blue)
                ),
                SyntaxError::MissingOutputPath => print!(
                    "{} Missing output path: No path for the output file was specified. Type {} for more details on IconBaker's usage.",
                    style("[Syntax Error]").with(Color::Red),
                    style("icon-baker -h").with(Color::Blue)
                ),
                _ => unimplemented!()
            },
            Error::Image(_err) => unimplemented!(),
            Error::Io((err, file)) => match err.kind() {
                io::ErrorKind::NotFound => println!(
                    "{} File {} could not be found on disk.",
                    style("[IO Error]").with(Color::Red),
                    style(file.clone().unwrap_or_default()).with(Color::Blue)
                ),
                io::ErrorKind::PermissionDenied => println!(
                    "{} Permission denied: File {} is inaccecible.",
                    style("[IO Error]").with(Color::Red),
                    style(file.clone().unwrap_or_default()).with(Color::Blue)
                ),
                io::ErrorKind::AddrInUse | io::ErrorKind::AddrNotAvailable => println!(
                    "{} File {} is unavaiable. Try closing any application that may be using it.",
                    style("[IO Error]").with(Color::Red),
                    style(file.clone().unwrap_or_default()).with(Color::Blue)
                ),
                io::ErrorKind::InvalidData | io::ErrorKind::InvalidInput => println!(
                    "{} File {} couln't be parsed. This file may be corrupted.",
                    style("[IO Error]").with(Color::Red),
                    style(file.clone().unwrap_or_default()).with(Color::Blue)
                ),
                _ => panic!("{:?}", err)
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args_os().map(|arg| arg.into_string().unwrap_or_default()).collect();
    let path_start = Regex::new(r"^\\\\\?\\").unwrap();

    match parse::args(&args) {
        Ok(cmd) => match cmd.exec() {
            Ok(meta) => {
                let out = cmd.output_path();
                let path = Path::new(&out);
                println!(
                    "{} File {} saved at {} [{} KB]",
                    style("[Ok]").with(Color::Green),
                    style(path.file_name().unwrap().to_string_lossy()).with(Color::Blue),
                    style(
                        path_start.replace_all(path.canonicalize().unwrap_or(env::current_dir().unwrap()).to_string_lossy().as_ref(), "")
                    ).with(Color::Blue),
                    meta.len() / 1000
                );
            },
            Err(err) => err.show()
        },
        Err(err)  => err.show()
    }
}
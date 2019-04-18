extern crate nsvg;
extern crate crossterm;
extern crate regex;

mod eval;
mod parse;

use std::{env, io, path::Path};
use eval::Command;
use crossterm::{style, Color};
use regex::Regex;

type Size = (u16, u16);

#[derive(Debug)]
pub enum Error {
    Syntax(SyntaxError),
    Eval(EvalError),
    Io(io::Error, String)
}

#[derive(Clone, Debug, PartialEq)]
pub enum SyntaxError {
    UnexpectedToken(String),
    MissingOutputFlag,
    MissingOutputPath,
    SizeReassignment(Size)
}

#[derive(Clone, Debug, PartialEq)]
pub enum EvalError {
    InvalidIcoSize(Size),
    InvalidIcnsSize(Size),
    IlligalDownsizing(Size, (u32, u32), String),
    UnsupportedOutputType(String),
    UnsupportedPngOutput(String)
}

const TITLE: &str = r"
 __  ___  __   __ _  ____   __   __ _  ____  ____ 
(  )/ __)/  \ (  ( \(  _ \ / _\ (  / )(  __)(  _ \
 )(( (__(  O )/    / ) _ (/    \ )  (  ) _)  )   /
(__)\___)\__/ \_)__)(____/\_/\_/(__\_)(____)(__\_)
BETA 0.0.1";
const USAGE: &str = "   icon-baker ((-f <file path> (<size>) [--interpolate | --proportional]) (-o <output path> | -png <output path>)) | -h\n";
const EXAMPLES: &str = "    icon-baker -f small.svg 16 20 24 -f big.png 32 64 -o output.ico
    icon-baker -f image.png 32x12 64x28 48 --interpolate -png output.zip\n";
const VALID_ICNS_SIZES: &str = "16x16, 32x32, 64x64, 128x128, 512x512 and 1024x1024";

macro_rules! help {
    () => {
        println!(
            "{}\n\n\n{}\n{}\n{}\n{}\n",
            style(TITLE).with(Color::Green),
            style("Usage").with(Color::Blue),
            USAGE,
            style("Examples").with(Color::Blue),
            EXAMPLES
        );
    };
}

impl Error {
    fn show(&self) {
        match &self {
            Error::Syntax(err) => match err {
                SyntaxError::MissingOutputFlag => println!(
                    "{} Missing output details. Type {} for more details on IconBaker's usage.",
                    style("[Syntax Error]").with(Color::Red),
                    style("icon-baker -h").with(Color::Blue)
                ),
                SyntaxError::MissingOutputPath => println!(
                    "{} Missing output path: No path for the output file was specified. Type {} for more details on IconBaker's usage.",
                    style("[Syntax Error]").with(Color::Red),
                    style("icon-baker -h").with(Color::Blue)
                ),
                SyntaxError::SizeReassignment((w, h)) => println!(
                    "{} The target size {}x{} is assigned to multiple files. Sizes can only be assigned to a single file.",
                    style("[Syntax Error]").with(Color::Red),
                    w, h
                ),
                SyntaxError::UnexpectedToken(token) => println!(
                    "{} Unexpected token: {}.",
                    style("[Syntax Error]").with(Color::Red),
                    style(token).with(Color::Red)
                )
            },
            Error::Eval(err) => match err {
                EvalError::IlligalDownsizing((w, h), (fw, fh), ext) => println!(
                    "{} A {}x{} {} file cannot be scaled to a {}x{} image. To enforce the scaling use the {} flag.",
                    style("[Scaling Error]").with(Color::Red),
                    fw, fh,
                    style(format!(".{}", ext.to_lowercase())).with(Color::Blue),
                    w, h,
                    style("--interpolate").with(Color::Blue)
                ),
                EvalError::InvalidIcnsSize((w, h)) => if w == h {
                    println!(
                        "{} The {} file format only supports {} icons: {}x{} icons aren't supported.",
                        style("[Icns Error]").with(Color::Red),
                        style(".icns").with(Color::Blue),
                        VALID_ICNS_SIZES,
                        w, h
                    )
                } else {
                    println!(
                        "{} The {} file format only supports square icons: {}x{} icons aren't supported.",
                        style("[Icns Error]").with(Color::Red),
                        style(".icns").with(Color::Blue),
                        w, h
                    )
                },
                EvalError::InvalidIcoSize((w, h)) => if w == h {
                    println!(
                        "{} The {} file format only supports icons of dimensions up to 256x256: {}x{} icons aren't supported.",
                        style("[Ico Error]").with(Color::Red),
                        style(".ico").with(Color::Blue),
                        w, h
                    )
                } else {
                    println!(
                        "{} The {} file format only supports square icons: {}x{} icons aren't supported.",
                        style("[Ico Error]").with(Color::Red),
                        style(".ico").with(Color::Blue),
                        w, h
                    )
                },
                EvalError::UnsupportedOutputType(ext) => println!(
                    "{} Files with the {} file extension are not supported.", 
                    style("[IO Error]").with(Color::Red),
                    style(format!(".{}", ext.to_lowercase())).with(Color::Blue)
                ),
                EvalError::UnsupportedPngOutput(ext) => println!(
                    "{} The {} option only supports the {} file format. The {} file extension is not supported",
                    style("[IO Error]").with(Color::Red),
                    style("-png").with(Color::Blue),
                    style(".zip").with(Color::Blue),
                    style(format!(".{}", ext.to_lowercase())).with(Color::Blue)
                )
            },
            Error::Io(err, file) => match err.kind() {
                io::ErrorKind::NotFound => println!(
                    "{} File {} could not be found on disk.",
                    style("[IO Error]").with(Color::Red),
                    style(file).with(Color::Blue)
                ),
                io::ErrorKind::PermissionDenied => println!(
                    "{} Permission denied: File {} is inaccecible.",
                    style("[IO Error]").with(Color::Red),
                    style(file).with(Color::Blue)
                ),
                io::ErrorKind::AddrInUse | io::ErrorKind::AddrNotAvailable => println!(
                    "{} File {} is unavaiable. Try closing any application that may be using it.",
                    style("[IO Error]").with(Color::Red),
                    style(file).with(Color::Blue)
                ),
                io::ErrorKind::InvalidData | io::ErrorKind::InvalidInput => println!(
                    "{} File {} couln't be parsed. This file may be corrupted.",
                    style("[IO Error]").with(Color::Red),
                    style(file).with(Color::Blue)
                ),
                _ => panic!("{:?}", err)
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args_os().map(|arg| arg.into_string().unwrap_or_default()).collect();
    let path_start = Regex::new(r"^\\\\\?\\").unwrap();

    match parse::args(args) {
        Ok(cmd) => match cmd {
            Command::Encode(cmd) => match cmd.exec() {
                Ok(meta) => {
                    let out = cmd.output_path();
                    let path = Path::new(&out);
                    println!(
                        "{} File {} saved at {} [{} KB]",
                        style("[IconBaker]").with(Color::Green),
                        style(path.file_name().unwrap().to_string_lossy()).with(Color::Blue),
                        style(
                            path_start.replace_all(path.canonicalize().unwrap_or(env::current_dir().unwrap()).to_string_lossy().as_ref(), "")
                        ).with(Color::Blue),
                        meta.len() / 1000
                    );
                },
                Err(err) => err.show()
            },
            Command::Help => help!()
        },
        Err(err)  => err.show()
    }
}
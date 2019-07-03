use std::{io, path::PathBuf, ffi::OsString};
use crossterm::{style, Color};

#[derive(Debug)]
pub enum Error {
    Syntax(SyntaxError),
    IconBaker(icon_baker::Error),
    Io(io::Error, PathBuf)
}

#[derive(Clone, Debug, PartialEq)]
pub enum SyntaxError {
    UnexpectedToken(usize),
    MissingOutputFlag,
    MissingOutputPath,
    UnsupportedOutputType(String, usize),
    UnsupportedPngOutput(String)
}

const VALID_ICNS_SIZES: &str = "16x16, 32x32, 64x64, 128x128, 512x512 and 1024x1024";

impl Error {
    pub fn show(&self, args: Vec<OsString>) -> io::Error {
        match &self {
            Error::Syntax(err)    => show_syntax(err, args),
            Error::IconBaker(err) => show_icon_baker(err),
            Error::Io(err, path)  => show_io(err, path.clone())
        }

        self.to_io()
    }

    pub fn to_io(&self) -> io::Error {
        match &self {
            Error::Syntax(_)  => io::Error::from(io::ErrorKind::InvalidInput),
            Error::Io(err, _) => io::Error::from(err.kind()),
            Error::IconBaker(err) => match err {
                icon_baker::Error::InvalidIcnsSize(_)
                | icon_baker::Error::InvalidIcoSize(_)
                | icon_baker::Error::SizeAlreadyIncluded(_) => io::Error::from(io::ErrorKind::InvalidInput),
                icon_baker::Error::Io(err) => io::Error::from(err.kind()),
                _ => io::Error::from(io::ErrorKind::Other)
            }
        }
    }
}

fn show_syntax(err: &SyntaxError, args: Vec<OsString>) {
    let args: Vec<String> = args.iter().map(|os_str| os_str.clone().into_string().unwrap_or_default()).collect();

    match err {
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
        SyntaxError::UnexpectedToken(err) => println!(
            "{} Unexpected token: {} {} {}",
            style("[Syntax Error]").with(Color::Red),
            style(args[..*err].join(" ")).with(Color::Blue),
            style(args[*err].clone()).with(Color::Red),
            style(args[(*err + 1)..].join(" ")).with(Color::Blue),
        ),
        SyntaxError::UnsupportedOutputType(ext, err) => println!(
            "{} The {} file extension is not supported: {} {} {}", 
            style("[IO Error]").with(Color::Red),
            style(format!(".{}", ext.to_lowercase())).with(Color::Blue),
            style(args[..*err].join(" ")).with(Color::Blue),
            style(args[*err].clone()).with(Color::Red),
            style(args[(*err + 1)..].join(" ")).with(Color::Blue)
        ),
        SyntaxError::UnsupportedPngOutput(ext) => println!(
            "{} The {} option only supports the {} file format. The {} file extension is not supported",
            style("[IO Error]").with(Color::Red),
            style("-png").with(Color::Blue),
            style(".zip").with(Color::Blue),
            style(format!(".{}", ext.to_lowercase())).with(Color::Blue)
        )
    }
}

fn show_icon_baker(err: &icon_baker::Error) {
    match err {
        icon_baker::Error::InvalidIcnsSize((w, h)) => if w == h {
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
        icon_baker::Error::InvalidIcoSize((w, h)) => if w == h {
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
        icon_baker::Error::SizeAlreadyIncluded((_w, _h)) => unimplemented!(),
        icon_baker::Error::Io(_) => unreachable!(),
        _ => panic!("{:?}", err)
    }
}

fn show_io(err: &io::Error, path: PathBuf) {
    match err.kind() {
        io::ErrorKind::NotFound => println!(
            "{} File {} could not be found on disk.",
            style("[IO Error]").with(Color::Red),
            style(path.display()).with(Color::Blue)
        ),
        io::ErrorKind::PermissionDenied => println!(
            "{} Permission denied: File {} is inaccecible.",
            style("[IO Error]").with(Color::Red),
            style(path.display()).with(Color::Blue)
        ),
        io::ErrorKind::AddrInUse | io::ErrorKind::AddrNotAvailable => println!(
            "{} File {} is unavaiable. Try closing any application that may be using it.",
            style("[IO Error]").with(Color::Red),
            style(path.display()).with(Color::Blue)
        ),
        io::ErrorKind::InvalidData | io::ErrorKind::InvalidInput => println!(
            "{} File {} couln't be parsed. This file may be corrupted.",
            style("[IO Error]").with(Color::Red),
            style(path.display()).with(Color::Blue)
        ),
        _ => panic!("{:?}", err)
    }
}
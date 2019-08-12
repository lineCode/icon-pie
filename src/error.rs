use std::{io, path::PathBuf, ffi::OsString};

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
    SizeAlreadyIncluded(usize, usize)
}

impl Error {
    pub fn exit_with(&self, args: Vec<OsString>) -> io::Error {
        match &self {
            Error::Syntax(err)    => show::syntax(err, args),
            Error::IconBaker(err) => show::icon_baker(err),
            Error::Io(err, path)  => show::io(err, path.clone())
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

mod show {
    use std::{io, path::PathBuf, ffi::OsString};
    use super::SyntaxError;
    use crossterm::{style, Color};

    const VALID_ICNS_SIZES: &str = "16x16, 32x32, 64x64, 128x128, 512x512 and 1024x1024";

    pub fn syntax(err: &SyntaxError, args: Vec<OsString>) {
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
            SyntaxError::SizeAlreadyIncluded(a, b)
                => size_already_included(*a, *b, args)
        }
    }
    
    pub fn icon_baker(err: &icon_baker::Error) {
        match err {
            icon_baker::Error::InvalidIcnsSize(size) => println!(
                "{0} The {1} file format only supports {2} icons: {3}x{3} icons aren't supported.",
                style("[Icns Error]").with(Color::Red),
                style(".icns").with(Color::Blue),
                VALID_ICNS_SIZES,
                size
            ),
            icon_baker::Error::InvalidIcoSize(size) => println!(
                "{0} The {1} file format only supports icons of dimensions up to 256x256: {2}x{2} icons aren't supported.",
                style("[Ico Error]").with(Color::Red),
                style(".ico").with(Color::Blue),
                size
            ),
            icon_baker::Error::SizeAlreadyIncluded(_) =>
                unreachable!("This error should have been scaped in an earlier stage."),
            icon_baker::Error::Io(_) => unreachable!(),
            _ => panic!("{:?}", err)
        }
    }
    
    pub fn io(err: &io::Error, path: PathBuf) {
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

    fn size_already_included(a: usize, b: usize, args: Vec<String>) {
        let (fst, scn) = if a < b { (a, b) } else { (b, a) };

        println!(
            /* TODO Implement this properly */
            "{} The same size is binded to multiple sources: {} {} {} {} {}",
            style("[Syntax Error]").with(Color::Red),
            style(args[..fst].join(" ")).with(Color::Blue),
            style(args[fst].clone()).with(Color::Red),
            style(args[(fst + 1)..scn].join(" ")).with(Color::Blue),
            style(args[scn].clone()).with(Color::Red),
            style(args[(scn + 1)..].join(" ")).with(Color::Blue),
        );
    }
}
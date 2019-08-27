use std::{io, error::Error, path::PathBuf};
use crate::Output;
use super::SyntaxError;
use crossterm::{style, Color};

pub fn syntax(err: &SyntaxError) {
    let args = crate::args();

    match err {
        SyntaxError::UnexpectedToken(err_c) => println!(
            "{} {} {} {} {}",
            style("[Unexpected Token]").with(Color::Red),
            style("icon-baker").with(Color::Blue),
            style(args[..*err_c].join(" ")).with(Color::Blue),
            style(args[*err_c].clone()).with(Color::Red),
            style(args[(*err_c + 1)..].join(" ")).with(Color::Blue)
        ),
        SyntaxError::UnexpectedEnd => println!(
            "{} {} {} {}\nType {} for more details on IconBaker's usage.",
            style("[Expected Additional Tokens]").with(Color::Red),
            style("icon-baker").with(Color::Blue),
            style(args.join(" ")).with(Color::Blue),
            style("▂").with(Color::Red),
            style("icon-baker -h").with(Color::Blue)
        )
    }
}

pub fn icon_baker(err: &icon_baker::Error) {
    if let icon_baker::Error::InvalidSize(size) = err {
        println!(
            "{0} The specified file format does not support {1}x{1} icons.",
            style("[Invalid Size]").with(Color::Red),
            size
        );
    } else {
        println!("{} {}", style("[Unknown Error]").with(Color::Red), err.description());
    }
}

pub fn io(err: &io::Error, out: Output) {
    match out {
        Output::Path(path) => io_path(err, path),
        Output::Stdout     => io_stdout(err)
    }
}

fn io_path(err: &io::Error, path: PathBuf) {
    match err.kind() {
        io::ErrorKind::NotFound => println!(
            "{} File {} could not be found on disk.",
            style("[IO Error]").with(Color::Red),
            style(path.display()).with(Color::Blue)
        ),
        io::ErrorKind::PermissionDenied => println!(
            "{} Permission denied: File {} is inaccessible.",
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
        _ => println!(
            "{} {}.",
            style("[IO Error]").with(Color::Red),
            err.description()
        )
    }
}

fn io_stdout(err: &io::Error) {
    unimplemented!("{:?}", err);
}
extern crate icon_baker;
extern crate crossterm;

mod parse;
mod eval;
mod error;

use std::{env, io, path::{PathBuf}, ffi::OsString, collections::HashMap};
use error::Error;
use icon_baker::{IconType, Size};
use crossterm::{style, Color};

pub enum Command {
    Help,
    Version,
    Icon(HashMap<Size, (PathBuf, bool)>, IconType, Output)
}

#[derive(Clone, Debug)]
pub enum Output {
    Path(PathBuf),
    Stdout
}

const VERSION: &str = "0.2.1-beta";
const TITLE: &str = r"
 __  ___  __   __ _  ____   __   __ _  ____  ____ 
(  )/ __)/  \ (  ( \(  _ \ / _\ (  / )(  __)(  _ \
 )(( (__(  O )/    / ) _ (/    \ )  (  ) _)  )   /
(__)\___)\__/ \_)__)(____/\_/\_/(__\_)(____)(__\_)";
const USAGE: &str = "icon-baker ((-e <file path> <size>... [-i | --interpolate])... (-ico | -icns | -png) [<output path>]) | -h | --help | -v | --version";
const EXAMPLES: [&str;3] = [
    "icon-baker -e small.svg 16 20 24 -e big.png 32 64 -ico output.ico",
    "icon-baker -e image.png 32 64 48 -i -png output.zip",
    "icon-baker -e image.jpeg 32 64 128 -i -icns"
];

const COMMANDS: [&str;7] = [
    "Specify an entries options.",
    "Outputs to a .ico file.",
    "Outputs to a .icns file.",
    "Outputs a .png sequence as a .zip file.",
    "Help.",
    "Display version information.",
    "Apply linear interpolation when resampling the image."
];

#[macro_export]
macro_rules! syntax {
    ($err:expr) => { Err(Error::Syntax($err)) };
}

fn main() -> Result<(), std::io::Error> {
    let args: Vec<OsString> = env::args_os().collect();

    match parse::args(args.clone()) {
        Ok(cmd) => match cmd {
            Command::Icon(entries, icon_type, output) => if let Err(err) = eval::icon(&entries, icon_type, output.clone()) {
                Err(err.exit_with(args))
            } else {
                if let Output::Path(path) = output {
                    println!(
                        "{} File {} saved at {}",
                        style("[IconBaker]").with(Color::Green),
                        style(path.file_name().unwrap_or_default().to_string_lossy()).with(Color::Blue),
                        style(path.canonicalize().unwrap_or(env::current_dir().unwrap()).display()).with(Color::Blue)
                    );
                }

                Ok(())
            },
            Command::Help => help(),
            Command::Version => version()
        },
        Err(err)  => Err(err.exit_with(args))
    }
}

fn help() -> Result<(), io::Error> {
    println!(
        "{}\n{}",
        style(TITLE).with(Color::Green),
        style(VERSION).with(Color::Green)
    );

    println!("\n{}   {}\n\n{}{}\n{}{}\n{}{}\n{}{}\n{}{}\n{}{}\n{}{}",
        style("Usage:").with(Color::Blue),
        style(USAGE).with(Color::Green),
        style("   -e (<options>)      ").with(Color::Green),
        COMMANDS[0],
        style("   -ico <output path>  ").with(Color::Green),
        COMMANDS[1],
        style("   -icns <output path> ").with(Color::Green),
        COMMANDS[2],
        style("   -png <output path>  ").with(Color::Green),
        COMMANDS[3],
        style("   -h, --help          ").with(Color::Green),
        COMMANDS[4],
        style("   -v, --version       ").with(Color::Green),
        COMMANDS[5],
        style("   -i, --interpolate   ").with(Color::Green),
        COMMANDS[6]
    );

    println!("\n{}\n   {}\n   {}\n   {}\n",
        style("Examples:").with(Color::Blue),
        style(EXAMPLES[0]).with(Color::Green),
        style(EXAMPLES[1]).with(Color::Green),
        style(EXAMPLES[2]).with(Color::Green)
    );

    Ok(())
}

fn version() -> Result<(), io::Error> {
    println!("icon-baker v{}", VERSION);
    Ok(())
}
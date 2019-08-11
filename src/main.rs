extern crate icon_baker;
extern crate crossterm;
extern crate regex;
#[macro_use]
extern crate lazy_static;

mod parse;
mod error;

use std::{env, io, path::{Path, PathBuf}, ffi::OsString, collections::HashMap};
use error::Error;
use icon_baker::{Entry, IconType};
use crossterm::{style, Color};

pub enum Command {
    Help,
    Version,
    Icon(HashMap<Entry, PathBuf>, IconType, PathBuf)
}

const VERSION: &str = "0.2.0-beta";
const TITLE: &str = r"
 __  ___  __   __ _  ____   __   __ _  ____  ____ 
(  )/ __)/  \ (  ( \(  _ \ / _\ (  / )(  __)(  _ \
 )(( (__(  O )/    / ) _ (/    \ )  (  ) _)  )   /
(__)\___)\__/ \_)__)(____/\_/\_/(__\_)(____)(__\_)";
const USAGE: &str = "icon-baker (-e <file path> <size>... [-i | --interpolate] [-p | --proportional])... (-ico <output path> | -icns <output path> | -png <output path>) | -h | --help | -v | --version";
const EXAMPLES: [&str;2] = [
    "icon-baker -e small.svg 16 20 24 -e big.png 32 64 -ico output.ico",
    "icon-baker -e image.png 32x12 64x28 48 -i -png output.zip"
];

const COMMANDS: [&str;6] = [
    "Specify an entries options.",
    "Outputs to a .ico file.",
    "Outputs to a .icns file.",
    "Outputs a .png sequence as a .zip file.",
    "Help.",
    "Display version information."
];

const OPTIONS:  [&str;3] = [
    "Apply linear interpolation when resampling the image.",
    "Preserves the aspect ratio of the image in the output.",
    "This option is only valid when outputting to png sequences."
];

macro_rules! catch {
    ($err:expr, $p:expr) => {
        match $err {
            Ok(()) => Ok(()),
            Err(icon_baker::Error::Io(err)) => Err(Error::Io(err, $p)),
            Err(err) => Err(Error::IconBaker(err))
        }
    };
}

fn main() -> Result<(), std::io::Error> {
    let args: Vec<OsString> = env::args_os().collect();

    match parse::args(args.clone()) {
        Ok(cmd) => match cmd {
            Command::Icon(entries, icon_type, output_path) => if let Err(err) = eval::icon(&entries, icon_type, &output_path) {
                Err(err.exit_with(args))
            } else {
                let path = Path::new(&output_path);
                println!(
                    "{} File {} saved at {}",
                    style("[IconBaker]").with(Color::Green),
                    style(path.file_name().unwrap_or_default().to_string_lossy()).with(Color::Blue),
                    style(path.canonicalize().unwrap_or(env::current_dir().unwrap()).display()).with(Color::Blue)
                );

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

    println!("\n{}   {}\n\n{}{}\n{}{}\n{}{}\n{}{}\n{}{}\n{}{}",
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
        COMMANDS[5]
    );

    println!("\n{}\n{}{}\n{}{}\n                       {}",
        style("Options:").with(Color::Blue),
        style("   -i, --interpolate   ").with(Color::Green),
        OPTIONS[0],
        style("   -p, --proportional  ").with(Color::Green),
        OPTIONS[1],
        OPTIONS[2]
    );

    println!("\n{}\n   {}\n   {}\n",
        style("Examples:").with(Color::Blue),
        style(EXAMPLES[0]).with(Color::Green),
        style(EXAMPLES[1]).with(Color::Green)
    );

    Ok(())
}

fn version() -> Result<(), io::Error> {
    println!("icon-baker v{}", VERSION);
    Ok(())
}

mod eval {
    use std::{io, fs, path::PathBuf, collections::HashMap};
    use crate::error::Error;
    use icon_baker::{Icon, Entry, IconType, SourceImage, FromPath};

    pub fn icon(entries: &HashMap<Entry, PathBuf>, icon_type: IconType, output_path: &PathBuf) -> Result<(), Error> {
        let mut source_map = HashMap::with_capacity(entries.len());
    
        for path in entries.values() {
            if let None = source_map.get(path) {
                if let Some(source) = SourceImage::from_path(path) {
                    source_map.insert(path, source);
                } else {
                    return Err(Error::Io(io::Error::from(io::ErrorKind::NotFound), path.clone()));
                }
            }
        }
    
        let mut icon = Icon::new(icon_type, source_map.len());
        for (opts, path) in entries {
            if let Err(err) = icon.add_entry(opts.clone(), source_map.get(path)
                .expect("Variable 'source_map' should have a key for String 'path'")) {
                return catch!(Err(err), path.clone());
            }
        }
    
        match fs::File::create(output_path) {
            Ok(file) => catch!(icon.write(file), output_path.clone()),
            Err(err) => Err(Error::Io(err, output_path.clone()))
        }
    } 
}
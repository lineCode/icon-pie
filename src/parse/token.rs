use std::path::PathBuf;
use crate::ResamplingFilter;
use icon_baker::Size;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Flag(Flag),
    Path(PathBuf),
    Size(Size),
    Filter(ResamplingFilter)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Flag {
    Entry,
    Ico,
    Icns,
    Png,
    Help,
    Version,
    Resample
}

impl<'a> From<&'a str> for Token {
    fn from(s: &str) -> Self {
        match s {
            "-e"      => Token::Flag(Flag::Entry),
            "-ico"    => Token::Flag(Flag::Ico),
            "-icns"   => Token::Flag(Flag::Icns),
            "-png"    => Token::Flag(Flag::Png),
            "-r"      => Token::Flag(Flag::Resample),
            "nearest" => Token::Filter(ResamplingFilter::Nearest),
            "linear"  => Token::Filter(ResamplingFilter::Linear),
            "cubic"   => Token::Filter(ResamplingFilter::Cubic),
            "-h" | "--help"        => Token::Flag(Flag::Help),
            "-v" | "--version"     => Token::Flag(Flag::Version),
            _ => if let Ok(size) = s.parse::<u32>() {
                Token::Size(size)
            } else {
                let mut p = PathBuf::new();
                p.push(s);
            
                Token::Path(p)
            }
        }
    }
}

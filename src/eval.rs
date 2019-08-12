use std::{io::{self, Write, stdout}, fs, path::PathBuf, collections::HashMap};
use crate::{error::Error, Output};
use icon_baker::{Icon, IconType, SourceImage, FromPath, Size};

pub fn icon(
    entries: &HashMap<Size, (PathBuf, bool)>,
    icon_type: IconType,
    output: Output
) -> Result<(), Error> {
    let mut source_map = HashMap::with_capacity(entries.len());

    for (path, _) in entries.values() {
        if let None = source_map.get(path) {
            if let Some(source) = SourceImage::from_path(path) {
                source_map.insert(path, source);
            } else {
                return Err(Error::Io(io::Error::from(io::ErrorKind::NotFound), path.clone()));
            }
        }
    }

    let mut icon = Icon::new(icon_type, source_map.len());
    for (&size, (path, _)) in entries {
        match icon.add_size(
            size,
            source_map.get(path)
                .expect("Variable 'source_map' should have a key for String 'path'")
        ) {
            Ok(()) => continue,
            Err(icon_baker::Error::SizeAlreadyIncluded(_)) 
                => unreachable!("This error should have been scaped in an earlier stage."),
            Err(icon_baker::Error::Io(err)) => return Err(Error::Io(err, path.clone())),
            Err(err) => return Err(Error::IconBaker(err))
        }
    }

    match output {
        Output::Path(path) => match fs::File::create(path.clone()) {
            Ok(file) => write(file, &icon, entries),
            Err(err) => Err(Error::Io(err, path))
        },
        Output::Stdout => write(stdout(), &icon, entries)
    }
}

fn write<W: Write>(
    w: W,
    icon: &Icon,
    entries: &HashMap<Size, (PathBuf, bool)>
) -> Result<(), Error> {
    icon.write(w,
        |src, size| match entries.get(&size) {
            Some((_, true)) => icon_baker::resample::linear(src, size),
            _ => icon_baker::resample::nearest_neighbor(src, size)
        }
    ).map_err(|err| Error::IconBaker(err))
}
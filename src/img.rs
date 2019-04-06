extern crate image;
extern crate zip;
extern crate png_encode_mini;
extern crate nsvg;
extern crate ico;
extern crate icns;

use std::{fs::{self, Metadata}, io, path::Path, collections::HashMap};
use image::{imageops, FilterType, DynamicImage, RgbaImage, GenericImageView, ImageError};
use super::{Error, SyntaxError, parse};

#[derive(Clone, Debug, PartialEq)]
pub enum OutputType {
    PngSequence,
    Ico,
    Icns
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FitType {
    Proportional,
    Exact
}

pub struct Command {
    files: Vec<parse::File>,
    output: String
}

impl Command {
    pub fn new(files: Vec<parse::File>, output: String) -> Command {
        Command {files, output}
    }

    pub fn output_path(&self) -> String {
        self.output.clone()
    }

    fn output_type(&self) -> Result<OutputType, Error> {
        match parse::ext(&self.output) {
            Some(ext) => {
                match ext.as_ref() {
                    "zip"  => Ok(OutputType::PngSequence),
                    "ico"  => Ok(OutputType::Ico),
                    "icns" => Ok(OutputType::Icns),
                    ext    => Err(Error::Syntax(SyntaxError::UnsupportedOutputType(String::from(ext))))
                }
            },
            None => Err(Error::Syntax(SyntaxError::UnknownOutputType))
        }
    }

    fn n_sizes(&self) -> Result<usize, Error> {
        let n_sizes = self.files.iter().map(|file| file.n_sizes()).fold(0, |sum, a| sum + a);
        let mut map: HashMap<(u16, u16), String> = HashMap::with_capacity(n_sizes);
        
        for file in &self.files {
            for (w, h) in file.sizes() {
                if let Some(other) = map.get(&(w, h)) {
                    return Err(Error::Syntax(SyntaxError::SizeReassignment((w, h, other.clone(), file.path().clone()))));
                } else {
                    map.insert((w, h), file.path().clone());
                }
            }
        }

        Ok(map.len())
    }

    fn images(&self) -> Result<Vec<RgbaImage>, Error> {
        let out = self.output_type()?;
        let mut bufs = Vec::with_capacity(self.n_sizes()?);

        for file in &self.files {
            if Some(String::from("svg")) == parse::ext(file.path()) {
                for (w, h) in file.sizes() {
                    if (out == OutputType::Ico || out == OutputType::Icns) && file.fit_type == FitType::Proportional && w != h {
                        return Err(Error::Syntax(SyntaxError::InvalidProportionalFlag((w, h))));
                    }

                    match nsvg::parse_file(Path::new(file.path()), nsvg::Units::Pixel, 96.0) {

                        Ok(svg) => match svg.rasterize_to_raw_rgba(f32::from(w) / svg.width()) {
                            Ok((width, height, buf)) => if let Some(rgba) = RgbaImage::from_raw(width, height, buf) {
                                if file.fit_type == FitType::Exact && (w != width as u16 || h != height as u16) {
                                    let resized = resize(&DynamicImage::from(image::ImageRgba8(rgba)), w as u32, h as u32, file.filter_type, file.fit_type);
                                    bufs.push(resized.to_rgba());
                                } else {
                                    bufs.push(rgba);
                                }
                            } else {
                                /* TODO implement error handling here */ unimplemented!();
                            },
                            Err(_err) => /* TODO implement error handling here */ unimplemented!()
                        },
                        Err(_) => return Err(Error::Io((io::Error::from(io::ErrorKind::InvalidInput), Some(file.path().clone()))))
                    }
                }
            } else {
                match image::open(file.path()) {
                    Ok(img) => {
                        for (w, h) in file.sizes() {
                            if (out == OutputType::Ico || out == OutputType::Icns) && file.fit_type == FitType::Proportional && w != h {
                                return Err(Error::Syntax(SyntaxError::InvalidProportionalFlag((w, h))));
                            }

                            if (w as u32) < img.width() || (h as u32) < img.height() {
                                if let FilterType::Nearest = file.filter_type {
                                    return Err(Error::Syntax(SyntaxError::InvalidDownsizingOpts((w, h))));
                                }
                            }

                            let resized = resize(&img, w as u32, h as u32, file.filter_type, file.fit_type);
                            bufs.push(resized.to_rgba());
                        }
                    },
                    Err(err) => {
                        match err {
                            ImageError::IoError(err) => return Err(Error::Io((err, Some(file.path().clone())))),
                            _ => return Err(Error::Image(err))
                        }
                    }
                }
            }
        }

        Ok(bufs)
    }

    pub fn exec(&self) -> Result<Metadata, Error> {
        let out = self.output_type()?;
        let bufs = self.images()?;

        match fs::File::create(&self.output) {
            Ok(file) => {
                match out {
                    OutputType::PngSequence => encode::png_sequence(&bufs, &file),
                    OutputType::Ico => encode::ico(&bufs, &file),
                    OutputType::Icns => encode::icns(&bufs, &file)
                }
            },
            Err(err) => Err(Error::Io((err, Some(self.output.clone()))))
        }
    }
}

mod encode {
    use std::{fs::{self, Metadata}, io::{self, Write}};
    use image::RgbaImage;
    use super::Error;

    pub fn png_sequence(bufs: &Vec<RgbaImage>, file: &fs::File) -> Result<Metadata, Error> {
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
        
        for buf in bufs {
            let (w, h) = buf.dimensions();

            // Encode the pixel data as PNG and store it in a Vec<u8>
            let mut data = Vec::with_capacity(buf.len() + 7);
            if let Err(err) = png_encode_mini::write_rgba_from_u8(&mut data, &buf.clone().into_raw(), w, h) {
                return Err(Error::Io((err, None)));
            }

            let file_name = if w == h {
                format!("{}.png", w)
            } else {
                format!("{}x{}.png", w, h)
            };

            if let Err(_err) = zip.start_file(file_name, options) {
                // TODO Return an error
                unimplemented!();
            }

            if let Err(_err) = zip.write_all(&data[..]) {
                // TODO Return an error
                unimplemented!();
            }
        }

        if let Err(_err) = zip.finish() {
            // TODO Return an error
            unimplemented!();
        } else {
            match file.metadata() {
                Ok(meta) => Ok(meta),
                Err(err) => Err(Error::Io((err, None)))
            }
        }
    }

    pub fn ico(bufs: &Vec<RgbaImage>, file: &fs::File) -> Result<Metadata, Error> {
        let mut output = ico::IconDir::new(ico::ResourceType::Icon);

        for buf in bufs {
            let (w, h) = buf.dimensions();
            let data = ico::IconImage::from_rgba_data(w, h, buf.clone().into_vec());

            output.add_entry(ico::IconDirEntry::encode(&data).unwrap());
        }

        match output.write(file) {
            Ok(_) => {
                match file.metadata() {
                    Ok(meta) => Ok(meta),
                    Err(err) => Err(Error::Io((err, None)))
                }
            },
            Err(err) => Err(Error::Io((err, None)))
        }
    }

    pub fn icns(bufs: &Vec<RgbaImage>, file: &fs::File) -> Result<Metadata, Error> {
        let mut output = icns::IconFamily::new();

        for buf in bufs {
            let (w, h) = buf.dimensions();

            match icns::Image::from_data(icns::PixelFormat::RGBA, w, h, buf.clone().into_vec()) {
                Ok(icon) => {
                    if let Err(err) = output.add_icon(&icon) {
                        return Err(Error::Io((err, None)))
                    }
                },
                Err(err) => return Err(Error::Io((err, None)))
            }
        }

        let buf_writer = io::BufWriter::new(file);
        match output.write(buf_writer) {
            Ok(_) => {
                match file.metadata() {
                    Ok(meta) => Ok(meta),
                    Err(err) => Err(Error::Io((err, None)))
                }
            },
            Err(err) => Err(Error::Io((err, None)))
        }
    }
}

fn resize(source: &DynamicImage, w: u32, h: u32, filter: FilterType, fit: FitType) -> DynamicImage {
    let mut img = source.clone();
    img = img.resize(w, h, filter);

    match fit {
        FitType::Proportional => img,
        FitType::Exact => {
            if img.width() == w && img.height() == h {
                img
            } else {
                let mut output = DynamicImage::new_rgba8(w, h);
                let dx = (output.width() - img.width()) / 2;
                let dy = (output.height() - img.height()) / 2;

                imageops::overlay(&mut output, &img, dx, dy);
                output
            }
        }
    }
}
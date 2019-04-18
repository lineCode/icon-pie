extern crate zip;
extern crate png_encode_mini;
extern crate nsvg;
extern crate ico;
extern crate icns;

use std::{fs::{self, Metadata}, collections::HashMap};
use super::{Size, Error, EvalError, SyntaxError, parse};

const MAX_ICO_SIZE: u16 = 256;
const VALID_ICNS_SIZES: [u16;7] = [16, 32, 64, 128, 256, 512, 1024];

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OutputType {
    PngSequence,
    Ico,
    Icns
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FitType {
    Proportional,
    Strict
}

#[derive(Clone)]
pub enum Command {
    Help,
    Encode(EncodeCommand)
}

impl Command {
    pub fn encode(files: Vec<parse::File>, output_path: String, output_type: OutputType) -> Command {
        Command::Encode(EncodeCommand {files, output_path, output_type})
    }
}

#[derive(Clone)]
pub struct EncodeCommand {
    files: Vec<parse::File>,
    output_path: String,
    output_type: OutputType
}

impl EncodeCommand {
    pub fn output_path(&self) -> String {
        self.output_path.clone()
    }

    fn n_sizes(&self) -> Result<usize, Error> {
        let n_sizes = self.files.iter().map(|file| file.n_sizes()).fold(0, |sum, a| sum + a);
        let mut map: HashMap<Size, String> = HashMap::with_capacity(n_sizes);
        
        for file in &self.files {
            for (w, h) in file.sizes() {
                if let Some(_other) = map.get(&(w, h)) {
                    return Err(Error::Syntax(SyntaxError::SizeReassignment((w, h))));
                } else {
                    map.insert((w, h), file.path().clone());
                }
            }
        }

        Ok(map.len())
    }

    fn validate(&self) -> Option<Error> {
        match self.output_type {
            OutputType::Ico => {
                for file in self.files.clone() {
                    for (w, h) in file.sizes() {
                        if w > MAX_ICO_SIZE || h > MAX_ICO_SIZE || w != h {
                            return Some(Error::Eval(EvalError::InvalidIcoSize((w, h))));
                        }
                    }
                }

                None
            },
            OutputType::Icns => {
                for file in self.files.clone() {
                    for (w, h) in file.sizes() {
                        if !VALID_ICNS_SIZES.contains(&w) || !VALID_ICNS_SIZES.contains(&h) || w != h {
                            return Some(Error::Eval(EvalError::InvalidIcnsSize((w, h))));
                        }
                    }
                }

                None
            },
            _ => None
        }
    }

    pub fn exec(&self) -> Result<Metadata, Error> {

        if let Some(err) = self.validate() {
            return Err(err);
        }

        let mut images = Vec::with_capacity(self.n_sizes()?);

        for file in self.files.clone() {
            let mut file_images = scale::file(&file)?;
            images.append(&mut file_images);
        }

        match fs::File::create(&self.output_path) {
            Ok(file) => match self.output_type {
                OutputType::PngSequence => encode::png_sequence(&images, &file, &self.output_path),
                OutputType::Ico => encode::ico(&images, &file, &self.output_path),
                OutputType::Icns => encode::icns(&images, &file, &self.output_path)
            },
            Err(err) => Err(Error::Io(err, self.output_path()))
        }
    }
}

mod encode {
    use std::{fs::{self, Metadata}, io::{self, Write}};
    use nsvg::image::RgbaImage;
    use super::Error;

    pub fn png_sequence(bufs: &Vec<RgbaImage>, file: &fs::File, path: &String) -> Result<Metadata, Error> {
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
        
        for buf in bufs {
            let (w, h) = buf.dimensions();

            // Encode the pixel data as PNG and store it in a Vec<u8>
            let mut data = Vec::with_capacity(buf.len());
            if let Err(err) = png_encode_mini::write_rgba_from_u8(&mut data, &buf.clone().into_raw(), w, h) {
                return Err(Error::Io(err, path.clone()));
            }

            let file_name = if w == h {
                format!("{}.png", w)
            } else {
                format!("{}x{}.png", w, h)
            };

            if let Err(err) = zip.start_file(file_name, options) {
                panic!(err);
            }

            if let Err(err) = zip.write_all(&data[..]) {
                panic!(err);
            }
        }

        if let Err(err) = zip.finish() {
            panic!(err);
        } else {
            match file.metadata() {
                Ok(meta) => Ok(meta),
                Err(err) => Err(Error::Io(err, path.clone()))
            }
        }
    }

    pub fn ico(bufs: &Vec<RgbaImage>, file: &fs::File, path: &String) -> Result<Metadata, Error> {
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
                    Err(err) => Err(Error::Io(err, path.clone()))
                }
            },
            Err(err) => Err(Error::Io(err, path.clone()))
        }
    }

    pub fn icns(bufs: &Vec<RgbaImage>, file: &fs::File, path: &String) -> Result<Metadata, Error> {
        let mut output = icns::IconFamily::new();

        for buf in bufs {
            let (w, h) = buf.dimensions();

            match icns::Image::from_data(icns::PixelFormat::RGBA, w, h, buf.clone().into_vec()) {
                Ok(icon) => if let Err(err) = output.add_icon(&icon) {
                    return Err(Error::Io(err, path.clone()))
                },
                Err(err) => return Err(Error::Io(err, path.clone()))
            }
        }

        let buf_writer = io::BufWriter::new(file);
        match output.write(buf_writer) {
            Ok(_) => {
                match file.metadata() {
                    Ok(meta) => Ok(meta),
                    Err(err) => Err(Error::Io(err, path.clone()))
                }
            },
            Err(err) => Err(Error::Io(err, path.clone()))
        }
    }
}

mod scale {
    use std::{io, path::Path};
    use nsvg::image::{self, imageops, FilterType, DynamicImage, RgbaImage, ImageError, GenericImage};
    use super::{parse::{self, File}, FitType, Error, EvalError};

    pub fn file(file: &File) -> Result<Vec<RgbaImage>, Error> {
        let sizes = file.sizes();
        let mut images = Vec::with_capacity(sizes.len());

        match parse::ext(file.path()).unwrap_or_default().as_ref() {
            "svg" => if let Ok(svg) =  nsvg::parse_file(Path::new(file.path()), nsvg::Units::Pixel, 96.0) {
                for (w, h) in sizes {
                    match svg.rasterize(f32::from(w) / svg.width()) {
                        Ok(buf) => if file.fit_type == FitType::Strict && (w as u32 != buf.width() || h as u32 != buf.height()) {
                            let din = DynamicImage::ImageRgba8(buf);
                            let reframed = reframe(&din, w as u32, h as u32);

                            images.push(reframed);
                        } else {
                            images.push(buf);
                        },
                        Err(err) => match err {
                            nsvg::Error::IoError(err) => return Err(Error::Io(err, file.path().clone())),
                            nsvg::Error::ParseError => return Err(Error::Io(io::Error::from(io::ErrorKind::InvalidInput), file.path().clone())),
                            err => panic!(err)
                        }
                    }
                }
            } else {
                return Err(Error::Io(io::Error::from(io::ErrorKind::InvalidInput), file.path().clone()))
            },
            _ => match image::open(file.path()) {
                Ok(img) => for (w, h) in file.sizes() {
                    if (w as u32) < img.width() || (h as u32) < img.height() {
                        if let FilterType::Nearest = file.filter_type {
                            let dim = img.dimensions();
                            let ext = parse::ext(file.path()).unwrap_or(String::from("*"));

                            return Err(Error::Eval(EvalError::IlligalDownsizing((w, h), dim, ext)));
                        }
                    }

                    let reframed = reframe(&img.resize(w as u32, h as u32, file.filter_type), w as u32, h as u32);
                    images.push(reframed);
                },
                Err(err) => match err {
                    ImageError::IoError(err) => return Err(Error::Io(err, file.path().clone())),
                    _ => panic!("{:?}", err)
                }
            }
        }

        Ok(images)
    }

    fn reframe(source: &DynamicImage, w: u32, h: u32) -> RgbaImage {
        if source.width() == w && source.height() == h {
            source.to_rgba()
        } else {
            let mut output = DynamicImage::new_rgba8(w, h);
            let dx = (output.width() - source.width()) / 2;
            let dy = (output.height() - source.height()) / 2;

            imageops::overlay(&mut output, &source, dx, dy);
            output.to_rgba()
        }
    }
}
# IconBaker

A simple command line tool for generating `.ico` and `.icns` icons.

## Intelligent Re-sampling
**Icon Baker** uses _[nearest-neighbor re-sampling](https://en.wikipedia.org/wiki/Nearest-neighbor_interpolation)_ by default, avoiding blurred edges when up-sizing small bitmap sources:

<img width="480px" height="248px" src="image1.png">

Furthermore, **Icon Baker** only up-sizes bitmap sources on an integer scale, filling the leftover pixels with a transparent border and providing pixel-perfect quality:

<img width="358px" height="222px" src="image2.png">

If you want your image to be linear interpolated, simply add the `-i` option to it's entry.

## Supported Image Formats
| Format | Supported?                                                                                 | 
| ------ | -------------------------------------------------------------------------------------------| 
| `PNG`  | All supported color types                                                                  | 
| `JPEG` | Baseline and progressive                                                                   | 
| `GIF`  | Yes                                                                                        | 
| `BMP`  | Yes                                                                                        | 
| `ICO`  | Yes                                                                                        | 
| `TIFF` | Baseline(no fax support), `LZW`, PackBits                                                  | 
| `WEBP` | Lossy(Luma channel only)                                                                   | 
| `PNM ` | `PBM`, `PGM`, `PPM`, standard `PAM`                                                        |
| `SVG`  | Limited([flat filled shapes only](https://github.com/GarkGarcia/icon_baker#svg-support))   |

## Usage
The formal syntax for using **IconBaker** is as follows:

`icon-baker (-e <file path> <size>... [-i | --interpolate] [-p | --proportional])... (-o <output path> | -png <output path) | -h | -v`

### Flags

|Flag                |Description                                |
|--------------------|-------------------------------------------|
|`-e (<options>)`    |Specify an entrys options.                 |
|`-o <output path>`  |Outputs to `.ico` or `.icns` file.         |
|`-png <output path>`|Outputs a `.png` sequence as a `.zip` file.|
|`-h`                |Help.                                      |
|`-v`                |Display version information.               |

### Options
|Option                |Description                                                                                                      |
|----------------------|-----------------------------------------------------------------------------------------------------------------|
|`-i`, `--interpolate` |Apply linear interpolation when resampling the image.                                                            |
|`-p`, `--proportional`|Preserves the aspect ratio of the image in the output. This option is only valid when outputing to png sequences.|

## Examples
* `icon-baker -e small.svg 16 20 24 -e big.png 32 64 -o output.ico`
* `icon-baker -e image.png 32x12 64x28 48 -i -png output.zip`

## Limitations
**IconBaker** has two main limitations: both `ICNS` and `SVG` are not fully supported. Due to the use of external dependencies, this app's author is not able to fully support the formal specifications of those two file formats.

However, the coverage provided by this external dependencies should be enough for most use cases.

| OSType | Description                             | Supported? |
|--------|-----------------------------------------|------------|
| `ICON` | 32×32 1-bit icon                        | No         |
| `ICN#` | 32×32 1-bit icon with 1-bit mask        | No         |
| `icm#` | 16×12 1-bit icon with 1-bit mask        | No         |
| `icm4` | 16×12 4-bit icon                        | No         |
| `icm8` | 16×12 8-bit icon                        | No         |
| `ics#` | 16×16 1-bit mask                        | No         |
| `ics4` | 16×16 4-bit icon                        | No         |
| `ics8` | 16x16 8-bit icon                        | No         |
| `is32` | 16×16 24-bit icon                       | Yes        |
| `s8mk` | 16x16 8-bit mask                        | Yes        |
| `icl4` | 32×32 4-bit icon                        | No         |
| `icl8` | 32×32 8-bit icon                        | No         |
| `il32` | 32x32 24-bit icon                       | Yes        |
| `l8mk` | 32×32 8-bit mask                        | Yes        |
| `ich#` | 48×48 1-bit mask                        | No         |
| `ich4` | 48×48 4-bit icon                        | No         |
| `ich8` | 48×48 8-bit icon                        | No         |
| `ih32` | 48×48 24-bit icon                       | Yes        |
| `h8mk` | 48×48 8-bit mask                        | Yes        |
| `it32` | 128×128 24-bit icon                     | Yes        |
| `t8mk` | 128×128 8-bit mask                      | Yes        |
| `icp4` | 16x16 32-bit PNG/JP2 icon               | PNG only   |
| `icp5` | 32x32 32-bit PNG/JP2 icon               | PNG only   |
| `icp6` | 64x64 32-bit PNG/JP2 icon               | PNG only   |
| `ic07` | 128x128 32-bit PNG/JP2 icon             | PNG only   |
| `ic08` | 256×256 32-bit PNG/JP2 icon             | PNG only   |
| `ic09` | 512×512 32-bit PNG/JP2 icon             | PNG only   |
| `ic10` | 512x512@2x "retina" 32-bit PNG/JP2 icon | PNG only   |
| `ic11` | 16x16@2x "retina" 32-bit PNG/JP2 icon   | PNG only   |
| `ic12` | 32x32@2x "retina" 32-bit PNG/JP2 icon   | PNG only   |
| `ic13` | 128x128@2x "retina" 32-bit PNG/JP2 icon | PNG only   |
| `ic14` | 256x256@2x "retina" 32-bit PNG/JP2 icon | PNG only   |
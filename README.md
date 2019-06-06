## Usage
The formal syntax for using **IconBaker** is as follows:

`icon-baker (-e <file path> <size>... [-i | --interpolate] [-p | --proportional])... (-o <output path> | -png <output path`

Here's a comprehensive list on the commands that can be issued:

|Syntax              |Command                                    |
|--------------------|-------------------------------------------|
|`-e (<options>)`    |Specify an entrys options.                 |
|`-o <output path>`  |Outputs to `.ico` or `.icns` file.         |
|`-png <output path>`|Outputs a `.png` sequence as a `.zip` file.|

## Options
|Syntax                |Option                                                                                                           |
|----------------------|-----------------------------------------------------------------------------------------------------------------|
|`-i`, `--interpolate` |Apply linear interpolation when resampling the image.                                                            |
|`-p`, `--proportional`|Preserves the aspect ratio of the image in the output. This option is only valid when outputing to png sequences.|
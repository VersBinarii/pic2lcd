# pic2lcd
Command line tool for converting images to LCD friendly format

# Installation
```
git clone https://github.com/VersBinarii/pic2lcd
cd pic2lcd
cargo install --path .
```

# Usage
```
USAGE:
    pic2lcd [FLAGS] [OPTIONS] <image>
FLAGS:
        --force-resize    Perform resizing without preserving aspect ratio
        --help            Prints help information
    -V, --version         Prints version information
    -v, --verbose
OPTIONS:
        --array-name <data-array-name>     [default: data]
    -h, --height <height>                 Output image might be lower than specified value to preserve aspect ratio. Use
                                          --force-resize if you dont care about aspect ratio.
    -O, --out-format <out-format>          [default: Rgb565]  [possible values: Monochrome, Rgb565]
    -o, --output <output>                  [default: out.h]
    -w, --width <width>                   Output image might be lower than specified value to preserve aspect ratio. Use
                                          --force-resize if you dont care about aspect ratio.
ARGS:
    <image>
```

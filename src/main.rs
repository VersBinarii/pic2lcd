use clap::arg_enum;
use image::{
    imageops::FilterType, io::Reader as ImageReader, GenericImageView,
};
use std::path::PathBuf;
use structopt::StructOpt;

arg_enum! {
    #[derive(Debug)]
    enum OutputFormat {
        Monochrome,
        Rgb565,
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "im2lcd", about = "Convert image to LCD friendly format.")]
struct Args {
    // Image file to be converted
    image: PathBuf,

    #[structopt(long, short, default_value = "out.h")]
    output: PathBuf,

    #[structopt(long="out-format", short="O", possible_values=&OutputFormat::variants(), case_insensitive=true, default_value = "Rgb565")]
    out_format: OutputFormat,

    #[structopt(
        long = "force-resize",
        requires_all(&["width", "height"]),
        help = "Perform resizing without preserving aspect ratio"
    )]
    force_resize: bool,

    #[structopt(
        short = "w",
        long = "width",
        help = "Output image might be lower than specified value to preserve aspect ratio. Use --force-resize if you dont care about aspect ratio."
    )]
    width: Option<u32>,

    #[structopt(
        short = "h",
        long = "height",
        help = "Output image might be lower than specified value to preserve aspect ratio. Use --force-resize if you dont care about aspect ratio."
    )]
    height: Option<u32>,

    #[structopt(long = "array-name", default_value = "data")]
    data_array_name: String,

    #[structopt(long, short)]
    verbose: bool,
}

#[derive(Debug)]
enum Orientation {
    Portrait,
    Landscape,
}

impl Orientation {
    fn from_dimensions((width, height): (u32, u32)) -> Self {
        if width < height {
            Orientation::Portrait
        } else {
            Orientation::Landscape
        }
    }
}

impl std::fmt::Display for Orientation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Orientation::Portrait => write!(f, "Portrait"),
            Orientation::Landscape => write!(f, "Landscape"),
        }
    }
}

fn main() {
    let args = Args::from_args();

    macro_rules! log_message {
        ($msg:expr) => {
            if args.verbose {
                println!("{}", $msg);
            }
        };
    }

    let img = ImageReader::open(&args.image)
        .expect("Failed to open image")
        .decode()
        .expect("Failed to decode image content");

    // Do we need to do any resizing?
    let must_resize = if args.width.is_some() || args.height.is_some() {
        true
    } else {
        false
    };

    // If the image is too large resize it to specified dimensions
    let orientation = Orientation::from_dimensions(img.dimensions());
    let (resize_width, resize_height) = match orientation {
        Orientation::Portrait => {
            log_message!("Orientation: portrait");
            let w = args.width.unwrap_or(240);
            let h = args.height.unwrap_or(320);
            (w, h)
        }
        Orientation::Landscape => {
            log_message!("Orientation landscape");
            let w = args.width.unwrap_or(320);
            let h = args.height.unwrap_or(240);
            (w, h)
        }
    };

    let resized_image = if must_resize {
        log_message!("Resizing image");
        if args.force_resize {
            img.resize_exact(resize_width, resize_height, FilterType::Lanczos3)
        } else {
            img.resize_to_fill(
                resize_width,
                resize_height,
                FilterType::Lanczos3,
            )
        }
    } else {
        img
    };

    let (width, height) = resized_image.dimensions();
    log_message!(format!("Resized dimensions: {}x{}", width, height));

    let out_buffer = match &args.out_format {
        OutputFormat::Monochrome => {
            let mut mono_pixels = vec![];
            let mut bit_count: u8 = 0;
            let mut mono_byte: u8 = 0;
            for y in 0..height {
                for x in 0..width {
                    let pixel = resized_image.get_pixel(x, y);
                    if pixel[0] > 130 || pixel[1] > 130 || pixel[2] > 130 {
                        mono_byte &= !(1 << 7 - bit_count)
                    } else {
                        mono_byte |= 1 << 7 - bit_count;
                    }
                    bit_count += 1;
                    if bit_count == 8 {
                        mono_pixels.push(mono_byte);
                        bit_count = 0;
                    }
                }
            }
            mono_pixels
        }
        OutputFormat::Rgb565 => {
            let mut out_pixels = vec![];
            for pixel in resized_image.to_rgb8().pixels() {
                let red = pixel[0];
                let green = pixel[1];
                let blue = pixel[2];

                let byte_16: u16 = ((red & 0xf8) as u16) << 8
                    | ((green & 0xfc) as u16) << 3
                    | (blue as u16) >> 3;
                out_pixels.push((byte_16 >> 8) as u8 & 0xff);
                out_pixels.push(byte_16 as u8 & 0xff);
            }
            out_pixels
        }
    };

    std::fs::write(
        args.output,
        format!(
            "/* {0} - {1}x{2} - {6}*/
const uint8_t {3}[{4}] = {{{5}}};",
            orientation,
            width,
            height,
            args.data_array_name,
            out_buffer.len(),
            out_buffer
                .into_iter()
                .map(|b| format!("\n    0x{:x},", b))
                .collect::<String>(),
            args.out_format
        ),
    )
    .unwrap();
}

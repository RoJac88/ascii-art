use clap::Parser;
use image::{DynamicImage, GenericImageView};
use std::fmt;
use std::io::{self, stdin};
use std::{thread, time};

const DEFAULT_CHARS: &str = ". ' , ^ \" ~ + - = # @ $";

#[derive(Parser)]
#[command(name = "Ascii-Art")]
#[command(author = "Rod J. <rodrigo.jacob@gmail.com>")]
#[command(about = "Converts an image to ascii art", long_about = None)]
struct Args {
    #[arg(
        short,
        long,
        default_value = DEFAULT_CHARS,
        help = "provide a list of chars from least to most intense, separated by whitespace"
    )]
    chars: Option<String>,

    #[arg(
        short,
        long,
        help = "scale factor as a positive integer",
        default_value = "3"
    )]
    scale: Option<u32>,
    src: Option<String>,
}

#[derive(Debug)]
struct AppError {
    kind: String,
    detail: String,
}
impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err_msg = match self.kind.as_str() {
            "io" => format!("io error: {}", self.detail),
            "img" => format!("load img error: {}", self.detail),
            "conf" => format!("configuration error: {}", self.detail),
            _ => format!("Unknown error occured: {}", self.detail),
        };
        write!(f, "{}", err_msg)
    }
}
impl From<io::Error> for AppError {
    fn from(error: io::Error) -> Self {
        AppError {
            kind: String::from("io"),
            detail: error.to_string(),
        }
    }
}

#[derive(Debug)]
struct AsciiPrinter {
    src_img: Option<Result<DynamicImage, AppError>>,
    chars: Vec<char>,
    scale: u32,
}
impl Default for AsciiPrinter {
    fn default() -> Self {
        let mut chars: Vec<char> = vec![' '];
        chars.extend(DEFAULT_CHARS.replace(' ', "").chars());
        Self {
            src_img: None,
            chars,
            scale: 3,
        }
    }
}
impl AsciiPrinter {
    fn load_image(self, src: &str) -> Self {
        let src_img = Some(image::open(src).map_err(|e| AppError {
            kind: "img".to_string(),
            detail: e.to_string(),
        }));
        AsciiPrinter {
            src_img,
            chars: self.chars,
            scale: self.scale,
        }
    }
    fn set_chars(self, intense_chars: String) -> Self {
        let mut chars = Vec::with_capacity(intense_chars.len() + 1);
        chars.push(' ');
        chars.extend(intense_chars.chars());
        AsciiPrinter {
            src_img: self.src_img,
            chars,
            scale: self.scale,
        }
    }
    fn set_scale(self, scale: u32) -> Self {
        AsciiPrinter {
            src_img: self.src_img,
            chars: self.chars,
            scale,
        }
    }
    fn get_char(&self, intensity: f32) -> char {
        let index = intensity / (256_f32 / self.chars.len() as f32);
        self.chars[index as usize]
    }

    fn get_pixel_intensity(red: u8, green: u8, blue: u8) -> f32 {
        0.2989 * red as f32 + 0.5870 * green as f32 + 0.1140 * blue as f32
    }

    fn into_print(self) -> Result<(), AppError> {
        if self.src_img.is_none() {
            return Err(AppError {
                kind: "img".to_string(),
                detail: "no image selected".to_string(),
            });
        };
        let src = self.src_img.as_ref().unwrap();
        if src.is_err() {
            return Err(AppError {
                kind: "img".to_string(),
                detail: src.as_ref().unwrap_err().to_string(),
            });
        }
        let src = src.as_ref().unwrap();
        let (width, height) = src.dimensions();
        for y in 0..height {
            for x in 0..width {
                if y % (self.scale * 2) == 0 && x % self.scale == 0 {
                    let pix = src.get_pixel(x, y);
                    let intensity = Self::get_pixel_intensity(pix[0], pix[1], pix[2]);
                    let char = self.get_char(intensity);
                    print!("{}", char);
                }
            }
            if y % (self.scale * 2) == 0 {
                println!();
            }
        }
        println!();
        Ok(())
    }
}

fn read_stdin() -> Result<String, AppError> {
    let (tx, rx) = std::sync::mpsc::channel();
    thread::spawn(move || {
        let mut buffer = String::new();
        let stdin = stdin();
        stdin.read_line(&mut buffer).unwrap();
        let line: String = buffer.trim().into();
        tx.send(line).unwrap();
    });
    thread::sleep(time::Duration::from_millis(10));
    let line = rx.try_recv();
    if let Err(e) = line {
        println!("No image selected, run with --help for more info");
        return Err(AppError {
            kind: "io".to_string(),
            detail: e.to_string(),
        });
    }
    let line = line.unwrap();
    if line.is_empty() {
        Err(AppError {
            kind: "io".to_string(),
            detail: "stdin is empty".to_string(),
        })
    } else {
        Ok(line)
    }
}
fn main() -> Result<(), AppError> {
    let args = Args::parse();
    let image_path = match args.src.as_deref() {
        Some(src) => src.to_string(),
        None => read_stdin()?,
    };
    println!("selected img {}", image_path);
    let printer = AsciiPrinter::default();
    printer
        .load_image(&image_path)
        .set_chars(args.chars.unwrap())
        .set_scale(args.scale.unwrap())
        .into_print()
}

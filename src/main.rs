extern crate image;
extern crate clap;
extern crate png;

use clap::{Arg, ArgMatches, App};
use std::fs::File;
use std::path::Path;
use std::vec::Vec;
use image::tiff;

fn main() {
    let matches = App::new("png2tiff")
                    .version("0.1")
                    .arg(Arg::with_name("INPUT_FILES")
                            .short("i")
                            .long("input_files")
                            .required(true)
                            .multiple(true)
                            .takes_value(true))
                    .arg(Arg::with_name("OUTPUT_DIR")
                            .short("o")
                            .long("output_dir")
                            .takes_value(true)
                            .use_delimiter(false))
                    .get_matches();

    let (inputs, odir) = parse_args(&matches);

    for i in &inputs {
        let file_name = i.to_str().unwrap_or("[unknown]");
        let png_file = match File::open(i) {
            Ok(v) => v,
            Err(_) => panic!("failed to open file {}.", file_name),
        };
        let png_dec = png::Decoder::new(png_file);
        let (png_info, mut png_reader) = match png_dec.read_info() {
            Ok(v) => v,
            Err(_) => panic!("failed to read file {}.", file_name),
        };

        let mut buf: Vec<u8> = vec![0; png_info.buffer_size()];
        match png_reader.next_frame(&mut buf) {
            Ok(_) => (),
            Err(_) => panic!("failed to decode png file {}.", file_name),
        };

        println!("{}: {:?}", file_name, png_info.color_type);
        let pbuf = match odir {
            Some(d) => {
                let mut fp = i.to_path_buf();
                let mut dp = d.to_path_buf();
                fp.set_extension("tiff");
                dp.push(fp.file_name().unwrap_or_else(|| panic!("???")));
                dp
            },
            None => {
                let mut fp = i.to_path_buf();
                fp.set_extension("tiff");
                fp
            }
        };

        let tiff_file = match File::create(pbuf.as_path()) {
            Ok(v) => v,
            Err(_) => panic!("failed to create file {}.",
                        pbuf.as_path().to_str().unwrap_or("[unknown]")),
        };

        let enc = tiff::TiffEncoder::new(&tiff_file);
        match enc.encode(&buf, png_info.width, png_info.height, image::ColorType::Rgba8) {
            Ok(_) => (),
            Err(_) => panic!("failed to encode tiff file {} correctly.", file_name),
        };
    }
}

fn parse_args<'a>(matches: &'a ArgMatches) -> (Vec<&'a Path>, Option<&'a Path>) {
    (match matches.values_of("INPUT_FILES") {
        Some(inputs) => {
            let v: Vec<&Path> = inputs.map(|i| { Path::new(i) }).collect();
            for p in &v {
                let file_name = p.to_str().unwrap_or("[unknown]");
                if !p.is_file() { panic!("{} is not file.", file_name) }
                if let Some(ext) = p.extension() {
                    if ext != "png" { panic!("{} is not png file.", file_name) };
                }
            }
            v
        },
        None => panic!("couldn't get the value of option.")
    },
    match matches.value_of("OUTPUT_DIR") {
        Some(o) => {
            let p = Path::new(o);
            if !p.exists() {
                panic!("unable to create output directory.");
            }
            if !p.is_dir() { panic!("{} is not dir.", o) }
            Some(p)
        },
        None => None
    })
}

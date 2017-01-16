#[macro_use]
extern crate clap;
extern crate image;
extern crate texsyn;

use clap::{Arg, App};
use image::*;

use texsyn::quilt::{Quilter, QuilterParams};

fn main() {
    let matches = App::new("Quilt").version(crate_version!())
                                   .arg(Arg::with_name("input")
                                            .help("Input image")
                                            .index(1)
                                            .required(true))
                                   .arg(Arg::with_name("output")
                                            .help("Output image")
                                            .default_value("quilt.png")
                                            .index(2))
                                   .arg(Arg::with_name("width")
                                            .help("Output image width")
                                            .takes_value(true)
                                            .short("w")
                                            .long("width")
                                            .default_value("1024"))
                                   .arg(Arg::with_name("height")
                                            .help("Output image height")
                                            .takes_value(true)
                                            .short("h")
                                            .long("height")
                                            .default_value("1024"))
                                   .arg(Arg::with_name("size")
                                            .help("Output image size")
                                            .takes_value(true)
                                            .short("s")
                                            .long("size")
                                            .conflicts_with("width")
                                            .conflicts_with("height")
                                            .default_value("1024"))
                                   .arg(Arg::with_name("blocksize")
                                            .help("Patch size")
                                            .takes_value(true)
                                            .short("b")
                                            .long("blocksize")
                                            .default_value("64"))
                                   .arg(Arg::with_name("overlap")
                                            .help("Overlap area size")
                                            .takes_value(true)
                                            .short("o")
                                            .long("overlap")
                                            .default_value("12"))
                                   .get_matches();

    let in_file = matches.value_of("input").unwrap();
    let out_file = matches.value_of("output").unwrap();
    let size = value_t!(matches, "size", u32);
    let (width, height) = if let Ok(s) = size { (s, s) }
                          else { (value_t!(matches, "width", u32).unwrap(), value_t!(matches, "height", u32).unwrap()) };
    let blocksize = value_t!(matches, "blocksize", u32).unwrap();
    let overlap = value_t!(matches, "overlap", u32).unwrap();

    let img = open(in_file).unwrap();
    let d = &texsyn::distance::l1;
    let params = QuilterParams::new((width, height), blocksize, overlap, None, None, d).unwrap();
    let mut quilter = Quilter::new(img.to_rgb(), params);

    let res = quilter.quilt_image().unwrap();
    res.save(out_file).unwrap();
}

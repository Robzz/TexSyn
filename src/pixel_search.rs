#[macro_use]
extern crate clap;
extern crate libtexsyn;

use clap::{Arg, App};

use libtexsyn::{PixelSearch, PixelSearchParams};
use libtexsyn::image::*;

fn main() {
    let matches = App::new("PixelSearch").version(crate_version!())
                                         .arg(Arg::with_name("input")
                                                  .help("Input image")
                                                  .index(1)
                                                  .required(true))
                                         .arg(Arg::with_name("output")
                                                  .help("Output image")
                                                  .default_value("search.png")
                                                  .index(2))
                                         .arg(Arg::with_name("width")
                                                  .help("Output image width")
                                                  .takes_value(true)
                                                  .short("w")
                                                  .long("width"))
                                         .arg(Arg::with_name("height")
                                                  .help("Output image height")
                                                  .takes_value(true)
                                                  .short("h")
                                                  .long("height"))
                                         .arg(Arg::with_name("size")
                                                  .help("Output image size")
                                                  .takes_value(true)
                                                  .short("s")
                                                  .long("size")
                                                  .conflicts_with("width")
                                                  .conflicts_with("height")
                                                  .default_value("1024"))
                                         .arg(Arg::with_name("window-size")
                                                  .help("Search window size. Must be odd.")
                                                  .takes_value(true)
                                                  .short("W")
                                                  .long("winsize")
                                                  .default_value("15"))
                                         .get_matches();

    let in_file = matches.value_of("input").unwrap();
    let out_file = matches.value_of("output").unwrap();
    let size = value_t!(matches, "size", u32);
    let (width, height) = if let Ok(s) = size { (s, s) }
                          else { (value_t!(matches, "width", u32).unwrap(), value_t!(matches, "height", u32).unwrap()) };
    let winsize = value_t!(matches, "window-size", u32).unwrap();

    let img = open(in_file).unwrap();
    let params = PixelSearchParams::new((width, height), winsize, None).unwrap();
    let mut ps = PixelSearch::new(img.to_rgb(), params).unwrap();

    let res = ps.synthesize();
    res.save(out_file).unwrap();
}

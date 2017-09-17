#[macro_use]
extern crate clap;
extern crate libtexsyn;
extern crate ndimage;

use clap::{Arg, App};

use libtexsyn::generators::per_pixel::{PixelSearch, PixelSearchParams};
use libtexsyn::image::*;

fn rgbimage_to_ndimage(img: &RgbImage) -> ndimage::Image2D<ndimage::Rgb<u8>> {
    let (w, h) = img.dimensions();
    let mut out_img = ndimage::Image2D::new(w, h);
    for x in 0..w {
        for y in 0..h {
            let p = img.get_pixel(x, y);
            out_img.put_pixel(x, y, ndimage::Rgb::new([p[0], p[1], p[2]]));
        }
    }
    out_img
}

fn ndimage_to_rgbimage(img: &ndimage::Image2D<ndimage::Rgb<u8>>) -> RgbImage {
    let (w, h) = img.dimensions();
    let mut out_img = RgbImage::new(w, h);
    for x in 0..w {
        for y in 0..h {
            let p = img.get_pixel(x, y);
            out_img.put_pixel(x, y, Rgb{ data: [p[0], p[1], p[2]] });
        }
    }
    out_img
}

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
    let ndimg = rgbimage_to_ndimage(&img.to_rgb());
    let mut ps = PixelSearch::new(ndimg, params).unwrap();

    let res = ps.synthesize();
    let ps_res = ndimage_to_rgbimage(&res);
    ps_res.save(out_file).unwrap();
}

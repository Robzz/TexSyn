#[macro_use]
extern crate clap;
extern crate libtexsyn;
#[macro_use(array)]
extern crate ndarray;

use clap::{Arg, App};
use ndarray::Array2;

use libtexsyn::generators::per_pixel::wei_levoy::{WeiLevoyParams, WeiLevoy};
use libtexsyn::neighbourhood::{Neighbourhood, NeighbourhoodElem};
use libtexsyn::image::*;
use libtexsyn::pyramid::GaussianPyramid;

fn main() {
    let matches = App::new("WeiLevoy").version(crate_version!())
                                      .arg(Arg::with_name("input")
                                               .help("Input image")
                                               .index(1)
                                               .required(true))
                                      .arg(Arg::with_name("output")
                                               .help("Output image")
                                               .default_value("wei_levoy.png")
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
                                      .arg(Arg::with_name("neighbourhood-size")
                                               .help("Neighbourhood size in pixels")
                                               .takes_value(true)
                                               .short("W")
                                               .long("nsize")
                                               .default_value("15"))
                                      .get_matches();

    let in_file = matches.value_of("input").unwrap();
    let out_file = matches.value_of("output").unwrap();
    let size = value_t!(matches, "size", u32);
    let (width, height) = if let Ok(s) = size { (s, s) }
                          else { (value_t!(matches, "width", u32).unwrap(), value_t!(matches, "height", u32).unwrap()) };
    let winsize = value_t!(matches, "neighbourhood-size", u32).unwrap();

    let img = open(in_file).unwrap().to_rgb();

    //let pyr = GaussianPyramid::new(img, 4).unwrap();
    //pyr.save("pyramid");

    let mut neighbourhood_array = Array2::from_elem((5, 3), NeighbourhoodElem::On);
    neighbourhood_array[[4, 2]] = NeighbourhoodElem::Off;
    neighbourhood_array[[4, 3]] = NeighbourhoodElem::Off;
    neighbourhood_array[[4, 4]] = NeighbourhoodElem::Off;
    let neighbourhood = Neighbourhood::new(neighbourhood_array, (4, 2));

    let params = WeiLevoyParams::new((width, height), vec!(neighbourhood), None);
    let mut wl = WeiLevoy::new(img, params).unwrap();

    let res = wl.synthesize();
    res.save(out_file).unwrap();
}

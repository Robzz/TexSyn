extern crate texsyn;
extern crate image;

use std::env;

use image::*;

use texsyn::{Quilter, QuilterParams};

fn main() {
    let args = env::args();
    let mut args_iter = args.into_iter();
    args_iter.next();
    let in_file = args_iter.next().unwrap();
    let img = open(in_file).unwrap();
    let d = &texsyn::distance::l1;
    let params = QuilterParams::new((1024, 1024), 64, 24, Some((120, 0)), None, d).unwrap();
    let mut quilter = Quilter::new(img.to_rgb(), params);

    let res = quilter.quilt_image().unwrap();
    res.save("quilt.png").unwrap();
}

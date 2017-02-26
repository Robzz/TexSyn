#![feature(try_from)]
#[macro_use]
extern crate approx;
extern crate conv;
#[macro_use]
extern crate error_chain;
extern crate imageproc;
extern crate image as img;
extern crate num_traits;
extern crate rand;
extern crate rayon;

mod common;
pub mod distance;
pub mod errors;
pub mod quilt;
pub mod search;

pub use quilt::{Quilter, QuilterParams};
pub use search::{PixelSearch, PixelSearchParams};
pub mod image {
    pub use img::*;
}

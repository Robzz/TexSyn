#![feature(try_from, ord_max_min)]
#[cfg(test)] #[macro_use] extern crate approx;
//extern crate conv;
#[macro_use]
extern crate error_chain;
extern crate float_cmp;
extern crate imageproc;
extern crate image as img;
#[macro_use(s)]
extern crate ndarray;
extern crate ndimage;
//extern crate noise;
extern crate num_traits;
extern crate rand;
extern crate rayon;
extern crate time;

mod common;
pub mod distance;
pub mod errors;
pub mod generators;
pub mod neighbourhood;
pub mod pyramid;
pub mod random;

pub mod image {
    pub use img::*;
}

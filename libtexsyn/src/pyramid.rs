//! This modules defines image pyramid types.

use float_cmp::ApproxOrdUlps;
use image::{Pixel};
use imageproc::definitions::{Image};
use imageproc::filter::gaussian_blur_f32;

use errors::*;

pub struct GaussianPyramid<P>
    where P: Pixel<Subpixel=u8> + 'static
{
    base_image: Image<P>,
    sublevels: Vec<Image<P>>
}

fn is_power_of_2(n: u32) -> bool {
    match n {
        0 => false,
        1 => true,
        _ => match n % 2 == 0 {
            false => false,
            true => is_power_of_2(n / 2)
        }
    }
}

fn downsample<P>(img: &Image<P>) -> Image<P>
    where P: Pixel<Subpixel=u8> + 'static
{
    let (w, h) = img.dimensions();
    let mut downsampled = Image::new(w / 2, h / 2);
    for x in 0..w/2 {
        for y in 0..h/2 {
            downsampled[(x, y)] = img[(2*x, 2*y)];
        }
    }

    downsampled
}

impl<P> GaussianPyramid<P>
    where P: Pixel<Subpixel=u8> + 'static
{
    pub fn new(base: Image<P>, levels: u32) -> Result<GaussianPyramid<P>> {
        println!("Dimensions: {}x{}", base.width(), base.height());
        if !is_power_of_2(base.width()) || !is_power_of_2(base.height()) {
            bail!(ErrorKind::InvalidArguments("Image dimensions must be a power of 2".to_owned()));
        }
        else if (base.width().min(base.height()) as f32).log2().approx_lt(&(levels as f32), 2) {
            bail!(ErrorKind::InvalidArguments("Too many levels for image size".to_owned()));
        }

        match levels {
            0 => Ok(GaussianPyramid { base_image: base, sublevels: vec!() }),
            _ => {
                let mut sublevels = Vec::with_capacity(levels as usize);
                sublevels.push(downsample(&gaussian_blur_f32::<P>(&base, 3.0)));
                for i in 1..levels {
                    let blurred = {
                        let previous_level = &sublevels[(i - 1) as usize];
                        gaussian_blur_f32::<P>(&previous_level, 3.0)
                    };
                    sublevels.push(downsample(&blurred));
                }
                Ok(GaussianPyramid { base_image: base, sublevels: sublevels })
            }
        }
    }

    pub fn save(&self, path_base: &str) -> Result<()> {
        try!(self.base_image.save(format!("{}_{}.png", path_base, "base")));
        for (i, img) in self.sublevels.iter().enumerate() {
            try!(img.save(format!("{}_{}.png", path_base, i)));
        }
        Ok(())
    }
}

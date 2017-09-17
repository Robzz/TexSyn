use image::{RgbImage, Rgb};

use errors::*;
use neighbourhood::{Neighbourhood};
use pyramid::GaussianPyramid;
use random::{Seed, new_rng, random_image_rgb_with_rng, random};

/// Parameters of the `WeiLevoy` algorithm.
pub struct WeiLevoyParams {
    /// Size of the image to synthesize
    pub size: (u32, u32),
    /// Size of the search neighbourhood (in number of pixels)
    pub neighbourhoods: Vec<Neighbourhood>,
    /// Seed of the internal random number generator
    pub seed: Option<Seed>
}

impl WeiLevoyParams {
    /// Create a new `WeiLevoyParams`
    pub fn new(size: (u32, u32), neighbourhoods: Vec<Neighbourhood>, seed: Option<Seed>) -> WeiLevoyParams {
        WeiLevoyParams { size: size, neighbourhoods: neighbourhoods, seed: seed }
    }
}

/// Per pixel texture synthesis algorithm. This is much faster than `PixelSearch` and of
/// comparable quality.
pub struct WeiLevoy {
    pyramid: GaussianPyramid<Rgb<u8>>,
    params: WeiLevoyParams,
}

impl WeiLevoy {
    /// Construct a new WeiLevoy instance
    pub fn new(source: RgbImage, params: WeiLevoyParams) -> Result<WeiLevoy> {
        let pyr = try!(GaussianPyramid::new(source, 4));
        Ok(WeiLevoy { pyramid: pyr, params: params })
    }

    pub fn synthesize(&self) -> RgbImage {
        let (w, h) = self.params.size;
        let mut res = RgbImage::new(w, h);
        let mut rng = new_rng(self.params.seed.unwrap_or(random()));
        random_image_rgb_with_rng(&mut res, &mut rng);

        res
    }
}

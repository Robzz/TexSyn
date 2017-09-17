use ndimage::*;
use num_traits::Zero;
use rand::{thread_rng, random, Rng};
use rayon::prelude::*;

use std::cmp::min;
use std::convert::TryFrom;

use common::{OrderedFloat, Rect};
use errors::*;

type RgbImage = Image2D<Rgb<u8>>;
type GrayImage = Image2D<Luma<u8>>;

pub struct PixelSearchParams {
    size: (u32, u32),
    window_size: u32,
    seed_coords: Option<(u32, u32)>
}

fn blit_rect<P>(dest: &mut Image2D<P>, src: &Image2D<P>, rect: &Rect, buf_coords: (u32, u32))
    where P: Pixel
{
    let mut dest_rect = dest.sub_image_mut(buf_coords.0, buf_coords.1, rect.size.0, rect.size.1);
    let src_rect = src.sub_image(rect.coords.0, rect.coords.1, rect.size.0, rect.size.1);

    for (pix_dest, pix_src) in dest_rect.into_iter().zip(src_rect.into_iter()) {
        *pix_dest = pix_src.clone();
    }
}

fn l2(p1: &Rgb<u8>, p2: &Rgb<u8>) -> f64 {
    let f = |c1, c2| {
        let n = (c1 as i32) - (c2 as i32);
        n * n
    };
    ((f(p1[0], p2[0]) + f(p1[1], p2[1]) + f(p1[2], p2[2])) as f64).sqrt()
}

/// Parameters of the Efros and Leung algorithm.
///
/// * `size`: size of the synthesized image
/// * `window_size`: size of the search window. Must be an odd number.
/// * `seed_coords`: coordinates of the top-left corner of the initial seed 3x3 patch. If set to None, will be chosen
/// randomly.
impl PixelSearchParams {
    pub fn new(size: (u32, u32), window_size: u32, seed_coords: Option<(u32, u32)>) -> Result<PixelSearchParams> {
        if window_size % 2 == 0 {
            bail!(ErrorKind::InvalidArguments("window_size must be odd".to_owned()));
        }
        Ok(PixelSearchParams { size: size, window_size: window_size, seed_coords: seed_coords })
    }
}

/// Implements the Efros and Leung algorithm. This is pretty slow...
pub struct PixelSearch {
    params: PixelSearchParams,
    source: RgbImage,
    buffer_opt: Option<RgbImage>
}

impl PixelSearch {
    /// Create a new `PixelSearch`
    pub fn new(source: RgbImage, params: PixelSearchParams) -> Result<PixelSearch> {
        if let Some(coords) = params.seed_coords {
            if coords.0 > source.width() - 3 || coords.1 > source.height() - 1 {
                bail!(ErrorKind::InvalidArguments("Seed patch is outside source image".to_owned()));
            }
        }
        Ok(PixelSearch { source: source, params: params, buffer_opt: None })
    }

    fn mask_on(mask: &GrayImage, x: u32, y: u32) -> bool {
        !mask.get_pixel(x, y).is_zero()
    }

    fn is_edge_pixel(mask: &GrayImage, x: u32, y: u32) -> bool {
        (if x != 0                 { Self::mask_on(mask, x - 1, y) } else { false }) ||
        (if x != mask.width() - 1  { Self::mask_on(mask, x + 1, y) } else { false }) ||
        (if y != 0                 { Self::mask_on(mask, x, y - 1) } else { false }) ||
        (if y != mask.height() - 1 { Self::mask_on(mask, x, y + 1) } else { false })
    }

    /// Synthesize an image using the Efros and Leung method.
    pub fn synthesize(&mut self) -> RgbImage {
        let (w, h) = self.params.size;
        self.buffer_opt = Some(RgbImage::new(w, h));
        let mut mask = GrayImage::new(w, h);
        {
            let mut mask_seed = mask.sub_image_mut(w / 2 - 1, h / 2 - 1, 3, 3);
            for pix in mask_seed {
                *pix = Luma::new([255u8]);
            }
        }
        //draw_filled_rect_mut(&mut mask, IPRect::at((w / 2 - 1) as i32, (h / 2 - 1) as i32).of_size(3, 3), Luma { data: [255] });

        // Copy the initial seed to the center of the buffer and grow an image from there
        let (sx, sy) = (random::<u32>() % (self.source.width() - 3), random::<u32>() % (self.source.height() - 3));
        blit_rect(self.buffer_opt.as_mut().unwrap(), &self.source, &Rect { coords: (sx, sy), size: (3, 3) }, (w / 2 - 1, h / 2 - 1));

        let mut n_pixels = mask.into_iter().filter(|p| p.is_zero()).count();
        //println!("Got {} zero pixels", n_pixels);
        while n_pixels > 0 {
            // Find the next pixel to synthesize
            let next_pixel = mask.enumerate_pixels().collect::<Vec<_>>().into_par_iter()
                                 .filter_map(|((x, y), p)|
                                             if p.data[0].is_zero() && Self::is_edge_pixel(&mask, x as u32, y as u32) {
                                                 Some((x as u32, y as u32))
                                             }
                                             else {
                                                 None
                                             })
                                 .map(|c| { (c, self.pixel_num_neigbours(&mask, c)) })
                                 .max_by_key(|&(_, n)| n).expect(":'('").0;

            // Synthesize the pixel and mark it as done
            let pixel = self.synthesize_pixel(&mask, next_pixel);
            self.buffer_opt.as_mut().unwrap().put_pixel(next_pixel.0, next_pixel.1, pixel);
            mask.put_pixel(next_pixel.0, next_pixel.1, Luma { data: [1] });
            n_pixels -= 1;
            println!("{} pixels left", n_pixels);
        }

        self.buffer_opt.take().unwrap()
    }

    // Compute the number of valid neighbours in the neighbourhood around the specified pixel
    fn pixel_num_neigbours(&self, mask: &GrayImage, coords: (u32, u32)) -> u32 {
        let d = (self.params.window_size - 1) / 2;
        let xs = if coords.0 <= d { 0 } else { coords.0 - d };
        let ys = if coords.1 <= d { 0 } else { coords.1 - d };
        let xe = min(mask.width() - 1, coords.0 + d) + 1; // +1 because for takes [a,b) ranges
        let ye = min(mask.width() - 1, coords.1 + d) + 1;

        let mut neighbours = 0;
        for x in xs..xe {
            for y in ys..ye {
                if (x != coords.0 || y != coords.1) && !mask.get_pixel(x, y).data[0].is_zero() {
                    neighbours += 1;
                }
            }
        }

        neighbours
    }

    // Synthesize one single pixel
    fn synthesize_pixel(&self, mask: &GrayImage, coords: (u32, u32)) -> Rgb<u8> {
        // Find all similar neighbourhoods and pick one wihin 10% tolerance
        let mut errors = self.source.enumerate_pixels().collect::<Vec<_>>().into_par_iter()
                                    .filter_map(|((x, y), _)|
                                                if let Some(err) = self.neighbourhood_error(mask, coords, (x as u32, y as u32)) {
                                                    Some((x as u32, y as u32, OrderedFloat::try_from(err).unwrap()))
                                                }
                                                else { None })
                                    .collect::<Vec<_>>();
        errors.sort_by_key(|&(_, _, e)| e);
        let bound = 1.1 * errors[0].2.as_float();
        let mut filtered_errors = errors.into_iter().take_while(|&(_, _, e)| e.as_float() <= bound).collect::<Vec<_>>();
        thread_rng().shuffle(&mut filtered_errors);
        let (x, y, _) = filtered_errors.pop().unwrap();
        self.source.get_pixel(x, y)
    }

    // Compute the error between the specified neighbourhood and the specified pixel
    fn neighbourhood_error(&self, mask: &GrayImage, pixel: (u32, u32), neighbourhood: (u32, u32)) -> Option<f64> {
        let d = ((self.params.window_size - 1) / 2) as i32;

        let (px, py) = (pixel.0 as i32, pixel.1 as i32);
        let (nx, ny) = (neighbourhood.0 as i32, neighbourhood.1 as i32);

        let xs = min(min(d, px), min(d, nx));
        let ys = min(min(d, py), min(d, ny));
        let xe = min(min(d, self.source.width() as i32 - nx - 1), min(d, mask.width() as i32 - px - 1));
        let ye = min(min(d, self.source.height() as i32 - ny - 1), min(d, mask.height() as i32 - py - 1));
        let mut error = 0.;
        let mut i = 0;
        for y in -ys..ye + 1 {
            for x in -xs..xe + 1 {
                let (pxx, pyy) = ((px + x) as u32, (py + y) as u32);
                let (nxx, nyy) = ((nx + x) as u32, (ny + y) as u32);
                if Self::mask_on(mask, pxx, pyy) {
                    error += l2(&self.source.get_pixel(nxx, nyy),
                                &self.buffer_opt.as_ref().unwrap().get_pixel(pxx, pyy));
                    i += 1;
                }
            }
        }

        match i {
            0 => None,
            _ => Some(error / i as f64)
        }
    }
}

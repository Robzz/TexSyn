use image::{Rgb, RgbImage};
use rand::{Rng, SeedableRng, XorShiftRng};
pub use rand::random;

pub type Seed = [u32; 4];

/// Create a new random number generator with the specified seed.
pub fn new_rng(seed: Seed) -> XorShiftRng {
    SeedableRng::from_seed(seed)
}

/// Create a new random number generator with a random seed.
pub fn new_rng_random_seed() -> XorShiftRng {
    let seed = random::<Seed>();
    SeedableRng::from_seed(seed)
}

/// Create a new unseeded random number generator, which will always return the same sequence.
pub fn new_deterministic_rng() -> XorShiftRng {
    XorShiftRng::new_unseeded()
}

/// Fill an RGB image with random noise using the specified Rng
pub fn random_image_rgb_with_rng<R: Rng>(img: &mut RgbImage, rng: &mut R) {
    for x in 0..img.width() {
        for y in 0..img.height() {
            let pixel = Rgb { data: [rng.gen(), rng.gen(), rng.gen()] };
            img.put_pixel(x, y, pixel);
        }
    }
}

/// Fill an RGB image with random noise using a randomly seeded rng
pub fn random_image_rgb<R: Rng>(img: &mut RgbImage) {
    random_image_rgb_with_rng(img, &mut new_rng_random_seed())
}

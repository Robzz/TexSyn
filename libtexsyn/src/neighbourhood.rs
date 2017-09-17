use ndarray::prelude::*;
use ndarray::iter::Iter;
use ndimage::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeighbourhoodElem {
    On,
    Off
}

/// Encodes a neighbourhood.
#[derive(Debug)]
pub struct Neighbourhood {
    elems: Array2<NeighbourhoodElem>,
    reference: (usize, usize)
}

impl Neighbourhood {
    /// Construct a new neighbourhood from a 2D array of `NeighbourhoodElem`.
    pub fn new(elems: Array2<NeighbourhoodElem>, reference: (usize, usize)) -> Neighbourhood {
        Neighbourhood { elems: elems, reference: reference }
    }

    pub fn difference<'a, P, S>(&'a self, p1: (usize, usize), img1: &'a Image2D<P>, p2: (usize, usize), img2: &'a Image2D<P>) -> f64
        where P: Pixel<Subpixel=S>,
              S: Primitive,
    {
        let l2 = |p1: &'a P, p2: &'a P| {
            let mut d = 0.;
            for (c1, c2) in p1.channels().into_iter().zip(p2.channels()) {
                let (min, max) = match c1 < c2 {
                    true => (c1, c2),
                    false => (c2, c1)
                };
                let diff = (*max - *min).to_f64().unwrap();
                let d2 = diff * diff;
                d = d + d2;
            }
            d.sqrt()
        };

        let iter1 = self.image_iter(img1, p1);
        let iter2 = self.image_iter(img2, p2);
        iter1.zip(iter2).map(|(p1, p2)| l2(p1, p2)).fold(0., |acc, d| acc + d)
    }

    pub fn image_iter<'a, P>(&'a self, img: &'a Image2D<P>, img_ref: (usize, usize)) -> NeighbourhoodIterator<'a, P>
        where P: Pixel
    {
        // Compute the rectangular region of the sub-image to iterate on.
        // For this, we need to account for the fact that the neighbourhood may extend over the
        // edges of the image, and in this case, crop the iteration region.
        let mut x_size_offset_left   = 0;
        let mut x_size_offset_right  = 0;
        let mut y_size_offset_top    = 0;
        let mut y_size_offset_bottom = 0;
        let x = match img_ref.0 < self.reference.0 {
            true => {
                x_size_offset_left += self.reference.0 - img_ref.0;
                0
            }
            false => img_ref.0 - self.reference.0
        };
        let y = match img_ref.1 < self.reference.1 {
            true => {
                y_size_offset_top += self.reference.1 - img_ref.1;
                0
            }
            false => img_ref.1 - self.reference.1
        };
        let (cols, rows) = (self.elems.cols(), self.elems.rows());
        if img_ref.0 + cols > img.width() as usize {
            x_size_offset_right += img_ref.0 + cols - img.width() as usize;
        }
        if img_ref.1 + rows > img.height() as usize {
            y_size_offset_bottom += img_ref.1 + rows - img.height() as usize;
        }

        let x_size_offset = x_size_offset_right + x_size_offset_left;
        let y_size_offset = y_size_offset_top + y_size_offset_bottom;
        let w = match x_size_offset > cols {
            true => 0,
            false => cols - x_size_offset
        };
        let h = match y_size_offset > rows {
            true => 0,
            false => rows - y_size_offset
        };
        let sub_image = img.sub_image(x as u32, y as u32, w as u32, h as u32);

        NeighbourhoodIterator::new(
            self.elems.slice(s![(x_size_offset_left as isize..(x_size_offset_left + w) as isize),
                                (y_size_offset_top as isize..(y_size_offset_top + h) as isize)])
                      .into_iter(),
            sub_image.into_iter()
        )
    }
}

pub struct NeighbourhoodIterator<'a, P>
    where P: Pixel + 'a
{
    neighbourhood_iter: Iter<'a, NeighbourhoodElem, Ix2>,
    img_iter: Iter<'a, P, Ix2>
}

impl<'a, P> NeighbourhoodIterator<'a, P>
    where P: Pixel + 'a
{
    fn new(neighbourhood_iter: Iter<'a, NeighbourhoodElem, Ix2>, img_iter: Iter<'a, P, Ix2>) -> NeighbourhoodIterator<'a, P> {
        NeighbourhoodIterator { neighbourhood_iter: neighbourhood_iter, img_iter: img_iter }
    }
}

impl<'a, P> Iterator for NeighbourhoodIterator<'a, P>
    where P: Pixel
{
    type Item = &'a P;

    fn next(&mut self) -> Option<&'a P> {
        match (self.neighbourhood_iter.next(), self.img_iter.next()) {
            (Some(elem), Some(pixel)) => if *elem == NeighbourhoodElem::On { Some(pixel) }
                                         else { self.next() },
            (None, None) => None,
            _ => unreachable!()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    //use approx;
    use ndimage::Luma;

    use std::iter::FromIterator;

    #[test]
    fn test_neighbourhood_iterator() {
        let mut neighbourhood_array = Array2::from_elem((3, 3), NeighbourhoodElem::On);
        neighbourhood_array[[1, 2]] = NeighbourhoodElem::Off;
        neighbourhood_array[[2, 1]] = NeighbourhoodElem::Off;
        neighbourhood_array[[2, 2]] = NeighbourhoodElem::Off;
        let neighbourhood = Neighbourhood::new(neighbourhood_array, (1, 1));

        let v = Vec::from_iter((0u8..25u8).map(|n| Luma::from(n)));
        let img = Image2D::from_vec(5, 5, v).unwrap();
        let iter_pixels = Vec::from_iter(neighbourhood.image_iter(&img, (2, 2)).map(|i| *i));
        let iter_pixels_ref = Vec::from_iter([6u8, 7, 8, 11, 12, 16].into_iter().map(|n| Luma::from(*n)));
        assert_eq!(iter_pixels, iter_pixels_ref);
    }

    #[test]
    fn test_neighbourhood_difference() {
        let img1 = Image2D::from_vec(5, 5, Vec::from_iter((0u8..25u8).map(|n| Luma::from(n)))).unwrap();
        let img2 = Image2D::from_vec(5, 5, Vec::from_iter((0u8..25u8).map(|n| Luma::from(2*n)))).unwrap();
        let mut neighbourhood_array = Array2::from_elem((3, 3), NeighbourhoodElem::On);
        neighbourhood_array[[1, 2]] = NeighbourhoodElem::Off;
        neighbourhood_array[[2, 1]] = NeighbourhoodElem::Off;
        neighbourhood_array[[2, 2]] = NeighbourhoodElem::Off;
        let neighbourhood = Neighbourhood::new(neighbourhood_array, (1, 1));

        let d = neighbourhood.difference((2, 2), &img1, (2, 2), &img2);
        assert_relative_eq!(d, 60.);
    }
}

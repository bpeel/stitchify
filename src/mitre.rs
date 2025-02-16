// Stichify – A utility to generate intarsia knitting patterns
// Copyright (C) 2025  Neil Roberts
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use super::fabric::{self, Fabric};
use super::stitch_image::{Image, Color};
use super::config::{Dimensions, Link};
use super::sampler::Sampler;

struct MitreImage {
    size: usize,
    pixels: Vec<Option<Color>>,
}

impl Image for MitreImage {
    fn width(&self) -> u32 {
        self.size as u32 * 2
    }

    fn height(&self) -> u32 {
        self.size as u32
    }

    fn get_pixel(&self, x: u32, y: u32) -> Option<Color> {
        self.pixels[y as usize * self.size * 2 + x as usize]
    }
}

impl MitreImage {
    pub fn new<I: Image>(
        image: &I,
        n_stitches: u16,
    ) -> MitreImage {
        let image_size = image.width().min(image.height());
        let sample_size = image_size as f32 / n_stitches as f32;
        let mut pixels =
            Vec::with_capacity((n_stitches * n_stitches * 2) as usize);
        let sampler = Sampler::new(image, sample_size, sample_size);

        for y in 0..n_stitches {
            let row_width = y + 1;

            for x in 0..row_width - 1 {
                pixels.push(sampler.sample(x, y, 1));
            }

            pixels.push(sampler.sample_lower_left_triangle(row_width - 1, y));

            // Add the empty middle section between the two halves
            let old_len = pixels.len();
            pixels.resize(
                old_len + (n_stitches - row_width) as usize * 2,
                None
            );

            pixels.push(sampler.sample_upper_right_triangle(row_width - 1, y));

            for x in 1..row_width {
                pixels.push(sampler.sample(y, row_width - 1 - x, 1));
            }
        }

        MitreImage {
            size: n_stitches as usize,
            pixels,
        }
    }
}

pub fn make_mitre_fabric<I: Image>(
    image: &I,
    dimensions: &Dimensions,
) -> Result<(Fabric, Dimensions), fabric::Error> {
    let image = MitreImage::new(image, dimensions.stitches);

    // Use stitches that are twice as wide as they are tall but force
    // garter stitch
    let mut dimensions = dimensions.clone();
    dimensions.gauge_rows = dimensions.gauge_stitches * 2;
    dimensions.duplicate_rows = 2;
    dimensions.stitches = image.width() as u16;

    dimensions.allow_link_gaps = true;

    // Automatically add links across the middle gaps
    if image.height() > 1 {
        let center = image.width() as u16 / 2;

        for y in 0..image.height() as u16 - 1 {
            let image_y = image.height() - 2 - y as u32;

            let left_x = center - y - 2;
            let right_x = center + y + 1;

            let left = image.get_pixel(left_x as u32, image_y);
            let right = image.get_pixel(right_x as u32, image_y);

            if left.is_none() || right.is_none() || left != right {
                continue;
            }

            let bottom_row = y * 2 + 3;

            dimensions.links.push(Link {
                source: (right_x + 1, bottom_row),
                dest: (left_x + 1, bottom_row),
            });
            dimensions.links.push(Link {
                source: (left_x + 1, bottom_row + 1),
                dest: (right_x + 1, bottom_row + 1),
            });
        }
    }

    fabric::Fabric::new(&image, &dimensions).map(|fabric| (fabric, dimensions))
}

#[cfg(test)]
mod test {
    use super::*;

    struct FakeImage {
    }

    impl Image for FakeImage {
        fn width(&self) -> u32 {
            24
        }

        fn height(&self) -> u32 {
            24
        }

        fn get_pixel(&self, x: u32, y: u32) -> Option<Color> {
            Some([x as u8, y as u8, 0])
        }
    }

    const UNLINKED_DIAGONAL_IMAGE_DATA: &[u8; 12 * 12] =
        b"abbbbbbbbbbb\
          aabbbbbbbbbb\
          aaabbbbbbbbb\
          aaaabbbbbbbb\
          aaaa    bbbb\
          aaaa    bbbb\
          aaaa    bbbb\
          aaaa    bbbb\
          aaaaaaaaabbb\
          aaaaaaaaaabb\
          aaaaaaaaaaab\
          aaaaaaaaaaaa";

    const LINKED_DIAGONAL_IMAGE_DATA: &[u8; 12 * 12] =
        b"aaaaaaaaaaaa\
          aaaaaaaaaaaa\
          aaaaaaaaaaaa\
          aaaaaaaaaaaa\
          aaaaabbbbbbb\
          aaaaaabbbbbb\
          aaaaaaabbbbb\
          aaaaaaaabbbb\
          aaaaaaaaabbb\
          aaaaaaaaaabb\
          aaaaaaaaaaab\
          aaaaaaaaaaaa";

    struct FakeDiagonalImage {
        data: [u8; 12 * 12],
    }

    impl Image for FakeDiagonalImage {
        fn width(&self) -> u32 {
            12
        }

        fn height(&self) -> u32 {
            12
        }

        fn get_pixel(&self, x: u32, y: u32) -> Option<Color> {
            match self.data[(y * 12 + x) as usize] {
                b'a' => Some([255, 0, 0]),
                b'b' => Some([0, 255, 0]),
                _ => None,
            }
        }
    }

    #[test]
    fn mitre_image() {
        let fake_image = FakeImage { };

        let image = MitreImage::new(&fake_image, 24);

        assert_eq!(image.width(), 48);
        assert_eq!(image.height(), 24);

        assert_eq!(image.get_pixel(0, 23), Some([0, 23, 0]));
        assert_eq!(image.get_pixel(23, 23), Some([23, 23, 0]));
        assert_eq!(image.get_pixel(0, 22), Some([0, 22, 0]));
        assert_eq!(image.get_pixel(22, 22), Some([22, 22, 0]));
        assert_eq!(image.get_pixel(23, 22), None);
        assert_eq!(image.get_pixel(0, 0), Some([0, 0, 0]));
        assert_eq!(image.get_pixel(1, 0), None);

        assert_eq!(image.get_pixel(24, 23), Some([23, 23, 0]));
        assert_eq!(image.get_pixel(47, 23), Some([23, 0, 0]));
        assert_eq!(image.get_pixel(24, 22), None);
        assert_eq!(image.get_pixel(25, 22), Some([22, 22, 0]));
        assert_eq!(image.get_pixel(47, 22), Some([22, 0, 0]));
        assert_eq!(image.get_pixel(24, 0), None);
        assert_eq!(image.get_pixel(46, 0), None);
        assert_eq!(image.get_pixel(47, 0), Some([0, 0, 0]));
    }

    #[test]
    fn unlinked_diagonal_image() {
        let fake_image = FakeDiagonalImage {
            data: UNLINKED_DIAGONAL_IMAGE_DATA.clone(),
        };

        let mut dimensions = Dimensions::default();
        dimensions.stitches = 3;

        let (fabric, dimensions) =
            make_mitre_fabric(&fake_image, &dimensions).unwrap();

        assert_eq!(fabric.n_rows(), 6);
        assert_eq!(fabric.n_stitches(), 6);
        assert_eq!(dimensions.stitches, 6);
        assert_eq!(dimensions.duplicate_rows, 2);
        assert_eq!(dimensions.gauge_rows, dimensions.gauge_stitches * 2);
        assert_eq!(dimensions.links.len(), 0);

        assert_eq!(
            fabric.stitches()[0].as_ref().unwrap().color,
            [255, 0, 0],
        );
        assert!(fabric.stitches()[1].is_none());
        assert!(fabric.stitches()[2].is_none());
        assert!(fabric.stitches()[3].is_none());
        assert!(fabric.stitches()[4].is_none());
        assert_eq!(
            fabric.stitches()[5].as_ref().unwrap().color,
            [0, 255, 0],
        );

        assert_eq!(
            fabric.stitches()[12].as_ref().unwrap().color,
            [255, 0, 0],
        );
        assert!(fabric.stitches()[13].is_none());
        assert!(fabric.stitches()[14].is_none());
        assert!(fabric.stitches()[15].is_none());
        assert!(fabric.stitches()[16].is_none());
        assert_eq!(
            fabric.stitches()[17].as_ref().unwrap().color,
            [0, 255, 0],
        );
    }

    #[test]
    fn linked_diagonal_image() {
        let fake_image = FakeDiagonalImage {
            data: LINKED_DIAGONAL_IMAGE_DATA.clone(),
        };

        let mut dimensions = Dimensions::default();
        dimensions.stitches = 3;

        let (fabric, dimensions) =
            make_mitre_fabric(&fake_image, &dimensions).unwrap();

        assert_eq!(fabric.n_rows(), 6);
        assert_eq!(fabric.n_stitches(), 6);
        assert_eq!(dimensions.stitches, 6);
        assert_eq!(dimensions.duplicate_rows, 2);
        assert_eq!(dimensions.gauge_rows, dimensions.gauge_stitches * 2);

        assert_eq!(dimensions.links.len(), 2);

        assert_eq!(dimensions.links[0].source, (6, 5));
        assert_eq!(dimensions.links[0].dest, (1, 5));
        assert_eq!(dimensions.links[1].source, (1, 6));
        assert_eq!(dimensions.links[1].dest, (6, 6));

        assert_eq!(
            fabric.stitches()[0].as_ref().unwrap().color,
            [255, 0, 0],
        );
        assert!(fabric.stitches()[1].is_none());
        assert!(fabric.stitches()[2].is_none());
        assert!(fabric.stitches()[3].is_none());
        assert!(fabric.stitches()[4].is_none());
        assert_eq!(
            fabric.stitches()[5].as_ref().unwrap().color,
            [255, 0, 0],
        );
    }
}

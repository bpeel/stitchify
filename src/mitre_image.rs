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

use super::fabric::{Image, Fabric, Color};

pub struct MitreImage<'a> {
    fabric: &'a Fabric,
}

impl<'a> Image for MitreImage<'a> {
    fn width(&self) -> u32 {
        self.fabric.n_stitches() as u32 * 2
    }

    fn height(&self) -> u32 {
        self.fabric.n_rows().min(self.fabric.n_stitches()) as u32
    }

    fn get_pixel(&self, x: u32, y: u32) -> Option<Color> {
        let n_stitches = self.fabric.n_stitches() as u32;
        let n_rows = self.height();
        let row_width = n_stitches + y + 1 - n_rows;

        if x < n_stitches {
            if x < row_width {
                self.fabric.stitches()[(x + y * n_rows) as usize]
                    .as_ref()
                    .map(|s| {
                        s.color
                    })
            } else {
                None
            }
        } else if x >= n_stitches * 2 - row_width {
            let x = x - n_stitches;

            self.fabric.stitches()[
                ((n_rows - 1 - x) * n_stitches + y) as usize
            ].as_ref().map(|s| s.color)
        } else {
            None
        }
    }
}

impl<'a> MitreImage<'a> {
    pub fn new(fabric: &'a Fabric) -> MitreImage<'a> {
        MitreImage {
            fabric
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::config::Dimensions;

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

    #[test]
    fn mitre_image() {
        let fake_image = FakeImage { };

        let mut dimensions = Dimensions::default();
        dimensions.gauge_stitches = 1;
        dimensions.gauge_rows = 1;
        dimensions.stitches = fake_image.width() as u16;

        let fabric = Fabric::new(&fake_image, &dimensions).unwrap();

        let image = MitreImage::new(&fabric);

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
}

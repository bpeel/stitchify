// Stichify – A utility to generate intarsia knitting patterns
// Copyright (C) 2024, 2025  Neil Roberts
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

use std::collections::HashMap;
use std::cell::Cell;
use super::stitch_image::{Image, Color};

struct SampleRange {
    start_x: u32,
    end_x: u32,
    start_y: u32,
    end_y: u32,
}

pub struct Sampler<'a, I: Image> {
    image: &'a I,
    sample_width: f32,
    sample_height: f32,
    // This is a helper hash map used to count the colors. It is
    // stored here to avoid having to reallocate the buffers every
    // time the image is sampled. Its contents aren’t reused between
    // sampling.
    counts: Cell<HashMap<Option<Color>, u32>>,
}

impl<'a, I: Image> Sampler<'a, I> {
    pub fn new(
        image: &'a I,
        sample_width: f32,
        sample_height: f32,
    ) -> Sampler<'a, I> {
        Sampler {
            image,
            sample_width,
            sample_height,
            counts: Cell::new(HashMap::new()),
        }
    }

    fn sample_range(&self, x: u16, y: u16, row_height: u16) -> SampleRange {
        SampleRange {
            start_x: (x as f32 * self.sample_width).round() as u32,
            end_x: (((x + 1) as f32 * self.sample_width).round() as u32)
                .min(self.image.width()),
            start_y: (y as f32 * self.sample_height).round() as u32,
            end_y: (((y + row_height) as f32 *
                     self.sample_height).round() as u32)
                .min(self.image.height()),
        }
    }

    fn start_counting(&self) -> HashMap<Option<Color>, u32> {
        let mut counts = self.counts.take();
        counts.clear();
        counts
    }

    fn end_counting(
        &self,
        counts: HashMap<Option<Color>, u32>,
    ) -> Option<Color> {
        let result = counts.keys()
            .max_by_key(|&color| counts[color])
            .cloned()
            .unwrap_or(None);

        self.counts.replace(counts);

        result
    }

    pub fn sample(
        &self,
        x: u16,
        y: u16,
        row_height: u16,
    ) -> Option<Color> {
        let sample_range = self.sample_range(x, y, row_height);

        let mut counts = self.start_counting();

        for y in sample_range.start_y..sample_range.end_y {
            for x in sample_range.start_x..sample_range.end_x {
                let color = self.image.get_pixel(x, y);
                *counts.entry(color.clone()).or_insert(0) += 1;
            }
        }

        self.end_counting(counts)
    }

    pub fn sample_lower_left_triangle(&self, x: u16, y: u16) -> Option<Color> {
        let sample_range = self.sample_range(x, y, 1);

        if sample_range.end_y <= sample_range.start_y {
            return None;
        }

        let mut counts = self.start_counting();

        let y_range = sample_range.end_y - sample_range.start_y;

        for y in sample_range.start_y..sample_range.end_y {
            let row_length = ((y + 1 - sample_range.start_y) *
                              (sample_range.end_x - sample_range.start_x) +
                              y_range / 2) /
                y_range;

            for x in sample_range.start_x..sample_range.start_x + row_length {
                let color = self.image.get_pixel(x, y);
                *counts.entry(color.clone()).or_insert(0) += 1;
            }
        }

        self.end_counting(counts)
    }

    pub fn sample_upper_right_triangle(&self, x: u16, y: u16) -> Option<Color> {
        let sample_range = self.sample_range(x, y, 1);

        if sample_range.end_y <= sample_range.start_y {
            return None;
        }

        let mut counts = self.start_counting();

        let y_range = sample_range.end_y - sample_range.start_y;

        for y in sample_range.start_y..sample_range.end_y {
            let row_length = ((y_range + sample_range.start_y - y) *
                              (sample_range.end_x - sample_range.start_x) +
                              y_range / 2) /
                y_range;

            for x in sample_range.end_x - row_length..sample_range.end_x {
                let color = self.image.get_pixel(x, y);
                *counts.entry(color.clone()).or_insert(0) += 1;
            }
        }

        self.end_counting(counts)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    static FAKE_IMAGE_DATA: &[u8; 3 * 4 * 3 * 4] =
        b"abbbbbbbbbbb\
          aabbbbbbbbbb\
          aaabbbbbbbbb\
          aaaabbbbbbbb\
          aaaaabbbbbbb\
          aaaaaabbbbbb\
          aaaaaaabbbbb\
          aaaaaaaabbbb\
          aaaaaaaaabbb\
          aaaaaaaaaabb\
          aaaaaaaaaaab\
          aaaaaaaaaaaa";

    struct FakeImage {
    }

    impl Image for FakeImage {
        fn width(&self) -> u32 {
            12
        }

        fn height(&self) -> u32 {
            12
        }

        fn get_pixel(&self, x: u32, y: u32) -> Option<Color> {
            match FAKE_IMAGE_DATA[(y * 12 + x) as usize] {
                b'a' => Some([255, 0, 0]),
                b'b' => Some([0, 255, 0]),
                _ => None,
            }
        }
    }

    #[test]
    fn sample_lower_left_triangle() {
        let image = FakeImage { };
        let sampler = Sampler::new(&image, 4.0, 4.0);

        assert_eq!(sampler.sample_lower_left_triangle(0, 0), Some([255, 0, 0]));
        assert_eq!(sampler.sample(1, 0, 1), Some([0, 255, 0]));

        assert_eq!(sampler.sample(0, 1, 1), Some([255, 0, 0]));
        assert_eq!(sampler.sample_lower_left_triangle(1, 1), Some([255, 0, 0]));
        assert_eq!(sampler.sample(2, 1, 1), Some([0, 255, 0]));

        assert_eq!(sampler.sample(1, 2, 1), Some([255, 0, 0]));
        assert_eq!(sampler.sample_lower_left_triangle(2, 2), Some([255, 0, 0]));
    }

    #[test]
    fn sample_upper_right_triangle() {
        let image = FakeImage { };
        let sampler = Sampler::new(&image, 4.0, 4.0);

        assert_eq!(
            sampler.sample_upper_right_triangle(0, 0),
            Some([0, 255, 0]),
        );
        assert_eq!(sampler.sample(1, 0, 1), Some([0, 255, 0]));

        assert_eq!(sampler.sample(0, 1, 1), Some([255, 0, 0]));
        assert_eq!(
            sampler.sample_upper_right_triangle(1, 1),
            Some([0, 255, 0]),
        );
        assert_eq!(sampler.sample(2, 1, 1), Some([0, 255, 0]));

        assert_eq!(sampler.sample(1, 2, 1), Some([255, 0, 0]));
        assert_eq!(
            sampler.sample_upper_right_triangle(2, 2),
            Some([0, 255, 0]),
        );
    }
}

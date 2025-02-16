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

    pub fn sample(
        &self,
        x: u16,
        y: u16,
        row_height: u16,
    ) -> Option<Color> {
        let start_x = (x as f32 * self.sample_width).round() as u32;
        let end_x = (((x + 1) as f32 * self.sample_width).round() as u32)
            .min(self.image.width());
        let start_y = (y as f32 * self.sample_height).round() as u32;
        let end_y = (((y + row_height) as f32 *
                      self.sample_height).round() as u32)
            .min(self.image.height());

        let mut counts = self.counts.take();

        counts.clear();

        for y in start_y..end_y {
            for x in start_x..end_x {
                let color = self.image.get_pixel(x, y);
                *counts.entry(color.clone()).or_insert(0) += 1;
            }
        }

        let result = counts.keys()
            .max_by_key(|&color| counts[color])
            .cloned()
            .unwrap_or(None);

        self.counts.replace(counts);

        result
    }
}

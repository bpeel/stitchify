// Stichify – A utility to generate intersia knitting patterns
// Copyright (C) 2024  Neil Roberts
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

use super::config::Dimensions;
use std::collections::HashMap;

pub type Color = [u8; 3];

pub trait Image {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn get_pixel(&self, x: u32, y: u32) -> Color;
}

#[derive(Clone)]
pub struct Stitch {
    pub color: Color,
    pub thread: u16,
}

pub struct Thread {
    pub x: u16,
    pub y: u16,
    pub id: u16,
    pub color: Color,
    pub stitch_count: u32,
}

pub struct Fabric {
    stitches: Vec<Stitch>,
    n_stitches: u16,
    n_rows: u16,
    threads: Vec<Thread>,
}

fn most_popular_color<I: Image>(
    image: &I,
    start_x: u32,
    end_x: u32,
    start_y: u32,
    end_y: u32,
) -> Color {
    let mut colors = HashMap::<Color, u32>::new();

    for y in start_y..end_y {
        for x in start_x..end_x {
            let color = image.get_pixel(x, y);
            colors.entry(color.clone())
                .and_modify(|e| *e += 1)
                .or_insert(1);
        }
    }

    colors.keys().max_by_key(|&color| colors[color]).unwrap().clone()
}

impl Fabric {
    pub fn new<I: Image>(
        image: &I,
        dimensions: &Dimensions,
    ) -> Fabric {
        let mut stitches = Vec::new();

        let sample_width = image.width() as f32 / dimensions.stitches as f32;
        let sample_height = sample_width
            * dimensions.gauge_stitches as f32
            / dimensions.gauge_rows as f32;
        let n_rows = (image.height() as f32 / sample_height).round() as u16;

        stitches.resize(
            (n_rows * dimensions.stitches) as usize,
            Stitch { color: [0, 0, 0], thread: 0 },
        );

        for y in (0..n_rows).rev().step_by(dimensions.duplicate_rows as usize) {
            let sample_start_y =
                (sample_height * (y as f32
                                  - (dimensions.duplicate_rows - 1) as f32))
                .round()
                .max(0.0) as u32;
            let sample_end_y = ((sample_height * (y + 1) as f32).round() as u32)
                .min(image.height());

            let (before_row, row) = stitches.split_at_mut(
                (y * dimensions.stitches) as usize
            );

            for x in 0..dimensions.stitches as u32 {
                let sample_start_x = (sample_width * x as f32).round() as u32;
                let sample_end_x = ((sample_width * (x as f32 + 1.0))
                                    .round() as u32)
                    .min(image.width());

                let color = most_popular_color(
                    image,
                    sample_start_x,
                    sample_end_x,
                    sample_start_y,
                    sample_end_y,
                );

                row[x as usize].color = color;
            }

            for i in 0..(dimensions.duplicate_rows - 1).min(y) {
                before_row[((y - i - 1) * dimensions.stitches) as usize
                           ..((y - i) * dimensions.stitches) as usize]
                    .clone_from_slice(&row[0..dimensions.stitches as usize]);
            }
        }

        let mut fabric = Fabric {
            stitches,
            n_stitches: dimensions.stitches,
            n_rows,
            threads: Vec::new(),
        };

        fabric.calculate_threads();

        fabric
    }

    fn calculate_threads(&mut self) {
        for y in (0..self.n_rows).rev() {
            for mut x in 0..self.n_stitches {
                if (self.n_rows - 1 - y) & 1 == 0 {
                    x = self.n_stitches - 1 - x;
                }

                let stitch_pos = (x + y * self.n_stitches) as usize;
                let color = self.stitches[stitch_pos].color.clone();
                let thread = self.find_thread(color, x, y);

                thread.stitch_count += 1;

                self.stitches[stitch_pos].thread = thread.id;
            }
        }

        self.threads.sort_unstable_by_key(|thread| thread.id);
    }

    fn find_thread(&mut self, color: Color, x: u16, y: u16) -> &mut Thread {
        for (i, thread) in self.threads.iter_mut().enumerate().rev() {
            if thread.y - y > 2 {
                break;
            }

            if thread.color != color {
                continue;
            }

            if thread.x.abs_diff(x) < 2 {
                let mut thread = self.threads.remove(i);
                thread.x = x;
                thread.y = y;
                self.threads.push(thread);
                return self.threads.last_mut().unwrap();
            }
        }

        let id = self.threads.len() as u16;

        self.threads.push(Thread {
            x,
            y,
            id,
            color: color.clone(),
            stitch_count: 0,
        });

        return self.threads.last_mut().unwrap();
    }

    pub fn threads(&self) -> &[Thread] {
        &self.threads
    }

    pub fn stitches(&self) -> &[Stitch] {
        &self.stitches
    }

    pub fn n_stitches(&self) -> u16 {
        self.n_stitches
    }

    pub fn n_rows(&self) -> u16 {
        self.n_rows
    }
}

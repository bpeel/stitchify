// Stichify – A utility to generate intarsia knitting patterns
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

use super::config::{Dimensions, Link};
use std::collections::HashMap;
use std::fmt;
use std::cmp::Ordering;

const MAX_ROW_GAP: u16 = 2;
const MAX_STITCH_GAP: u16 = 1;

pub type Color = [u8; 3];

pub trait Image {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn get_pixel(&self, x: u32, y: u32) -> Option<Color>;
}

#[derive(Clone, Debug)]
pub struct Stitch {
    pub color: Color,
    pub thread: u16,
}

#[derive(Debug)]
pub struct Thread {
    pub x: u16,
    pub y: u16,
    pub id: u16,
    pub color: Color,
    pub stitch_count: u32,
}

#[derive(Debug)]
pub struct Fabric {
    stitches: Vec<Option<Stitch>>,
    n_stitches: u16,
    n_rows: u16,
    threads: Vec<Thread>,
}

#[derive(Debug)]
pub enum Error {
    LinkNotFound(Link),
    LinkTooFar(Link),
    LinkToDifferentColor(Link),
    PosOutsideOfFabric(u16, u16),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::LinkNotFound(link) => {
                write!(f, "No thread found for link {}", link)
            },
            Error::LinkTooFar(link) => {
                write!(f, "Link is too far: {}", link)
            },
            Error::LinkToDifferentColor(link) => {
                write!(f, "Colors don’t match for link: {}", link)
            },
            Error::PosOutsideOfFabric(x, y) => {
                write!(f, "Position {},{} is outside of the fabric", x, y)
            },
        }
    }
}

fn most_popular_color<I: Image>(
    image: &I,
    start_x: u32,
    end_x: u32,
    start_y: u32,
    end_y: u32,
) -> Option<Color> {
    let mut colors = HashMap::<Option<Color>, u32>::new();

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
    ) -> Result<Fabric, Error> {
        let mut stitches = Vec::new();

        let sample_width = image.width() as f32 / dimensions.stitches as f32;
        let sample_height = sample_width
            * dimensions.gauge_stitches as f32
            / dimensions.gauge_rows as f32;
        let n_rows = (image.height() as f32 / sample_height).round() as u16;

        stitches.resize(
            (n_rows * dimensions.stitches) as usize,
            None,
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

                row[x as usize] = color.map(|color| Stitch {
                    color,
                    thread: 0,
                });
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

        let link_map = fabric.links_to_hash(dimensions)?;
        fabric.calculate_threads(&link_map)?;

        Ok(fabric)
    }

    fn validate_link_pos(&self, (x, y): (u16, u16)) -> Result<(), Error> {
        if x == 0 || x > self.n_stitches || y == 0 || y > self.n_rows
            || self.look_up_link_position((x, y)).is_none()
        {
            Err(Error::PosOutsideOfFabric(x, y))
        } else {
            Ok(())
        }
    }

    fn look_up_link_position(&self, (x, y): (u16, u16)) -> Option<&Color> {
        self.stitches[
            (self.n_stitches - x
             + (self.n_rows - y) * self.n_stitches) as usize
        ].as_ref().map(|stitch| &stitch.color)
    }

    fn links_to_hash(
        &self,
        dimensions: &Dimensions
    ) -> Result<HashMap<(u16, u16), (u16, u16)>, Error> {
        let mut link_map = HashMap::new();

        for link in dimensions.links.iter() {
            self.validate_link_pos(link.source)?;
            self.validate_link_pos(link.dest)?;

            if !dimensions.allow_link_gaps
                && (link.source.0.abs_diff(link.dest.0) > MAX_STITCH_GAP
                    || link.source.1.abs_diff(link.dest.1) > MAX_ROW_GAP)
            {
                return Err(Error::LinkTooFar(link.clone()));
            }

            if self.look_up_link_position(link.source)
                != self.look_up_link_position(link.dest)
            {
                return Err(Error::LinkToDifferentColor(link.clone()));
            }

            if self.compare_position_thread_order(
                link.source,
                link.dest,
            ).is_gt() {
                link_map.insert(link.source, link.dest);
            } else {
                link_map.insert(link.dest, link.source);
            };
        }

        Ok(link_map)
    }

    fn calculate_threads(
        &mut self,
        link_map: &HashMap<(u16, u16), (u16, u16)>,
    ) -> Result<(), Error> {
        for y in (0..self.n_rows).rev() {
            for mut x in 0..self.n_stitches {
                if (self.n_rows - 1 - y) & 1 == 0 {
                    x = self.n_stitches - 1 - x;
                }

                let stitch_pos = (x + y * self.n_stitches) as usize;

                if let Some(stitch) = self.stitches[stitch_pos].as_ref() {
                    let thread = self.find_thread(
                        link_map,
                        stitch.color.clone(),
                        x,
                        y
                    )?;

                    thread.stitch_count += 1;

                    self.stitches[stitch_pos]
                        .as_mut()
                        .unwrap()
                        .thread = thread.id;
                }
            }
        }

        self.threads.sort_unstable_by_key(|thread| thread.id);

        Ok(())
    }

    fn find_thread_in_links(
        &self,
        link_map: &HashMap<(u16, u16), (u16, u16)>,
        x: u16,
        y: u16,
    ) -> Result<Option<usize>, Error> {
        let source = (self.n_stitches - x, self.n_rows - y);

        if let Some(&dest) = link_map.get(&source) {
            for (i, thread) in self.threads.iter().enumerate() {
                if thread.x == self.n_stitches - dest.0
                    && thread.y == self.n_rows - dest.1
                {
                    return Ok(Some(i));
                }
            }

            return Err(Error::LinkNotFound(Link { source, dest }));
        }

        Ok(None)
    }

    fn find_neighboring_thread(
        &self,
        color: Color,
        x: u16,
        y: u16,
    ) -> Option<usize> {
        for (i, thread) in self.threads.iter().enumerate().rev() {
            if thread.y - y > MAX_ROW_GAP {
                break;
            }

            if thread.color != color {
                continue;
            }

            if thread.x.abs_diff(x) <= MAX_STITCH_GAP {
                return Some(i);
            }
        }

        None
    }

    fn find_thread(
        &mut self,
        link_map: &HashMap<(u16, u16), (u16, u16)>,
        color: Color,
        x: u16,
        y: u16,
    ) -> Result<&mut Thread, Error> {
        if let Some(thread_index) =
            self.find_thread_in_links(link_map, x, y)?
            .or_else(|| self.find_neighboring_thread(color, x, y))
        {
            let mut thread = self.threads.remove(thread_index);
            thread.x = x;
            thread.y = y;
            self.threads.push(thread);
        } else {
            let id = self.threads.len() as u16;

            self.threads.push(Thread {
                x,
                y,
                id,
                color: color.clone(),
                stitch_count: 0,
            });
        }

        return Ok(self.threads.last_mut().unwrap());
    }

    fn compare_position_thread_order(
        &self,
        a: (u16, u16),
        b: (u16, u16)
    ) -> Ordering {
        a.1.cmp(&b.1).then_with(|| {
            // On odd rows (where the last row is 1), the ordering is
            // right-to-left, otherwise it is left-to-right. The
            // lowest numbered stitch is on the right so right-to-left
            // means increasing numbers.

            if a.1 & 1 == 0 {
                b.0.cmp(&a.0)
            } else {
                a.0.cmp(&b.0)
            }
        })
    }

    pub fn threads(&self) -> &[Thread] {
        &self.threads
    }

    pub fn stitches(&self) -> &[Option<Stitch>] {
        &self.stitches
    }

    pub fn n_stitches(&self) -> u16 {
        self.n_stitches
    }

    pub fn n_rows(&self) -> u16 {
        self.n_rows
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const FAKE_IMAGE_DATA: &'static [u8] =
        b"##  ##\
          ##  ##\
          \x20#### \
          \x20#### \
          ##  ##\
          ##  ##";

    struct FakeImage {
        width: u32,
        data: &'static [u8],
    }

    impl Default for FakeImage {
        fn default() -> FakeImage {
            FakeImage::new(FAKE_IMAGE_DATA, 6)
        }
    }

    impl FakeImage {
        fn new(data: &'static [u8], width: u32) -> FakeImage {
            FakeImage {
                width,
                data
            }
        }
    }

    impl Image for FakeImage {
        fn width(&self) -> u32 {
            self.width
        }

        fn height(&self) -> u32 {
            self.data.len() as u32 / self.width()
        }

        fn get_pixel(&self, x: u32, y: u32) -> Option<Color> {
            match self.data[(y * self.width() + x) as usize] {
                b' ' => Some([255, 255, 255]),
                b'x' => None,
                _ => Some([0, 0, 0]),
            }
        }
    }

    fn assert_threads(fabric: &Fabric, thread_nums: &[u16]) {
        let fabric_threads = fabric.stitches().iter().map(|stitch| {
            stitch.as_ref().unwrap().thread
        }).collect::<Vec<u16>>();

        assert_eq!(&fabric_threads, thread_nums);
    }

    #[test]
    fn links() {
        let image = FakeImage::default();
        let mut dimensions = Dimensions::default();

        dimensions.gauge_stitches = 1;
        dimensions.gauge_rows = 1;
        dimensions.stitches = image.width() as u16;

        let fabric = Fabric::new(&image, &dimensions).unwrap();

        assert_eq!(fabric.n_stitches(), image.width() as u16);
        assert_eq!(fabric.n_rows(), image.height() as u16);

        assert_eq!(fabric.threads().len(), 7);

        assert_eq!(fabric.threads()[0].stitch_count, 16);
        assert_eq!(fabric.threads()[1].stitch_count, 4);
        assert_eq!(fabric.threads()[2].stitch_count, 4);
        assert_eq!(fabric.threads()[3].stitch_count, 2);
        assert_eq!(fabric.threads()[4].stitch_count, 2);
        assert_eq!(fabric.threads()[5].stitch_count, 4);
        assert_eq!(fabric.threads()[6].stitch_count, 4);

        for (i, thread) in fabric.threads().iter().enumerate() {
            assert_eq!(thread.id as usize, i);
            assert_eq!(
                thread.color,
                fabric.stitches[
                    (thread.x + thread.y * fabric.n_stitches())
                        as usize
                ].as_ref().unwrap().color,
            );
        }

        assert_threads(
            &fabric,
            &[
                6, 6, 5, 5, 0, 0,
                6, 6, 5, 5, 0, 0,
                4, 0, 0, 0, 0, 3,
                4, 0, 0, 0, 0, 3,
                2, 2, 1, 1, 0, 0,
                2, 2, 1, 1, 0, 0,
            ],
        );

        const LINKED_THREADS: [u16; 36] = [
                2, 2, 5, 5, 0, 0,
                2, 2, 5, 5, 0, 0,
                4, 2, 2, 0, 0, 3,
                4, 2, 2, 0, 0, 3,
                2, 2, 1, 1, 0, 0,
                2, 2, 1, 1, 0, 0,
        ];

        dimensions.links = vec![
            Link { source: (4, 3), dest: (5, 2) },
            Link { source: (3, 4), dest: (3, 3) },
        ];

        let fabric = Fabric::new(&image, &dimensions).unwrap();

        assert_eq!(fabric.threads().len(), 6);

        assert_threads(&fabric, &LINKED_THREADS);

        // Try again with the source and dest swapped
        dimensions.links = vec![
            Link { source: (5, 2), dest: (4, 3) },
            Link { source: (3, 3), dest: (3, 4) },
        ];

        let fabric = Fabric::new(&image, &dimensions).unwrap();

        assert_eq!(fabric.threads().len(), 6);

        assert_threads(&fabric, &LINKED_THREADS);
    }

    #[test]
    fn thread_order() {
        let image = FakeImage::default();
        let mut dimensions = Dimensions::default();

        dimensions.gauge_stitches = 1;
        dimensions.gauge_rows = 1;
        dimensions.stitches = image.width() as u16;

        let fabric = Fabric::new(&image, &dimensions).unwrap();

        assert_eq!(fabric.n_rows(), 6);

        assert_eq!(
            fabric.compare_position_thread_order((2, 5), (3, 5)),
            Ordering::Less,
        );
        assert_eq!(
            fabric.compare_position_thread_order((3, 5), (2, 5)),
            Ordering::Greater,
        );
        assert_eq!(
            fabric.compare_position_thread_order((3, 5), (3, 5)),
            Ordering::Equal,
        );

        assert_eq!(
            fabric.compare_position_thread_order((2, 4), (3, 4)),
            Ordering::Greater,
        );
        assert_eq!(
            fabric.compare_position_thread_order((3, 4), (2, 4)),
            Ordering::Less,
        );
        assert_eq!(
            fabric.compare_position_thread_order((3, 4), (3, 4)),
            Ordering::Equal,
        );
    }

    #[test]
    fn horizontal_link() {
        let image = FakeImage::default();
        let mut dimensions = Dimensions::default();

        dimensions.gauge_stitches = 1;
        dimensions.gauge_rows = 1;
        dimensions.stitches = image.width() as u16;
        dimensions.links.push(Link { source: (1, 1), dest: (2, 1) });

        Fabric::new(&image, &dimensions).unwrap();
    }

    #[test]
    fn allow_link_gaps() {
        let image = FakeImage::default();
        let mut dimensions = Dimensions::default();

        dimensions.gauge_stitches = 1;
        dimensions.gauge_rows = 1;
        dimensions.stitches = image.width() as u16;
        dimensions.links.push(Link { source: (5, 1), dest: (2, 1) });

        assert_eq!(
            Fabric::new(&image, &dimensions).unwrap_err().to_string(),
            "Link is too far: 5,1->2,1",
        );

        dimensions.allow_link_gaps = true;

        Fabric::new(&image, &dimensions).unwrap();
    }

    #[test]
    fn link_to_missing_stitch() {
        const IMAGE_DATA: &'static [u8] = b"x  x";
        let image = FakeImage::new(IMAGE_DATA, 4);
        let mut dimensions = Dimensions::default();

        dimensions.gauge_stitches = 1;
        dimensions.gauge_rows = 1;
        dimensions.stitches = image.width() as u16;

        dimensions.links.push(Link { source: (3, 1), dest: (2, 1) });

        Fabric::new(&image, &dimensions).unwrap();

        dimensions.links[0] = Link { source: (4, 1), dest: (3, 1) };

        assert_eq!(
            Fabric::new(&image, &dimensions).unwrap_err().to_string(),
            "Position 4,1 is outside of the fabric",
        );

        dimensions.links[0] = Link { source: (2, 1), dest: (1, 1) };

        assert_eq!(
            Fabric::new(&image, &dimensions).unwrap_err().to_string(),
            "Position 1,1 is outside of the fabric",
        );
    }
}

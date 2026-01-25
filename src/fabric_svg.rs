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

use super::fabric::{Fabric, Thread};
use super::stitch_image::Color;
use super::dimensions::{Dimensions, StitchText};
use std::collections::HashMap;
use std::fmt::Write;
use std::fmt;

const BOX_WIDTH: f32 = 20.0;
const LINE_WIDTH: f32 = BOX_WIDTH / 6.0;

pub trait Document {
    type Element: Element;

    fn create_element(&self, name: &str) -> Self::Element;
}

pub trait Element {
    fn set_root_namespace(&mut self, namespace: &str);
    fn add_namespace(&mut self, keyword: &str, namespace: &str);
    fn add_child(&mut self, child: Self);
    fn add_text(&mut self, value: impl ToString);
    fn add_attribute(&mut self, name: &str, value: impl ToString);
    fn add_attribute_ns(
        &mut self,
        keyword: &str,
        name: &str,
        value: impl ToString
    );
}

struct SvgGenerator<'a, D: Document> {
    box_width: f32,
    box_height: f32,
    fabric: &'a Fabric,
    dimensions: &'a Dimensions,
    document: &'a D,
}

impl<'a, D: Document> SvgGenerator<'a, D> {
    fn generate_box(
        &self,
        x: u16,
        y: u16,
        color: Color,
    ) -> D::Element {
        let mut path = self.document.create_element("path");

        path.add_attribute(
            "fill",
            format!(
                "rgb({}%, {}%, {}%)",
                color[0] as f32 * 100.0 / 255.0,
                color[1] as f32 * 100.0 / 255.0,
                color[2] as f32 * 100.0 / 255.0,
            ),
        );

        path.add_attribute(
            "d",
            format!(
                "M {} {} l {} 0 l 0 {} l -{} 0 z",
                x as f32 * self.box_width,
                y as f32 * self.box_height,
                self.box_width,
                self.box_height,
                self.box_width,
            ),
        );

        path
    }

    fn generate_boxes(&self) -> D::Element {
        let mut group = self.document.create_element("g");

        group.add_attribute("id", "boxes");

        for (stitch_num, stitch) in self.fabric.stitches().iter().enumerate() {
            if let Some(stitch) = stitch {
                let x = stitch_num as u16 % self.fabric.n_stitches() + 1;
                let y = stitch_num as u16 / self.fabric.n_stitches() + 1;

                group.add_child(self.generate_box(x, y, stitch.color));
            }
        }

        group
    }

    fn generate_grid_no_id(&self, n_columns: u16, n_rows: u16) -> D::Element {
        let mut path = self.document.create_element("path");

        path.add_attribute("stroke-width", LINE_WIDTH);
        path.add_attribute("stroke-linecap", "square");
        path.add_attribute("stroke-linejoin", "miter");
        path.add_attribute("stroke", "rgb(71%, 71%, 71%)");

        let mut path_str = String::new();

        for x in 0..=n_columns {
            if x != 0 {
                path_str.push(' ');
            }

            write!(
                &mut path_str,
                "M {} 0 l 0 {}",
                x as f32 * self.box_width,
                n_rows as f32 * self.box_height,
            ).unwrap();
        }

        for y in 0..=n_rows {
            if y != 0 {
                path_str.push(' ');
            }

            write!(
                &mut path_str,
                "M 0 {} l {} 0",
                y as f32 * self.box_height,
                n_columns as f32 * self.box_width,
            ).unwrap();
        }

        path.add_attribute("d", path_str);

        path
    }

    fn generate_grid(&self) -> D::Element {
        let mut group = self.document.create_element("g");

        group.add_attribute("id", "grid");

        group.add_attribute(
            "transform",
            format!(
                "translate({} {})",
                self.box_width,
                self.box_height
            ),
        );

        let path = self.generate_grid_no_id(
            self.fabric.n_stitches(),
            self.fabric.n_rows()
        );

        group.add_child(path);

        group
    }

    fn set_text_appearance(&self, element: &mut D::Element) {
        element.add_attribute("font-family", "Sans");
        element.add_attribute("font-size", self.box_height * 0.6);
    }

    fn set_text_y(&self, text: &mut D::Element, y: f32) {
        text.add_attribute("y", y + 0.7 * self.box_height);
    }

    fn set_text_position(
        &self,
        text: &mut D::Element,
        x: f32,
        y: f32,
    ) {
        text.add_attribute("x", x + 0.5 * self.box_width);
        self.set_text_y(text, y);
        text.add_attribute("text-anchor", "middle");
    }

    fn generate_rulers(&self) -> D::Element {
        let mut group = self.document.create_element("g");

        group.add_attribute("id", "rulers");

        self.set_text_appearance(&mut group);

        let mut left_rulers = self.document.create_element("g");

        left_rulers.add_attribute("id", "left-rulers");

        for y in 0..self.fabric.n_rows() {
            let mut text = self.document.create_element("text");

            self.set_text_position(
                &mut text,
                0.0,
                (y + 1) as f32 * self.box_height,
            );

            text.add_text(self.fabric.n_rows() - y);

            left_rulers.add_child(text);
        }

        group.add_child(left_rulers);

        let mut right_rulers = self.document.create_element("use");
        right_rulers.add_attribute("xlink:href", "#left-rulers");
        right_rulers.add_attribute(
            "x",
            BOX_WIDTH * (self.fabric.n_stitches() + 1) as f32,
        );

        group.add_child(right_rulers);

        let mut top_rulers = self.document.create_element("g");

        top_rulers.add_attribute("id", "top-rulers");

        for x in 0..self.fabric.n_stitches() {
            let mut text = self.document.create_element("text");

            self.set_text_position(
                &mut text,
                (x + 1) as f32 * self.box_width,
                0.0,
            );

            text.add_text(self.fabric.n_stitches() - x);

            top_rulers.add_child(text);
        }

        group.add_child(top_rulers);

        let mut bottom_rulers = self.document.create_element("use");
        bottom_rulers.add_attribute("xlink:href", "#top-rulers");
        bottom_rulers.add_attribute(
            "y",
            self.box_height * (self.fabric.n_rows() + 1) as f32,
        );

        group.add_child(bottom_rulers);

        group
    }

    fn generate_run_counts(&self) -> D::Element {
        let mut group = self.document.create_element("g");

        group.add_attribute("id", "run-counts");

        self.set_text_appearance(&mut group);

        let stitches = self.fabric.stitches();
        let n_stitches = self.fabric.n_stitches();

        for row in 0..self.fabric.n_rows() {
            let y = self.fabric.n_rows() - 1 - row;
            let mut run_count = 0;

            for stitch in 0..n_stitches {
                run_count += 1;

                let x = if (row & 1) == 0 {
                    n_stitches - 1 - stitch
                } else {
                    stitch
                };

                let color = stitches[(y * n_stitches + x) as usize].clone();

                let change = stitch >= n_stitches - 1
                    || {
                        let next = if (row & 1) == 0 {
                            x - 1
                        } else {
                            x + 1
                        };
                        color != stitches[(y * n_stitches + next) as usize]
                    };

                if change {
                    let mut text = self.document.create_element("text");

                    let x = if row & 1 == 0 {
                        x + run_count - 1
                    } else {
                        x + 1 - run_count
                    };

                    self.set_text_position(
                        &mut text,
                        (x + 1) as f32 * BOX_WIDTH,
                        (y + 1) as f32 * self.box_height,
                    );

                    text.add_text(run_count);

                    if let Some(stitch) = color {
                        set_text_color(&mut text, stitch.color);
                    }

                    group.add_child(text);

                    run_count = 0;
                }
            }
        }

        group
    }

    fn generate_midline_rulers(&self) -> D::Element {
        let mut group = self.document.create_element("g");

        group.add_attribute("id", "midline-rulers");

        self.set_text_appearance(&mut group);

        self.fabric.stitches().iter().enumerate().fold(
            self.fabric.stitches()[0].clone(),
            |prev, (i, stitch)| {
                let x = i as u16 % self.fabric.n_stitches();

                if x > 0 && prev != *stitch {
                    let y = i as u16 / self.fabric.n_stitches();

                    let mut text = self.document.create_element("text");

                    self.set_text_position(
                        &mut text,
                        (x + 1) as f32 * BOX_WIDTH,
                        (y + 1) as f32 * self.box_height,
                    );

                    text.add_text(self.fabric.n_rows() - y);

                    if let Some(stitch) = stitch {
                        set_text_color(&mut text, stitch.color);
                    }

                    group.add_child(text);
                }

            stitch.clone()
        });

        group
    }

    fn generate_missing_stitches(&self) -> D::Element {
        let mut group = self.document.create_element("g");

        group.add_attribute("id", "missing-stitches");

        for (stitch_num, stitch) in self.fabric.stitches().iter().enumerate() {
            if stitch.is_none() {
                let x = stitch_num as u16 % self.fabric.n_stitches() + 1;
                let y = stitch_num as u16 / self.fabric.n_stitches() + 1;

                let mut path = self.document.create_element("path");

                path.add_attribute("stroke-width", LINE_WIDTH / 2.0);
                path.add_attribute("stroke", "rgb(71%, 71%, 71%)");

                path.add_attribute(
                    "d",
                    format!(
                        "M {} {} l {} {} M {} {} l {} {}",
                        x as f32 * self.box_width,
                        y as f32 * self.box_height,
                        self.box_width,
                        self.box_height,
                        (x + 1) as f32 * self.box_width,
                        y as f32 * self.box_height,
                        -self.box_width,
                        self.box_height,
                    ),
                );

                group.add_child(path);
            }
        }

        group
    }

    fn generate_box_thread_text(
        &self,
        thread: u16,
        x: f32,
        y: f32,
        color: Color,
    ) -> D::Element {
        let mut element = self.document.create_element("use");

        element.add_attribute_ns(
            "xlink",
            "href",
            format!("#thread-{}", thread)
        );
        element.add_attribute("x", x);
        element.add_attribute("y", y);

        set_text_color(&mut element, color);

        element
    }

    fn generate_box_threads(&self) -> D::Element {
        let mut group = self.document.create_element("g");

        group.add_attribute("id", "box-threads");

        for (stitch_num, stitch) in self.fabric.stitches().iter().enumerate() {
            if let Some(stitch) = stitch {
                let x = stitch_num as u16 % self.fabric.n_stitches() + 1;
                let y = stitch_num as u16 / self.fabric.n_stitches() + 1;

                group.add_child(self.generate_box_thread_text(
                    stitch.thread,
                    x as f32 * self.box_width,
                    y as f32 * self.box_height,
                    stitch.color,
                ));
            }
        }

        group
    }

    fn generate_stitch_count(&self, y: usize, count: u32) -> D::Element {
        let mut count_text = self.document.create_element("text");
        count_text.add_attribute("x", self.box_width as f32 * 1.5);
        self.set_text_y(&mut count_text, y as f32 * self.box_height);

        count_text.add_text(
            stitch_count_text(&self.dimensions, count)
        );

        count_text
    }

    fn generate_thread_counts(&self) -> D::Element {
        let mut group = self.document.create_element("g");

        group.add_attribute("id", "thread-counts");

        group.add_attribute(
            "transform",
            format!(
                "translate({} {})",
                self.box_width * 2.0,
                self.box_height * (self.fabric.n_rows() + 3) as f32,
            ),
        );

        let mut counts = self.document.create_element("g");

        self.set_text_appearance(&mut counts);

        for (y, thread) in self.fabric.threads().iter().enumerate() {
            group.add_child(self.generate_box(
                0,
                y as u16,
                thread.color,
            ));

            group.add_child(self.generate_box_thread_text(
                thread.id,
                0.0,
                y as f32 * self.box_height,
                thread.color,
            ));

            counts.add_child(self.generate_stitch_count(
                y,
                thread.stitch_count
            ));
        }

        group.add_child(self.generate_grid_no_id(
            1,
            self.fabric.threads().len() as u16,
        ));

        group.add_child(counts);

        group
    }

    fn generate_color_counts(&self) -> D::Element {
        let mut group = self.document.create_element("g");

        group.add_attribute("id", "color-counts");

        let x_offset = if self.dimensions.show_thread_counts {
            7.0
        } else {
            1.0
        };

        group.add_attribute(
            "transform",
            format!(
                "translate({} {})",
                self.box_width * x_offset,
                self.box_height * (self.fabric.n_rows() + 3) as f32,
            ),
        );

        let mut text_group = self.document.create_element("g");

        self.set_text_appearance(&mut text_group);

        let counts = count_color_stitches(&self.fabric.threads());

        for (y, &(color, count)) in counts.iter().enumerate() {
            group.add_child(self.generate_box(
                0,
                y as u16,
                color,
            ));

            text_group.add_child(self.generate_stitch_count(y, count));
        }

        group.add_child(self.generate_grid_no_id(
            1,
            counts.len() as u16,
        ));

        group.add_child(text_group);

        group
    }

    fn generate_defs(&self) -> D::Element {
        let mut defs = self.document.create_element("defs");

        for thread in self.fabric.threads().iter() {
            let text = if thread.id == 0 {
                "A".to_string()
            } else {
                let mut parts = Vec::new();
                let mut id = thread.id;

                while id > 0 {
                    parts.push(
                        char::from_u32('A' as u32 + id as u32 % 26).unwrap()
                    );
                    id /= 26;
                }

                parts.iter().rev().collect::<String>()
            };

            let mut element = self.document.create_element("text");

            self.set_text_appearance(&mut element);
            self.set_text_position(&mut element, 0.0, 0.0);

            element.add_text(text);
            element.add_attribute("id", format!("thread-{}", thread.id));

            defs.add_child(element);
        }

        defs
    }
}

fn calc_n_visible_rows(dimensions: &Dimensions, fabric: &Fabric) -> usize
{
    let n_fabric_rows = fabric.n_rows() as usize + 2;

    if dimensions.show_thread_counts {
        n_fabric_rows + 1 + fabric.threads().len()
    } else if dimensions.show_color_counts {
        n_fabric_rows + 1 + count_color_stitches(fabric.threads()).len()
    } else {
        n_fabric_rows
    }
}

pub fn convert<D: Document>(
    document: &D,
    dimensions: &Dimensions,
    fabric: &Fabric
) -> D::Element {
    let generator = SvgGenerator {
        dimensions: dimensions,
        box_width: BOX_WIDTH,
        box_height: BOX_WIDTH
            * dimensions.gauge_stitches
            / dimensions.gauge_rows,
        fabric,
        document,
    };

    let n_visible_rows = calc_n_visible_rows(dimensions, fabric);

    let svg_width = (fabric.n_stitches() + 2) as f32 * BOX_WIDTH;
    let svg_height = (n_visible_rows as f32 * generator.box_height)
        + LINE_WIDTH / 2.0;

    let mut svg = document.create_element("svg");

    svg.set_root_namespace("http://www.w3.org/2000/svg");
    svg.add_namespace("xlink", "http://www.w3.org/1999/xlink");
    svg.add_attribute("width", svg_width);
    svg.add_attribute("height", svg_height);
    svg.add_attribute(
        "viewBox",
        format!(
            "0 0 {} {}",
            svg_width,
            svg_height
        )
    );

    svg.add_child(generator.generate_defs());

    svg.add_child(generator.generate_boxes());

    svg.add_child(generator.generate_grid());

    svg.add_child(generator.generate_rulers());

    svg.add_child(generator.generate_missing_stitches());

    match dimensions.stitch_text {
        StitchText::None => (),
        StitchText::Thread => svg.add_child(generator.generate_box_threads()),
        StitchText::Runs => svg.add_child(generator.generate_run_counts()),
        StitchText::Ruler => svg.add_child(generator.generate_midline_rulers()),
    }

    if dimensions.show_thread_counts {
        svg.add_child(generator.generate_thread_counts());
    }

    if dimensions.show_color_counts {
        svg.add_child(generator.generate_color_counts());
    }

    svg
}

fn mm_to_text(mut out: impl Write, mm: u32) -> fmt::Result {
    if mm < 10 {
        write!(out, "{}mm", mm)?;
    } else {
        let cm = (mm + 5) / 10;

        if cm < 100 {
            write!(out, "{}cm", cm)?;
        } else {
            write!(out, "{}", cm / 100)?;

            let mut rem_cm = cm % 100;

            if rem_cm > 0 {
                out.write_char('.')?;

                while rem_cm > 0 {
                    write!(out, "{}", rem_cm / 10)?;
                    rem_cm = rem_cm * 10 % 100;
                }
            }

            out.write_char('m')?;
        }
    }

    Ok(())
}

fn stitch_count_text(dimensions: &Dimensions, n_stitches: u32) -> String {
    let mut count = format!("{} (", n_stitches);

    let mm = match dimensions.cm_per_stitch {
        Some(cm_per_stitch) => {
            (n_stitches as f32 * cm_per_stitch * 10.0).round() as u32
        },
        None => {
            // Multiply by 100 because the gauge is probably in
            // stitches per 10cm. Multiply by 3 because there is a
            // rule of thumb that it takes approximately 3 times as
            // much yarn to make the stitch than the resulting width
            // of the stitch. Add half of the gauge to round to the
            // nearest integer instead of rounding down.
            ((n_stitches as f32 * 100.0 * 3.0 + dimensions.gauge_stitches / 2.0)
             / dimensions.gauge_stitches)
                .round() as u32
        },
    };

    mm_to_text(&mut count, mm).unwrap();

    count.push(')');

    count
}

fn count_color_stitches(threads: &[Thread]) -> Vec<(Color, u32)> {
    let mut counts = HashMap::<Color, u32>::new();

    for thread in threads.iter() {
        counts.entry(thread.color.clone())
            .and_modify(|count| *count += thread.stitch_count)
            .or_insert(thread.stitch_count);
    }

    let mut counts = counts.into_iter().collect::<Vec<(Color, u32)>>();

    counts.sort_by_key(|count| u32::MAX - count.1);

    counts
}

fn set_text_color<E: Element>(
    element: &mut E,
    box_color: Color,
) {
    if box_color.iter().map(|&x| x as u16).sum::<u16>() < 384 {
        element.add_attribute("fill", "rgb(100%, 100%, 100%)");
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn mm_to_string(mm: u32) -> String {
        let mut result = String::new();

        mm_to_text(&mut result, mm).unwrap();

        result
    }

    #[test]
    fn test_mm_to_text() {
        assert_eq!(&mm_to_string(1), "1mm");
        assert_eq!(&mm_to_string(9), "9mm");
        assert_eq!(&mm_to_string(10), "1cm");
        assert_eq!(&mm_to_string(11), "1cm");
        assert_eq!(&mm_to_string(15), "2cm");
        assert_eq!(&mm_to_string(19), "2cm");
        assert_eq!(&mm_to_string(994), "99cm");
        assert_eq!(&mm_to_string(995), "1m");
        assert_eq!(&mm_to_string(1000), "1m");
        assert_eq!(&mm_to_string(1100), "1.1m");
        assert_eq!(&mm_to_string(1120), "1.12m");
        assert_eq!(&mm_to_string(1126), "1.13m");
    }

    #[test]
    fn test_stitch_count_text() {
        let mut dimensions = Dimensions::default();

        dimensions.gauge_stitches = 31.0;

        assert_eq!(
            &stitch_count_text(&dimensions, 31),
            "31 (30cm)"
        );

        assert_eq!(
            &stitch_count_text(&dimensions, 46),
            "46 (45cm)"
        );

        dimensions.cm_per_stitch = Some(100.0);

        assert_eq!(
            &stitch_count_text(&dimensions, 345),
            "345 (345m)",
        );
    }
}

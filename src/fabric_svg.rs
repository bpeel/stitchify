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

use super::fabric::{Fabric, Color};
use simple_xml_builder::XMLElement;
use super::config::Dimensions;
use std::fmt::Write;

const BOX_WIDTH: u16 = 20;
const LINE_WIDTH: f32 = BOX_WIDTH as f32 / 6.0;

fn generate_box(
    box_width: u16,
    box_height: u16,
    x: u16,
    y: u16,
    color: Color,
) -> XMLElement {
    let mut path = XMLElement::new("path");

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
            x * box_width,
            y * box_height,
            box_width,
            box_height,
            box_width,
        ),
    );

    path
}

fn generate_boxes(
    box_width: u16,
    box_height: u16,
    fabric: &Fabric,
) -> XMLElement {
    let mut group = XMLElement::new("g");

    group.add_attribute("id", "boxes");

    for (stitch_num, stitch) in fabric.stitches().iter().enumerate() {
        let x = stitch_num as u16 % fabric.n_stitches();
        let y = stitch_num as u16 / fabric.n_stitches();

        group.add_child(generate_box(
            box_width,
            box_height,
            x,
            y,
            stitch.color,
        ));
    }

    group
}

fn generate_grid_no_id(
    box_width: u16,
    box_height: u16,
    n_columns: u16,
    n_rows: u16,
) -> XMLElement {
    let mut path = XMLElement::new("path");

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
            x * box_width,
            n_rows * box_height,
        ).unwrap();
    }

    for y in 0..=n_rows {
        if y != 0 {
            path_str.push(' ');
        }

        write!(
            &mut path_str,
            "M 0 {} l {} 0",
            y * box_height,
            n_columns * box_width,
        ).unwrap();
    }

    path.add_attribute("d", path_str);

    path
}

fn generate_grid(
    box_width: u16,
    box_height: u16,
    n_columns: u16,
    n_rows: u16,
) -> XMLElement {
    let mut path = generate_grid_no_id(
        box_width,
        box_height,
        n_columns,
        n_rows
    );

    path.add_attribute("id", "grid");

    path
}

fn set_text_appearance(
    element: &mut XMLElement,
    box_height: u16,
) {
    element.add_attribute("font-family", "Sans");
    element.add_attribute("font-size", box_height as f32 * 0.6);
}

fn set_text_position(
    text: &mut XMLElement,
    box_width: u16,
    box_height: u16,
    x: u16,
    y: u16,
) {
    text.add_attribute("x", x as f32 + 0.5 * box_width as f32);
    text.add_attribute("y", y as f32 + 0.7 * box_height as f32);
    text.add_attribute("text-anchor", "middle");
}

fn generate_rulers(
    box_width: u16,
    box_height: u16,
    n_columns: u16,
    n_rows: u16,
) -> XMLElement {
    let mut group = XMLElement::new("g");

    group.add_attribute("id", "rulers");

    set_text_appearance(&mut group, box_height);

    for y in 0..n_rows {
        let mut text = XMLElement::new("text");

        set_text_position(
            &mut text,
            box_width,
            box_height,
            n_columns * box_width,
            y * box_height,
        );

        text.add_text(n_rows - y);

        group.add_child(text);
    }

    for x in 0..n_columns {
        let mut text = XMLElement::new("text");

        set_text_position(
            &mut text,
            box_width,
            box_height,
            x * box_width,
            n_rows * box_height,
        );

        text.add_text(n_columns - x);

        group.add_child(text);
    }

    group
}

fn generate_box_thread_text(
    thread: u16,
    x: u16,
    y: u16,
    color: Color,
) -> XMLElement {
    let mut element = XMLElement::new("use");

    element.add_attribute(
        "xlink:href",
        format!("#thread-{}", thread)
    );
    element.add_attribute("x", x);
    element.add_attribute("y", y);

    if color.iter().map(|&x| x as u16).sum::<u16>() < 384 {
        element.add_attribute("fill", "rgb(100%, 100%, 100%)");
    }

    element
}

fn generate_box_threads(
    box_width: u16,
    box_height: u16,
    fabric: &Fabric,
) -> XMLElement {
    let mut group = XMLElement::new("g");

    group.add_attribute("id", "box-threads");

    for (stitch_num, stitch) in fabric.stitches().iter().enumerate() {
        let x = stitch_num as u16 % fabric.n_stitches();
        let y = stitch_num as u16 / fabric.n_stitches();

        group.add_child(generate_box_thread_text(
            stitch.thread,
            x * box_width,
            y * box_height,
            stitch.color,
        ));
    }

    group
}

fn generate_thread_counts(
    box_width: u16,
    box_height: u16,
    fabric: &Fabric,
) -> XMLElement {
    let mut group = XMLElement::new("g");

    group.add_attribute("id", "thread-counts");

    group.add_attribute(
        "transform",
        format!(
            "translate({} {})",
            box_width,
            box_height * (fabric.n_rows() + 2),
        ),
    );

    let mut counts = XMLElement::new("g");

    set_text_appearance(&mut counts, box_height);

    for (y, thread) in fabric.threads().iter().enumerate() {
        group.add_child(generate_box(
            box_width,
            box_height,
            0,
            y as u16,
            thread.color,
        ));

        group.add_child(generate_box_thread_text(
            thread.id,
            0,
            y as u16 * box_height,
            thread.color,
        ));

        let mut count_text = XMLElement::new("text");
        set_text_position(
            &mut count_text,
            box_width,
            box_height,
            box_width,
            y as u16 * box_height,
        );
        count_text.add_text(format!("{}", thread.stitch_count));
        counts.add_child(count_text);
    }

    group.add_child(generate_grid_no_id(
        box_width,
        box_height,
        1,
        fabric.threads().len() as u16,
    ));

    group.add_child(counts);

    group
}

fn generate_defs(
    fabric: &Fabric,
    box_width: u16,
    box_height: u16,
) -> XMLElement {
    let mut defs = XMLElement::new("defs");

    for thread in fabric.threads().iter() {
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

        let mut element = XMLElement::new("text");

        set_text_appearance(&mut element, box_height);
        set_text_position(&mut element, box_width, box_height, 0, 0);

        element.add_text(text);
        element.add_attribute("id", format!("thread-{}", thread.id));

        defs.add_child(element);
    }

    defs
}

pub fn convert(dimensions: &Dimensions, fabric: &Fabric) -> XMLElement {
    let box_height = BOX_WIDTH
        * dimensions.gauge_stitches
        / dimensions.gauge_rows;

    let mut svg = XMLElement::new("svg");

    let svg_width = ((fabric.n_stitches() + 1) * BOX_WIDTH) as f32
        + LINE_WIDTH / 2.0;
    let svg_height = ((fabric.n_rows() + 2 + fabric.threads().len() as u16)
                      * box_height) as f32
        + LINE_WIDTH;

    svg.add_attribute("xmlns", "http://www.w3.org/2000/svg");
    svg.add_attribute("xmlns:xlink", "http://www.w3.org/1999/xlink");
    svg.add_attribute("width", svg_width);
    svg.add_attribute("height", svg_height);
    svg.add_attribute("viewBox", format!("0 0 {} {}", svg_width, svg_height));

    svg.add_child(generate_defs(fabric, BOX_WIDTH, box_height));

    let mut translation = XMLElement::new("g");
    translation.add_attribute(
        "transform",
        format!("translate({} {})", LINE_WIDTH / 2.0, LINE_WIDTH / 2.0),
    );

    translation.add_child(generate_boxes(
        BOX_WIDTH,
        box_height,
        fabric,
    ));

    translation.add_child(generate_grid(
        BOX_WIDTH,
        box_height,
        fabric.n_stitches(),
        fabric.n_rows(),
    ));

    translation.add_child(generate_rulers(
        BOX_WIDTH,
        box_height,
        fabric.n_stitches(),
        fabric.n_rows(),
    ));

    translation.add_child(generate_box_threads(
        BOX_WIDTH,
        box_height,
        fabric,
    ));

    translation.add_child(generate_thread_counts(
        BOX_WIDTH,
        box_height,
        fabric,
    ));

    svg.add_child(translation);

    svg
}

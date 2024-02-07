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

use super::fabric::Fabric;
use simple_xml_builder::XMLElement;
use super::config::Dimensions;
use std::fmt::Write;

const BOX_WIDTH: u16 = 20;
const LINE_WIDTH: f32 = BOX_WIDTH as f32 / 6.0;

fn generate_boxes(
    box_width: u16,
    box_height: u16,
    fabric: &Fabric,
) -> XMLElement {
    let mut group = XMLElement::new("g");

    for (stitch_num, stitch) in fabric.stitches().iter().enumerate() {
        let x = stitch_num as u16 % fabric.n_stitches();
        let y = stitch_num as u16 / fabric.n_stitches();

        let mut path = XMLElement::new("path");

        path.add_attribute(
            "fill",
            format!(
                "rgb({}%, {}%, {}%)",
                stitch.color[0] as f32 * 100.0 / 255.0,
                stitch.color[1] as f32 * 100.0 / 255.0,
                stitch.color[2] as f32 * 100.0 / 255.0,
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

        group.add_child(path);
    }

    group
}

fn generate_grid(
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

    group.add_attribute("font-family", "Sans");
    group.add_attribute("font-size", box_height as f32 * 0.6);

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

pub fn convert(dimensions: &Dimensions, fabric: &Fabric) -> XMLElement {
    let box_height = BOX_WIDTH
        * dimensions.gauge_stitches
        / dimensions.gauge_rows;

    let mut svg = XMLElement::new("svg");

    let svg_width = ((fabric.n_stitches() + 1) * BOX_WIDTH) as f32
        + LINE_WIDTH / 2.0;
    let svg_height = ((fabric.n_rows() + 1) * box_height) as f32
        + LINE_WIDTH / 2.0;

    svg.add_attribute("xmlns", "http://www.w3.org/2000/svg");
    svg.add_attribute("width", svg_width);
    svg.add_attribute("height", svg_height);
    svg.add_attribute("viewBox", format!("0 0 {} {}", svg_width, svg_height));

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

    svg.add_child(translation);

    svg
}

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

mod config;
mod fabric;
mod fabric_svg;
mod mitre_image;

use std::process::ExitCode;
use std::fs::File;
use image::DynamicImage;
use image::buffer::ConvertBuffer;
use fabric::Image;

struct ImageBufWrapper(image::RgbaImage);

impl Image for ImageBufWrapper {
    fn width(&self) -> u32 {
        self.0.width()
    }

    fn height(&self) -> u32 {
        self.0.height()
    }

    fn get_pixel(&self, x: u32, y: u32) -> Option<fabric::Color> {
        let pixel = self.0.get_pixel(x, y);

        if pixel[3] >= 128 {
            Some([pixel[0], pixel[1], pixel[2]])
        } else {
            None
        }
    }
}

fn build_fabric<I: Image>(
    image: &I,
    config: &config::Config,
) -> Result<fabric::Fabric, fabric::Error> {
    if config.mitre {
        // First generate the fabric with square stitches and without
        // the links
        let mut dimensions = config.dimensions.clone();
        dimensions.gauge_rows = config.dimensions.gauge_stitches;
        dimensions.duplicate_rows = 1;
        dimensions.links.clear();
        let fabric = fabric::Fabric::new(image, &dimensions)?;

        let image = mitre_image::MitreImage::new(&fabric);

        // Next use stitches that are twice as wide as they are tall
        // but force garter stitch
        let mut dimensions = config.dimensions.clone();
        dimensions.gauge_stitches = 1;
        dimensions.gauge_rows = 2;
        dimensions.duplicate_rows = 2;
        dimensions.stitches = image.width() as u16;

        dimensions.allow_link_gaps = true;

        // Automatically add links across the middle gaps
        if image.height() > 1 {
            let center = image.width() as u16 / 2;

            for y in 2..=image.height() as u16 {
                dimensions.links.push(config::Link {
                    source: (center - y + 1, y * 2 - 1),
                    dest: (center + y, y * 2 - 1),
                });
                dimensions.links.push(config::Link {
                    source: (center + y, y * 2),
                    dest: (center - y + 1, y * 2),
                });
            }
        }

        fabric::Fabric::new(&image, &dimensions)
    } else {
        fabric::Fabric::new(image, &config.dimensions)
    }
}

fn main() -> ExitCode {
    let config = config::Config::parse();

    let image = match image::io::Reader::open(&config.files.input) {
        Ok(image) => image,
        Err(e) => {
            eprintln!("{}: {}", config.files.input, e);
            return ExitCode::FAILURE;
        },
    };

    let image = match image.decode() {
        Ok(image) => image,
        Err(e) => {
            eprintln!("{}: {}", config.files.input, e);
            return ExitCode::FAILURE;
        },
    };

    let image = match image {
        DynamicImage::ImageRgb8(image) => image.convert(),
        DynamicImage::ImageLuma8(image) => image.convert(),
        DynamicImage::ImageLumaA8(image) => image.convert(),
        DynamicImage::ImageRgba8(image) => image,
        DynamicImage::ImageLuma16(image) => image.convert(),
        DynamicImage::ImageLumaA16(image) => image.convert(),
        DynamicImage::ImageRgb16(image) => image.convert(),
        DynamicImage::ImageRgba16(image) => image.convert(),
        DynamicImage::ImageRgb32F(image) => image.convert(),
        DynamicImage::ImageRgba32F(image) => image.convert(),
        _ => {
            eprintln!("{}: unsupported image format", config.files.input);
            return ExitCode::FAILURE;
        },
    };

    let fabric = match build_fabric(
        &ImageBufWrapper(image),
        &config,
    ) {
        Ok(fabric) => fabric,
        Err(e) => {
            eprintln!("{}", e);
            return ExitCode::FAILURE;
        },
    };

    let svg = fabric_svg::convert(&config.dimensions, &fabric);

    let output = match File::create(&config.files.output) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("{}: {}", config.files.output, e);
            return ExitCode::FAILURE;
        },
    };

    match svg.write(output) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{}: {}", config.files.output, e);
            ExitCode::FAILURE
        },
    }
}

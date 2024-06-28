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

use std::process::ExitCode;
use std::fs::File;
use image::DynamicImage;
use image::buffer::ConvertBuffer;

struct ImageBufWrapper(image::RgbaImage);

impl fabric::Image for ImageBufWrapper {
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

    let fabric = match fabric::Fabric::new(
        &ImageBufWrapper(image),
        &config.dimensions,
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

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

use clap::Parser;

#[derive(Parser)]
#[command(name = "Stitchify")]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    input: String,
    #[arg(short, long, value_name = "FILE")]
    output: String,
    #[arg(short, long, value_name = "COUNT", default_value_t = 22,
          value_parser = clap::value_parser!(u16).range(1..))]
    stitches: u16,
    #[arg(long, value_name = "COUNT", default_value_t = 22,
          value_parser = clap::value_parser!(u16).range(1..))]
    gauge_stitches: u16,
    #[arg(long, value_name = "COUNT", default_value_t = 30,
          value_parser = clap::value_parser!(u16).range(1..))]
    gauge_rows: u16,
    #[arg(short, long)]
    garter: bool,
}

#[derive(Clone)]
pub struct Dimensions {
    pub stitches: u16,
    pub gauge_stitches: u16,
    pub gauge_rows: u16,
    pub duplicate_rows: u16,
}

pub struct Files {
    pub input: String,
    pub output: String,
}

pub struct Config {
    pub dimensions: Dimensions,
    pub files: Files,
}

impl Config {
    pub fn parse() -> Config {
        let Cli {
            input,
            output,
            stitches,
            gauge_stitches,
            gauge_rows,
            garter,
        } = Cli::parse();

        Config {
            dimensions: Dimensions {
                stitches,
                gauge_stitches,
                gauge_rows,
                duplicate_rows: garter as u16 + 1,
            },
            files: Files { input, output },
        }
    }
}

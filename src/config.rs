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
use std::str::FromStr;
use std::fmt;

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
    #[arg(long, value_name = "CM")]
    cm_per_stitch: Option<f32>,
    #[arg(short, long)]
    garter: bool,
    #[arg(short, long = "link", value_name = "LINK",
          value_parser = clap::value_parser!(Link))]
    links: Vec<Link>,
}

#[derive(Clone, Debug)]
pub struct Link {
    pub source: (u16, u16),
    pub dest: (u16, u16),
}

#[derive(Debug)]
pub enum LinkParseError {
    MissingElement,
    TooManyElements,
    ParseIntError(std::num::ParseIntError),
}

impl From<std::num::ParseIntError> for LinkParseError {
    fn from(e: std::num::ParseIntError) -> LinkParseError {
        LinkParseError::ParseIntError(e)
    }
}

impl std::error::Error for LinkParseError {
}

impl fmt::Display for LinkParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            LinkParseError::ParseIntError(e) => write!(f, "{}", e),
            LinkParseError::MissingElement
                | LinkParseError::TooManyElements =>
            {
                write!(f, "Link argument must be of the form “x,y,x,y”")
            },
        }
    }
}

impl FromStr for Link {
    type Err = LinkParseError;

    fn from_str(s: &str) -> Result<Link, LinkParseError> {
        let mut value_count = 0usize;
        let mut link = Link { source: (0, 0), dest: (0, 0) };

        for part in s.split(',') {
            let part = part.parse::<u16>()?;

            match value_count {
                0 => link.source.0 = part,
                1 => link.source.1 = part,
                2 => link.dest.0 = part,
                3 => link.dest.1 = part,
                _ => return Err(LinkParseError::TooManyElements),
            }

            value_count += 1;
        }

        if value_count < 4 {
            Err(LinkParseError::MissingElement)
        } else {
            Ok(link)
        }
    }
}

impl fmt::Display for Link {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{},{}->{},{}",
            self.source.0,
            self.source.1,
            self.dest.0,
            self.dest.1,
        )
    }
}

#[derive(Clone)]
pub struct Dimensions {
    pub stitches: u16,
    pub gauge_stitches: u16,
    pub gauge_rows: u16,
    pub cm_per_stitch: Option<f32>,
    pub duplicate_rows: u16,
    pub links: Vec<Link>,
}

impl Default for Dimensions {
    fn default() -> Dimensions {
        Dimensions {
            stitches: 22,
            gauge_stitches: 22,
            gauge_rows: 30,
            cm_per_stitch: None,
            duplicate_rows: 1,
            links: Vec::new(),
        }
    }
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
            cm_per_stitch,
            garter,
            links,
        } = Cli::parse();

        Config {
            dimensions: Dimensions {
                stitches,
                gauge_stitches,
                gauge_rows,
                cm_per_stitch,
                duplicate_rows: garter as u16 + 1,
                links,
            },
            files: Files { input, output },
        }
    }
}

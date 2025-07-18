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

use clap::{Parser, ValueEnum, builder::PossibleValue};
use super::dimensions::{Dimensions, Link, StitchText};
use super::gauge;

impl ValueEnum for StitchText {
    fn value_variants<'a>() -> &'a [Self] {
        &[StitchText::None, StitchText::Thread, StitchText::Runs]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self {
            StitchText::None => {
                PossibleValue::new("none").help("No thread text")
            },
            StitchText::Thread => {
                PossibleValue::new("thread").help("Thread letter")
            },
            StitchText::Runs => {
                PossibleValue::new("runs").help("Counts of runs of same color")
            },
        })
    }
}

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
    #[arg(long, value_name = "COUNT", default_value_t = 22.0,
          value_parser = gauge::parse)]
    gauge_stitches: f32,
    #[arg(long, value_name = "COUNT", default_value_t = 30.0,
          value_parser = gauge::parse)]
    gauge_rows: f32,
    #[arg(long, value_name = "CM")]
    cm_per_stitch: Option<f32>,
    #[arg(short, long)]
    garter: bool,
    #[arg(short, long)]
    mitre: bool,
    #[arg(short = 't', long, default_value = "thread")]
    stitch_text: StitchText,
    #[arg(short = 'G', long)]
    allow_link_gaps: bool,
    #[arg(short, long = "link", value_name = "LINK",
          value_parser = clap::value_parser!(Link))]
    links: Vec<Link>,
}

pub struct Files {
    pub input: String,
    pub output: String,
}

pub struct Config {
    pub dimensions: Dimensions,
    pub mitre: bool,
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
            mitre,
            stitch_text,
            allow_link_gaps,
            links,
        } = Cli::parse();

        Config {
            dimensions: Dimensions {
                stitches,
                gauge_stitches,
                gauge_rows,
                cm_per_stitch,
                duplicate_rows: garter as u16 + 1,
                allow_link_gaps,
                links,
                stitch_text,
            },
            mitre,
            files: Files { input, output },
        }
    }
}

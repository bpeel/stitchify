// Stichify – A utility to generate intarsia knitting patterns
// Copyright (C) 2025  Neil Roberts
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

use std::num::ParseFloatError;
use std::str::FromStr;
use std::fmt;

const CM_PER_INCH: f32 = 2.54;

#[derive(Debug)]
pub enum Error {
    FloatParseError(ParseFloatError),
    TooSmall(f32),
    Abnormal(f32),
    BothPartsLength,
    BothPartsItems,
}

#[derive(Clone, Copy)]
enum PartType {
    Length,
    Items,
}

struct Part {
    part_type: PartType,
    value: f32,
}

static SUFFIXES: [(&'static str, f32); 4] = [
    ("cm", 1.0),
    ("mm", 0.1),
    ("\"", CM_PER_INCH),
    ("in", CM_PER_INCH),
];

impl From<ParseFloatError> for Error {
    fn from(e: ParseFloatError) -> Error {
        Error::FloatParseError(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::FloatParseError(e) => e.fmt(f),
            Error::TooSmall(v) => {
                write!(f, "gauge {} is too small", v)
            },
            Error::Abnormal(v) => {
                write!(f, "invalid gauge: {}", v)
            },
            Error::BothPartsLength => {
                write!(f, "both parts of the gauge are a length")
            },
            Error::BothPartsItems => {
                write!(f, "both parts of the gauge are stitches or rows")
            },
        }
    }
}

impl std::error::Error for Error {
}

fn parse_float_value(s: &str) -> Result<f32, Error> {
    let value = s.parse()?;

    if value <= 0.0 {
        Err(Error::TooSmall(value))
    } else if !value.is_normal() {
        Err(Error::Abnormal(value))
    } else {
        Ok(value)
    }
}

impl FromStr for Part {
    type Err = Error;

    fn from_str(s: &str) -> Result<Part, Error> {
        for (suffix, multiplier) in SUFFIXES.iter() {
            if let Some(value_str) = s.strip_suffix(suffix) {
                return Ok(Part {
                    part_type: PartType::Length,
                    value: parse_float_value(value_str)? * multiplier,
                });
            }
        }

        // No valid suffix was found so it should be a number of items
        Ok(Part {
            part_type: PartType::Items,
            value: parse_float_value(s)?,
        })
    }
}

pub fn parse(arg: &str) -> Result<f32, Error> {
    match arg.split_once('/') {
        Some((left, right)) => {
            let left = left.parse::<Part>()?;
            let right = right.parse::<Part>()?;

            match (left.part_type, right.part_type) {
                (PartType::Length, PartType::Items) => {
                    Ok(right.value / left.value * 10.0)
                },
                (PartType::Items, PartType::Length) => {
                    Ok(left.value / right.value * 10.0)
                },
                (PartType::Items, PartType::Items) => {
                    Err(Error::BothPartsItems)
                },
                (PartType::Length, PartType::Length) => {
                    Err(Error::BothPartsLength)
                },
            }
        },
        None => {
            // If there’s no slash in the string then assume it’s
            // directly a number of units in 10cm
            parse_float_value(arg)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn assert_approx_equal(a: f32, b: f32) {
        if (a - b).abs() >= 0.0001 {
            panic!("{} != {}", a, b);
        }
    }

    #[test]
    fn one_part() {
        assert_approx_equal(parse("12").unwrap(), 12.0);
        assert_approx_equal(parse("12.0").unwrap(), 12.0);
    }

    #[test]
    fn cm() {
        assert_approx_equal(parse("5/10cm").unwrap(), 5.0);
        assert_approx_equal(parse("5/20cm").unwrap(), 2.5);
        assert_approx_equal(parse("10cm/5").unwrap(), 5.0);
        assert_approx_equal(parse("1/0.1cm").unwrap(), 100.0);
    }

    #[test]
    fn mm() {
        assert_approx_equal(parse("1/1mm").unwrap(), 100.0);
        assert_approx_equal(parse("1/0.5mm").unwrap(), 200.0);
    }

    #[test]
    fn inches() {
        assert_approx_equal(parse("30/4\"").unwrap(), 29.52755905511811);
        assert_approx_equal(parse("30/4in").unwrap(), 29.52755905511811);
    }

    #[test]
    fn both_length() {
        assert_eq!(
            parse("12in/6cm").unwrap_err().to_string(),
            "both parts of the gauge are a length",
        );
    }

    #[test]
    fn both_items() {
        assert_eq!(
            parse("12/6").unwrap_err().to_string(),
            "both parts of the gauge are stitches or rows",
        );
    }

    #[test]
    fn too_small() {
        assert_eq!(
            parse("-1").unwrap_err().to_string(),
            "gauge -1 is too small",
        );
        assert_eq!(
            parse("-1/1.0cm").unwrap_err().to_string(),
            "gauge -1 is too small",
        );
    }

    #[test]
    fn abnormal() {
        assert_eq!(
            parse("inf").unwrap_err().to_string(),
            "invalid gauge: inf",
        );
    }

    #[test]
    fn bad_float() {
        assert!(matches!(parse("12/foocm"), Err(Error::FloatParseError(_))));
    }
}

use core::str::FromStr;

use crate::{ansi::AnsiColor, Color};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// An error type for parsing colors
pub enum ParseColorError {
    /// An invalid hex digit was detected
    InvalidHexDigit,
    /// Value overflowed a u8
    U8Overflow,
    /// An unknown color format
    UnknownColor,
}

#[inline(always)]
const fn parse_hex_digit(x: u8) -> Result<u8, ParseColorError> {
    match x {
        b'0'..=b'9' => Ok(x - b'0'),
        b'A'..=b'F' => Ok(x - b'A' + 10),
        b'a'..=b'f' => Ok(x - b'a' + 10),
        _ => Err(ParseColorError::InvalidHexDigit),
    }
}

const fn merge(a: u8, b: u8) -> u8 {
    a << 4 | b
}

impl FromStr for Color {
    type Err = ParseColorError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.as_bytes() {
            &[b'#', a, b, c, d, e, f] => {
                let a = parse_hex_digit(a)?;
                let b = parse_hex_digit(b)?;
                let c = parse_hex_digit(c)?;
                let d = parse_hex_digit(d)?;
                let e = parse_hex_digit(e)?;
                let f = parse_hex_digit(f)?;

                Self::Rgb(crate::rgb::RgbColor {
                    red: merge(a, b),
                    green: merge(c, d),
                    blue: merge(e, f),
                })
            }
            &[a @ b'0'..=b'9'] => Self::Xterm((a - b'0').into()),
            &[a @ b'0'..=b'9', b @ b'0'..=b'9'] => Self::Xterm(((a - b'0') * 10 + b).into()),
            &[a @ b'0'..=b'1', b @ b'0'..=b'9', c @ b'0'..=b'9']
            | &[a @ b'2', b @ b'0'..=b'4', c @ b'0'..=b'9']
            | &[a @ b'2', b @ b'5', c @ b'0'..=b'5'] => {
                Self::Xterm(((a - b'0') * 100 + (b - b'0') * 10 + (c - b'0')).into())
            }
            &[b'0'..=b'9', b'0'..=b'9', b'0'..=b'9'] => return Err(ParseColorError::U8Overflow),
            &[b'#', a] => Self::Xterm(parse_hex_digit(a)?.into()),
            &[b'#', a, b] => Self::Xterm(merge(parse_hex_digit(a)?, parse_hex_digit(b)?).into()),
            b"black" => Self::Ansi(AnsiColor::Black),
            b"red" => Self::Ansi(AnsiColor::Red),
            b"green" => Self::Ansi(AnsiColor::Green),
            b"yellow" => Self::Ansi(AnsiColor::Yellow),
            b"blue" => Self::Ansi(AnsiColor::Blue),
            b"magenta" | b"purple" => Self::Ansi(AnsiColor::Magenta),
            b"cyan" => Self::Ansi(AnsiColor::Cyan),
            b"white" => Self::Ansi(AnsiColor::White),
            b"bright black" => Self::Ansi(AnsiColor::BrightBlack),
            b"bright red" => Self::Ansi(AnsiColor::BrightRed),
            b"bright green" => Self::Ansi(AnsiColor::BrightGreen),
            b"bright yellow" => Self::Ansi(AnsiColor::BrightYellow),
            b"bright blue" => Self::Ansi(AnsiColor::BrightBlue),
            b"bright magenta" => Self::Ansi(AnsiColor::BrightMagenta),
            b"bright cyan" => Self::Ansi(AnsiColor::BrightCyan),
            b"bright white" => Self::Ansi(AnsiColor::BrightWhite),
            _ => return Err(ParseColorError::UnknownColor),
        })
    }
}

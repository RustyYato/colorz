//! 48-bit color values. Not as widely supported as standard ANSI or Xterm.

use crate::{ColorSpec, WriteColor};

#[cfg(doc)]
use crate::Color;

/// An Rgb value for color
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct RgbColor {
    /// The red component of the color
    pub red: u8,
    /// The green component of the color
    pub green: u8,
    /// The blue component of the color
    pub blue: u8,
}

struct RgbBuffer {
    data: [u8; 19],
    len: u8,
}

#[repr(u8)]
enum Layer {
    Foreground,
    Background,
    Underline,
}

impl RgbBuffer {
    fn new() -> Self {
        RgbBuffer {
            // we chose `;` so we don't need to write the seperator each time
            // which saves a little bit of time
            data: [b';'; 19],
            len: 0,
        }
    }

    fn write_escape_start(&mut self, layer: Layer) {
        self.write(match layer {
            Layer::Foreground => "\x1b[38;2;",
            Layer::Background => "\x1b[48;2;",
            Layer::Underline => "\x1b[58;2;",
        })
    }

    fn write_args_header(&mut self, layer: Layer) {
        self.write(match layer {
            Layer::Foreground => "38;2;",
            Layer::Background => "48;2;",
            Layer::Underline => "58;2;",
        })
    }

    fn write_sep(&mut self) {
        self.len += 1;
    }

    fn write_escape_end(&mut self) {
        self.write_char(b'm')
    }

    fn write(&mut self, s: &str) {
        self.data[self.len as usize..][..s.len()].copy_from_slice(s.as_bytes());
        self.len += s.len() as u8;
    }

    fn write_char(&mut self, x: u8) {
        self.data[self.len as usize] = x;
        self.len += 1;
    }

    fn write_u8(&mut self, mut x: u8) {
        let mut len = 0;
        let data = &mut self.data[self.len as usize..][..3];

        if x >= 100 {
            data[len] = x / 100 + b'0';
            x %= 100;
            len += 1;
        }

        if x >= 10 {
            data[len] = x / 10 + b'0';
            x %= 10;
            len += 1;
        }

        data[len] = x + b'0';
        self.len += len as u8 + 1;
    }

    fn to_str(&self) -> &str {
        core::str::from_utf8(&self.data[..self.len as usize]).unwrap()
    }

    #[inline(always)]
    fn write_args(&mut self, red: u8, green: u8, blue: u8) {
        self.write_u8(red);
        self.write_sep();
        self.write_u8(green);
        self.write_sep();
        self.write_u8(blue);
    }
}

impl crate::seal::Seal for RgbColor {}
impl WriteColor for RgbColor {
    fn color_kind(self) -> crate::mode::ColorKind {
        crate::mode::ColorKind::Rgb
    }

    fn fmt_foreground_args(self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut buffer = RgbBuffer::new();
        buffer.write_args_header(Layer::Foreground);
        buffer.write_args(self.red, self.green, self.blue);
        f.write_str(buffer.to_str())
    }

    fn fmt_background_args(self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut buffer = RgbBuffer::new();
        buffer.write_args_header(Layer::Background);
        buffer.write_args(self.red, self.green, self.blue);
        f.write_str(buffer.to_str())
    }

    fn fmt_underline_args(self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut buffer = RgbBuffer::new();
        buffer.write_args_header(Layer::Underline);
        buffer.write_args(self.red, self.green, self.blue);
        f.write_str(buffer.to_str())
    }

    fn fmt_foreground(self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut buffer = RgbBuffer::new();
        buffer.write_escape_start(Layer::Foreground);
        buffer.write_args(self.red, self.green, self.blue);
        buffer.write_escape_end();
        f.write_str(buffer.to_str())
    }

    fn fmt_background(self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut buffer = RgbBuffer::new();
        buffer.write_escape_start(Layer::Background);
        buffer.write_args(self.red, self.green, self.blue);
        buffer.write_escape_end();
        f.write_str(buffer.to_str())
    }

    fn fmt_underline(self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut buffer = RgbBuffer::new();
        buffer.write_escape_start(Layer::Underline);
        buffer.write_args(self.red, self.green, self.blue);
        buffer.write_escape_end();
        f.write_str(buffer.to_str())
    }
}

const fn raw_args(r: u8, g: u8, b: u8) -> [u8; 13] {
    let mut output = *b"2;rrr;ggg;bbb";

    const fn d(mut x: u8, mut n: u8) -> u8 {
        while n != 0 {
            x /= 10;
            n -= 1;
        }

        x % 10 + b'0'
    }

    output[2] = d(r, 2);
    output[3] = d(r, 1);
    output[4] = d(r, 0);

    output[6] = d(g, 2);
    output[7] = d(g, 1);
    output[8] = d(g, 0);

    output[10] = d(b, 2);
    output[11] = d(b, 1);
    output[12] = d(b, 0);

    output
}

const fn args(first: u8, r: u8, g: u8, b: u8) -> [u8; 16] {
    let mut output = *b"x8;2;rrr;ggg;bbb";
    output[0] = first;

    const fn d(mut x: u8, mut n: u8) -> u8 {
        while n != 0 {
            x /= 10;
            n -= 1;
        }

        x % 10 + b'0'
    }

    output[5] = d(r, 2);
    output[6] = d(r, 1);
    output[7] = d(r, 0);

    output[9] = d(g, 2);
    output[10] = d(g, 1);
    output[11] = d(g, 0);

    output[13] = d(b, 2);
    output[14] = d(b, 1);
    output[15] = d(b, 0);

    output
}

const fn escape(payload: [u8; 16]) -> [u8; 19] {
    let mut output = [0; 19];
    let mut i = 0;

    output[0] = 0x1b;
    output[1] = b'[';
    output[18] = b'm';

    while i < 16 {
        output[i + 2] = payload[i];
        i += 1;
    }

    output
}

const fn to_str(x: &[u8]) -> &str {
    match core::str::from_utf8(x) {
        Ok(x) => x,
        Err(_) => unreachable!(),
    }
}

/// A compile time Rgb color type
///
/// You can convert this type to [`Rgb`] via [`From`] or [`Self::DYNAMIC`]
/// and to [`Color`] or [`Option<Color>`] via [`From`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgb<const RED: u8, const GREEN: u8, const BLUE: u8>;

impl<const RED: u8, const GREEN: u8, const BLUE: u8> Rgb<RED, GREEN, BLUE> {
    /// The corrosponding value of [`RgbColor`]
    pub const DYNAMIC: RgbColor = RgbColor {
        red: RED,
        green: GREEN,
        blue: BLUE,
    };

    const FOREGROUND_ARGS_DATA: [u8; 16] = args(b'3', RED, GREEN, BLUE);
    const BACKGROUND_ARGS_DATA: [u8; 16] = args(b'4', RED, GREEN, BLUE);
    const UNDERLINE_ARGS_DATA: [u8; 16] = args(b'5', RED, GREEN, BLUE);

    /// The ANSI color args
    pub const ARGS: &'static str = to_str(&raw_args(RED, GREEN, BLUE));

    /// The ANSI foreground color arguments
    pub const FOREGROUND_ARGS: &'static str = to_str(&Self::FOREGROUND_ARGS_DATA);
    /// The ANSI background color arguments
    pub const BACKGROUND_ARGS: &'static str = to_str(&Self::BACKGROUND_ARGS_DATA);
    /// The ANSI underline color arguments
    pub const UNDERLINE_ARGS: &'static str = to_str(&Self::UNDERLINE_ARGS_DATA);

    /// The ANSI foreground color sequence
    pub const FOREGROUND_ESCAPE: &'static str = to_str(&escape(Self::FOREGROUND_ARGS_DATA));
    /// The ANSI background color sequence
    pub const BACKGROUND_ESCAPE: &'static str = to_str(&escape(Self::BACKGROUND_ARGS_DATA));
    /// The ANSI underline color sequence
    pub const UNDERLINE_ESCAPE: &'static str = to_str(&escape(Self::UNDERLINE_ARGS_DATA));
}

impl<const RED: u8, const GREEN: u8, const BLUE: u8> crate::seal::Seal for Rgb<RED, GREEN, BLUE> {}
impl<const RED: u8, const GREEN: u8, const BLUE: u8> ColorSpec for Rgb<RED, GREEN, BLUE> {
    type Dynamic = RgbColor;

    const KIND: crate::mode::ColorKind = crate::mode::ColorKind::Rgb;

    #[inline]
    fn into_dynamic(self) -> Self::Dynamic {
        Self::DYNAMIC
    }

    #[inline]
    fn foreground_args(self) -> &'static str {
        Self::FOREGROUND_ARGS
    }

    #[inline]
    fn background_args(self) -> &'static str {
        Self::BACKGROUND_ARGS
    }

    #[inline]
    fn underline_args(self) -> &'static str {
        Self::UNDERLINE_ARGS
    }

    #[inline]
    fn foreground_escape(self) -> &'static str {
        Self::FOREGROUND_ESCAPE
    }

    #[inline]
    fn background_escape(self) -> &'static str {
        Self::BACKGROUND_ESCAPE
    }

    #[inline]
    fn underline_escape(self) -> &'static str {
        Self::UNDERLINE_ESCAPE
    }
}

impl From<RgbColor> for crate::Color {
    #[inline(always)]
    fn from(color: RgbColor) -> Self {
        crate::Color::Rgb(color)
    }
}

impl From<RgbColor> for Option<crate::Color> {
    #[inline(always)]
    fn from(color: RgbColor) -> Self {
        Some(crate::Color::Rgb(color))
    }
}

impl<const RED: u8, const GREEN: u8, const BLUE: u8> From<Rgb<RED, GREEN, BLUE>> for RgbColor {
    #[inline(always)]
    fn from(_: Rgb<RED, GREEN, BLUE>) -> Self {
        RgbColor {
            red: RED,
            green: GREEN,
            blue: BLUE,
        }
    }
}

impl<const RED: u8, const GREEN: u8, const BLUE: u8> From<Rgb<RED, GREEN, BLUE>> for crate::Color {
    #[inline(always)]
    fn from(color: Rgb<RED, GREEN, BLUE>) -> Self {
        crate::Color::Rgb(color.into())
    }
}

impl<const RED: u8, const GREEN: u8, const BLUE: u8> From<Rgb<RED, GREEN, BLUE>>
    for Option<crate::Color>
{
    #[inline(always)]
    fn from(color: Rgb<RED, GREEN, BLUE>) -> Self {
        Some(color.into())
    }
}

impl<const RED: u8, const GREEN: u8, const BLUE: u8> crate::ComptimeColor
    for Rgb<RED, GREEN, BLUE>
{
    const VALUE: Option<crate::Color> = Some(crate::Color::Rgb(Self::DYNAMIC));
}

//! 48-bit color values. Not as widely supported as standard ANSI or Xterm.

use crate::{ColorSpec, WriteColor};

#[cfg(doc)]
use crate::Color;

/// An Rgb value for color
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RgbColor {
    /// The red component of the color
    pub red: u8,
    /// The green component of the color
    pub green: u8,
    /// The blue component of the color
    pub blue: u8,
}

// At stack only buffer which has two uses
// *  allows optimizing the number of calls to core::fmt::Formatter::write_str
//      which can save quite a bit of time since, Formatter is a huge optimization barrier
// * allows computing the color codes at compile time
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
    const fn new() -> Self {
        RgbBuffer {
            // using `;` so we don't need to write the separator each time
            // which saves a little bit of time
            data: [b';'; 19],
            len: 0,
        }
    }

    const fn write_escape_start(&mut self, layer: Layer) {
        self.write(match layer {
            Layer::Foreground => "\x1b[38;2;",
            Layer::Background => "\x1b[48;2;",
            Layer::Underline => "\x1b[58;2;",
        })
    }

    const fn write_args_header(&mut self, layer: Layer) {
        self.write(match layer {
            Layer::Foreground => "38;2;",
            Layer::Background => "48;2;",
            Layer::Underline => "58;2;",
        })
    }

    const fn write_sep(&mut self) {
        // separators are set when RgbBuffer is initialized, so we don't need to write anything
        // and RgbBuffers are not reused, so `self.data[self.len]` must equal `;`
        debug_assert!(self.data[self.len as usize] == b';');
        self.len += 1;
    }

    const fn write_escape_end(&mut self) {
        self.write_char(b'm')
    }

    const fn write(&mut self, s: &str) {
        let data = self.data.split_at_mut(self.len as usize).1;
        let data = data.split_at_mut(s.len()).0;
        data.copy_from_slice(s.as_bytes());
        self.len += s.len() as u8;
    }

    const fn write_char(&mut self, x: u8) {
        self.data[self.len as usize] = x;
        self.len += 1;
    }

    const fn write_u8(&mut self, x: u8) {
        // makes LLVM's peephole optimizer trigger, giving better codegen
        // this also lifts the bounds check to the top, and no other
        // bounds checks are done

        let mut len = 0;
        let data = self
            .data
            .split_at_mut(self.len as usize)
            .1
            .first_chunk_mut::<3>()
            .unwrap();

        if x >= 100 {
            data[len] = x / 100 + b'0';
            len += 1;
        }

        if x >= 10 {
            data[len] = x % 100 / 10 + b'0';
            len += 1;
        }

        data[len] = x % 10 + b'0';
        self.len += len as u8 + 1;
    }

    #[inline]
    const fn to_str(&self) -> &str {
        debug_assert!(self.data.is_ascii());
        let data = self.data.split_at(self.len as usize).0;
        match core::str::from_utf8(data) {
            Ok(x) => x,
            Err(_) => unreachable!(),
        }
    }

    // inline(always) gives a measurable perf boost
    #[inline(always)]
    const fn write_args(&mut self, red: u8, green: u8, blue: u8) {
        self.write_u8(red);
        self.write_sep();
        self.write_u8(green);
        self.write_sep();
        self.write_u8(blue);
    }

    const fn raw_args_payload(&self) -> RgbBuffer {
        let mut data = [0; 19];
        let mut i = 0;
        while i < self.len as usize - 1 - 5 {
            data[i] = self.data[i + 5];
            i += 1;
        }
        RgbBuffer {
            len: self.len - 1 - 5,
            data,
        }
    }

    const fn args_payload(&self) -> RgbBuffer {
        let mut data = [0; 19];
        let mut i = 0;
        while i < self.len as usize - 1 - 2 {
            data[i] = self.data[i + 2];
            i += 1;
        }
        RgbBuffer {
            len: self.len - 1 - 2,
            data,
        }
    }
}

impl crate::seal::Seal for RgbColor {}
impl WriteColor for RgbColor {
    #[inline]
    fn color_kind(self) -> crate::mode::ColorKind {
        crate::mode::ColorKind::Rgb
    }

    #[inline]
    fn fmt_foreground_args(self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut buffer = RgbBuffer::new();
        buffer.write_args_header(Layer::Foreground);
        buffer.write_args(self.red, self.green, self.blue);
        f.write_str(buffer.to_str())
    }

    #[inline]
    fn fmt_background_args(self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut buffer = RgbBuffer::new();
        buffer.write_args_header(Layer::Background);
        buffer.write_args(self.red, self.green, self.blue);
        f.write_str(buffer.to_str())
    }

    #[inline]
    fn fmt_underline_args(self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut buffer = RgbBuffer::new();
        buffer.write_args_header(Layer::Underline);
        buffer.write_args(self.red, self.green, self.blue);
        f.write_str(buffer.to_str())
    }

    #[inline]
    fn fmt_foreground(self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut buffer = RgbBuffer::new();
        buffer.write_escape_start(Layer::Foreground);
        buffer.write_args(self.red, self.green, self.blue);
        buffer.write_escape_end();
        f.write_str(buffer.to_str())
    }

    #[inline]
    fn fmt_background(self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut buffer = RgbBuffer::new();
        buffer.write_escape_start(Layer::Background);
        buffer.write_args(self.red, self.green, self.blue);
        buffer.write_escape_end();
        f.write_str(buffer.to_str())
    }

    #[inline]
    fn fmt_underline(self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut buffer = RgbBuffer::new();
        buffer.write_escape_start(Layer::Underline);
        buffer.write_args(self.red, self.green, self.blue);
        buffer.write_escape_end();
        f.write_str(buffer.to_str())
    }
}

const fn const_rgb_buffer(layer: Layer, r: u8, g: u8, b: u8) -> RgbBuffer {
    let mut buffer = RgbBuffer::new();
    buffer.write_escape_start(layer);
    buffer.write_args(r, g, b);
    buffer.write_escape_end();
    buffer
}

/// A compile time Rgb color type
///
/// You can convert this type to [`Rgb`] via [`From`] or [`Self::DYNAMIC`]
/// and to [`Color`] or [`Option<Color>`] via [`From`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgb<const RED: u8, const GREEN: u8, const BLUE: u8>;

impl<const RED: u8, const GREEN: u8, const BLUE: u8> Rgb<RED, GREEN, BLUE> {
    /// The corresponding value of [`RgbColor`]
    pub const DYNAMIC: RgbColor = RgbColor {
        red: RED,
        green: GREEN,
        blue: BLUE,
    };

    const FOREGROUND_DATA: RgbBuffer = const_rgb_buffer(Layer::Foreground, RED, GREEN, BLUE);
    const BACKGROUND_DATA: RgbBuffer = const_rgb_buffer(Layer::Background, RED, GREEN, BLUE);
    const UNDERLINE_DATA: RgbBuffer = const_rgb_buffer(Layer::Underline, RED, GREEN, BLUE);

    const FOREGROUND_ARGS_DATA: RgbBuffer = Self::FOREGROUND_DATA.args_payload();
    const BACKGROUND_ARGS_DATA: RgbBuffer = Self::BACKGROUND_DATA.args_payload();
    const UNDERLINE_ARGS_DATA: RgbBuffer = Self::UNDERLINE_DATA.args_payload();

    const DATA: RgbBuffer = Self::FOREGROUND_DATA.raw_args_payload();

    /// The ANSI color args
    pub const ARGS: &'static str = Self::DATA.to_str();

    /// The ANSI foreground color arguments
    pub const FOREGROUND_ARGS: &'static str = Self::FOREGROUND_ARGS_DATA.to_str();
    /// The ANSI background color arguments
    pub const BACKGROUND_ARGS: &'static str = Self::BACKGROUND_ARGS_DATA.to_str();
    /// The ANSI underline color arguments
    pub const UNDERLINE_ARGS: &'static str = Self::UNDERLINE_ARGS_DATA.to_str();

    /// The ANSI foreground color sequence
    pub const FOREGROUND_ESCAPE: &'static str = Self::FOREGROUND_DATA.to_str();
    /// The ANSI background color sequence
    pub const BACKGROUND_ESCAPE: &'static str = Self::BACKGROUND_DATA.to_str();
    /// The ANSI underline color sequence
    pub const UNDERLINE_ESCAPE: &'static str = Self::UNDERLINE_DATA.to_str();
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

#[test]
fn test_write_u8() {
    let mut buffer = RgbBuffer::new();
    buffer.write_args(205, 101, 200);
    assert_eq!(buffer.to_str(), "205;101;200");
}

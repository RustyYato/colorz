//! 48-bit color values. Not as widely supported as standard ANSI or Xterm.

use crate::{ColorSpec, WriteColor};

use crate::ascii;

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
    data: [ascii::Char; 19],
    len: u8,
}

// Use a slice here since it makes it simpler for LLVM to optimize out any panics
// also keep a copy of the length, since again it makes it simpler for LLVM to optimize
// By using this abstraction, we can ensure that LLVM knows that self.len + self.data.len() <= 19
// throughout every `fmt_` function.
struct RgbWriter<'a> {
    data: &'a mut [ascii::Char],
    len: u8,
}

#[repr(u8)]
enum Layer {
    Foreground,
    Background,
    Underline,
}

impl<'a> RgbWriter<'a> {
    const fn new(buffer: &'a mut [ascii::Char; 19]) -> Self {
        *buffer = [ascii::Char::Semicolon; 19];
        RgbWriter {
            data: buffer,
            len: 0,
        }
    }

    #[allow(clippy::mem_replace_with_default)]
    const fn chunk<const N: usize>(&mut self) -> &'a mut [ascii::Char; N] {
        let (first, data) = core::mem::replace(&mut self.data, &mut [])
            .split_first_chunk_mut::<N>()
            .unwrap();
        self.data = data;
        self.len += N as u8;
        first
    }

    const fn write_escape_start(&mut self, layer: Layer) {
        assert!(self.data.len() == 19);

        *self.chunk() = [ascii::Char::Escape, ascii::Char::LeftSquareBracket];
        self.write_args_header(layer);
    }

    const fn write_args_header(&mut self, layer: Layer) {
        *self.chunk() = [
            ascii::Char::from_digit(layer as u8 + 3),
            ascii::Char::Digit8,
            ascii::Char::Semicolon,
            ascii::Char::Digit2,
        ];
        self.write_sep();
    }

    const fn write_sep(&mut self) {
        let val = self.chunk::<1>()[0];
        // separators are set when RgbBuffer is initialized, so we don't need to write anything
        // and RgbBuffers are not reused, so `self.data[self.len]` must equal `;`
        debug_assert!(val as u8 == b';');
    }

    const fn write_escape_end(&mut self) {
        *self.chunk() = [ascii::Char::SmallM];
    }

    #[inline]
    const fn write_u8(&mut self, x: u8) {
        #[cold]
        #[inline(never)]
        const fn write_u8_failed() -> ! {
            unreachable!()
        }

        // makes LLVM's peephole optimizer trigger, giving better codegen
        // this also lifts the bounds check to the top, and no other
        // bounds checks are done

        if self.data.len() <= 3 {
            write_u8_failed()
        }

        if x >= 100 {
            *self.chunk() = [ascii::Char::from_digit(x / 100)];
        }

        if x >= 10 {
            *self.chunk() = [ascii::Char::from_digit(x / 10 % 10)];
        }

        *self.chunk() = [ascii::Char::from_digit(x % 10)];
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
}

impl RgbBuffer {
    const fn new() -> Self {
        RgbBuffer {
            // using `;` so we don't need to write the separator each time
            // which saves a little bit of time
            data: [ascii::Char::Semicolon; 19],
            len: 0,
        }
    }

    #[inline]
    const fn to_str(&self) -> &str {
        self.to_str_with(self.len)
    }

    #[inline]
    const fn to_str_with(&self, len: u8) -> &str {
        let data = self.data.split_at(len as usize).0;
        ascii::as_str(data)
    }

    const fn raw_args_payload(&self) -> RgbBuffer {
        let mut data = [ascii::Char::Semicolon; 19];
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
        let mut data = [ascii::Char::Semicolon; 19];
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
        let mut writer = RgbWriter::new(&mut buffer.data);
        writer.write_args_header(Layer::Foreground);
        writer.write_args(self.red, self.green, self.blue);
        let len = writer.len;
        f.write_str(buffer.to_str_with(len))
    }

    #[inline(never)]
    fn fmt_background_args(self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut buffer = RgbBuffer::new();
        let mut writer = RgbWriter::new(&mut buffer.data);
        writer.write_args_header(Layer::Background);
        writer.write_args(self.red, self.green, self.blue);
        let len = writer.len;
        f.write_str(buffer.to_str_with(len))
    }

    #[inline]
    fn fmt_underline_args(self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut buffer = RgbBuffer::new();
        let mut writer = RgbWriter::new(&mut buffer.data);
        writer.write_args_header(Layer::Underline);
        writer.write_args(self.red, self.green, self.blue);
        let len = writer.len;
        f.write_str(buffer.to_str_with(len))
    }

    #[inline]
    fn fmt_foreground(self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut buffer = RgbBuffer::new();
        let mut writer = RgbWriter::new(&mut buffer.data);
        writer.write_escape_start(Layer::Foreground);
        writer.write_args(self.red, self.green, self.blue);
        writer.write_escape_end();
        let len = writer.len;
        f.write_str(buffer.to_str_with(len))
    }

    #[inline]
    fn fmt_background(self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut buffer = RgbBuffer::new();
        let mut writer = RgbWriter::new(&mut buffer.data);
        writer.write_escape_start(Layer::Background);
        writer.write_args(self.red, self.green, self.blue);
        writer.write_escape_end();
        let len = writer.len;
        f.write_str(buffer.to_str_with(len))
    }

    #[inline]
    fn fmt_underline(self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut buffer = RgbBuffer::new();
        let mut writer = RgbWriter::new(&mut buffer.data);
        writer.write_escape_start(Layer::Underline);
        writer.write_args(self.red, self.green, self.blue);
        writer.write_escape_end();
        let len = writer.len;
        f.write_str(buffer.to_str_with(len))
    }
}

const fn const_rgb_buffer(layer: Layer, r: u8, g: u8, b: u8) -> RgbBuffer {
    let mut buffer = RgbBuffer::new();
    let mut writer = RgbWriter::new(&mut buffer.data);
    writer.write_escape_start(layer);
    writer.write_args(r, g, b);
    writer.write_escape_end();
    let len = writer.len;
    buffer.len = len;
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
    let mut writer = RgbWriter::new(&mut buffer.data);
    writer.write_args(205, 101, 200);
    let len = writer.len;
    assert_eq!(buffer.to_str_with(len), "205;101;200");
    let mut buffer = RgbBuffer::new();
    let mut writer = RgbWriter::new(&mut buffer.data);
    writer.write_escape_start(Layer::Foreground);
    writer.write_args(0xff, 0xff, 0xff);
    writer.write_escape_end();
    assert_eq!(writer.len, 19);
}

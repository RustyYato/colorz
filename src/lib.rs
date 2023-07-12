#![no_std]
#![forbid(unsafe_code)]

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

pub mod ansi;
pub mod css;
pub mod mode;
pub mod rgb;
mod style;
mod value;
pub mod xterm;

#[non_exhaustive]
pub struct StyledValue<T, F = NoColor, B = NoColor, U = NoColor> {
    pub value: T,
    pub style: Style<F, B, U>,
    pub stream: Stream,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stream {
    Stdout,
    Stderr,
    Stdin,
    AlwaysColor,
    NeverColor,
}

impl<T: ?Sized> Colorize for T {}
pub use value::Colorize;

pub use style::{Effect, EffectFlags, EffectFlagsIter, Style};

#[derive(Debug, Clone, Copy)]
pub struct NoColor;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    Ansi(ansi::AnsiColor),
    Css(css::CssColor),
    Xterm(xterm::XtermColor),
    Rgb(rgb::RgbColor),
}

pub trait AnsiColorCode {
    type Dynamic;

    #[doc(hidden)]
    const KIND: CodeKind = CodeKind::Unknown;

    fn into_dynamic(self) -> Self::Dynamic;

    fn code(&self) -> &'static str;

    fn foreground_code(&self) -> &'static str;

    fn background_code(&self) -> &'static str;

    fn underline_code(&self) -> &'static str;

    fn foreground_escape(&self) -> &'static str;

    fn background_escape(&self) -> &'static str;

    fn underline_escape(&self) -> &'static str;
}

impl<C: AnsiColorCode> WriteColor for C {
    #[doc(hidden)]
    #[inline(always)]
    fn code_kind(&self) -> CodeKind {
        C::KIND
    }

    fn fmt_code(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.code())
    }

    fn fmt_foreground_code(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.foreground_code())
    }

    fn fmt_background_code(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.background_code())
    }

    fn fmt_underline_code(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.underline_code())
    }

    fn fmt_foreground(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.foreground_escape())
    }

    fn fmt_background(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.background_escape())
    }

    fn fmt_underline(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.underline_escape())
    }
}

pub trait WriteColor {
    #[doc(hidden)]
    #[inline(always)]
    fn code_kind(&self) -> CodeKind {
        CodeKind::Unknown
    }

    fn fmt_code(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result;

    fn fmt_foreground_code(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result;

    fn fmt_background_code(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result;

    fn fmt_underline_code(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result;

    fn fmt_foreground(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("\x1b[")?;
        self.fmt_foreground_code(f)?;
        f.write_str("m")
    }

    fn fmt_background(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("\x1b[")?;
        self.fmt_background_code(f)?;
        f.write_str("m")
    }

    fn fmt_underline(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("\x1b[58;")?;
        self.fmt_code(f)?;
        f.write_str("m")
    }
}

impl WriteColor for Color {
    #[doc(hidden)]
    #[inline(always)]
    fn code_kind(&self) -> CodeKind {
        match self {
            Color::Ansi(_) => CodeKind::Ansi,
            Color::Css(_) => CodeKind::Rgb,
            Color::Xterm(_) => CodeKind::Xterm,
            Color::Rgb(_) => CodeKind::Rgb,
        }
    }

    fn fmt_code(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Color::Ansi(color) => color.fmt_code(f),
            Color::Css(color) => color.fmt_code(f),
            Color::Xterm(color) => color.fmt_code(f),
            Color::Rgb(color) => color.fmt_code(f),
        }
    }

    fn fmt_foreground_code(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Color::Ansi(color) => color.fmt_foreground_code(f),
            Color::Css(color) => color.fmt_foreground_code(f),
            Color::Xterm(color) => color.fmt_foreground_code(f),
            Color::Rgb(color) => color.fmt_background_code(f),
        }
    }

    fn fmt_background_code(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Color::Ansi(color) => color.fmt_background_code(f),
            Color::Css(color) => color.fmt_background_code(f),
            Color::Xterm(color) => color.fmt_background_code(f),
            Color::Rgb(color) => color.fmt_background_code(f),
        }
    }

    fn fmt_underline_code(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Color::Ansi(color) => color.fmt_underline_code(f),
            Color::Css(color) => color.fmt_underline_code(f),
            Color::Xterm(color) => color.fmt_underline_code(f),
            Color::Rgb(color) => color.fmt_underline_code(f),
        }
    }

    fn fmt_foreground(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Color::Ansi(color) => color.fmt_foreground(f),
            Color::Css(color) => color.fmt_foreground(f),
            Color::Xterm(color) => color.fmt_foreground(f),
            Color::Rgb(color) => color.fmt_foreground(f),
        }
    }

    fn fmt_background(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Color::Ansi(color) => color.fmt_background(f),
            Color::Css(color) => color.fmt_background(f),
            Color::Xterm(color) => color.fmt_background(f),
            Color::Rgb(color) => color.fmt_background(f),
        }
    }

    fn fmt_underline(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Color::Ansi(color) => color.fmt_underline(f),
            Color::Css(color) => color.fmt_underline(f),
            Color::Xterm(color) => color.fmt_underline(f),
            Color::Rgb(color) => color.fmt_underline(f),
        }
    }
}

impl WriteColor for core::convert::Infallible {
    fn fmt_code(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {}
    }

    fn fmt_foreground_code(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {}
    }

    fn fmt_background_code(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {}
    }

    fn fmt_underline_code(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {}
    }
}

#[doc(hidden)]
pub enum Kind {
    MaybeSome,
    AlwaysSome,
    NeverSome,
}

#[doc(hidden)]
pub enum CodeKind {
    Ansi,
    Xterm,
    Rgb,
    Unknown,
}

pub trait OptionalColor {
    type Color: WriteColor;

    #[doc(hidden)]
    const KIND: Kind = Kind::MaybeSome;

    fn get(&self) -> Option<Self::Color>;
}

impl<C: WriteColor + Clone> OptionalColor for C {
    type Color = Self;

    const KIND: Kind = Kind::AlwaysSome;

    #[inline]
    fn get(&self) -> Option<Self::Color> {
        Some(self.clone())
    }
}

impl<C: OptionalColor> OptionalColor for Option<C> {
    type Color = C::Color;

    #[inline]
    fn get(&self) -> Option<Self::Color> {
        self.as_ref().and_then(C::get)
    }
}

impl OptionalColor for NoColor {
    type Color = core::convert::Infallible;

    const KIND: Kind = Kind::NeverSome;

    #[inline]
    fn get(&self) -> Option<Self::Color> {
        None
    }
}

struct Ref<'a, T: ?Sized>(&'a T);

impl<T: ?Sized + OptionalColor> OptionalColor for Ref<'_, T> {
    type Color = T::Color;

    const KIND: Kind = T::KIND;

    #[inline]
    fn get(&self) -> Option<Self::Color> {
        self.0.get()
    }
}

pub trait ComptimeColor {
    const VALUE: Option<Color>;
}

impl ComptimeColor for NoColor {
    const VALUE: Option<Color> = None;
}

impl From<NoColor> for Option<Color> {
    #[inline(always)]
    fn from(_value: NoColor) -> Self {
        None
    }
}

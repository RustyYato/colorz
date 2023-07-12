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
pub struct StyledValue<T, F = NoColor, B = NoColor> {
    pub value: T,
    pub style: Style<F, B>,
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

pub use style::{Effect, EffectFlags, Style};

#[derive(Debug, Clone, Copy)]
pub struct NoColor;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    Ansi(ansi::AnsiColor),
    Css(css::CssColor),
    Xterm(xterm::XtermColor),
    Rgb(rgb::Rgb),
}

pub trait AnsiColorCode {
    type Dynamic;

    fn into_dynamic(self) -> Self::Dynamic;

    fn foreground_code(&self) -> &'static str;

    fn background_code(&self) -> &'static str;

    fn foreground_escape(&self) -> &'static str;

    fn background_escape(&self) -> &'static str;
}

impl<C: AnsiColorCode> WriteColor for C {
    fn fmt_foreground_code(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.foreground_code())
    }

    fn fmt_background_code(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.background_code())
    }

    fn fmt_foreground(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.foreground_escape())
    }

    fn fmt_background(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.background_escape())
    }
}

pub trait WriteColor {
    fn fmt_foreground_code(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result;

    fn fmt_background_code(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result;

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
}

impl WriteColor for Color {
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
            Color::Css(color) => color.fmt_background(f),
            Color::Xterm(color) => color.fmt_background_code(f),
            Color::Rgb(color) => color.fmt_background_code(f),
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
}

impl WriteColor for core::convert::Infallible {
    fn fmt_foreground_code(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {}
    }

    fn fmt_background_code(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {}
    }
}

pub trait OptionalColor {
    type Color: WriteColor;

    fn get(&self) -> Option<Self::Color>;
}

impl<C: WriteColor + Copy> OptionalColor for C {
    type Color = Self;

    #[inline]
    fn get(&self) -> Option<Self::Color> {
        Some(*self)
    }
}

impl<C: WriteColor + Copy> OptionalColor for Option<C> {
    type Color = C;

    #[inline]
    fn get(&self) -> Option<Self::Color> {
        *self
    }
}

impl OptionalColor for NoColor {
    type Color = core::convert::Infallible;

    #[inline]
    fn get(&self) -> Option<Self::Color> {
        None
    }
}

struct Ref<'a, T: ?Sized>(&'a T);

impl<T: ?Sized + OptionalColor> OptionalColor for Ref<'_, T> {
    type Color = T::Color;

    #[inline]
    fn get(&self) -> Option<Self::Color> {
        self.0.get()
    }
}

#![doc = include_str!("../README.md")]
#![no_std]
#![feature(rustdoc_missing_doc_code_examples)]
#![forbid(unsafe_code, missing_docs, clippy::missing_panics_doc)]
#![deny(
    missing_debug_implementations,
    missing_copy_implementations,
    clippy::missing_const_for_fn,
    clippy::missing_inline_in_public_items,
    // rustdoc::missing_doc_code_examples
)]
#![warn(rustdoc::missing_doc_code_examples)]
#![cfg_attr(doc, feature(doc_cfg))]

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

#[macro_use]
pub mod xterm;

pub mod ansi;
pub mod css;
mod from_str;
pub mod mode;
pub mod rgb;
mod style;
mod value;

pub use from_str::ParseColorError;

/// A styled value, created from [`Colorize`] or [`StyledValue::new`]
///
/// This represents a value with a style applied to it, and an associated stream
/// that this value will be written to.
///
/// ```rust
/// use colorz::{Colorize, StyledValue, ansi};
///
/// let hello: StyledValue<_, ansi::Blue> = "Hello ".blue();
/// println!("{hello} world");
/// ```
#[non_exhaustive]
#[derive(Clone, Copy)]
pub struct StyledValue<T, F = NoColor, B = NoColor, U = NoColor> {
    /// The value to style
    pub value: T,
    /// The style to use
    pub style: Style<F, B, U>,
    /// The stream to use
    pub stream: Option<mode::Stream>,
}

impl<T: ?Sized> Colorize for T {}
pub use value::Colorize;

pub use style::{Effect, EffectFlags, EffectFlagsIter, Style};

/// A no color placeholder type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoColor;

/// A runtime color args
///
/// # FromStr
///
/// you can parse a color from a string, here are the supported formats
/// * `#rrggbb` - where each `r`, `g`, or `b` is a hex character. This will parse to `Color::Rgb`,
/// * [0-9]{1,3} will parse to a `Color::Xterm` color code. Only supports values in the range 0..=255
/// * `#xx` or `#x` - where each `x` is a hex character. This will parse to `Color::Xterm` color code,
/// * the name of any ANSI color code case sensitive,  i.e. `red` or `bright blue` will parse to `Color::Ansi`
///
/// There isn't a way to parse to a `CssColor` at this time.
///
/// ```
/// use colorz::{Color, xterm, ansi, rgb};
///
/// assert_eq!("#ff".parse::<Color>(), Ok(Color::Xterm(xterm::XtermColor::from_code(0xff))));
/// assert_eq!("red".parse::<Color>(), Ok(Color::Ansi(ansi::AnsiColor::Red)));
/// assert_eq!("bright blue".parse::<Color>(), Ok(Color::Ansi(ansi::AnsiColor::BrightBlue)));
/// assert_eq!("#abcdef".parse::<Color>(), Ok(Color::Rgb(rgb::RgbColor { red: 0xab, green: 0xcd, blue: 0xef })));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    /// The ANSI color type (see [`ansi`] for details)
    Ansi(ansi::AnsiColor),
    /// The Xterm color type (see [`xterm`] for details)
    Xterm(xterm::XtermColor),
    /// The CSS color type (see [`css`] for details)
    Css(css::CssColor),
    /// The Rgb color type (see [`rgb`] for details)
    Rgb(rgb::RgbColor),
}

mod seal {
    pub trait Seal: Copy {}
}

/// A sealed trait for describing ANSI color args. This is largely only used to
/// implement [`WriteColor`] and to provide lower level tools to access code codes and color arguments.
///
/// This is a common trait for all single-color types specified by this crate,
/// and [`AnsiColor`](ansi::AnsiColor), [`XtermColor`](xterm::XtermColor), [`CssColor`](css::CssColor).
///
/// Note: [`rgb::RgbColor`] and [`Color`] don't implement this trait. Instead it implements `WriteColor` directly.
pub trait ColorSpec: seal::Seal {
    /// The runtime version of the color
    ///
    /// For all single-color types specified by this crate, this is the corresponding `*Color` type.
    /// For [`AnsiColor`](ansi::AnsiColor), [`XtermColor`](xterm::XtermColor), [`CssColor`](css::CssColor), it is themselves
    type Dynamic: WriteColor;

    /// The color kind of this Color
    ///
    /// used to detect wether to color on a given terminal
    const KIND: mode::ColorKind;

    /// Convert to the runtime version of the color
    fn into_dynamic(self) -> Self::Dynamic;

    /// The foreground color arguments only, excluding the leading `\x1b[` prefix
    fn foreground_args(self) -> &'static str;

    /// The background color arguments only, excluding the leading `\x1b[` prefix
    fn background_args(self) -> &'static str;

    /// The underline color arguments only, excluding the leading `\x1b[` prefix
    fn underline_args(self) -> &'static str;

    /// The foreground color sequence, including the leading `\x1b[` prefix
    fn foreground_escape(self) -> &'static str;

    /// The background color sequence, including the leading `\x1b[` prefix
    fn background_escape(self) -> &'static str;

    /// The underline color sequence, including the leading `\x1b[58;` prefix
    fn underline_escape(self) -> &'static str;
}

impl<C: ColorSpec> WriteColor for C {
    #[inline]
    fn color_kind(self) -> mode::ColorKind {
        C::KIND
    }

    #[inline]
    fn fmt_foreground_args(self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.foreground_args())
    }

    #[inline]
    fn fmt_background_args(self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.background_args())
    }

    #[inline]
    fn fmt_underline_args(self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.underline_args())
    }

    #[inline]
    fn fmt_foreground(self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.foreground_escape())
    }

    #[inline]
    fn fmt_background(self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.background_escape())
    }

    #[inline]
    fn fmt_underline(self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.underline_escape())
    }
}

/// A sealed trait for describing how to write ANSI color args
///
/// if you are using this trait directly, then you should use
/// [`mode::should_color`] to allow users to disable coloring
pub trait WriteColor: seal::Seal {
    /// The color kind of this Color
    ///
    /// used to detect wether to color is available on a given terminal if the `supports-color` feature is enabled
    fn color_kind(self) -> mode::ColorKind;

    /// write the foreground color arguments
    fn fmt_foreground_args(self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result;

    /// write the background color arguments
    fn fmt_background_args(self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result;

    /// write the underline color arguments
    fn fmt_underline_args(self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result;

    /// write the foreground color sequence
    #[inline]
    fn fmt_foreground(self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("\x1b[")?;
        self.fmt_foreground_args(f)?;
        f.write_str("m")
    }

    /// write the background color sequence
    #[inline]
    fn fmt_background(self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("\x1b[")?;
        self.fmt_background_args(f)?;
        f.write_str("m")
    }

    /// write the underline color sequence
    #[inline]
    fn fmt_underline(self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("\x1b[58;")?;
        self.fmt_underline_args(f)?;
        f.write_str("m")
    }
}

impl seal::Seal for Color {}
impl WriteColor for Color {
    #[inline]
    fn color_kind(self) -> mode::ColorKind {
        match self {
            Color::Ansi(_) => mode::ColorKind::Ansi,
            Color::Xterm(_) => mode::ColorKind::Xterm,
            Color::Css(_) => mode::ColorKind::Rgb,
            Color::Rgb(_) => mode::ColorKind::Rgb,
        }
    }

    #[inline]
    fn fmt_foreground_args(self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Color::Ansi(color) => color.fmt_foreground_args(f),
            Color::Css(color) => color.fmt_foreground_args(f),
            Color::Xterm(color) => color.fmt_foreground_args(f),
            Color::Rgb(color) => color.fmt_background_args(f),
        }
    }

    #[inline]
    fn fmt_background_args(self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Color::Ansi(color) => color.fmt_background_args(f),
            Color::Css(color) => color.fmt_background_args(f),
            Color::Xterm(color) => color.fmt_background_args(f),
            Color::Rgb(color) => color.fmt_background_args(f),
        }
    }

    #[inline]
    fn fmt_underline_args(self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Color::Ansi(color) => color.fmt_underline_args(f),
            Color::Css(color) => color.fmt_underline_args(f),
            Color::Xterm(color) => color.fmt_underline_args(f),
            Color::Rgb(color) => color.fmt_underline_args(f),
        }
    }

    #[inline]
    fn fmt_foreground(self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Color::Ansi(color) => color.fmt_foreground(f),
            Color::Css(color) => color.fmt_foreground(f),
            Color::Xterm(color) => color.fmt_foreground(f),
            Color::Rgb(color) => color.fmt_foreground(f),
        }
    }

    #[inline]
    fn fmt_background(self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Color::Ansi(color) => color.fmt_background(f),
            Color::Css(color) => color.fmt_background(f),
            Color::Xterm(color) => color.fmt_background(f),
            Color::Rgb(color) => color.fmt_background(f),
        }
    }

    #[inline]
    fn fmt_underline(self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Color::Ansi(color) => color.fmt_underline(f),
            Color::Css(color) => color.fmt_underline(f),
            Color::Xterm(color) => color.fmt_underline(f),
            Color::Rgb(color) => color.fmt_underline(f),
        }
    }
}

impl seal::Seal for core::convert::Infallible {}
impl WriteColor for core::convert::Infallible {
    #[inline]
    fn color_kind(self) -> mode::ColorKind {
        match self {}
    }

    #[inline]
    fn fmt_foreground_args(self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {}
    }

    #[inline]
    fn fmt_background_args(self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {}
    }

    #[inline]
    fn fmt_underline_args(self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {}
    }
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
pub enum Kind {
    MaybeSome,
    AlwaysSome,
    NeverSome,
}

/// An optional color type. This is used to abstract over [`WriteColor`] and `Option<W>` where `W: WriteColor`
pub trait OptionalColor: seal::Seal {
    /// The color type
    type Color: WriteColor;

    #[doc(hidden)]
    const KIND: Kind = Kind::MaybeSome;

    /// Get the color value
    fn get(self) -> Option<Self::Color>;

    /// Get the [color kind](mode::ColorKind), this is used to check if
    /// formatting this color is supported on the current terminal when
    /// the `supports-color` feature is enabled
    #[inline]
    fn color_kind(self) -> mode::ColorKind {
        self.get()
            .map(WriteColor::color_kind)
            .unwrap_or(mode::ColorKind::NoColor)
    }
}

impl<C: WriteColor> OptionalColor for C {
    type Color = Self;

    const KIND: Kind = Kind::AlwaysSome;

    #[inline]
    fn get(self) -> Option<Self::Color> {
        Some(self)
    }
}

impl<C: seal::Seal> seal::Seal for Option<C> {}
impl<C: OptionalColor> OptionalColor for Option<C> {
    type Color = C::Color;

    #[inline]
    fn get(self) -> Option<Self::Color> {
        self.and_then(C::get)
    }
}

impl seal::Seal for NoColor {}
impl OptionalColor for NoColor {
    type Color = core::convert::Infallible;

    const KIND: Kind = Kind::NeverSome;

    #[inline]
    fn get(self) -> Option<Self::Color> {
        None
    }
}

/// A compile time color value, only implemented by single-color types
pub trait ComptimeColor: seal::Seal {
    /// The corresponding [`Color`] value
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

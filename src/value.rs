use core::fmt::{self, Display};

use crate::{ansi, mode::Stream, Effect, OptionalColor, Style, StyledValue};

impl<T, F, B, U> StyledValue<T, F, B, U> {
    /// Create a new styled value
    pub const fn new(value: T, style: Style<F, B, U>, stream: Option<Stream>) -> Self {
        Self {
            value,
            style,
            stream,
        }
    }
}

macro_rules! AnsiColorMethods {
    (
        ($(#[$fg:meta] #[$bg:meta] $color:ident $fun:ident $into_fun:ident $on_fun:ident $into_on_fun:ident)*)
        ($(#[$doc:meta] $effect:ident $effect_fun:ident $into_effect_fun:ident)*)
    ) => {
        /// An extension trait for all values which adds convenience formatting functions
        pub trait Colorize {
            /// Convert a value to a `StyledValue`
            fn into_style(self) -> StyledValue<Self>
            where
                Self: Sized,
            {
                StyledValue {
                    value: self,
                    style: Style::new(),
                    stream: None,
                }
            }

            /// Convert a value to a `StyledValue`
            fn style(&self) -> StyledValue<&Self> {
                StyledValue {
                    value: self,
                    style: Style::new(),
                    stream: None
                }
            }

            /// Convert a value to a `StyledValue` and applies the given style
            fn into_style_with<F, B, U>(self, style: Style<F, B, U>) -> StyledValue<Self, F, B, U>
            where
                Self: Sized,
            {
                StyledValue {
                    value: self,
                    style,
                    stream: None,
                }
            }

            /// Convert a value to a `StyledValue` and applies the given style
            fn style_with<F, B, U>(&self, style: Style<F, B, U>) -> StyledValue<&Self, F, B, U> {
                StyledValue {
                    value: self,
                    style,
                    stream: None,
                }
            }

            /// Changes the foreground color
            fn fg<C>(&self, color: C) -> StyledValue<&Self, C> {
                self.style().fg(color)
            }

            /// Changes the foreground color
            fn into_fg<C>(self, color: C) -> StyledValue<Self, C> where Self: Sized {
                self.into_style().fg(color)
            }

            /// Changes the background color
            fn bg<C>(&self, color: C) -> StyledValue<&Self, crate::NoColor, C> {
                self.style().bg(color)
            }

            /// Changes the background color
            fn into_bg<C>(self, color: C) -> StyledValue<Self, crate::NoColor, C> where Self: Sized {
                self.into_style().bg(color)
            }

            /// Changes the underline color
            fn underline_color<C>(&self, color: C) -> StyledValue<&Self, crate::NoColor, crate::NoColor, C> {
                self.style().underline_color(color)
            }

            /// Changes the underline color
            fn into_underline_color<C>(self, color: C) -> StyledValue<Self, crate::NoColor, crate::NoColor, C> where Self: Sized {
                self.into_style().underline_color(color)
            }

            $(#[$fg] fn $fun(&self) -> StyledValue<&Self, ansi::$color> {
                self.style().$fun()
            })*

            $(#[$bg] fn $on_fun(&self) -> StyledValue<&Self, crate::NoColor, ansi::$color> {
                self.style().$on_fun()
            })*

            $(#[$fg] fn $into_fun(self) -> StyledValue<Self, ansi::$color> where Self: Sized{
                self.into_style().$fun()
            })*

            $(#[$bg] fn $into_on_fun(self) -> StyledValue<Self, crate::NoColor, ansi::$color> where Self: Sized {
                self.into_style().$on_fun()
            })*

            $(#[$doc] fn $effect_fun(&self) -> StyledValue<&Self> {
                self.style().$effect_fun()
            })*

            $(#[$doc] fn $into_effect_fun(self) -> StyledValue<Self> where Self: Sized {
                self.into_style().$effect_fun()
            })*
        }

        impl<T, F: OptionalColor, B: OptionalColor, U: OptionalColor> StyledValue<T, F, B, U> {
            /// Wrap this styled value in another one (this allows setting conditional formatting differently for each layer)
            #[inline]
            pub const fn style(&self) -> StyledValue<&Self> {
                StyledValue {
                    value: self,
                    style: Style::new(),
                    stream: None,
                }
            }

            /// Wrap this styled value in another one (this allows setting conditional formatting differently for each layer)
            #[inline]
            pub const fn into_style(self) -> StyledValue<Self> {
                StyledValue {
                    value: self,
                    style: Style::new(),
                    stream: None,
                }
            }

            /// Change the foreground color
            #[inline]
            pub fn fg<C>(self, color: C) -> StyledValue<T, C, B, U> {
                StyledValue {
                    value: self.value,
                    style: self.style.fg(color),
                    stream: self.stream,
                }
            }

            /// Change the background color
            #[inline]
            pub fn bg<C>(self, color: C) -> StyledValue<T, F, C, U> {
                StyledValue {
                    value: self.value,
                    style: self.style.bg(color),
                    stream: self.stream,
                }
            }

            /// Change the underline color
            #[inline]
            pub fn underline_color<C>(self, color: C) -> StyledValue<T ,F, B, C> {
                StyledValue {
                    value: self.value,
                    style: self.style.underline_color(color),
                    stream: self.stream,
                }
            }

            $(#[inline] #[$fg] pub fn $fun(self) -> StyledValue<T, ansi::$color, B, U> {
                self.fg(ansi::$color)
            })*

            $(#[inline] #[$bg] pub fn $on_fun(self) -> StyledValue<T, F, ansi::$color, U> {
                self.bg(ansi::$color)
            })*

            $(#[inline] #[$doc] pub fn $effect_fun(self) -> StyledValue<T, F, B, U> {
                StyledValue {
                    value: self.value,
                    style: self.style.with(Effect::$effect),
                    stream: self.stream,
                }
            })*

            /// Sets the stream for the given value
            #[inline]
            pub const fn stream(mut self, stream: Stream) -> Self  {
                self.stream = Some(stream);
                self
            }

            /// Sets the stream for the given value
            #[inline]
            pub const fn stream_opt(mut self, stream: Option<Stream>) -> Self  {
                self.stream = stream;
                self
            }
        }

        fn _all_effects_accounted_for(e: Effect) {
            match e {
                $(Effect::$effect => (),)*
            }
        }
    };
}

AnsiColorMethods! {
    (
        /// Changes the foreground to black
        /// Changes the background to black
        Black   black into_black on_black into_on_black
        /// Changes the foreground to red
        /// Changes the background to red
        Red     red into_red on_red into_on_red
        /// Changes the foreground to green
        /// Changes the background to green
        Green   green into_green on_green into_on_green
        /// Changes the foreground to yellow
        /// Changes the background to yellow
        Yellow  yellow into_yellow on_yellow into_on_yellow
        /// Changes the foreground to blue
        /// Changes the background to blue
        Blue    blue into_blue on_blue into_on_blue
        /// Changes the foreground to magenta
        /// Changes the background to magenta
        Magenta magenta into_magenta on_magenta into_on_magenta
        /// Changes the foreground to cyan
        /// Changes the background to cyan
        Cyan    cyan into_cyan on_cyan into_on_cyan
        /// Changes the foreground to white
        /// Changes the background to white
        White   white into_white on_white into_on_white

        /// Changes the foreground to bright black
        /// Changes the background to bright black
        BrightBlack   bright_black into_bright_black on_bright_black into_on_bright_black
        /// Changes the foreground to bright red
        /// Changes the background to bright red
        BrightRed     bright_red into_bright_red on_bright_red into_on_bright_red
        /// Changes the foreground to bright green
        /// Changes the background to bright green
        BrightGreen   bright_green into_bright_green on_bright_green into_on_bright_green
        /// Changes the foreground to bright yellow
        /// Changes the background to bright yellow
        BrightYellow  bright_yellow into_bright_yellow on_bright_yellow into_on_bright_yellow
        /// Changes the foreground to bright blue
        /// Changes the background to bright blue
        BrightBlue    bright_blue into_bright_blue on_bright_blue into_on_bright_blue
        /// Changes the foreground to bright magenta
        /// Changes the background to bright magenta
        BrightMagenta bright_magenta into_bright_magenta on_bright_magenta into_on_bright_magenta
        /// Changes the foreground to bright cyan
        /// Changes the background to bright cyan
        BrightCyan    bright_cyan into_bright_cyan on_bright_cyan into_on_bright_cyan
        /// Changes the foreground to bright white
        /// Changes the background to bright white
        BrightWhite   bright_white into_bright_white on_bright_white into_on_bright_white
    )
    (
        /// Applies the bold effect
        Bold bold into_bold
        /// Applies the dimmed effect
        Dimmed dimmed into_dimmed
        /// Applies the italics effect
        Italic italics into_italics
        /// Applies the underline effect
        Underline underline into_underline
        /// Applies the double underline effect
        DoubleUnderline double_underline into_double_underline
        /// Applies the blink effect
        Blink blink into_blink
        /// Applies the blink fast effect
        BlinkFast blink_fast into_blink_fast
        /// Applies the reverse effect
        Reversed reverse into_reverse
        /// Applies the hide effect
        Hidden hide into_hide
        /// Applies the strikethrough effect
        Strikethrough strikethrough into_strikethrough
        /// Applies the overline effect
        Overline overline into_overline
        /// Applies the superscript effect
        SuperScript superscript into_superscript
        /// Applies the subscript effect
        SubScript subscript into_subscript
    )
}

impl<T, F: OptionalColor, B: OptionalColor, U: OptionalColor> StyledValue<T, F, B, U> {
    /// Writes a styled value with the given value formatter
    pub fn fmt_with(
        &self,
        fmt: &mut fmt::Formatter<'_>,
        f: impl FnOnce(&T, &mut fmt::Formatter<'_>) -> fmt::Result,
    ) -> fmt::Result {
        let use_colors = self.style.should_color(self.stream);

        if use_colors {
            self.style.apply().fmt(fmt)?;
        }
        f(&self.value, fmt)?;
        if use_colors {
            self.style.clear().fmt(fmt)?;
        }
        Ok(())
    }
}

macro_rules! fmt_impl {
    ($name:ident) => {
        impl<T: fmt::$name, F: OptionalColor, B: OptionalColor, U: OptionalColor> fmt::$name
            for StyledValue<T, F, B, U>
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.fmt_with(f, fmt::$name::fmt)
            }
        }
    };
}

fmt_impl!(Display);
fmt_impl!(Debug);
fmt_impl!(Binary);
fmt_impl!(Octal);
fmt_impl!(Pointer);
fmt_impl!(LowerExp);
fmt_impl!(UpperExp);
fmt_impl!(LowerHex);
fmt_impl!(UpperHex);

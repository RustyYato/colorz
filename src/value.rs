use core::fmt::{self, Display};

use crate::{ansi, Effect, OptionalColor, Stream, Style, StyledValue};

impl<T, F, B, U> StyledValue<T, F, B, U> {
    pub const fn new(value: T, style: Style<F, B, U>, stream: Stream) -> Self {
        Self {
            value,
            style,
            stream,
        }
    }
}

macro_rules! AnsiColorMethods {
    (
        ($($color:ident $fun:ident $into_fun:ident $on_fun:ident $into_on_fun:ident)*)
        ($($effect:ident $effect_fun:ident $into_effect_fun:ident)*)
    ) => {
        pub trait Colorize {
            fn into_style(self) -> StyledValue<Self>
            where
                Self: Sized,
            {
                StyledValue {
                    value: self,
                    style: Style::new(),
                    stream: crate::Stream::AlwaysColor,
                }
            }

            fn style(&self) -> StyledValue<&Self> {
                StyledValue {
                    value: self,
                    style: Style::new(),
                    stream: crate::Stream::AlwaysColor,
                }
            }

            fn color<C>(&self, color: C) -> StyledValue<&Self, C> {
                self.style().color(color)
            }

            fn into_color<C>(self, color: C) -> StyledValue<Self, C> where Self: Sized {
                self.into_style().color(color)
            }

            fn on_color<C>(&self, color: C) -> StyledValue<&Self, crate::NoColor, C> {
                self.style().on_color(color)
            }

            fn into_on_color<C>(self, color: C) -> StyledValue<Self, crate::NoColor, C> where Self: Sized {
                self.into_style().on_color(color)
            }

            fn underline_color<C>(&self, color: C) -> StyledValue<&Self, crate::NoColor, crate::NoColor, C> {
                self.style().underline_color(color)
            }

            fn into_underline_color<C>(self, color: C) -> StyledValue<Self, crate::NoColor, crate::NoColor, C> where Self: Sized {
                self.into_style().underline_color(color)
            }

            $(fn $fun(&self) -> StyledValue<&Self, ansi::$color> {
                self.style().$fun()
            })*

            $(fn $on_fun(&self) -> StyledValue<&Self, crate::NoColor, ansi::$color> {
                self.style().$on_fun()
            })*

            $(fn $into_fun(self) -> StyledValue<Self, ansi::$color> where Self: Sized{
                self.into_style().$fun()
            })*

            $(fn $into_on_fun(self) -> StyledValue<Self, crate::NoColor, ansi::$color> where Self: Sized {
                self.into_style().$on_fun()
            })*

            $(fn $effect_fun(&self) -> StyledValue<&Self> {
                self.style().$effect_fun()
            })*

            $(fn $into_effect_fun(self) -> StyledValue<Self> where Self: Sized {
                self.into_style().$effect_fun()
            })*
        }

        impl<T, F, B, U> StyledValue<T, F, B, U> {
            #[inline]
            pub fn color<C>(self, color: C) -> StyledValue<T, C, B, U> {
                StyledValue {
                    value: self.value,
                    style: self.style.foreground(color),
                    stream: self.stream,
                }
            }

            #[inline]
            pub fn on_color<C>(self, color: C) -> StyledValue<T, F, C, U> {
                StyledValue {
                    value: self.value,
                    style: self.style.background(color),
                    stream: self.stream,
                }
            }

            #[inline]
            pub fn underline_color<C>(self, color: C) -> StyledValue<T ,F, B, C> {
                StyledValue {
                    value: self.value,
                    style: self.style.underline_color(color),
                    stream: self.stream,
                }
            }

            $(#[inline] pub fn $fun(self) -> StyledValue<T, ansi::$color, B, U> {
                StyledValue {
                    value: self.value,
                    style: self.style.foreground(ansi::$color),
                    stream: self.stream,
                }
            })*

            $(#[inline] pub fn $on_fun(self) -> StyledValue<T, F, ansi::$color, U> {
                StyledValue {
                    value: self.value,
                    style: self.style.background(ansi::$color),
                    stream: self.stream,
                }
            })*

            $(#[inline] pub fn $effect_fun(self) -> StyledValue<T, F, B, U> {
                StyledValue {
                    value: self.value,
                    style: self.style.with(Effect::$effect),
                    stream: self.stream,
                }
            })*

            #[inline]
            pub fn stream(self, stream: Stream) -> Self  {
                Self {
                    value: self.value,
                    style: self.style,
                    stream,
                }
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
        Black   black into_black on_black into_on_black
        Red     red into_red on_red into_on_red
        Green   green into_green on_green into_on_green
        Yellow  yellow into_yellow on_yellow into_on_yellow
        Blue    blue into_blue on_blue into_on_blue
        Magenta magenta into_magenta on_magenta into_on_magenta
        Cyan    cyan into_cyan on_cyan into_on_cyan
        White   white into_white on_white into_on_white

        BrightBlack   bright_black into_bright_black on_bright_black into_on_bright_black
        BrightRed     bright_red into_bright_red on_bright_red into_on_bright_red
        BrightGreen   bright_green into_bright_green on_bright_green into_on_bright_green
        BrightYellow  bright_yellow into_bright_yellow on_bright_yellow into_on_bright_yellow
        BrightBlue    bright_blue into_bright_blue on_bright_blue into_on_bright_blue
        BrightMagenta bright_magenta into_bright_magenta on_bright_magenta into_on_bright_magenta
        BrightCyan    bright_cyan into_bright_cyan on_bright_cyan into_on_bright_cyan
        BrightWhite   bright_white into_bright_white on_bright_white into_on_bright_white
    )
    (
        Bold bold into_bold
        Dimmed dimmed into_dimmed
        Italic italics into_italics
        Underline underline into_underline
        DoubleUnderline double_underline into_double_underline
        Blink blink into_blink
        BlinkFast blink_fast into_blink_fast
        Reversed reversed into_reverse
        Hidden hide into_hide
        Strikethrough strikethrough into_strikethrough
        Overline overline into_overline
        SuperScript superscript into_superscript
        SubScript subscript into_subscript
    )
}

impl<T, F: OptionalColor, B: OptionalColor, U: OptionalColor> StyledValue<T, F, B, U> {
    pub fn fmt_with(
        &self,
        fmt: &mut fmt::Formatter<'_>,
        f: impl FnOnce(&T, &mut fmt::Formatter<'_>) -> fmt::Result,
    ) -> fmt::Result {
        let use_colors = crate::mode::should_color(self.stream);

        if use_colors {
            self.style.as_ref().apply().fmt(fmt)?;
        }
        f(&self.value, fmt)?;
        if use_colors {
            self.style.as_ref().clear().fmt(fmt)?;
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

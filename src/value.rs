use core::fmt::{self, Display};

use crate::{ansi, Effect, OptionalColor, Style, StyledValue};

impl<T, F, B> StyledValue<T, F, B> {
    pub const fn new(value: T, style: Style<F, B>) -> Self {
        Self { value, style }
    }
}

macro_rules! AnsiColorMethods {
    (
        ($($color:ident $fun:ident $into_fun:ident)*)
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
                }
            }

            fn style(&self) -> StyledValue<&Self> {
                StyledValue {
                    value: self,
                    style: Style::new(),
                }
            }

            $(fn $fun(&self) -> StyledValue<&Self, ansi::$color> {
                self.style().$fun()
            })*

            $(fn $into_fun(self) -> StyledValue<Self, ansi::$color> where Self: Sized{
                self.into_style().$fun()
            })*
        }

        impl<T, F, B> StyledValue<T, F, B> {
            $(pub fn $fun(self) -> StyledValue<T, ansi::$color, B> {
                StyledValue {
                    value: self.value,
                    style: self.style.foreground(ansi::$color),
                }
            })*

            $(pub fn $effect_fun(self) -> StyledValue<T, F, B> {
                StyledValue {
                    value: self.value,
                    style: self.style.with(Effect::$effect),
                }
            })*
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
        Black   black into_black
        Red     red into_red
        Green   green into_green
        Yellow  yellow into_yellow
        Blue    blue into_blue
        Magenta magenta into_magenta
        Cyan    cyan into_cyan
        White   white into_white

        BrightBlack   bright_black into_bright_black
        BrightRed     bright_red into_bright_red
        BrightGreen   bright_green into_bright_green
        BrightYellow  bright_yellow into_bright_yellow
        BrightBlue    bright_blue into_bright_blue
        BrightMagenta bright_magenta into_bright_magenta
        BrightCyan    bright_cyan into_bright_cyan
        BrightWhite   bright_white into_bright_white
    )
    (
        Bold bold into_bold
        Dimmed dimmed into_dimmed
        Italic italics into_italics
        Underline underlined into_underlined
        Blink blink into_blink
        BlinkFast blink_fast into_blink_fast
        Reversed reversed into_reversed
        Hidden hide into_hide
        Strikethrough strikethrough into_strikethrough
        Overline overline into_overline
    )
}

impl<T, F: OptionalColor, B: OptionalColor> StyledValue<T, F, B> {
    pub fn fmt_with(
        &self,
        fmt: &mut fmt::Formatter<'_>,
        f: impl FnOnce(&T, &mut fmt::Formatter<'_>) -> fmt::Result,
    ) -> fmt::Result {
        self.style.as_ref().apply().fmt(fmt)?;
        f(&self.value, fmt)?;
        self.style.as_ref().clear().fmt(fmt)
    }
}

macro_rules! fmt_impl {
    ($name:ident) => {
        impl<T: fmt::$name, F: OptionalColor, B: OptionalColor> fmt::$name
            for StyledValue<T, F, B>
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

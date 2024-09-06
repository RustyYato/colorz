//! Basic ANSI color codes, which are widely supported on most terminals

#[cfg(doc)]
use crate::Color;
use crate::ColorSpec;

macro_rules! MkAnsiColor {
    (
        $($xterm:tt $name:ident $fg:literal $bg:literal)*
    ) => {
        /// A runtime ANSI color type
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum AnsiColor {
            $(
                #[doc = concat!("The runtime version of [`", stringify!($name), "`](struct@self::", stringify!($name), ")")]
                #[doc = concat!(" repesenting the color args ", stringify!($fg), " on the foreground and ", stringify!($bg), " on the background")]
                $name,
            )*
        }

        const _: [(); core::mem::size_of::<AnsiColor>()] = [(); 1];

        $(
            /// A compile time ANSI color type
            #[doc = concat!(" repesenting the color args ", stringify!($fg), " on the foreground and ", stringify!($bg), " on the background")]
            ///
            /// You can convert this type to [`AnsiColor`] via [`From`] or [`Self::DYNAMIC`]
            /// and to [`Color`] or [`Option<Color>`] via [`From`]
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            pub struct $name;
        )*

        impl From<AnsiColor> for crate::Color {
            #[inline(always)]
            fn from(color: AnsiColor) -> Self {
                crate::Color::Ansi(color)
            }
        }

        impl From<AnsiColor> for Option<crate::Color> {
            #[inline(always)]
            fn from(color: AnsiColor) -> Self {
                Some(crate::Color::Ansi(color))
            }
        }

        $(
            impl From<$name> for AnsiColor {
                #[inline(always)]
                fn from(_: $name) -> Self {
                    Self::$name
                }
            }

            impl From<$name> for crate::Color {
                #[inline(always)]
                fn from(_: $name) -> Self {
                    crate::Color::Ansi($name::DYNAMIC)
                }
            }

            impl From<$name> for Option<crate::Color> {
                #[inline(always)]
                fn from(_: $name) -> Self {
                    <$name as crate::ComptimeColor>::VALUE
                }
            }

            impl crate::ComptimeColor for $name {
                const VALUE: Option<crate::Color> = Some(crate::Color::Ansi(Self::DYNAMIC));
            }
        )*

        impl AnsiColor {
            #[inline]
            /// The ANSI foreground color args
            pub const fn foreground_args(self) -> &'static str {
                match self {
                    $(Self::$name => $name::FOREGROUND_ARGS,)*
                }
            }

            #[inline]
            /// The ANSI background color args
            pub const fn background_args(self) -> &'static str {
                match self {
                    $(Self::$name => $name::BACKGROUND_ARGS,)*
                }
            }

            #[inline]
            /// The ANSI background color args
            pub const fn underline_args(self) -> &'static str {
                self.to_xterm().underline_args()
            }

            #[inline]
            /// The ANSI foreground color escape sequence
            pub const fn foreground_escape(self) -> &'static str {
                match self {
                    $(Self::$name => $name::FOREGROUND_ESCAPE,)*
                }
            }

            #[inline]
            /// The ANSI background color escape sequence
            pub const fn background_escape(self) -> &'static str {
                match self {
                    $(Self::$name => $name::BACKGROUND_ESCAPE,)*
                }
            }

            #[inline]
            /// The ANSI underline color escape sequence
            pub const fn underline_escape(self) -> &'static str {
                self.to_xterm().underline_escape()
            }

            #[inline]
            /// The corropsonding Xterm color
            pub const fn to_xterm(self) -> crate::xterm::XtermColor {
                match self {
                    $(Self::$name => $name::DYNAMIC_XTERM,)*
                }
            }
        }

        impl From<AnsiColor> for crate::xterm::XtermColor {
            #[inline]
            fn from(color: AnsiColor) -> Self {
                color.to_xterm()
            }
        }

        impl crate::seal::Seal for AnsiColor {}
        impl ColorSpec for AnsiColor {
            type Dynamic = Self;

            const KIND: crate::mode::ColorKind = crate::mode::ColorKind::Ansi;

            #[inline]
            fn into_dynamic(self) -> Self::Dynamic {
                self
            }

            #[inline]
            fn underline_args(self) -> &'static str {
                self.to_xterm().underline_args()
            }

            #[inline]
            fn foreground_args(self) -> &'static str {
                self.foreground_args()
            }

            #[inline]
            fn background_args(self) -> &'static str {
                self.background_args()
            }

            #[inline]
            fn foreground_escape(self) -> &'static str {
                self.foreground_escape()
            }

            #[inline]
            fn background_escape(self) -> &'static str {
                self.background_escape()
            }

            #[inline]
            fn underline_escape(self) -> &'static str {
                self.to_xterm().underline_escape()
            }
        }

        $(
            impl $name {
                /// The corrosponding variant on [`AnsiColor`]
                pub const DYNAMIC: AnsiColor = AnsiColor::$name;
                /// The corrosponding [Xterm](crate::xterm) color
                pub const XTERM: xterm_from_code!($xterm) = xterm_from_code!($xterm);
                /// The corrosponding [`XtermColor`](crate::xterm::XtermColor) color
                pub const DYNAMIC_XTERM: crate::xterm::XtermColor = crate::xterm::XtermColor::from_code($xterm);

                /// The ANSI foreground color arguments
                pub const FOREGROUND_ARGS: &'static str = stringify!($fg);
                /// The ANSI background color arguments
                pub const BACKGROUND_ARGS: &'static str = stringify!($bg);

                /// The ANSI foreground color escape sequence
                pub const FOREGROUND_ESCAPE: &'static str = concat!("\x1b[", stringify!($fg) ,"m");
                /// The ANSI background color escape sequence
                pub const BACKGROUND_ESCAPE: &'static str = concat!("\x1b[", stringify!($bg) ,"m");
            }

            impl crate::seal::Seal for $name {}
            impl ColorSpec for $name {
                type Dynamic = AnsiColor;

                const KIND: crate::mode::ColorKind = crate::mode::ColorKind::Ansi;

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
                    <xterm_from_code!($xterm)>::UNDERLINE_ARGS
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
                    <xterm_from_code!($xterm)>::UNDERLINE_ESCAPE
                }
            }
        )*

    };
}

MkAnsiColor! {
    0 Black   30 40
    1 Red     31 41
    2 Green   32 42
    3 Yellow  33 43
    4 Blue    34 44
    5 Magenta 35 45
    6 Cyan    36 46
    7 White   37 47

    8 BrightBlack   90 100
    9 BrightRed     91 101
    10 BrightGreen   92 102
    11 BrightYellow  93 103
    12 BrightBlue    94 104
    13 BrightMagenta 95 105
    14 BrightCyan    96 106
    15 BrightWhite   97 107

    16 Default   39 49
}

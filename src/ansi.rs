//! Basic ANSI color codes, which are widely supported on most terminals

use crate::AnsiColorCode;
#[cfg(doc)]
use crate::Color;

macro_rules! MkAnsiColor {
    (
        $($name:ident $fg:literal $bg:literal)*
    ) => {
        /// A runtime ANSI color type
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum AnsiColor {
            $(
                #[doc = concat!("The runtime version of [`", stringify!($name), "`](self::", stringify!($name), ")")]
                #[doc = concat!(" repesenting the color code ", stringify!($fg), " on the foreground and ", stringify!($bg), " on the background")]
                $name,
            )*
        }

        const _: [(); core::mem::size_of::<AnsiColor>()] = [(); 1];

        $(
            /// A compile time ANSI color type
            #[doc = concat!(" repesenting the color code ", stringify!($fg), " on the foreground and ", stringify!($bg), " on the background")]
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
            /// The ANSI foreground color code
            pub const fn foreground_code(self) -> &'static str {
                match self {
                    $(Self::$name => $name::FOREGROUND_CODE,)*
                }
            }

            #[inline]
            /// The ANSI background color code
            pub const fn background_code(self) -> &'static str {
                match self {
                    $(Self::$name => $name::BACKGROUND_CODE,)*
                }
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
        }

        impl AnsiColorCode for AnsiColor {
            type Dynamic = Self;

            #[doc(hidden)]
            const KIND: crate::CodeKind = crate::CodeKind::Ansi;

            #[inline]
            fn into_dynamic(self) -> Self::Dynamic {
                self
            }

            #[inline]
            fn code(&self) -> &'static str {
                ""
            }

            #[inline]
            fn underline_code(&self) -> &'static str {
                ""
            }

            #[inline]
            fn foreground_code(&self) -> &'static str {
                (*self).foreground_code()
            }

            #[inline]
            fn background_code(&self) -> &'static str {
                (*self).background_code()
            }

            #[inline]
            fn foreground_escape(&self) -> &'static str {
                (*self).foreground_escape()
            }

            #[inline]
            fn background_escape(&self) -> &'static str {
                (*self).background_escape()
            }

            #[inline]
            fn underline_escape(&self) -> &'static str {
                ""
            }
        }

        $(
            impl $name {
                /// The corrosponding variant on [`AnsiColor`]
                pub const DYNAMIC: AnsiColor = AnsiColor::$name;

                /// The ANSI foreground color arguments
                pub const FOREGROUND_CODE: &'static str = stringify!($fg);
                /// The ANSI background color arguments
                pub const BACKGROUND_CODE: &'static str = stringify!($bg);

                /// The ANSI foreground color escape sequence
                pub const FOREGROUND_ESCAPE: &'static str = concat!("\x1b[", stringify!($fg) ,"m");
                /// The ANSI background color escape sequence
                pub const BACKGROUND_ESCAPE: &'static str = concat!("\x1b[", stringify!($bg) ,"m");
            }

            impl AnsiColorCode for $name {
                type Dynamic = AnsiColor;

                #[doc(hidden)]
                const KIND: crate::CodeKind = crate::CodeKind::Ansi;

                #[inline]
                fn into_dynamic(self) -> Self::Dynamic {
                    Self::DYNAMIC
                }

                #[inline]
                fn code(&self) -> &'static str {
                    (*self).foreground_code()
                }

                #[inline]
                fn foreground_code(&self) -> &'static str {
                    Self::FOREGROUND_CODE
                }

                #[inline]
                fn background_code(&self) -> &'static str {
                    Self::BACKGROUND_CODE
                }

                #[inline]
                fn underline_code(&self) -> &'static str {
                    ""
                }

                #[inline]
                fn foreground_escape(&self) -> &'static str {
                    Self::FOREGROUND_ESCAPE
                }

                #[inline]
                fn background_escape(&self) -> &'static str {
                    Self::BACKGROUND_ESCAPE
                }

                #[inline]
                fn underline_escape(&self) -> &'static str {
                    ""
                }
            }
        )*

    };
}

MkAnsiColor! {
    Black   30 40
    Red     31 41
    Green   32 42
    Yellow  33 43
    Blue    34 44
    Magenta 35 45
    Cyan    36 46
    White   37 47
    Default   39 49

    BrightBlack   90 100
    BrightRed     91 101
    BrightGreen   92 102
    BrightYellow  93 103
    BrightBlue    94 104
    BrightMagenta 95 105
    BrightCyan    96 106
    BrightWhite   97 107
}

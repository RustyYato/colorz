use crate::AnsiColorCode;

macro_rules! AnsiColor {
    (
        $($name:ident $fg:literal $bg:literal)*
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum AnsiColor {
            $($name,)*
        }

        $(
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            pub struct $name;
        )*

        impl AnsiColor {
            pub const fn foreground_code(&self) -> &'static str {
                match self {
                    $(Self::$name => $name::FOREGROUND_CODE,)*
                }
            }

            pub const fn background_code(&self) -> &'static str {
                match self {
                    $(Self::$name => $name::BACKGROUND_CODE,)*
                }
            }

            pub const fn foreground_escape(&self) -> &'static str {
                match self {
                    $(Self::$name => $name::FOREGROUND_ESCAPE,)*
                }
            }

            pub const fn background_escape(&self) -> &'static str {
                match self {
                    $(Self::$name => $name::BACKGROUND_ESCAPE,)*
                }
            }
        }

        impl AnsiColorCode for AnsiColor {
            type Dynamic = Self;

            #[inline]
            fn into_dynamic(self) -> Self::Dynamic {
                self
            }

            #[inline]
            fn foreground_code(&self) -> &'static str {
                self.foreground_code()
            }

            #[inline]
            fn background_code(&self) -> &'static str {
                self.background_code()
            }

            #[inline]
            fn foreground_escape(&self) -> &'static str {
                self.foreground_escape()
            }

            #[inline]
            fn background_escape(&self) -> &'static str {
                self.background_escape()
            }
        }

        $(
            impl $name {
                pub const DYNAMIC: AnsiColor = AnsiColor::$name;

                pub const FOREGROUND_CODE: &'static str = stringify!($fg);
                pub const BACKGROUND_CODE: &'static str = stringify!($bg);

                pub const FOREGROUND_ESCAPE: &'static str = concat!("\x1b[", stringify!($fg) ,"m");
                pub const BACKGROUND_ESCAPE: &'static str = concat!("\x1b[", stringify!($bg) ,"m");
            }

            impl AnsiColorCode for $name {
                type Dynamic = AnsiColor;

                #[inline]
                fn into_dynamic(self) -> Self::Dynamic {
                    Self::DYNAMIC
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
                fn foreground_escape(&self) -> &'static str {
                    Self::FOREGROUND_ESCAPE
                }

                #[inline]
                fn background_escape(&self) -> &'static str {
                    Self::BACKGROUND_ESCAPE
                }
            }
        )*

    };
}

AnsiColor! {
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

use crate::{AnsiColorCode, WriteColor};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgb {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl WriteColor for Rgb {
    fn fmt_code(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "2;{};{};{}", self.red, self.green, self.blue)
    }

    fn fmt_foreground_code(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "38;2;{};{};{}", self.red, self.green, self.blue)
    }

    fn fmt_background_code(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "48;2;{};{};{}", self.red, self.green, self.blue)
    }
}

const fn raw_code(r: u8, g: u8, b: u8) -> [u8; 13] {
    let mut output = *b"2;rrr;ggg;bbb";

    const fn d(mut x: u8, mut n: u8) -> u8 {
        while n != 0 {
            x /= 10;
            n -= 1;
        }

        x % 10 + b'0'
    }

    output[2] = d(r, 2);
    output[3] = d(r, 1);
    output[4] = d(r, 0);

    output[6] = d(g, 2);
    output[7] = d(g, 1);
    output[8] = d(g, 0);

    output[10] = d(b, 2);
    output[11] = d(b, 1);
    output[12] = d(b, 0);

    output
}

const fn code(first: u8, r: u8, g: u8, b: u8) -> [u8; 16] {
    let mut output = *b"x8;2;rrr;ggg;bbb";
    output[0] = first;

    const fn d(mut x: u8, mut n: u8) -> u8 {
        while n != 0 {
            x /= 10;
            n -= 1;
        }

        x % 10 + b'0'
    }

    output[5] = d(r, 2);
    output[6] = d(r, 1);
    output[7] = d(r, 0);

    output[9] = d(g, 2);
    output[10] = d(g, 1);
    output[11] = d(g, 0);

    output[13] = d(b, 2);
    output[14] = d(b, 1);
    output[15] = d(b, 0);

    output
}

const fn escape(payload: [u8; 16]) -> [u8; 19] {
    let mut output = [0; 19];
    let mut i = 0;

    output[0] = 0x1b;
    output[1] = b'[';
    output[18] = b'm';

    while i < 16 {
        output[i + 2] = payload[i];
        i += 1;
    }

    output
}

const fn to_str(x: &[u8]) -> &str {
    match core::str::from_utf8(x) {
        Ok(x) => x,
        Err(_) => unreachable!(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RGB<const RED: u8, const GREEN: u8, const BLUE: u8>;

impl<const RED: u8, const GREEN: u8, const BLUE: u8> RGB<RED, GREEN, BLUE> {
    pub const DYNAMIC: Rgb = Rgb {
        red: RED,
        green: GREEN,
        blue: BLUE,
    };

    const FOREGROUND_CODE_DATA: [u8; 16] = code(b'3', RED, GREEN, BLUE);
    const BACKGROUND_CODE_DATA: [u8; 16] = code(b'4', RED, GREEN, BLUE);

    pub const CODE: &'static str = to_str(&raw_code(RED, GREEN, BLUE));

    pub const FOREGROUND_CODE: &'static str = to_str(&Self::FOREGROUND_CODE_DATA);
    pub const BACKGROUND_CODE: &'static str = to_str(&Self::BACKGROUND_CODE_DATA);

    pub const FOREGROUND_ESCAPE: &'static str = to_str(&escape(Self::FOREGROUND_CODE_DATA));
    pub const BACKGROUND_ESCAPE: &'static str = to_str(&escape(Self::BACKGROUND_CODE_DATA));
}

impl<const RED: u8, const GREEN: u8, const BLUE: u8> AnsiColorCode for RGB<RED, GREEN, BLUE> {
    type Dynamic = Rgb;

    #[inline]
    fn into_dynamic(self) -> Self::Dynamic {
        Self::DYNAMIC
    }

    #[inline]
    fn code(&self) -> &'static str {
        Self::CODE
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

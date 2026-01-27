#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Char {
    Escape = b'\x1b',
    Digit0 = b'0',
    Digit1 = b'1',
    Digit2 = b'2',
    Digit3 = b'3',
    Digit4 = b'4',
    Digit5 = b'5',
    Digit6 = b'6',
    Digit7 = b'7',
    Digit8 = b'8',
    Digit9 = b'9',
    Semicolon = b';',
    LeftSquareBracket = b'[',
    SmallM = b'm',
}

impl Char {
    pub const fn from_digit(n: u8) -> Self {
        match n {
            0 => Char::Digit0,
            1 => Char::Digit1,
            2 => Char::Digit2,
            3 => Char::Digit3,
            4 => Char::Digit4,
            5 => Char::Digit5,
            6 => Char::Digit6,
            7 => Char::Digit7,
            8 => Char::Digit8,
            9 => Char::Digit9,
            _ => unreachable!(),
        }
    }
}

#[inline]
pub const fn as_bytes(slice: &[Char]) -> &[u8] {
    // SAFETY: Char is repr(u8) and fieldless, so it is trivially castable to bytes
    unsafe { core::slice::from_raw_parts(slice.as_ptr().cast(), slice.len()) }
}

#[inline]
pub const fn as_str(slice: &[Char]) -> &str {
    // SAFETY: Char's is represented by ascii bytes, and a slice of ascii bytes is always valid utf8
    unsafe { core::str::from_utf8_unchecked(as_bytes(slice)) }
}

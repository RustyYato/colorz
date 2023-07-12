//! Flags to control if any styling should occur
//!
//! There are three levels, in order of precedence
//! * feature flags - compile time (`strip-colors`)
//! * global - runtime [`set`], [`set_from_env`], [`replace`], [`replace_if_current_is`]
//! * per value - runtime [`StyledValue::stream`]
//!
//! higher precedence options forces coloring or no-coloring even if lower precedence options
//! specify otherwise.
//!
//! For example, using [`StyledValue::stream`] to [`Stream::AlwaysColor`] doesn't gurantee
//! that any coloring will happen. For example, if the `strip-colors` feature flag is set
//! or if `set(Mode::Never)` was called before.
//!
//! However, these flags only control coloring on [`StyledValue`], so using
//! the color types directly to color values will always be supported (even with `strip-colors`).

#[cfg(doc)]
use crate::{Stream, StyledValue};

use core::sync::atomic::{AtomicU8, Ordering};

static MODE: AtomicU8 = AtomicU8::new(Mode::Detect as u8);

const STREAM_NEVER: u8 = 0;
const STREAM_ALWAYS: u8 = 1;
const STREAM_UNDETECTED: u8 = 2;
static STREAMS: [AtomicU8; 3] = [
    AtomicU8::new(STREAM_UNDETECTED),
    AtomicU8::new(STREAM_UNDETECTED),
    AtomicU8::new(STREAM_UNDETECTED),
];

/// The coloring mode
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    /// use [`StyledValue::stream`] to pick when to color (by default always color if stream isn't specified)
    Detect,
    /// Never color [`StyledValue`]
    Never,
    /// Always color [`StyledValue`]
    Always,
}

const DETECT: u8 = Mode::Detect as u8;
const NEVER: u8 = Mode::Never as u8;
const ALWAYS: u8 = Mode::Always as u8;

fn decode(mode: u8) -> Mode {
    match mode {
        self::DETECT => Mode::Detect,
        self::NEVER => Mode::Never,
        self::ALWAYS => Mode::Always,
        _ => unreachable!(),
    }
}

#[cold]
#[cfg(feature = "std")]
fn detect_stream(stream: crate::Stream) -> bool {
    use std::io::{stderr, stdin, stdout, IsTerminal};

    let output = match stream {
        crate::Stream::AlwaysColor => return true,
        crate::Stream::NeverColor => return false,
        crate::Stream::Stdout => stdout().is_terminal(),
        crate::Stream::Stderr => stderr().is_terminal(),
        crate::Stream::Stdin => stdin().is_terminal(),
    };

    STREAMS[stream as usize].store(output as u8, Ordering::Relaxed);

    std::sync::atomic::fence(Ordering::SeqCst);
    output
}

#[cfg(not(feature = "std"))]
pub fn detect_stream(_stream: crate::Stream) -> bool {
    true
}

#[inline(always)]
#[cfg(feature = "strip-colors")]
pub fn should_color(stream: crate::Stream) -> bool {
    false
}

/// Should you color for a given stream
#[inline]
#[cfg(not(feature = "strip-colors"))]
pub fn should_color(stream: crate::Stream) -> bool {
    match get() {
        Mode::Detect => {
            match stream {
                crate::Stream::AlwaysColor => return true,
                crate::Stream::NeverColor => return false,
                _ => (),
            }

            should_color_slow(stream)
        }
        Mode::Never => false,
        Mode::Always => true,
    }
}

#[cold]
#[cfg(not(feature = "strip-colors"))]
fn should_color_slow(stream: crate::Stream) -> bool {
    let stream_info = &STREAMS[stream as usize];
    match stream_info.load(Ordering::Relaxed) {
        self::STREAM_UNDETECTED => detect_stream(stream),
        self::STREAM_NEVER => false,
        self::STREAM_ALWAYS => true,
        _ => unreachable!(),
    }
}

/// Get the global coloring mode (default [`Mode::Detect`])
pub fn get() -> Mode {
    if cfg!(feature = "strip-colors") {
        return Mode::Never;
    }

    decode(MODE.load(Ordering::Acquire))
}

/// Set the global coloring mode
pub fn set(mode: Mode) {
    if cfg!(feature = "strip-colors") {
        return;
    }

    MODE.store(mode as u8, Ordering::Release);
}

impl Mode {
    #[cfg(feature = "std")]
    /// Get the coloring mode from the environment variables `NO_COLOR` and `ALWAYS_COLOR`
    ///
    /// if `NO_COLOR` is present, then pick [`Mode::Never`]
    /// else if `ALWAYS_COLOR` is present, then pick [`Mode::Always`]
    /// else pick [`Mode::Detect`]
    pub fn from_env() -> Mode {
        if cfg!(feature = "strip-colors") {
            return Mode::Never;
        }

        if std::env::var_os("NO_COLOR").is_some() {
            Mode::Never
        } else if std::env::var_os("ALWAYS_COLOR").is_some() {
            Mode::Always
        } else {
            Mode::Detect
        }
    }
}

/// Set the coloring mode from the environment variables `NO_COLOR` and `ALWAYS_COLOR`
///
/// if `NO_COLOR` is present, then set [`Mode::Never`]
/// else if `ALWAYS_COLOR` is present, then set [`Mode::Always`]
///
/// If neither are set, the global coloring mode isn't changed
#[cfg(feature = "std")]
pub fn set_from_env() {
    if cfg!(feature = "strip-colors") {
        return;
    }

    if std::env::var_os("NO_COLOR").is_some() {
        set(Mode::Never);
    } else if std::env::var_os("ALWAYS_COLOR").is_some() {
        set(Mode::Always);
    }
}

/// Replace the global coloring mode
pub fn replace(mode: Mode) -> Mode {
    if cfg!(feature = "strip-colors") {
        return Mode::Never;
    }

    decode(MODE.swap(mode as u8, Ordering::Release))
}

/// Replace the global coloring mode if it is currently at `current`
pub fn replace_if_current_is(current: Mode, mode: Mode) -> Result<(), Mode> {
    if cfg!(feature = "strip-colors") {
        return Err(Mode::Never);
    }

    MODE.compare_exchange(
        current as u8,
        mode as u8,
        Ordering::Release,
        Ordering::Relaxed,
    )
    .map(drop)
    .map_err(decode)
}

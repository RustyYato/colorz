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
use crate::StyledValue;

use core::sync::atomic::AtomicU8;

static COLORING_MODE: AtomicU8 = AtomicU8::new(Mode::DETECT);
static DEFAULT_STREAM: AtomicU8 = AtomicU8::new(Stream::AlwaysColor.encode());
static STDOUT_SUPPORT: AtomicU8 = AtomicU8::new(ColorSupport::DETECT);
static STDERR_SUPPORT: AtomicU8 = AtomicU8::new(ColorSupport::DETECT);

/// The coloring mode
pub enum Mode {
    /// use [`StyledValue::stream`] to pick when to color (by default always color if stream isn't specified)
    Detect,
    /// Always color [`StyledValue`]
    Always,
    /// Never color [`StyledValue`]
    Never,
}

/// The stream to detect when to color on
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stream {
    /// Detect via [`std::io::stdout`] if feature `std` or `supports-color` is enabled
    Stdout,
    /// Detect via [`std::io::stderr`] if feature `std` or `supports-color` is enabled
    Stderr,
    /// Always color, used to pick the coloring mode at runtime for a particular value
    ///
    /// The default coloring mode for streams
    AlwaysColor,
    /// Never color, used to pick the coloring mode at runtime for a particular value
    NeverColor,
}

/// The coloring kinds
#[repr(u8)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorKind {
    /// A basic ANSI color
    Ansi,
    /// A 256-color
    Xterm,
    /// A 48-bit color
    Rgb,
    /// No color at all
    NoColor,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ColorSupport {
    ansi: bool,
    xterm: bool,
    rgb: bool,
}

impl ColorSupport {
    const DETECT: u8 = 0x80;

    #[cfg(feature = "supports-color")]
    fn encode(self) -> u8 {
        u8::from(self.ansi) | u8::from(self.xterm) << 1 | u8::from(self.rgb) << 2
    }

    #[cfg(feature = "supports-color")]
    fn decode(x: u8) -> Self {
        Self {
            ansi: x & 0b001 != 0,
            xterm: x & 0b010 != 0,
            rgb: x & 0b100 != 0,
        }
    }
}

impl Mode {
    const DETECT: u8 = Self::Detect.encode();

    const fn encode(self) -> u8 {
        match self {
            Mode::Always => 0,
            Mode::Never => 1,
            Mode::Detect => 2,
        }
    }

    const fn decode(x: u8) -> Self {
        match x {
            0 => Self::Always,
            1 => Self::Never,
            _ => Self::Detect,
        }
    }
}

impl Stream {
    const fn encode(self) -> u8 {
        match self {
            Stream::Stdout => 0,
            Stream::Stderr => 1,
            Stream::AlwaysColor => 2,
            Stream::NeverColor => 3,
        }
    }

    const fn decode(x: u8) -> Self {
        match x {
            0 => Self::Stdout,
            1 => Self::Stderr,
            2 => Self::AlwaysColor,
            3 => Self::NeverColor,
            _ => unreachable!(),
        }
    }
}

/// Set the global coloring mode (this allows forcing colors on or off despite stream preferences)
pub fn set_coloring_mode(mode: Mode) {
    if cfg!(feature = "strip-colors") {
        return;
    }

    COLORING_MODE.store(Mode::encode(mode), core::sync::atomic::Ordering::Release)
}

/// Get the global coloring mode
pub fn get_coloring_mode() -> Mode {
    if cfg!(feature = "strip-colors") {
        return Mode::Never;
    }

    Mode::decode(COLORING_MODE.load(core::sync::atomic::Ordering::Acquire))
}

/// Set the default stream if one isn't chosen per value
pub fn set_default_stream(stream: Stream) {
    DEFAULT_STREAM.store(
        Stream::encode(stream),
        core::sync::atomic::Ordering::Release,
    )
}

/// Get the default stream
pub fn get_default_stream() -> Stream {
    Stream::decode(DEFAULT_STREAM.load(core::sync::atomic::Ordering::Acquire))
}

pub(crate) fn should_color(stream: Option<Stream>, kinds: &[ColorKind]) -> bool {
    if cfg!(feature = "strip-colors") {
        return false;
    }

    match get_coloring_mode() {
        Mode::Always => return true,
        Mode::Never => return false,
        Mode::Detect => (),
    }

    let stream = stream.unwrap_or_else(get_default_stream);

    let is_stdout = match stream {
        Stream::Stdout => true,
        Stream::Stderr => false,
        Stream::AlwaysColor => return true,
        Stream::NeverColor => return false,
    };

    should_color_slow(is_stdout, kinds)
}

#[cold]
#[cfg(all(not(feature = "std"), not(feature = "supports-color")))]
fn should_color_slow(_is_stdout: bool, _kinds: &[ColorKind]) -> bool {
    true
}

#[cold]
#[cfg(all(feature = "std", not(feature = "supports-color")))]
fn should_color_slow(is_stdout: bool, _kinds: &[ColorKind]) -> bool {
    use core::sync::atomic::Ordering;
    use std::io::IsTerminal;

    let support_ref = match is_stdout {
        true => &STDOUT_SUPPORT,
        false => &STDERR_SUPPORT,
    };

    #[cold]
    #[inline(never)]
    fn detect(is_stdout: bool, support: &AtomicU8) -> bool {
        let s = if is_stdout {
            std::io::stdout().is_terminal()
        } else {
            std::io::stderr().is_terminal()
        };

        support.store(s as u8, Ordering::Relaxed);

        core::sync::atomic::fence(Ordering::SeqCst);

        s
    }

    match support_ref.load(Ordering::Acquire) {
        ColorSupport::DETECT => detect(is_stdout, support_ref),
        0 => false,
        _ => true,
    }
}

#[cold]
#[cfg(feature = "supports-color")]
fn should_color_slow(is_stdout: bool, kinds: &[ColorKind]) -> bool {
    use core::sync::atomic::Ordering;

    use supports_color::Stream;

    let (stream, support_ref) = match is_stdout {
        true => (Stream::Stdout, &STDOUT_SUPPORT),
        false => (Stream::Stderr, &STDERR_SUPPORT),
    };

    let support = support_ref.load(Ordering::Acquire);

    #[cold]
    #[inline(never)]
    fn detect(s: Stream, support: &AtomicU8) -> ColorSupport {
        let s = supports_color::on(s).map_or(
            ColorSupport {
                ansi: false,
                xterm: false,
                rgb: false,
            },
            |level| ColorSupport {
                ansi: level.has_basic,
                xterm: level.has_256,
                rgb: level.has_16m,
            },
        );

        support.store(s.encode(), Ordering::Relaxed);

        core::sync::atomic::fence(Ordering::SeqCst);

        s
    }

    let support = if support == ColorSupport::DETECT {
        detect(stream, support_ref)
    } else {
        ColorSupport::decode(support)
    };

    for &kind in kinds {
        let supported = match kind {
            ColorKind::Ansi => support.ansi,
            ColorKind::Xterm => support.xterm,
            ColorKind::Rgb => support.rgb,
            ColorKind::NoColor => continue,
        };

        if !supported {
            return false;
        }
    }

    true
}

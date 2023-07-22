//! Flags to control if any styling should occur
//!
//! There are three levels, in order of precedence
//! * feature flags - compile time (`strip-colors`)
//! * global - runtime [`set_coloring_mode`], [`set_coloring_mode_from_env`]
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

use core::{str::FromStr, sync::atomic::AtomicU8};

static COLORING_MODE: AtomicU8 = AtomicU8::new(Mode::DETECT);
static DEFAULT_STREAM: AtomicU8 = AtomicU8::new(Stream::AlwaysColor.encode());
#[cfg(any(feature = "std", feature = "supports-color"))]
static STDOUT_SUPPORT: AtomicU8 = AtomicU8::new(ColorSupport::DETECT);
#[cfg(any(feature = "std", feature = "supports-color"))]
static STDERR_SUPPORT: AtomicU8 = AtomicU8::new(ColorSupport::DETECT);

/// The coloring mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// use [`StyledValue::stream`] to pick when to color (by default always color if stream isn't specified)
    Detect,
    /// Always color [`StyledValue`]
    Always,
    /// Never color [`StyledValue`]
    Never,
}

/// An error if deserializing a mode from a string fails
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModeFromStrError;

#[cfg(feature = "std")]
impl std::error::Error for ModeFromStrError {}

impl core::fmt::Display for ModeFromStrError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(r#"Invalid mode: valid options include "detect", "always", "never""#)
    }
}

const ASCII_CASE_MASK: u8 = 0b0010_0000;
const ASCII_CASE_MASK_SIMD: u64 = u64::from_ne_bytes([ASCII_CASE_MASK; 8]);

impl FromStr for Mode {
    type Err = ModeFromStrError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_ascii_bytes(s.as_bytes())
    }
}

impl Mode {
    /// Parse the mode from some ascii encoded bytes
    #[inline]
    pub const fn from_ascii_bytes(s: &[u8]) -> Result<Self, ModeFromStrError> {
        const DETECT_STR: u64 = u64::from_ne_bytes(*b"detect\0\0") | ASCII_CASE_MASK_SIMD;
        const ALWAYS_STR: u64 = u64::from_ne_bytes(*b"always\0\0") | ASCII_CASE_MASK_SIMD;
        const NEVER_STR: u64 = u64::from_ne_bytes(*b"never\0\0\0") | ASCII_CASE_MASK_SIMD;

        let data = match *s {
            [a, b, c, d, e] => u64::from_ne_bytes([a, b, c, d, e, 0, 0, 0]),
            [a, b, c, d, e, f] => u64::from_ne_bytes([a, b, c, d, e, f, 0, 0]),
            _ => return Err(ModeFromStrError),
        };

        let data = data | ASCII_CASE_MASK_SIMD;

        match data {
            DETECT_STR => Ok(Mode::Detect),
            ALWAYS_STR => Ok(Mode::Always),
            NEVER_STR => Ok(Mode::Never),
            _ => Err(ModeFromStrError),
        }
    }
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

/// An error if deserializing a mode from a string fails
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamFromStrError;

#[cfg(feature = "std")]
impl std::error::Error for StreamFromStrError {}

impl core::fmt::Display for StreamFromStrError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(r#"Invalid mode: valid options include "stdout", "stderr", "always", "never""#)
    }
}

impl FromStr for Stream {
    type Err = StreamFromStrError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_ascii_bytes(s.as_bytes())
    }
}

impl Stream {
    /// Parse the mode from some ascii encoded bytes
    #[inline]
    pub const fn from_ascii_bytes(s: &[u8]) -> Result<Self, StreamFromStrError> {
        const STDOUT_STR: u64 = u64::from_ne_bytes(*b"stdout\0\0") | ASCII_CASE_MASK_SIMD;
        const STDERR_STR: u64 = u64::from_ne_bytes(*b"stderr\0\0") | ASCII_CASE_MASK_SIMD;
        const ALWAYS_STR: u64 = u64::from_ne_bytes(*b"always\0\0") | ASCII_CASE_MASK_SIMD;
        const NEVER_STR: u64 = u64::from_ne_bytes(*b"never\0\0\0") | ASCII_CASE_MASK_SIMD;

        let data = match *s {
            [a, b, c, d, e] => u64::from_ne_bytes([a, b, c, d, e, 0, 0, 0]),
            [a, b, c, d, e, f] => u64::from_ne_bytes([a, b, c, d, e, f, 0, 0]),
            _ => return Err(StreamFromStrError),
        };

        let data = data | ASCII_CASE_MASK_SIMD;

        match data {
            STDERR_STR => Ok(Stream::Stderr),
            STDOUT_STR => Ok(Stream::Stdout),
            ALWAYS_STR => Ok(Stream::AlwaysColor),
            NEVER_STR => Ok(Stream::NeverColor),
            _ => Err(StreamFromStrError),
        }
    }
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
    #[cfg(any(feature = "std", feature = "supports-color"))]
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

    /// Reads the current mode from the environment
    ///
    /// * If `NO_COLOR` is set to a non-zero value, [`Mode::Never`] is returned
    ///
    /// * If `ALWAYS_COLOR`, `CLICOLOR_FORCE`, `FORCE_COLOR` is set to a non-zero value, [`Mode::Always`] is returned
    ///
    /// * otherwise None is returned
    #[cfg(feature = "std")]
    #[cfg_attr(doc, doc(cfg(feature = "std")))]
    pub fn from_env() -> Option<Self> {
        if std::env::var_os("NO_COLOR").is_some_and(|x| x != "0") {
            return Some(Self::Never);
        }

        if std::env::var_os("ALWAYS_COLOR").is_some_and(|x| x != "0") {
            return Some(Self::Always);
        }

        if std::env::var_os("CLICOLOR_FORCE").is_some_and(|x| x != "0") {
            return Some(Self::Always);
        }

        if std::env::var_os("FORCE_COLOR").is_some_and(|x| x != "0") {
            return Some(Self::Always);
        }

        None
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

/// Reads the current mode from the environment
///
/// if no relevant environment variables are set, then the coloring mode is left unchanged
///
/// see [`Mode::from_env`] for details on which env vars are supported
#[cfg(feature = "std")]
#[cfg_attr(doc, doc(cfg(feature = "std")))]
pub fn set_coloring_mode_from_env() {
    if cfg!(feature = "strip-colors") {
        return;
    }

    if let Some(mode) = Mode::from_env() {
        set_coloring_mode(mode)
    }
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

#[inline]
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

#[inline]
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

#[cfg(test)]
mod test {
    use crate::mode::Mode;

    use super::Stream;

    extern crate std;

    #[allow(clippy::needless_range_loop)]
    fn test_case_insensitive_mode_from_str<const N: usize>(input: [u8; N], mode: Mode) {
        for i in 0..1 << N {
            let mut input = input;
            for j in 0..input.len() {
                if i & (1 << j) != 0 {
                    input[j] = input[j].to_ascii_uppercase();
                };
            }

            assert_eq!(Mode::from_ascii_bytes(&input), Ok(mode));
        }
    }

    #[allow(clippy::needless_range_loop)]
    fn test_case_insensitive_stream_from_str<const N: usize>(input: [u8; N], stream: Stream) {
        for i in 0..1 << N {
            let mut input = input;
            for j in 0..input.len() {
                if i & (1 << j) != 0 {
                    input[j] = input[j].to_ascii_uppercase();
                };
            }

            assert_eq!(Stream::from_ascii_bytes(&input), Ok(stream));
        }
    }

    #[test]
    fn mode_from_str_never() {
        test_case_insensitive_mode_from_str(*b"never", Mode::Never);
    }

    #[test]
    fn mode_from_str_always() {
        test_case_insensitive_mode_from_str(*b"always", Mode::Always);
    }

    #[test]
    fn mode_from_str_detect() {
        test_case_insensitive_mode_from_str(*b"detect", Mode::Detect);
    }

    #[test]
    fn stream_from_str_never() {
        test_case_insensitive_stream_from_str(*b"never", Stream::NeverColor);
    }

    #[test]
    fn stream_from_str_always() {
        test_case_insensitive_stream_from_str(*b"always", Stream::AlwaysColor);
    }

    #[test]
    fn stream_from_str_stdout() {
        test_case_insensitive_stream_from_str(*b"stdout", Stream::Stdout);
    }

    #[test]
    fn stream_from_str_stderr() {
        test_case_insensitive_stream_from_str(*b"stderr", Stream::Stderr);
    }
}

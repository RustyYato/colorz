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

#[repr(u8)]
pub enum Mode {
    Detect,
    Never,
    Always,
}

const DETECT: u8 = Mode::Detect as u8;
const NEVER: u8 = Mode::Never as u8;
const ALWAYS: u8 = Mode::Always as u8;

fn decode(mode: u8) -> Mode {
    match mode {
        self::DETECT => Mode::Detect,
        self::NEVER => Mode::Never,
        self::ALWAYS => Mode::Detect,
        _ => unreachable!(),
    }
}

#[cold]
#[cfg(feature = "std")]
pub fn detect_stream(stream: crate::Stream) -> bool {
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

#[cfg(not(feature = "strip-colors"))]
pub fn should_color(stream: crate::Stream) -> bool {
    match get() {
        Mode::Detect => {
            match stream {
                crate::Stream::AlwaysColor => return true,
                crate::Stream::NeverColor => return false,
                _ => (),
            }

            let stream_info = &STREAMS[stream as usize];
            match stream_info.load(Ordering::Relaxed) {
                self::STREAM_UNDETECTED => detect_stream(stream),
                self::STREAM_NEVER => false,
                self::STREAM_ALWAYS => true,
                _ => unreachable!(),
            }
        }
        Mode::Never => false,
        Mode::Always => true,
    }
}

pub fn get() -> Mode {
    if cfg!(feature = "strip-colors") {
        return Mode::Never;
    }

    decode(MODE.load(Ordering::Acquire))
}

pub fn set(mode: Mode) {
    if cfg!(feature = "strip-colors") {
        return;
    }

    MODE.store(mode as u8, Ordering::Release);
}

pub fn replace(mode: Mode) -> Mode {
    if cfg!(feature = "strip-colors") {
        return Mode::Never;
    }

    decode(MODE.swap(mode as u8, Ordering::Release))
}

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

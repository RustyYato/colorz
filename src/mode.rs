use core::sync::atomic::{AtomicU8, Ordering};

static MODE: AtomicU8 = AtomicU8::new(Mode::Detect as u8);

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

pub fn get() -> Mode {
    decode(MODE.load(Ordering::Acquire))
}

pub fn set(mode: Mode) {
    MODE.store(mode as u8, Ordering::Release);
}

pub fn replace(mode: Mode) -> Mode {
    decode(MODE.swap(mode as u8, Ordering::Release))
}

pub fn replace_if_current_is(current: Mode, mode: Mode) -> Result<(), Mode> {
    MODE.compare_exchange(
        current as u8,
        mode as u8,
        Ordering::Release,
        Ordering::Relaxed,
    )
    .map(drop)
    .map_err(decode)
}

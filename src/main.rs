use colorize::Colorize;
use std::io::Write;

fn main() {
    let mut buffer = [0; 1024];
    let start = std::time::Instant::now();
    for _ in 0..100_000_000 {
        let mut buffer = &mut buffer[..];

        let _ = write!(
            buffer,
            "{}",
            format_args!("Hello {} world", "my red".fg(colorize::ansi::Red)).on_blue()
        );
    }
    dbg!(start.elapsed());
}

use colorize::{mode, Colorize, Stream::Stderr};

fn main() {
    // NOTE: this example uses `stream(Stderr)` even though it's writing
    // to stdout to make it easier to see how `mode::Mode::Detect` works
    // try running this command wihtout redirecting stderr and while
    // redirecting stderr to `/dev/null` (or some file) to see how it
    // colors things differently
    //
    // ```
    // cargo run --example mode
    // cargo run --example mode 2> /dev/null
    // ```
    //
    // NOTE: that colorize currently uses `std::io::IsTerminal` to detect
    // if stdin/stderr/stdout have been redirected

    println!("color mode=always");
    mode::set(mode::Mode::Always);
    println!("{}", "blue".blue().stream(Stderr));

    println!("color mode=detect");
    mode::set(mode::Mode::Detect);
    println!("{}", "blue".blue().stream(Stderr));

    println!("color mode=never");
    mode::set(mode::Mode::Never);
    println!("{}", "blue".blue().stream(Stderr));
}

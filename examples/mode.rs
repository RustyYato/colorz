use colorize::{
    mode,
    mode::Stream::{AlwaysColor, NeverColor, Stderr, Stdout},
    Colorize,
};

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

    // cargo run --example mode --features strip-colors
    // cargo run --example mode --features strip-colors 2> /dev/null
    // ```
    //
    // NOTE: that colorize currently uses `std::io::IsTerminal` to detect
    // if stdin/stderr/stdout have been redirected

    assert_eq!(mode::get_default_stream(), AlwaysColor);

    mode::set_default_stream(Stderr);

    assert_eq!(mode::get_default_stream(), Stderr);

    println!("color mode=always");
    mode::set_coloring_mode(mode::Mode::Always);
    println!("{}", "blue stderr".blue());
    println!("{}", "blue stdout".blue().stream(Stdout));
    println!("{}", "blue always".blue().stream(AlwaysColor));
    println!("{}", "blue never".blue().stream(NeverColor));

    println!("color mode=detect");
    mode::set_coloring_mode(mode::Mode::Detect);
    println!("{}", "blue stderr".blue());
    println!("{}", "blue stdout".blue().stream(Stdout));
    println!("{}", "blue always".blue().stream(AlwaysColor));
    println!("{}", "blue never".blue().stream(NeverColor));

    println!("color mode=never");
    mode::set_coloring_mode(mode::Mode::Never);
    println!("{}", "blue stderr".blue());
    println!("{}", "blue stdout".blue().stream(Stdout));
    println!("{}", "blue always".blue().stream(AlwaysColor));
    println!("{}", "blue never".blue().stream(NeverColor));
}

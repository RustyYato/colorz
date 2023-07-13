use colorize::{ansi::AnsiColor, rgb::RgbColor, xterm::XtermColor, Color, Colorize};

fn random_number() -> u32 {
    2
}

fn main() {
    let mut color = AnsiColor::Red;
    println!("{}", "red".fg(color));

    color = AnsiColor::Blue;
    println!("{}", "blue".fg(color));

    let color = XtermColor::Fuchsia;
    println!("{}", "fuchsia".fg(color));

    let color = RgbColor {
        red: 141,
        green: 59,
        blue: 212,
    };
    println!("{}", "custom purple".fg(color));

    let color = match random_number() {
        1 => Color::Rgb(colorize::rgb::RgbColor {
            red: 141,
            green: 59,
            blue: 212,
        }),
        2 => Color::Ansi(AnsiColor::BrightGreen),
        3 => "#F3F3F3".parse().unwrap(),
        _ => Color::Xterm(XtermColor::Aqua),
    };

    println!("{}", "mystery color".fg(color));
}

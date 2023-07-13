use colorize::Style;

#[test]
fn test_effects() {
    let style = Style::new().bold().blink();

    assert_eq!(format!("{}", style.apply()), "\x1b[1;5m")
}

#[test]
fn test_foreground() {
    let style = Style::new()
        .fg(colorize::Color::Ansi(colorize::ansi::AnsiColor::Blue))
        .dimmed();

    assert_eq!(format!("{}", style.apply()), "\x1b[2m\x1b[34m")
}

#[test]
fn test_background() {
    let style = Style::new()
        .bg(colorize::Color::Ansi(colorize::ansi::AnsiColor::Blue))
        .dimmed();

    assert_eq!(format!("{}", style.apply()), "\x1b[2m\x1b[44m");
}

#[test]
fn test_partial() {
    let background = Style::new().bg(colorize::Color::Ansi(colorize::ansi::AnsiColor::Red));
    let style = Style::new().fg(colorize::Color::Ansi(colorize::ansi::AnsiColor::Blue));

    let x = format!(
        "{}hello {}my{} world{}",
        background.apply(),
        style.apply(),
        style.clear(),
        background.clear()
    );

    assert_eq!(x, "\x1b[41mhello \x1b[34mmy\x1b[39m world\x1b[49m")
}

#[test]
fn test_rgb() {
    let style = Style::new().bg(colorize::Color::Rgb(colorize::rgb::RgbColor {
        red: 255,
        green: 128,
        blue: 0,
    }));

    assert_eq!(format!("{}", style.apply()), "\x1b[48;2;255;128;0m");
}

#[test]
fn test_rgb_const() {
    let style = Style::new().bg(colorize::rgb::Rgb::<255, 0, 18>);

    assert_eq!(format!("{}", style.apply()), "\x1b[48;2;255;0;18m");

    assert_eq!(
        colorize::rgb::Rgb::<255, 0, 18>::FOREGROUND_ARGS,
        "38;2;255;0;18"
    );
}

#[test]
fn test_rgb_to_runtime() {
    let style = Style::new()
        .bg(colorize::rgb::Rgb::<255, 128, 0>)
        .into_runtime_style();

    assert_eq!(format!("{}", style.apply()), "\x1b[48;2;255;128;0m");
}

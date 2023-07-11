use colorize::Style;

#[test]
fn test_effects() {
    let style = Style::new().bold().blink();

    assert_eq!(format!("{}", style.apply()), "\x1b[1;5m")
}

#[test]
fn test_foreground() {
    let style = Style::new()
        .foreground(colorize::Color::Ansi(colorize::ansi::AnsiColors::Blue))
        .dimmed();

    assert_eq!(format!("{}", style.apply()), "\x1b[34;2m")
}

#[test]
fn test_background() {
    let style = Style::new()
        .background(colorize::Color::Ansi(colorize::ansi::AnsiColors::Blue))
        .dimmed();

    assert_eq!(format!("{}", style.apply()), "\x1b[44;2m");
}

#[test]
fn test_partial() {
    let background =
        Style::new().background(colorize::Color::Ansi(colorize::ansi::AnsiColors::Red));
    let style = Style::new().foreground(colorize::Color::Ansi(colorize::ansi::AnsiColors::Blue));

    let x = format!(
        "{}hello {}my{} world{}",
        background.apply(),
        style.apply(),
        style.clear(),
        background.clear()
    );

    if cfg!(feature = "nested-formats") {
        assert_eq!(x, "\x1b[41mhello \x1b[34mmy\x1b[39m world\x1b[49m")
    } else {
        assert_eq!(x, "\x1b[41mhello \x1b[34mmy\x1b[0m world\x1b[0m")
    }
}

#[test]
fn test_rgb() {
    let style = Style::new()
        .background(colorize::Color::Rgb(colorize::rgb::Rgb {
            red: 255,
            green: 128,
            blue: 0,
        }))
        .dimmed();

    println!("{}hello{}", style.apply(), style.clear());

    assert_eq!(format!("{}", style.apply()), "\x1b[44;2m");
}

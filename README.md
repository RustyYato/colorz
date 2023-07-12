# colorize

A zero-alloc, `no_std` compatible way of coloring your terminal outputs!

This is a "fork" of [owo-colors](https://github.com/jam1garner/owo-colors) with more features.

```rust
use colorize::{Colorize, xterm};

// will print `hello world` in red
println!("{}", "hello world".red());
// will print `hello world` on a red background
println!("{}", "hello world".on_red());
// will print `hello world` on a with an Aqua underline
println!("{}", "hello world".underline_color(xterm::Aqua).underline());
```

Features:
* Format using any `std` formatting trait, (`Display`, `Debug`, etc.)
* Format using a custom format function (`StyledValue::fmt_with`)
* Per-value conditional styling via `StyledValue::stream`
* Global conditional styling for all `StyledValue`s via
    * `colorize::mode`
    * `strip-colors` feature flag
* zero-dependency by default
* Standard names for Ansi, Xterm, and Css colors
* Rgb color support
* Ansi modifier (bold, italics, underline)
* Multi-color support (foreground, background, and underline color)
* mostly a drop-in replacement for `owo-colors` for simple cases
    * (some xterm color names may be different, some methods are called a little differently)
* compile-time selection of xterm colors by color code
* compile-time style construction
* compile-time style value construction
* `NO_COLOR`/`ALWAYS_COLOR` environment variables: `colorize::mode::{Mode::from_env, set_from_env}` (requires `std` feature)

Format any value
```rust
use colorize::{Colorize, xterm, css};

#[derive(Debug)]
struct MyType {
    value: String,
}

// will print `hello world` in red
println!("{}", "hello world".red());
// will print `100` on an aquamarine background
println!("{}", 100.on_color(css::Aquamarine));
// will print `hello world` on a with an Aqua underline
println!("{:?}", MyType { value: "hello world".into() }.underline_color(xterm::Aqua).underline());
```

With conditional formatting per value
```rust
use colorize::{Colorize, xterm, Stream::*};

// will print `hello world` in red if Stdin points to a terminal
println!("{}", "hello world".red().stream(Stdin));
```

Easily turn it off at any time
```rust
use colorize::{Colorize, xterm, Stream::*};

colorize::mode::set(colorize::mode::Mode::Never);

// doesn't style the value
println!("{}", "hello world".red());

assert_eq!(format!("{}", "hello world".red()), "hello world");
```

Create compile time style sheets
```rust
use colorize::{Colorize, Style, Effect, xterm};

const MY_STYLE: Style = Style::new()
    .const_fg(xterm::ForestGreen)
    .const_effects([Effect::Italic, Effect::Bold])
    .const_into_runtime_style();

// styles `my text` in forest green with italics and bold
println!("{}", "my text".style_with(MY_STYLE));
```

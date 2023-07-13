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
* `NO_COLOR`/`ALWAYS_COLOR` environment variables: `colorize::mode::{Mode::from_env, set_coloring_mode_from_env}`
    * requires `std` or `supports-color` feature

## Feature Flags

This crate has a few feature flags
* `strip-colors` - removes all coloring for `StyledValue`'s formatting methods
* `std` - this enables the standard library (since this library is `no_std` by default)
* `supports-color` - this enables the `supports-color` crate (which also uses the `std` library)

None of the feature is enabled by default. And they should only be turned on by the final binary crate.

If these features are turned off, then only the global mode settings is respected, and no stream-based
color detection is done.

if `strip-colors` is enabled, then `colorize::mode::get_coloring_mode` will always
return `Mode::Never`, and `StyledValue` will never be colored.

else if `supports-color` is enabled, then the `supports-color` crate is used to detect if
ANSI, Xterm or RGB colors are supports. If a `StyledValue` tries to use any unsupported
color types, then it will not do any coloring. 
For example, if you terminal doesn't support Xterm colors, and you write

```rust
use colorize::{Colorize, xterm};

println!("{}", "hello world".fg(xterm::Red));
```

Then you will see `hello world` in your default terminal color.

finally if `std` is enabled, then if the stream is a terminal then all coloring types will be used.
    and if the stream isn't a terminal then no coloring will be chosen.

## Coloring Mode

There are many ways to specify the coloring mode for `colorize`, and it may not be obvious how
they interact, so here is a precedence list. To figure out how colorize chooses to colorize, go
down the list, and the first element that applies will be selected.

* if the feature flag `strip-colors` is enabled -> NO COLORING
* if the global coloring mode is `Mode::Always` -> DO COLOR
* if the global coloring mode is `Mode::NEVER`  -> NO COLORING
* if the per-value stream if set to
    * `Stream::AlwaysColor` -> DO COLOR
    * `Stream::NeverColor` -> NO COLORING
    * `Stream::Stdout`/`Stream::Stderr` -> detect coloring using `std` or `support-color` (see docs on feature flags for details)
* if global stream is set to
    * `Stream::AlwaysColor` -> DO COLOR
    * `Stream::NeverColor` -> NO COLORING
    * `Stream::Stdout`/`Stream::Stderr` -> detect coloring using `std` or `support-color` (see docs on feature flags for details)

The global stream is always set to one of the possible `Stream` values,
so one option on the list will always be chosen.

NOTE that setting the coloring mode from the environment sets the global coloring mode, so either the second or third option on the list.

## Examples

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
println!("{}", 100.bg(css::Aquamarine));
// will print `hello world` on a with an Aqua underline
println!("{:?}", MyType { value: "hello world".into() }.underline_color(xterm::Aqua).underline());
```

With conditional formatting per value
```rust
use colorize::{Colorize, xterm, mode::Stream::*};

// will print `hello world` in red if Stdout points to a terminal
println!("{}", "hello world".red().stream(Stdout));
```

Easily turn it off at any time
```rust
use colorize::{Colorize, xterm, mode::Stream::*};

colorize::mode::set_coloring_mode(colorize::mode::Mode::Never);

// doesn't style the value
println!("{}", "hello world".red());

assert_eq!(format!("{}", "hello world".red()), "hello world");
```

Create compile time style sheets
```rust
use colorize::{Colorize, Style, Effect, xterm};

const MY_STYLE: Style = Style::new()
    .fg(xterm::ForestGreen)
    .effects_array([Effect::Italic, Effect::Bold])
    .const_into_runtime_style();

// styles `my text` in forest green with italics and bold
println!("{}", "my text".style_with(MY_STYLE));
```

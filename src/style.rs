use core::{fmt, num::NonZeroU16};

use crate::{ansi, mode::Stream, Color, ComptimeColor, OptionalColor, WriteColor};

/// A generic style format, this specifies the colors of the foreground, background, underline,
/// and what effects the text should have (bold, italics, etc.)
///
/// This type can be constructed via the various builder methods ([`foreground`](Self::foreground), [`bold`](Self::bold), etc.)
///
/// ```
/// use colorz::{Colorize, Style, ansi};
///
/// let style = Style::new().fg(ansi::Red).bg(ansi::Yellow).bold();
///
/// let x = "hello world".style_with(style);
/// ```
///
/// Then the style can be converted to a common [`Style`] type via [`Style::into_runtime_style`]
///
///
/// ```
/// # use colorz::{Colorize, Style, ansi};
/// # let style = Style::new().fg(ansi::Red).bg(ansi::Yellow).bold();
/// let style = style.into_runtime_style();
///
/// let x = "hello world".style_with(style);
/// ```
#[non_exhaustive]
#[must_use = "A `Style` value doesn't do anything on it's own"]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Style<F = Option<Color>, B = Option<Color>, U = Option<Color>> {
    /// The foreground color
    pub foreground: F,
    /// The background color
    pub background: B,
    /// The underline color
    pub underline_color: U,
    /// The various effects (like bold, italics, etc.)
    pub effects: EffectFlags,
}

const _: [(); core::mem::size_of::<Style>()] = [(); 14];

/// A collection of [`Effect`]s
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct EffectFlags {
    data: u16,
}

impl core::fmt::Debug for EffectFlags {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(*self).finish()
    }
}

macro_rules! Effect {
    ($($(#[$meta:meta])* $name:ident $apply:literal $clear:literal -> $set_func:ident,)*) => {
        /// An effect that can be applied to values
        #[repr(u8)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Effect {
            $($(#[$meta])* $name,)*
        }

        #[allow(non_upper_case_globals)]
        mod val {
            $(pub const $name: u8 = super::Effect::$name as u8;)*
        }

        #[allow(non_upper_case_globals)]
        mod apply {
            $(pub const $name: &str = stringify!($apply);)*
        }

        #[allow(non_upper_case_globals)]
        mod disable {
            $(pub const $name: &str = stringify!($clear);)*
        }

        #[allow(non_upper_case_globals)]
        mod apply_escape {
            $(pub const $name: &str = concat!("\x1b[", stringify!($apply), "m");)*
        }

        #[allow(non_upper_case_globals)]
        mod disable_escape {
            $(pub const $name: &str = concat!("\x1b[", stringify!($clear), "m");)*
        }

        const ALL_EFFECTS: EffectFlags = EffectFlags::new() $(.with(Effect::$name))*;

        impl Effect {
            fn decode(x: u8) -> Self {
                #[cold]
                #[inline(never)]
                fn bad_style() -> ! {
                    unreachable!("Bad style value decoded")
                }

                match x {
                    $(val::$name => Self::$name,)*
                    _ => bad_style(),
                }
            }

            /// The ANSI effect args
            #[inline]
            pub const fn apply_args(self) -> &'static str {
                match self {
                    $(Self::$name => apply::$name,)*
                }
            }

            /// The ANSI effect removal args
            #[inline]
            pub const fn clear_args(self) -> &'static str {
                match self {
                    $(Self::$name => disable::$name,)*
                }
            }

            /// The ANSI effect sequence
            #[inline]
            pub const fn apply_escape(self) -> &'static str {
                match self {
                    $(Self::$name => apply_escape::$name,)*
                }
            }

            /// The ANSI effect removal sequence
            #[inline]
            pub const fn clear_escape(self) -> &'static str {
                match self {
                    $(Self::$name => disable_escape::$name,)*
                }
            }

            const fn mask(self) -> u16 {
                1 << self as u8
            }
        }

        impl<F: OptionalColor, B: OptionalColor, U: OptionalColor> Style<F, B, U> {$(
            $(#[$meta])*
            #[inline(always)]
            pub const fn $set_func(self) -> Self {
                self.with(Effect::$name)
            }
        )*}
    };
}

impl EffectFlags {
    /// Create an empty set of effects
    #[inline(always)]
    pub const fn new() -> Self {
        Self { data: 0 }
    }

    /// Create a set of all effects
    #[inline(always)]
    pub const fn all() -> Self {
        ALL_EFFECTS
    }

    /// Create a set of effects from an array
    #[inline(always)]
    pub const fn from_array<const N: usize>(effects: [Effect; N]) -> Self {
        let mut e = EffectFlags::new();
        let mut i = 0;

        while i < N {
            e = e.with(effects[i]);
            i += 1;
        }

        e
    }

    /// Are there no effects
    #[inline(always)]
    pub const fn is_plain(self) -> bool {
        self.data == 0
    }

    #[inline(always)]
    const fn at_most_one_effect(self) -> bool {
        // self.data == 0 || self.data.is_power_of_two()
        self.data & self.data.wrapping_sub(1) == 0
    }

    /// Is this effect in the collection
    #[inline(always)]
    pub const fn is(self, opt: Effect) -> bool {
        self.data & opt.mask() != 0
    }

    /// Do these two collections intersect
    #[inline(always)]
    pub const fn is_any(self, opt: EffectFlags) -> bool {
        self.data & opt.data != 0
    }

    /// Add an effect to the set
    #[must_use = "EffectFlags::with returns a new instance without modifying the original"]
    #[inline(always)]
    pub const fn with(self, opt: Effect) -> Self {
        Self {
            data: self.data | opt.mask(),
        }
    }

    /// Remove an effect from the set
    #[must_use = "EffectFlags::without returns a new instance without modifying the original"]
    #[inline(always)]
    pub const fn without(self, opt: Effect) -> Self {
        Self {
            data: self.data & !opt.mask(),
        }
    }

    /// Toggle an effect in the set
    #[must_use = "EffectFlags::toggled returns a new instance without modifying the original"]
    #[inline(always)]
    pub const fn toggled(self, opt: Effect) -> Self {
        Self {
            data: self.data ^ opt.mask(),
        }
    }

    /// Add an effect to the set in place
    #[inline(always)]
    pub fn set(&mut self, opt: Effect) {
        *self = self.with(opt)
    }

    /// Remove an effect from the set in place
    #[inline(always)]
    pub fn unset(&mut self, opt: Effect) {
        *self = self.without(opt)
    }

    /// Toggle an effect in the set in place
    #[inline(always)]
    pub fn toggle(&mut self, opt: Effect) {
        *self = self.toggled(opt)
    }

    /// Iterate over all effects
    #[inline]
    pub const fn iter(self) -> EffectFlagsIter {
        EffectFlagsIter { data: self.data }
    }
}

impl Default for EffectFlags {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Style<crate::NoColor, crate::NoColor, crate::NoColor> {
    /// Create a new style
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            foreground: crate::NoColor,
            background: crate::NoColor,
            underline_color: crate::NoColor,
            effects: EffectFlags::new(),
        }
    }

    fn fmt_clear_all(f: &mut fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("\x1b[0m")
    }

    /// Clear a styling
    #[inline]
    pub fn clear_all() -> impl core::fmt::Display + core::fmt::Debug {
        struct ClearAll;

        impl core::fmt::Display for ClearAll {
            #[inline]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                Style::fmt_clear_all(f)
            }
        }

        impl core::fmt::Debug for ClearAll {
            #[inline]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                Style::fmt_clear_all(f)
            }
        }

        ClearAll
    }
}

impl Default for Style<crate::NoColor, crate::NoColor, crate::NoColor> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<F: OptionalColor, B: OptionalColor, U: OptionalColor> Style<F, B, U> {
    /// Set the foreground color
    #[inline(always)]
    pub const fn fg<T>(self, color: T) -> Style<T, B, U> {
        Style {
            foreground: color,
            background: self.background,
            underline_color: self.underline_color,
            effects: self.effects,
        }
    }

    /// Set the background color
    #[inline(always)]
    pub const fn bg<T>(self, color: T) -> Style<F, T, U> {
        Style {
            foreground: self.foreground,
            background: color,
            underline_color: self.underline_color,
            effects: self.effects,
        }
    }

    /// Set the underline color
    #[inline(always)]
    pub const fn underline_color<T>(self, color: T) -> Style<F, B, T> {
        Style {
            foreground: self.foreground,
            background: self.background,
            underline_color: color,
            effects: self.effects,
        }
    }

    /// Does this style apply any colors or effects
    #[inline(always)]
    pub fn is_plain(&self) -> bool {
        self.effects.is_plain()
            && self.foreground.get().is_none()
            && self.background.get().is_none()
    }

    /// Does this style use the effect
    #[inline(always)]
    pub const fn is(&self, opt: Effect) -> bool {
        self.effects.is(opt)
    }

    /// Set which effects are used
    #[inline(always)]
    pub fn effects<I: IntoIterator>(self, flags: I) -> Self
    where
        I::Item: Into<Effect>,
    {
        Self {
            effects: EffectFlags::from_iter(flags),
            ..self
        }
    }

    /// Set which effects are used
    #[inline(always)]
    pub const fn effects_array<const N: usize>(self, effects: [Effect; N]) -> Self {
        Style {
            effects: EffectFlags::from_array(effects),
            ..self
        }
    }

    /// Set which effects are used
    #[inline(always)]
    pub const fn effect_flags(self, effects: EffectFlags) -> Self {
        Style { effects, ..self }
    }

    /// Clear all effects
    #[inline(always)]
    pub const fn clear_effects(self) -> Self {
        self.effect_flags(EffectFlags::new())
    }

    /// Add the given effect
    #[inline(always)]
    pub const fn with(self, opt: Effect) -> Self {
        Style {
            effects: self.effects.with(opt),
            ..self
        }
    }

    /// Remove the given effect
    #[inline(always)]
    pub const fn without(self, opt: Effect) -> Self {
        Style {
            effects: self.effects.without(opt),
            ..self
        }
    }

    /// Toggle the effect
    #[inline(always)]
    pub const fn toggled(self, opt: Effect) -> Self {
        Style {
            effects: self.effects.toggled(opt),
            ..self
        }
    }
}

impl<F: Into<Option<Color>>, B: Into<Option<Color>>, U: Into<Option<Color>>> Style<F, B, U> {
    /// Convert to a type-erased style
    #[inline]
    pub fn into_runtime_style(self) -> Style {
        Style {
            foreground: self.foreground.into(),
            background: self.background.into(),
            underline_color: self.underline_color.into(),
            effects: self.effects,
        }
    }
}

impl<F: ComptimeColor, B: ComptimeColor, U: ComptimeColor> Style<F, B, U> {
    /// Convert to a type-erased style
    #[inline]
    pub const fn const_into_runtime_style(self) -> Style {
        Style {
            foreground: F::VALUE,
            background: B::VALUE,
            underline_color: U::VALUE,
            effects: self.effects,
        }
    }
}

Effect! {
    /// Makes the value bolded
    ///
    /// ```
    /// use colorz::Colorize;
    ///
    /// println!("{}", "hello world".bold());
    /// ```
    Bold 1 22 -> bold,

    /// Makes the value faint
    ///
    /// ```
    /// use colorz::Colorize;
    ///
    /// println!("{}", "hello world".dimmed());
    /// ```
    Dimmed 2 22 -> dimmed,

    /// Makes the value italics
    ///
    /// ```
    /// use colorz::Colorize;
    ///
    /// println!("{}", "hello world".italics());
    /// ```
    Italic 3 23 -> italics,

    /// Makes the value underlined
    ///
    /// ```
    /// use colorz::Colorize;
    ///
    /// println!("{}", "hello world".underline());
    /// ```
    Underline 4 24 -> underline,

    /// Makes the value double underlined
    ///
    /// ```
    /// use colorz::Colorize;
    ///
    /// println!("{}", "hello world".double_underline());
    /// ```
    DoubleUnderline 21 24 -> double_underline,

    /// Makes the value blink
    ///
    /// ```
    /// use colorz::Colorize;
    ///
    /// println!("{}", "hello world".blink());
    /// ```
    Blink 5 25 -> blink,

    /// Makes the value blink fast
    ///
    /// ```
    /// use colorz::Colorize;
    ///
    /// println!("{}", "hello world".blink_fast());
    /// ```
    BlinkFast 6 25 -> blink_fast,

    /// Makes the value reversed
    ///
    /// ```
    /// use colorz::Colorize;
    ///
    /// println!("{}", "hello world".reverse());
    /// ```
    Reversed 7 27 -> reverse,

    /// Makes the value hidden
    ///
    /// ```
    /// use colorz::Colorize;
    ///
    /// println!("{}", "hello world".hide());
    /// ```
    Hidden 8 28 -> hide,

    /// Applies a strikethrough to the value
    ///
    /// ```
    /// use colorz::Colorize;
    ///
    /// println!("{}", "hello world".strikethrough());
    /// ```
    Strikethrough 9 29 -> strikethrough,

    /// Applies an overline to the value
    ///
    /// ```
    /// use colorz::Colorize;
    ///
    /// println!("{}", "hello world".overline());
    /// ```
    Overline 53 55 -> overline,

    /// Makes the value a superscript
    ///
    /// ```
    /// use colorz::Colorize;
    ///
    /// println!("{}", "hello world".superscript());
    /// ```
    SuperScript 73 75 -> superscript,

    /// Makes the value a subscript
    ///
    /// ```
    /// use colorz::Colorize;
    ///
    /// println!("{}", "hello world".subscript());
    /// ```
    SubScript 73 75 -> subscript,
}

const ANY_UNDERLINE: EffectFlags = EffectFlags::new()
    .with(Effect::Underline)
    .with(Effect::DoubleUnderline);

impl<F: OptionalColor, B: OptionalColor, U: OptionalColor> Style<F, B, U> {
    /// Should you color based on the current coloring mode
    ///
    /// See `Coloring Mode` in the crate docs for details
    #[inline]
    pub fn should_color(&self, stream: impl Into<Option<Stream>>) -> bool {
        crate::mode::should_color(
            stream.into(),
            &[
                self.foreground.color_kind(),
                self.background.color_kind(),
                self.underline_color.color_kind(),
            ],
        )
    }

    fn fmt_apply(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.effects.is_any(ANY_UNDERLINE) {
            if let Some(color) = self.underline_color.get() {
                color.fmt_underline(f)?
            }
        }

        match (F::KIND, B::KIND) {
            (_, crate::Kind::MaybeSome) | (crate::Kind::MaybeSome, _) => (),
            (crate::Kind::NeverSome, crate::Kind::NeverSome) => {
                if self.effects.data.is_power_of_two() {
                    let effect = self.effects.iter().next().unwrap();
                    return f.write_str(effect.apply_escape());
                }
            }
            (crate::Kind::AlwaysSome, crate::Kind::AlwaysSome) => {
                // for now
            }
            (crate::Kind::AlwaysSome, crate::Kind::NeverSome) => {
                if self.effects.at_most_one_effect() {
                    if !self.effects.is_plain() {
                        let effect = self.effects.iter().next().unwrap();
                        f.write_str(effect.apply_escape())?;
                    }

                    return self.foreground.get().unwrap().fmt_foreground(f);
                }
            }
            (crate::Kind::NeverSome, crate::Kind::AlwaysSome) => {
                if self.effects.at_most_one_effect() {
                    if !self.effects.is_plain() {
                        let effect = self.effects.iter().next().unwrap();
                        f.write_str(effect.apply_escape())?;
                    }

                    if let Some(bg) = self.background.get() {
                        return bg.fmt_background(f);
                    }
                }
            }
        }

        self.fmt_apply_slow(f)
    }

    fn fmt_apply_slow(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.effects.at_most_one_effect() {
            if let Some(effect) = self.effects.iter().next() {
                f.write_str(effect.apply_escape())?;
            }

            if let Some(fg) = self.foreground.get() {
                fg.fmt_foreground(f)?;
            }

            if let Some(bg) = self.background.get() {
                bg.fmt_background(f)?;
            }

            return Ok(());
        }

        let mut semicolon = false;
        macro_rules! semi {
            () => {
                if semicolon {
                    f.write_str(";")?
                }
            };
        }

        if self.is_plain() {
            return Ok(());
        }

        f.write_str("\x1b[")?;

        if let Some(fg) = self.foreground.get() {
            semicolon = true;
            fg.fmt_foreground_args(f)?;
        }

        if let Some(bg) = self.background.get() {
            semi!();
            semicolon = true;
            bg.fmt_background_args(f)?;
        }

        if !self.effects.at_most_one_effect() {
            self.effects.iter().try_for_each(|effect| {
                semi!();
                semicolon = true;
                f.write_str(effect.apply_args())?;
                Ok(())
            })?;
        }

        f.write_str("m")?;

        Ok(())
    }

    fn fmt_clear(&self, f: &mut fmt::Formatter<'_>) -> core::fmt::Result {
        if self.effects.is_any(ANY_UNDERLINE) && self.underline_color.get().is_some() {
            f.write_str("\x1b[59m")?
        }

        match (F::KIND, B::KIND) {
            (_, crate::Kind::MaybeSome) | (crate::Kind::MaybeSome, _) => (),
            (crate::Kind::NeverSome, crate::Kind::NeverSome) => {
                if self.effects.data.is_power_of_two() {
                    let effect = self.effects.iter().next().unwrap();
                    return f.write_str(effect.clear_escape());
                }
            }
            (crate::Kind::AlwaysSome, crate::Kind::AlwaysSome) => {
                // for now
            }
            (crate::Kind::AlwaysSome, crate::Kind::NeverSome) => {
                if self.effects.at_most_one_effect() {
                    if !self.effects.is_plain() {
                        let effect = self.effects.iter().next().unwrap();
                        f.write_str(effect.clear_escape())?;
                    }

                    return ansi::Default.fmt_foreground(f);
                }
            }
            (crate::Kind::NeverSome, crate::Kind::AlwaysSome) => {
                if self.effects.at_most_one_effect() {
                    if !self.effects.is_plain() {
                        let effect = self.effects.iter().next().unwrap();
                        f.write_str(effect.clear_escape())?;
                    }

                    return ansi::Default.fmt_background(f);
                }
            }
        }

        self.fmt_clear_slow(f)
    }

    #[cold]
    fn fmt_clear_slow(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let at_most_one_effect = self.effects.at_most_one_effect();
        if at_most_one_effect {
            if let Some(effect) = self.effects.iter().next() {
                f.write_str(effect.clear_escape())?;
            }

            if self.foreground.get().is_some() {
                ansi::Default.fmt_foreground(f)?;
            }

            if self.background.get().is_some() {
                ansi::Default.fmt_background(f)?;
            }

            return Ok(());
        }

        let mut semicolon = false;
        macro_rules! semi {
            () => {
                if semicolon {
                    f.write_str(";")?
                }
            };
        }

        if self.is_plain() {
            return Ok(());
        }

        f.write_str("\x1b[")?;

        if self.foreground.get().is_some() {
            semicolon = true;
            ansi::Default.fmt_foreground_args(f)?;
        }

        if self.background.get().is_some() {
            semi!();
            semicolon = true;
            ansi::Default.fmt_background_args(f)?;
        }

        if !at_most_one_effect {
            self.effects.iter().try_for_each(|effect| {
                semi!();
                semicolon = true;
                f.write_str(effect.clear_args())?;
                Ok(())
            })?;
        }

        f.write_str("m")?;

        Ok(())
    }

    /// Writes the ANSI color and effect codes
    #[inline]
    pub fn apply(self) -> impl core::fmt::Display + core::fmt::Debug {
        struct Prefix<F, B, U> {
            style: Style<F, B, U>,
        }

        impl<F: OptionalColor, B: OptionalColor, U: OptionalColor> core::fmt::Display for Prefix<F, B, U> {
            #[inline]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.style.fmt_apply(f)
            }
        }

        impl<F: OptionalColor, B: OptionalColor, U: OptionalColor> core::fmt::Debug for Prefix<F, B, U> {
            #[inline]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.style.fmt_apply(f)
            }
        }

        Prefix { style: self }
    }

    /// Writes the ANSI color and effect clear codes (reverses whatever [`apply`](Self::apply) did)
    #[inline]
    pub fn clear(self) -> impl core::fmt::Display + core::fmt::Debug {
        struct Suffix<F, B, U> {
            style: Style<F, B, U>,
        }

        impl<F: OptionalColor, B: OptionalColor, U: OptionalColor> core::fmt::Display for Suffix<F, B, U> {
            #[inline]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.style.fmt_clear(f)
            }
        }

        impl<F: OptionalColor, B: OptionalColor, U: OptionalColor> core::fmt::Debug for Suffix<F, B, U> {
            #[inline]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.style.fmt_clear(f)
            }
        }

        Suffix { style: self }
    }
}

/// An iterator for the [`EffectFlags`] type, which yields [`Effect`]s
#[derive(Clone)]
pub struct EffectFlagsIter {
    data: u16,
}

impl core::fmt::Debug for EffectFlagsIter {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.clone()).finish()
    }
}

impl<'a> From<&'a Effect> for Effect {
    #[inline(always)]
    fn from(value: &'a Effect) -> Self {
        *value
    }
}

impl<'a> From<&'a mut Effect> for Effect {
    #[inline(always)]
    fn from(value: &'a mut Effect) -> Self {
        *value
    }
}

impl<E: Into<Effect>> FromIterator<E> for EffectFlags {
    #[inline]
    fn from_iter<T: IntoIterator<Item = E>>(iter: T) -> Self {
        iter.into_iter()
            .map(Into::into)
            .fold(Self::new(), Self::with)
    }
}

impl IntoIterator for EffectFlags {
    type Item = Effect;
    type IntoIter = EffectFlagsIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl Iterator for EffectFlagsIter {
    type Item = Effect;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let data = NonZeroU16::new(self.data)?;
        let zeros = data.trailing_zeros();
        self.data ^= 1 << zeros;
        Some(Effect::decode(zeros as u8))
    }
}

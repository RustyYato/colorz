use core::{fmt, num::NonZeroU16};

use crate::{ansi, Color, ComptimeColor, OptionalColor, WriteColor};

#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Style<F = Option<Color>, B = Option<Color>, U = Option<Color>> {
    pub foreground: F,
    pub background: B,
    pub underline_color: U,
    pub effects: EffectFlags,
}

const _: [(); core::mem::size_of::<Style>()] = [(); 14];

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct EffectFlags {
    data: u16,
}

macro_rules! Effect {
    ($($name:ident $apply:literal $clear:literal -> $set_func:ident,)*) => {
        #[repr(u8)]
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Effect {
            $($name,)*
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

            #[inline]
            fn apply_args(self) -> &'static str {
                match self {
                    $(Self::$name => apply::$name,)*
                }
            }

            #[inline]
            fn clear_args(self) -> &'static str {
                match self {
                    $(Self::$name => disable::$name,)*
                }
            }

            #[inline]
            fn apply_escape(self) -> &'static str {
                match self {
                    $(Self::$name => apply_escape::$name,)*
                }
            }

            #[inline]
            fn clear_escape(self) -> &'static str {
                match self {
                    $(Self::$name => disable_escape::$name,)*
                }
            }

            const fn mask(self) -> u16 {
                1 << self as u8
            }
        }

        impl<F, B, U> Style<F, B, U> {$(
            #[inline(always)]
            pub fn $set_func(self) -> Self {
                self.with(Effect::$name)
            }
        )*}
    };
}

impl EffectFlags {
    #[inline(always)]
    pub const fn new() -> Self {
        Self { data: 0 }
    }

    #[inline(always)]
    pub const fn all() -> Self {
        Self { data: 0x1ff }
    }

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

    #[inline(always)]
    pub const fn is_plain(self) -> bool {
        self.data == 0
    }

    #[inline(always)]
    pub const fn is(self, opt: Effect) -> bool {
        self.data & opt.mask() != 0
    }

    #[inline(always)]
    pub const fn is_any(self, opt: EffectFlags) -> bool {
        self.data & opt.data != 0
    }

    #[inline(always)]
    pub const fn with(self, opt: Effect) -> Self {
        Self {
            data: self.data | opt.mask(),
        }
    }

    #[inline(always)]
    pub const fn without(self, opt: Effect) -> Self {
        Self {
            data: self.data & !opt.mask(),
        }
    }

    #[inline(always)]
    pub const fn toggled(self, opt: Effect) -> Self {
        Self {
            data: self.data ^ opt.mask(),
        }
    }

    #[inline(always)]
    pub fn set(&mut self, opt: Effect) {
        *self = self.with(opt)
    }

    #[inline(always)]
    pub fn unset(&mut self, opt: Effect) {
        *self = self.without(opt)
    }

    #[inline(always)]
    pub fn toggle(&mut self, opt: Effect) {
        *self = self.toggled(opt)
    }

    #[inline]
    pub const fn iter(self) -> EffectFlagsIter {
        EffectFlagsIter { data: self.data }
    }

    #[inline(always)]
    pub fn try_for_each<F: FnMut(Effect) -> Result<(), E>, E>(self, f: F) -> Result<(), E> {
        self.iter().try_for_each(f)
    }
}

impl Style<crate::NoColor, crate::NoColor, crate::NoColor> {
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

impl<F, B, U> Style<F, B, U> {
    #[inline(always)]
    pub fn foreground<T>(self, color: T) -> Style<T, B, U> {
        Style {
            foreground: color,
            background: self.background,
            underline_color: self.underline_color,
            effects: self.effects,
        }
    }

    #[inline(always)]
    pub fn background<T>(self, color: T) -> Style<F, T, U> {
        Style {
            foreground: self.foreground,
            background: color,
            underline_color: self.underline_color,
            effects: self.effects,
        }
    }

    #[inline(always)]
    pub fn underline_color<T>(self, color: T) -> Style<F, B, T> {
        Style {
            foreground: self.foreground,
            background: self.background,
            underline_color: color,
            effects: self.effects,
        }
    }

    #[inline(always)]
    pub(crate) fn as_ref(&self) -> Style<crate::Ref<F>, crate::Ref<B>, crate::Ref<U>> {
        Style {
            foreground: crate::Ref(&self.foreground),
            background: crate::Ref(&self.background),
            underline_color: crate::Ref(&self.underline_color),
            effects: self.effects,
        }
    }
}

impl<F: Copy, B: Copy, U: Copy> Style<F, B, U> {
    #[inline(always)]
    pub const fn const_foreground<T>(self, color: T) -> Style<T, B, U> {
        Style {
            foreground: color,
            background: self.background,
            underline_color: self.underline_color,
            effects: self.effects,
        }
    }

    #[inline(always)]
    pub const fn const_background<T>(self, color: T) -> Style<F, T, U> {
        Style {
            foreground: self.foreground,
            background: color,
            underline_color: self.underline_color,
            effects: self.effects,
        }
    }

    #[inline(always)]
    pub fn const_underline_color<T>(self, color: T) -> Style<F, B, T> {
        Style {
            foreground: self.foreground,
            background: self.background,
            underline_color: color,
            effects: self.effects,
        }
    }
}

impl<F: Into<Option<Color>>, B: Into<Option<Color>>, U: Into<Option<Color>>> Style<F, B, U> {
    pub fn into_runtime_style(self) -> Style {
        Style {
            foreground: self.foreground.into(),
            background: self.background.into(),
            underline_color: self.underline_color.into(),
            effects: self.effects,
        }
    }
}

impl<F: ComptimeColor + Copy, B: ComptimeColor + Copy, U: ComptimeColor + Copy> Style<F, B, U> {
    pub const fn const_into_runtime_style(self) -> Style {
        Style {
            foreground: F::VALUE,
            background: B::VALUE,
            underline_color: U::VALUE,
            effects: self.effects,
        }
    }
}

impl<F: OptionalColor, B: OptionalColor, U: OptionalColor> Style<F, B, U> {
    #[inline(always)]
    pub fn is_plain(&self) -> bool {
        self.effects.is_plain()
            && self.foreground.get().is_none()
            && self.background.get().is_none()
    }

    #[inline(always)]
    pub fn is_complete(&self) -> bool {
        self.effects == EffectFlags::all()
            && self.foreground.get().is_some()
            && self.background.get().is_some()
    }

    #[inline(always)]
    pub const fn is(&self, opt: Effect) -> bool {
        self.effects.is(opt)
    }
}

impl<F, B, U> Style<F, B, U> {
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

    #[inline(always)]
    pub fn effect_flags(self, effects: EffectFlags) -> Self {
        Style { effects, ..self }
    }

    #[inline(always)]
    pub fn clear_effects(self) -> Self {
        self.effect_flags(EffectFlags::new())
    }

    #[inline(always)]
    pub fn with(self, opt: Effect) -> Self {
        Style {
            effects: self.effects.with(opt),
            ..self
        }
    }

    #[inline(always)]
    pub fn without(self, opt: Effect) -> Self {
        Style {
            effects: self.effects.without(opt),
            ..self
        }
    }

    #[inline(always)]
    pub fn toggled(self, opt: Effect) -> Self {
        Style {
            effects: self.effects.toggled(opt),
            ..self
        }
    }
}

impl<F: Copy, B: Copy, U: Copy> Style<F, B, U> {
    #[inline(always)]
    pub const fn const_effect_flags(self, effects: EffectFlags) -> Self {
        Style {
            foreground: self.foreground,
            background: self.background,
            underline_color: self.underline_color,
            effects,
        }
    }

    #[inline(always)]
    pub const fn const_effects<const N: usize>(self, effects: [Effect; N]) -> Self {
        Style {
            foreground: self.foreground,
            background: self.background,
            underline_color: self.underline_color,
            effects: EffectFlags::from_array(effects),
        }
    }

    #[inline(always)]
    pub const fn const_clear_effects(self) -> Self {
        self.const_effect_flags(EffectFlags::new())
    }

    #[inline(always)]
    pub const fn const_with(self, opt: Effect) -> Self {
        self.const_effect_flags(self.effects.with(opt))
    }

    #[inline(always)]
    pub const fn const_without(self, opt: Effect) -> Self {
        self.const_effect_flags(self.effects.without(opt))
    }

    #[inline(always)]
    pub const fn const_toggled(self, opt: Effect) -> Self {
        self.const_effect_flags(self.effects.toggled(opt))
    }
}

Effect! {
    Bold 1 22 -> bold,
    Dimmed 2 22 -> dimmed,
    Italic 3 23 -> italics,
    Underline 4 24 -> underline,
    DoubleUnderline 21 24 -> double_underline,
    Blink 5 25 -> blink,
    BlinkFast 6 25 -> blink_fast,
    Reversed 7 27 -> reverse,
    Hidden 8 28 -> hide,
    Strikethrough 9 29 -> strikethrough,
    Overline 53 55 -> overline,
    SuperScript 73 75 -> superscript,
    SubScript 73 75 -> subscript,
}

const ANY_UNDERLINE: EffectFlags = EffectFlags::new()
    .with(Effect::Underline)
    .with(Effect::DoubleUnderline);

impl<F: OptionalColor, B: OptionalColor, U: OptionalColor> Style<F, B, U> {
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
                if self.effects.data.count_ones() <= 1 {
                    if self.effects.data.is_power_of_two() {
                        let effect = self.effects.iter().next().unwrap();
                        f.write_str(effect.apply_escape())?;
                    }

                    if let Some(fg) = self.foreground.get() {
                        return fg.fmt_foreground(f);
                    }
                }
            }
            (crate::Kind::NeverSome, crate::Kind::AlwaysSome) => {
                if self.effects.data.count_ones() <= 1 {
                    if self.effects.data.is_power_of_two() {
                        let effect = self.effects.iter().next().unwrap();
                        f.write_str(effect.apply_escape())?;
                    }

                    if let Some(bg) = self.background.get() {
                        return bg.fmt_background(f);
                    }
                }
            }
        }

        if self.effects.data.is_power_of_two() {
            let effect = self.effects.iter().next().unwrap();
            f.write_str(effect.apply_escape())?;

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

        if !self.is_plain() {
            f.write_str("\x1b[")?
        }

        if let Some(fg) = self.foreground.get() {
            semicolon = true;
            fg.fmt_foreground_args(f)?;
        }

        if let Some(bg) = self.background.get() {
            semi!();
            semicolon = true;
            bg.fmt_background_args(f)?;
        }

        if !self.effects.data.is_power_of_two() {
            self.effects.try_for_each(|effect| {
                semi!();
                semicolon = true;
                f.write_str(effect.apply_args())?;
                Ok(())
            })?;
        }

        if !self.is_plain() {
            f.write_str("m")?
        }

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
                if self.effects.data.count_ones() <= 1 {
                    if self.effects.data.is_power_of_two() {
                        let effect = self.effects.iter().next().unwrap();
                        f.write_str(effect.clear_escape())?;
                    }

                    return ansi::Default.fmt_foreground(f);
                }
            }
            (crate::Kind::NeverSome, crate::Kind::AlwaysSome) => {
                if self.effects.data.count_ones() <= 1 {
                    if self.effects.data.is_power_of_two() {
                        let effect = self.effects.iter().next().unwrap();
                        f.write_str(effect.clear_escape())?;
                    }

                    return ansi::Default.fmt_background(f);
                }
            }
        }

        if self.effects.data.is_power_of_two() {
            let effect = self.effects.iter().next().unwrap();
            f.write_str(effect.clear_escape())?;

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

        if self.is_complete() {
            return Style::fmt_clear_all(f);
        }

        if !self.is_plain() {
            f.write_str("\x1b[")?
        }

        if self.foreground.get().is_some() {
            semicolon = true;
            ansi::Default.fmt_foreground_args(f)?;
        }

        if self.background.get().is_some() {
            semi!();
            semicolon = true;
            ansi::Default.fmt_background_args(f)?;
        }

        if !self.effects.data.is_power_of_two() {
            self.effects.try_for_each(|effect| {
                semi!();
                semicolon = true;
                f.write_str(effect.clear_args())?;
                Ok(())
            })?;
        }

        if !self.is_plain() {
            f.write_str("m")?
        }

        Ok(())
    }

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

pub struct EffectFlagsIter {
    data: u16,
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

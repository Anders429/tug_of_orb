use deranged::{RangedU16, RangedU8};

#[derive(Debug)]
#[repr(transparent)]
pub struct DisplayStatus(u16);

impl DisplayStatus {
    pub const ENABLE_VBLANK_INTERRUPTS: Self = Self(0b0000_0000_0000_1000);
}

#[derive(Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct DisplayControl(u16);

impl DisplayControl {
    pub const fn new() -> Self {
        Self(0)
    }

    pub const fn with_bg_mode(self, mode: RangedU8<0, 5>) -> Self {
        Self(self.0 & !7 | mode.get() as u16)
    }

    pub const fn with_obj_vram_1d(self, set: bool) -> Self {
        Self(self.0 & !(1 << 6) | ((set as u16) << 6))
    }

    pub const fn with_bg0(self, show: bool) -> Self {
        Self(self.0 & !(1 << 8) | ((show as u16) << 8))
    }

    pub const fn with_bg1(self, show: bool) -> Self {
        Self(self.0 & !(1 << 9) | ((show as u16) << 9))
    }

    pub const fn with_bg2(self, show: bool) -> Self {
        Self(self.0 & !(1 << 10) | ((show as u16) << 10))
    }

    pub const fn with_bg3(self, show: bool) -> Self {
        Self(self.0 & !(1 << 11) | ((show as u16) << 11))
    }

    pub const fn with_obj(self, show: bool) -> Self {
        Self(self.0 & !(1 << 12) | ((show as u16) << 12))
    }
}

#[derive(Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct BackgroundControl(u16);

impl BackgroundControl {
    pub const fn new() -> Self {
        Self(0)
    }

    /// Set the priority.
    ///
    /// Note that 0 is *highest* priority, and 3 is *lowest*.
    pub const fn with_priority(self, priority: RangedU8<0, 3>) -> Self {
        Self(self.0 & !3 | priority.get() as u16)
    }

    pub const fn with_charblock(self, charblock: RangedU8<0, 3>) -> Self {
        Self(self.0 & !(3 << 2) | ((charblock.get() as u16) << 2))
    }

    pub const fn with_8bpp(self, set: bool) -> Self {
        Self(self.0 & !(1 << 7) | (set as u16) << 7)
    }

    pub const fn with_screenblock(self, screenblock: RangedU8<0, 31>) -> Self {
        Self(self.0 & !(31 << 8) | ((screenblock.get() as u16) << 8))
    }

    pub const fn with_screen_size(self, size: RangedU8<0, 3>) -> Self {
        Self(self.0 & !(3 << 14) | ((size.get() as u16) << 14))
    }
}

#[derive(Debug)]
#[repr(u16)]
pub enum ColorEffect {
    None,
    Blend,
    Brighten,
    Darken,
}

#[derive(Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct BlendControl(u16);

impl BlendControl {
    pub const fn new() -> Self {
        Self(0)
    }

    pub const fn with_target1_bg0(self, set: bool) -> Self {
        Self(self.0 & !1 | set as u16)
    }

    pub const fn with_target1_bg1(self, set: bool) -> Self {
        Self(self.0 & !(1 << 1) | (set as u16) << 1)
    }

    pub const fn with_target1_bg2(self, set: bool) -> Self {
        Self(self.0 & !(1 << 2) | (set as u16) << 2)
    }

    pub const fn with_target1_bg3(self, set: bool) -> Self {
        Self(self.0 & !(1 << 3) | (set as u16) << 3)
    }

    pub const fn with_target1_obj(self, set: bool) -> Self {
        Self(self.0 & !(1 << 4) | (set as u16) << 4)
    }

    pub const fn with_target1_backdrop(self, set: bool) -> Self {
        Self(self.0 & !(1 << 5) | (set as u16) << 5)
    }

    pub const fn with_color_effect(self, effect: ColorEffect) -> Self {
        Self(self.0 & !(3 << 6) | (effect as u16) << 6)
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Color(u16);

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct TextScreenEntry(u16);

impl TextScreenEntry {
    pub const fn new() -> Self {
        Self(0)
    }

    pub const fn with_tile(self, tile: RangedU16<0, 1023>) -> Self {
        Self(self.0 & !1023 | tile.get())
    }

    pub const fn with_hflip(self, flipped: bool) -> Self {
        Self(self.0 & !(1 << 10) | ((flipped as u16) << 10))
    }

    pub const fn with_vflip(self, flipped: bool) -> Self {
        Self(self.0 & !(1 << 11) | ((flipped as u16) << 11))
    }

    pub const fn with_palette(self, palette: RangedU8<0, 15>) -> Self {
        Self(self.0 & !(15 << 12) | ((palette.get() as u16) << 12))
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct ObjectAttributes(u64);

impl ObjectAttributes {
    pub const fn new() -> Self {
        Self(0)
    }

    pub const fn with_y(self, y: u8) -> Self {
        Self(self.0 & !255 | (y as u64))
    }

    pub const fn with_disabled(self, disabled: bool) -> Self {
        Self(self.0 & !(1 << 9) | ((disabled as u64) << 9))
    }

    // TODO: Make this use a `RangedU16`.
    pub const fn with_x(self, x: u16) -> Self {
        Self(self.0 & !(511 << 16) | ((x as u64) << 16))
    }

    pub const fn with_size(self, size: RangedU8<0, 3>) -> Self {
        Self(self.0 & !(3 << 30) | ((size.get() as u64) << 30))
    }

    pub const fn with_tile(self, tile: RangedU16<0, 1023>) -> Self {
        Self(self.0 & !(1023 << 32) | ((tile.get() as u64) << 32))
    }

    pub const fn with_palette(self, palette: RangedU8<0, 15>) -> Self {
        Self(self.0 & !(15 << 44) | ((palette.get() as u64) << 44))
    }
}

#[cfg(test)]
mod tests {
    use super::{BackgroundControl, BlendControl, ColorEffect, DisplayControl};
    use deranged::RangedU8;
    use gba_test::test;

    #[test]
    fn background_control_priority() {
        assert_eq!(
            BackgroundControl::new().with_priority(RangedU8::new_static::<2>()),
            BackgroundControl(0b0000_0000_0000_0010)
        );
    }

    #[test]
    fn background_control_charblock() {
        assert_eq!(
            BackgroundControl::new().with_charblock(RangedU8::new_static::<1>()),
            BackgroundControl(0b0000_0000_0000_0100)
        );
    }

    #[test]
    fn background_control_8bpp() {
        assert_eq!(
            BackgroundControl::new().with_8bpp(true),
            BackgroundControl(0b0000_0000_1000_0000)
        );
    }

    #[test]
    fn background_control_screenblock() {
        assert_eq!(
            BackgroundControl::new().with_screenblock(RangedU8::new_static::<25>()),
            BackgroundControl(0b0001_1001_0000_0000)
        );
    }

    #[test]
    fn background_control_screen_size() {
        assert_eq!(
            BackgroundControl::new().with_screen_size(RangedU8::new_static::<3>()),
            BackgroundControl(0b1100_0000_0000_0000)
        );
    }

    #[test]
    fn display_control_bg_mode() {
        assert_eq!(
            DisplayControl::new().with_bg_mode(RangedU8::new_static::<5>()),
            DisplayControl(0b0000_0000_0000_0101)
        )
    }

    #[test]
    fn display_control_obj_vram_1d() {
        assert_eq!(
            DisplayControl::new().with_obj_vram_1d(true),
            DisplayControl(0b0000_0000_0100_0000)
        )
    }

    #[test]
    fn display_control_bg0() {
        assert_eq!(
            DisplayControl::new().with_bg0(true),
            DisplayControl(0b0000_0001_0000_0000)
        )
    }

    #[test]
    fn display_control_bg1() {
        assert_eq!(
            DisplayControl::new().with_bg1(true),
            DisplayControl(0b0000_0010_0000_0000)
        )
    }

    #[test]
    fn display_control_bg2() {
        assert_eq!(
            DisplayControl::new().with_bg2(true),
            DisplayControl(0b0000_0100_0000_0000)
        )
    }

    #[test]
    fn display_control_bg3() {
        assert_eq!(
            DisplayControl::new().with_bg3(true),
            DisplayControl(0b0000_1000_0000_0000)
        )
    }

    #[test]
    fn display_control_obj() {
        assert_eq!(
            DisplayControl::new().with_obj(true),
            DisplayControl(0b0001_0000_0000_0000)
        )
    }

    #[test]
    fn blend_control_target1_bg0() {
        assert_eq!(
            BlendControl::new().with_target1_bg0(true),
            BlendControl(0b0000_0000_0000_0001)
        );
    }

    #[test]
    fn blend_control_target1_bg1() {
        assert_eq!(
            BlendControl::new().with_target1_bg1(true),
            BlendControl(0b0000_0000_0000_0010)
        );
    }

    #[test]
    fn blend_control_target1_bg2() {
        assert_eq!(
            BlendControl::new().with_target1_bg2(true),
            BlendControl(0b0000_0000_0000_0100)
        );
    }

    #[test]
    fn blend_control_target1_bg3() {
        assert_eq!(
            BlendControl::new().with_target1_bg3(true),
            BlendControl(0b0000_0000_0000_1000)
        );
    }

    #[test]
    fn blend_control_target1_obj() {
        assert_eq!(
            BlendControl::new().with_target1_obj(true),
            BlendControl(0b0000_0000_0001_0000)
        );
    }

    #[test]
    fn blend_control_target1_backdrop() {
        assert_eq!(
            BlendControl::new().with_target1_backdrop(true),
            BlendControl(0b0000_0000_0010_0000)
        );
    }

    #[test]
    fn blend_control_color_effect() {
        assert_eq!(
            BlendControl::new().with_color_effect(ColorEffect::Darken),
            BlendControl(0b0000_0000_1100_0000)
        );
    }
}

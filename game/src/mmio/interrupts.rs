#[derive(Debug)]
#[repr(transparent)]
pub struct Interrupts(u16);

impl Interrupts {
    pub const VBLANK: Self = Self(0b0000_0000_0000_0001);
}

#[derive(Debug)]
#[repr(u8)]
pub enum Prescaler {
    Freq1 = 0,
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Control(u16);

impl Control {
    pub const fn new() -> Self {
        Self(0)
    }

    pub const fn with_prescaler(self, prescaler: Prescaler) -> Self {
        Self(self.0 & !3 | (prescaler as u16))
    }

    pub const fn with_enable(self, set: bool) -> Self {
        Self(self.0 & !(1 << 7) | ((set as u16) << 7))
    }
}

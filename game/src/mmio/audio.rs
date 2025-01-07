#[derive(Debug)]
#[repr(transparent)]
pub struct Enable(u16);

impl Enable {
    pub const fn new() -> Self {
        Self(0)
    }

    pub const fn master_enable(self, set: bool) -> Self {
        Self(self.0 & !(1 << 7) | ((set as u16) << 7))
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Control(u16);

impl Control {
    pub const fn new() -> Self {
        Self(0)
    }

    pub const fn sound_a_right(self, set: bool) -> Self {
        Self(self.0 & !(1 << 8) | ((set as u16) << 8))
    }

    pub const fn sound_a_left(self, set: bool) -> Self {
        Self(self.0 & !(1 << 9) | ((set as u16) << 9))
    }

    pub const fn sound_a_fifo_reset(self, set: bool) -> Self {
        Self(self.0 & !(1 << 11) | ((set as u16) << 11))
    }
}

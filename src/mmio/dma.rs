#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub(crate) struct DmaControl(u16);

impl DmaControl {
    pub(crate) const fn new() -> Self {
        Self(0)
    }

    pub(crate) const fn with_transfer_32bit(self) -> Self {
        Self(self.0 | 0b0000_0100_0000_0000)
    }

    pub(crate) const fn with_enabled(self) -> Self {
        Self(self.0 | 0b1000_0000_0000_0000)
    }

    pub(crate) const fn to_u16(self) -> u16 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::DmaControl;
    use gba_test::test;

    #[test]
    fn dma_control_empty() {
        assert_eq!(DmaControl::new().to_u16(), 0);
    }

    #[test]
    fn dma_control_with_transfer_32bit() {
        assert_eq!(DmaControl::new().with_transfer_32bit().to_u16(), 1024);
    }

    #[test]
    fn dma_control_with_enabled() {
        assert_eq!(DmaControl::new().with_enabled().to_u16(), 32768);
    }

    #[test]
    fn dma_control_with_all() {
        assert_eq!(
            DmaControl::new()
                .with_transfer_32bit()
                .with_enabled()
                .to_u16(),
            33792
        );
    }
}

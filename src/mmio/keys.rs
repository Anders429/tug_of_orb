#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct KeyInput(u16);

impl KeyInput {
    pub const NONE: Self = Self(0b0000_0011_1111_1111);
    pub const A: Self = Self(0b0000_0011_1111_1110);
    pub const B: Self = Self(0b0000_0011_1111_1101);
    pub const SELECT: Self = Self(0b0000_0011_1111_1011);
    pub const START: Self = Self(0b0000_0011_1111_0111);
    pub const RIGHT: Self = Self(0b0000_0011_1110_1111);
    pub const LEFT: Self = Self(0b0000_0011_1101_1111);
    pub const UP: Self = Self(0b0000_0011_1011_1111);
    pub const DOWN: Self = Self(0b0000_0011_0111_1111);
    pub const R: Self = Self(0b0000_0010_1111_1111);
    pub const L: Self = Self(0b0000_0001_1111_1111);

    pub const fn contains(self, other: Self) -> bool {
        (Self::NONE.0 ^ self.0) & (Self::NONE.0 ^ other.0) == (Self::NONE.0 ^ other.0)
    }
}

#[cfg(test)]
mod tests {
    use super::KeyInput;
    use gba_test::test;

    #[test]
    fn key_input_none_contains_none() {
        assert!(KeyInput::NONE.contains(KeyInput::NONE))
    }

    #[test]
    fn key_input_none_contains_a() {
        assert!(!KeyInput::NONE.contains(KeyInput::A))
    }

    #[test]
    fn key_input_a_contains_none() {
        assert!(KeyInput::A.contains(KeyInput::NONE))
    }

    #[test]
    fn key_input_a_b_contains_b() {
        assert!(KeyInput(0b0000_0011_1111_1100).contains(KeyInput::B))
    }

    #[test]
    fn key_input_all_contains_all() {
        assert!(KeyInput(0).contains(KeyInput(0)))
    }

    #[test]
    fn key_input_all_contains_none() {
        assert!(KeyInput(0).contains(KeyInput::NONE))
    }
}

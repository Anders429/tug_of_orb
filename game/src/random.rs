//! Random number generations.
//!
//! Any RNG used in this project is interacted with through the `rand` crate. As such, they should
//! always implement `RngCore`.

use rand_core::RngCore;

/// An implementation of a permuted congruential generator, utilizing a multiplicative congruential
/// generator instead of a linear congruential generator.
///
/// This PCG implementation is optimized for both speed and distribution quality. While it uses a
/// 64-bit state, the multiplier is only 32 bits and therefore allows for less instructions on a
/// 32-bit architecture.
#[derive(Debug)]
pub struct Pcg32Fast<const MULTIPLIER: u32 = 0xf13283ad> {
    /// The generator's current state.
    state: u64,
}

impl<const MULTIPLIER: u32> Pcg32Fast<MULTIPLIER> {
    /// Creates a new generator from the given seed.
    pub const fn new(seed: u64) -> Self {
        Self {
            // The state must be odd.
            state: seed.wrapping_mul(2).wrapping_add(1),
        }
    }
}

impl<const MULTIPLIER: u32> RngCore for Pcg32Fast<MULTIPLIER> {
    fn next_u32(&mut self) -> u32 {
        let count = self.state >> 61;
        self.state = self.state.wrapping_mul(MULTIPLIER.into());
        self.state ^= self.state >> 22;
        (self.state >> (22 + count)) as u32
    }

    fn next_u64(&mut self) -> u64 {
        rand_core::impls::next_u64_via_u32(self)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        rand_core::impls::fill_bytes_via_next(self, dest);
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // These tests are mostly to make sure that the distribution is sufficiently random (for
    // example, we aren't just repeating between even and odd numbers, and consequentially we
    // aren't just repeating between true and false values), as well as to make sure the RNG isn't
    // accidentally changed. If the RNG is purposefully changed, just verify that the results are
    // still sufficiently random.

    use super::Pcg32Fast;
    use gba_test::test;
    use rand::{Rng, RngCore};

    #[test]
    fn pcg_next_u32() {
        let mut pcg = Pcg32Fast::<0xf13283ad>::new(0xcafef00dd15ea5e5);

        assert_eq!(pcg.next_u32(), 4_129_627_752);
        assert_eq!(pcg.next_u32(), 2_392_381_265);
        assert_eq!(pcg.next_u32(), 4_143_941_195);
        assert_eq!(pcg.next_u32(), 220_930_253);
        assert_eq!(pcg.next_u32(), 4_257_109_229);
        assert_eq!(pcg.next_u32(), 972_448_136);
        assert_eq!(pcg.next_u32(), 934_764_305);
        assert_eq!(pcg.next_u32(), 873_243_406);
        assert_eq!(pcg.next_u32(), 383_284_308);
        assert_eq!(pcg.next_u32(), 780_176_459);
    }

    #[test]
    fn pcg_next_u64() {
        let mut pcg = Pcg32Fast::<0xf13283ad>::new(0xcafef00dd15ea5e5);

        assert_eq!(pcg.next_u64(), 10_275_199_296_867_737_192);
        assert_eq!(pcg.next_u64(), 948_888_215_475_947_083);
        assert_eq!(pcg.next_u64(), 4_176_632_945_433_269_485);
        assert_eq!(pcg.next_u64(), 3_750_551_871_152_414_481);
        assert_eq!(pcg.next_u64(), 3_350_832_376_897_369_172);
        assert_eq!(pcg.next_u64(), 8_157_516_009_857_891_989);
        assert_eq!(pcg.next_u64(), 3_756_168_090_588_794_237);
        assert_eq!(pcg.next_u64(), 9_568_290_154_139_263_811);
        assert_eq!(pcg.next_u64(), 2_725_406_287_625_284_761);
        assert_eq!(pcg.next_u64(), 3_927_429_650_481_207_189);
    }

    #[test]
    fn pcg_gen_bool() {
        let mut pcg = Pcg32Fast::<0xf13283ad>::new(0xcafef00dd15ea5e5);

        assert_eq!(pcg.gen::<bool>(), true);
        assert_eq!(pcg.gen::<bool>(), true);
        assert_eq!(pcg.gen::<bool>(), true);
        assert_eq!(pcg.gen::<bool>(), false);
        assert_eq!(pcg.gen::<bool>(), true);
        assert_eq!(pcg.gen::<bool>(), false);
        assert_eq!(pcg.gen::<bool>(), false);
        assert_eq!(pcg.gen::<bool>(), false);
        assert_eq!(pcg.gen::<bool>(), false);
        assert_eq!(pcg.gen::<bool>(), false);
    }
}

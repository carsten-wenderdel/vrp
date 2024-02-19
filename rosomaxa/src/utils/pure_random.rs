use rand::prelude::*;
use rand::Error;


/// Provides the way to use randomized values in generic way.
pub trait PureRandom {
    /// Creates a new RNG, different for every call.
    fn new_pure_random(&mut self) -> Self;

    /// Produces integral random value, uniformly distributed on the closed interval [min, max]
    fn uniform_int(&mut self, min: i32, max: i32) -> i32;

    /// Produces real random value, uniformly distributed on the closed interval [min, max)
    fn uniform_real(&mut self, min: f64, max: f64) -> f64;
    /// Flips a coin and returns true if it is "heads", false otherwise.
    fn is_head_not_tails(&mut self) -> bool;

    /// Tests probability value in (0., 1.) range.
    fn is_hit(&mut self, probability: f64) -> bool;

    /// Returns an index from collected with probability weight.
    /// Uses exponential distribution where the weights are the rate of the distribution (lambda)
    /// and selects the smallest sampled value.
    fn weighted(&mut self, weights: &[usize]) -> usize;
}

/// A default random implementation.
pub struct DefaultPureRandom {
    rng: SmallRng,
}

impl DefaultPureRandom {
    /// This behaves like the trait `Copy`, but we don't implement the trait, as usually this behaviour
    /// is not desired for random number generators
    ///
    /// In general, we probably don't want to use this method too often as the return value will
    /// behave exactly as the original value. If we have a single (global) instance of this struct,
    /// it should probably be mutable, so that newly constructed instances can be different for each call.
    #[inline]
    pub fn generate_copy(&self) -> Self {
        Self { rng: self.rng.clone() }
    }

    /// Creates an instance of `DefaultPureRandom` with repeatable (predictable) random generation.
    #[inline]
    pub fn with_seed(seed: u64) -> Self {
        Self { rng: SmallRng::seed_from_u64(seed) }
    }

    /// Creates an instance of `DefaultPureRandom` with reproducible behavior.
    #[inline]
    pub fn for_tests() -> Self {
        Self { rng: SmallRng::seed_from_u64(1234567890) }
    }

    /// Creates a randomly initialized instance of `DefaultPureRandom`.
    #[inline]
    pub fn new_random() -> Self {
        Self { rng: SmallRng::from_rng(thread_rng()).expect("cannot get RNG from thread rng") }
    }
}

impl PureRandom for DefaultPureRandom {
    #[inline]
    fn new_pure_random(&mut self) -> Self {
        Self { rng: SmallRng::seed_from_u64(self.next_u64())}
    }
    #[inline]
    fn uniform_int(&mut self, min: i32, max: i32) -> i32 {
        if min == max {
            return min;
        }

        assert!(min < max);
        self.rng.gen_range(min..max + 1)
    }

    #[inline]
    fn uniform_real(&mut self, min: f64, max: f64) -> f64 {
        if (min - max).abs() < f64::EPSILON {
            return min;
        }

        assert!(min < max);
        self.rng.gen_range(min..max)
    }

    #[inline]
    fn is_head_not_tails(&mut self) -> bool {
        self.rng.gen_bool(0.5)
    }

    #[inline]
    fn is_hit(&mut self, probability: f64) -> bool {
        self.gen_bool(probability.clamp(0., 1.))
    }

    #[inline]
    fn weighted(&mut self, weights: &[usize]) -> usize {
        weights
            .iter()
            .zip(0_usize..)
            .map(|(&weight, index)| (-self.uniform_real(0., 1.).ln() / weight as f64, index))
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
            .unwrap()
            .1
    }
}

/// Reimplementing RngCore helps to set breakpoints and also hides the usage of SmallRng.
impl RngCore for DefaultPureRandom {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        self.rng.next_u32()
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        self.rng.next_u64()
    }

    #[inline]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.rng.fill_bytes(dest)
    }

    #[inline]
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        self.rng.try_fill_bytes(dest)
    }
}

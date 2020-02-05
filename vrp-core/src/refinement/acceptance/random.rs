#[cfg(test)]
#[path = "../../../tests/unit/refinement/acceptance/greedy_test.rs"]
mod greedy_test;

use crate::refinement::acceptance::{Acceptance, Greedy};
use crate::refinement::{Individuum, RefinementContext};

/// Decorates existing acceptance method by adding extra logic which accepts some solutions
/// randomly with given probability.
pub struct RandomProbability {
    other: Box<dyn Acceptance>,
    probability: f64,
}

impl RandomProbability {
    /// Creates a new instance.
    pub fn new(other: Box<dyn Acceptance>, probability: f64) -> Self {
        Self { other, probability }
    }
}

impl Default for RandomProbability {
    fn default() -> Self {
        Self::new(Box::new(Greedy::default()), 0.001)
    }
}

impl Acceptance for RandomProbability {
    fn is_accepted(&self, refinement_ctx: &mut RefinementContext, solution: &Individuum) -> bool {
        let random = solution.0.random.clone();

        self.other.is_accepted(refinement_ctx, solution) || self.probability > random.uniform_real(0., 1.)
    }
}
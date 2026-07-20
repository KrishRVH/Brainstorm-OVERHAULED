use crate::engine::rng::RngState;
use crate::seed::Seed;

#[derive(Clone, Debug)]
pub(crate) struct SearchState {
    pub seed: Seed,
    pub hashed_seed: f64,
    pub rng: RngState,
}

impl SearchState {
    pub(crate) fn from_id(id: i64) -> Self {
        let mut seed = Seed::from_id(id);
        let hashed_seed = seed.pseudohash(0);
        Self {
            seed,
            hashed_seed,
            rng: RngState::default(),
        }
    }

    pub(crate) fn next(&mut self) {
        self.hashed_seed = self.seed.next_and_pseudohash_zero();
        self.rng.clear();
    }
}

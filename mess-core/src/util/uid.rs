use std::collections::HashSet;

use rand::{
    prelude::ThreadRng,
    thread_rng,
    RngCore,
};

/// Convenience struct for generating unique u64s
pub struct UIDGenerator {
    uid_set: HashSet<u64>,
    rng: ThreadRng,
}

impl Default for UIDGenerator {
    fn default() -> UIDGenerator {
        UIDGenerator {
            uid_set: HashSet::new(),
            rng: thread_rng(),
        }
    }
}

impl UIDGenerator {
    pub fn generate(&mut self) -> u64 {
        let mut uid = self.rng.next_u64();
        while self.uid_set.contains(&uid) {
            uid = self.rng.next_u64();
        }
        self.uid_set.insert(uid);
        uid
    }
}

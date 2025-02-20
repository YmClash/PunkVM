//src/pvm/cache_stats.rs

use std::fmt;


#[derive(Debug, Default, Clone)]
pub struct CacheStatistics {
    pub hits: usize,
    pub misses: usize,
    pub write_backs: usize,
    pub invalidations: usize,
    pub coherence_misses: usize,
    pub write_hits: usize,
    pub write_misses: usize,
    pub evictions: usize,
}
impl CacheStatistics {
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    pub fn write_back_rate(&self) -> f64 {
        let total_writes = self.write_hits + self.write_misses;
        if total_writes == 0 {
            0.0
        } else {
            self.write_backs as f64 / total_writes as f64
        }
    }

    pub fn merge_with_next_level(&mut self, next_level: &CacheStatistics) {
        self.hits += next_level.hits;
        self.misses += next_level.misses;
        self.write_backs += next_level.write_backs;
        self.invalidations += next_level.invalidations;
        self.coherence_misses += next_level.coherence_misses;
        self.write_hits += next_level.write_hits;
        self.write_misses += next_level.write_misses;
        self.evictions += next_level.evictions;
    }

    pub fn total_accesses(&self) -> usize {
        self.hits + self.misses
    }

}

impl fmt::Display for CacheStatistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Cache Statistics:\n\
             Hit Rate: {:.2}%\n\
             Write Back Rate: {:.2}%\n\
             Hits: {}\n\
             Misses: {}\n\
             Write Backs: {}\n\
             Invalidations: {}\n\
             Coherence Misses: {}\n\
             Write Hits: {}\n\
             Write Misses: {}\n\
             Evictions: {}\n",
            self.hit_rate() * 100.0,
            self.write_back_rate() * 100.0,
            self.hits,
            self.misses,
            self.write_backs,
            self.invalidations,
            self.coherence_misses,
            self.write_hits,
            self.write_misses,
            self.evictions
        )
    }
}
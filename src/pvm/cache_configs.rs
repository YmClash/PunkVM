//src/pvm/cache_configs.rs
use crate::pvm::caches::Cache;
use crate::pvm::vm_errors::VMResult;
use crate::pvm::cache_stats::CacheStatistics;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WritePolicy {
    WriteThrough,
    WriteBack,
}

#[derive(Debug, Clone, Copy)]
pub enum ReplacementPolicy {
    LRU,
    FIFO,
    Random,
}


#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub size: usize,         // Taille totale du cache
    pub lines_size: usize,   // Taille d'une ligne de cache
    pub associativity: usize,// Niveau d'associativité
    pub write_policy: WritePolicy,
    pub replacement_policy: ReplacementPolicy,
}

impl CacheConfig {
    pub fn new_l1() -> Self {
        Self {
            size: 64 * 1024,   // 64KB
            lines_size: 64,    // 64 octets par ligne
            associativity: 4,  // 4-way set associative
            write_policy: WritePolicy::WriteThrough,
            replacement_policy: ReplacementPolicy::LRU,
        }
    }

    pub fn new_l2() -> Self {
        Self {
            size: 256 * 1024,  // 256KB
            lines_size: 64,    // 64 octets par ligne
            associativity: 8,  // 8-way set associative
            write_policy: WritePolicy::WriteBack,
            replacement_policy: ReplacementPolicy::LRU,
        }
    }

    pub fn num_sets(&self) -> usize {
        self.size / (self.lines_size * self.associativity)
    }

    pub fn is_valid(&self) -> bool {
        self.size.is_power_of_two() &&
            self.lines_size.is_power_of_two() &&
            self.associativity.is_power_of_two() &&
            self.size >= self.lines_size &&
            self.lines_size >= 8 &&
            self.associativity >= 1
    }
}

#[derive(Debug)]
pub struct CacheSystem {
    pub level1: Cache,
    pub level2: Box<Cache>,
    pub statistics: CacheStatistics,
}

impl CacheSystem {
    pub fn new() -> Self {
        let l2_config = CacheConfig::new_l2();
        let l1_config = CacheConfig::new_l1();

        // Créer L2 d'abord
        let level2 = Box::new(Cache::new(l2_config.clone(), None));
        
        // Créer L1 avec référence à L2
        // Note: Pour l'instant on ne peut pas partager le Box, donc L1 n'aura pas de next_level
        let level1 = Cache::new(l1_config, None);

        Self {
            level1,
            level2,
            statistics: CacheStatistics::default(),
        }
    }

    pub fn write(&mut self, addr: u32, value: u8) -> VMResult<()> {
        match self.level1.write(addr, value) {
            Ok(_) => {
                self.statistics.write_hits += 1;
                Ok(())
            },
            Err(_) => {
                self.statistics.write_misses += 1;
                self.level2.write(addr, value)
            }
        }
    }

    pub fn read(&mut self, addr: u32) -> VMResult<u64> {
        match self.level1.read(addr) {
            Ok(value) => {
                self.statistics.hits += 1;
                Ok(value)
            },
            Err(_) => {
                self.statistics.misses += 1;
                self.level2.read(addr)
            }
        }
    }

    pub fn reset(&mut self) -> VMResult<()> {
        self.level1.reset()?;
        self.level2.reset()?;
        self.statistics = CacheStatistics::default();
        Ok(())
    }

    pub fn get_statistics(&self) -> String {
        format!(
            "Cache System Statistics:\n{}\n\nL1 Cache:\n{}\nL2 Cache:\n{}",
            self.statistics,
            self.level1.get_detailed_stats(),
            self.level2.get_detailed_stats()
        )
    }
    // Ajouter la méthode pour obtenir la politique d'écriture
    pub fn write_policy(&self) -> WritePolicy {
        // On retourne la politique du cache L1 par défaut
        self.level1.get_write_policy()
    }
}




////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_config_l1() {
        let l1_config = CacheConfig::new_l1();
        assert_eq!(l1_config.write_policy, WritePolicy::WriteThrough);
        assert!(l1_config.is_valid());
    }

    #[test]
    fn test_cache_config_l2() {
        let l2_config = CacheConfig::new_l2();
        assert_eq!(l2_config.write_policy, WritePolicy::WriteBack);
        assert!(l2_config.is_valid());
    }

    #[test]
    fn test_num_sets_calculation() {
        let config = CacheConfig {
            size: 1024,
            lines_size: 64,
            associativity: 4,
            write_policy: WritePolicy::WriteThrough,
            replacement_policy: ReplacementPolicy::LRU,
        };
        assert_eq!(config.num_sets(), 4); // 1024 / (64 * 4) = 4
    }

    #[test]
    fn test_cache_system() {
        let mut cache_system = CacheSystem::new();

        // Test write puis read
        cache_system.write(0x1000, 42).unwrap();
        assert_eq!(cache_system.read(0x1000).unwrap(), 42);

        // Vérifier les statistiques
        assert!(cache_system.statistics.hits > 0);
        assert!(cache_system.statistics.write_hits > 0);
    }
}
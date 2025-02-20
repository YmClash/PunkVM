//src/pvm/memorys.rs

use std::collections::HashMap;
use crate::pvm::cache_configs::{CacheConfig, ReplacementPolicy, WritePolicy};
use crate::pvm::cache_stats::CacheStatistics;
use crate::pvm::caches::Cache;
use crate::pvm::vm_errors::{VMError, VMResult};

const DEFAULT_MEMORY_SIZE: usize = 1024 * 1024; // 1MB par défaut

pub struct Memory {
    data: HashMap<u64, u64>,
    cache: Cache,
}

pub struct MemoryController {
    pub main_memory: Vec<u8>,
    pub cache: Cache,
}

impl Memory {
    pub fn new() -> VMResult<Self> {
        let l2_config = CacheConfig::new_l2();
        let l1_config = CacheConfig::new_l1();

        let l2_cache = Box::new(Cache::new(l2_config, None));
        let l1_cache = Cache::new(l1_config, Some(l2_cache));

        Ok(Self {
            data: HashMap::new(),
            cache: l1_cache,
        })
    }

    pub fn read(&mut self, addr: u64) -> VMResult<u64> {
        match self.cache.read(addr) {
            Ok(value) => Ok(value),
            Err(_) => {
                self.data
                    .get(&addr)
                    .copied()
                    .ok_or_else(|| VMError::memory_error(&format!("Address {:#x} not found", addr)))
            }
        }
    }

    pub fn write(&mut self, addr: u64, value: u64) -> VMResult<()> {
        self.cache.write(addr, value)?;

        if self.cache.get_write_policy() == WritePolicy::WriteThrough {
            self.data.insert(addr, value);
        }
        Ok(())
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.cache.reset().unwrap_or_default();
    }

    pub fn get_cache_stats(&self) -> String {
        self.cache.get_detailed_stats()
    }
}

impl MemoryController {
    pub fn new(memory_size: usize, cache_size: usize) -> VMResult<Self> {
        let l1_config = CacheConfig {
            size: cache_size,
            lines_size: 64,
            associativity: 4,
            write_policy: WritePolicy::WriteThrough,
            replacement_policy: ReplacementPolicy::LRU,
        };

        Ok(Self {
            main_memory: vec![0; memory_size],
            cache: Cache::new(l1_config, None),
        })
    }

    pub fn with_default_size() -> VMResult<Self> {
        Self::new(DEFAULT_MEMORY_SIZE, DEFAULT_MEMORY_SIZE / 4)
    }

    pub fn reset(&mut self) -> VMResult<()> {
        self.main_memory.fill(0);
        self.cache.reset()
    }

    pub fn read(&mut self, addr: u64) -> VMResult<u64> {
        let addr = addr as usize;
        self.check_bounds(addr, 8)?;

        // Vérifier d'abord dans le cache
        match self.cache.read(addr as u64) {
            Ok(value) => Ok(value),
            Err(_) => {
                // Cache miss, lire depuis la mémoire principale
                let mut bytes = [0u8; 8];
                bytes.copy_from_slice(&self.main_memory[addr..addr + 8]);
                let value = u64::from_le_bytes(bytes);

                // Mettre à jour le cache
                self.cache.write(addr as u64, value)?;
                Ok(value)
            }
        }
    }

    pub fn write(&mut self, addr: u64, value: u64) -> VMResult<()> {
        let addr = addr as usize;
        self.check_bounds(addr, 8)?;

        // Mettre à jour le cache
        self.cache.write(addr as u64, value)?;

        // Écrire dans la mémoire principale si write-through
        if self.cache.get_write_policy() == WritePolicy::WriteThrough {
            let bytes = value.to_le_bytes();
            self.main_memory[addr..addr + 8].copy_from_slice(&bytes);
        }

        Ok(())
    }

    pub fn get_cache_stats(&self) -> VMResult<CacheStatistics> {
        Ok(self.cache.get_statistics().clone())
    }

    fn check_bounds(&self, addr: usize, size: usize) -> VMResult<()> {
        if addr + size > self.main_memory.len() {
            return Err(VMError::memory_error(&format!(
                "Memory access out of bounds at address 0x{:X}",
                addr
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_basic_operations() {
        let mut memory = MemoryController::new(1024, 256).unwrap();

        memory.write(0, 0x1234_5678_9ABC_DEF0).unwrap();
        assert_eq!(memory.read(0).unwrap(), 0x1234_5678_9ABC_DEF0);

        memory.write(8, 0xFEDC_BA98_7654_3210).unwrap();
        assert_eq!(memory.read(8).unwrap(), 0xFEDC_BA98_7654_3210);
        assert_eq!(memory.read(0).unwrap(), 0x1234_5678_9ABC_DEF0);
    }

    #[test]
    fn test_memory_bounds() {
        let mut memory = MemoryController::new(16, 256).unwrap();
        assert!(memory.write(16, 0x1234).is_err());
        assert!(memory.read(16).is_err());
    }

    #[test]
    fn test_memory_alignment() {
        let mut memory = MemoryController::new(1024, 256).unwrap();

        for addr in (0..32).step_by(8) {
            memory.write(addr, addr as u64).unwrap();
            assert_eq!(memory.read(addr).unwrap(), addr as u64);
        }
    }

    #[test]
    fn test_memory_cache_coherence() {
        let mut memory = Memory::new().unwrap();

        // Test write-through behavior
        memory.write(0x1000, 42).unwrap();
        assert_eq!(memory.read(0x1000).unwrap(), 42);

        // Test cache hit
        assert_eq!(memory.read(0x1000).unwrap(), 42);

        // Test overwrite
        memory.write(0x1000, 84).unwrap();
        assert_eq!(memory.read(0x1000).unwrap(), 84);

        // Test clear
        memory.clear();
        assert!(memory.read(0x1000).is_err());
    }

    #[test]
    fn test_cache_statistics() {
        let mut memory = MemoryController::with_default_size().unwrap();

        // Perform some memory accesses
        memory.write(0, 42).unwrap();
        memory.read(0).unwrap();
        memory.read(0).unwrap(); // Should be a cache hit

        let stats = memory.get_cache_stats().unwrap();
        assert!(stats.hits > 0);
        assert!(stats.total_accesses() > stats.hits);
    }
}







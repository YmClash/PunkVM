use crate::pvm::caches::{Cache, CacheStatistics, CacheStats};
use crate::pvm::vm_errors::{VMError, VMResult};

/// Contrôleur de mémoire
pub struct MemoryController {
    pub main_memory: Vec<u8>,
    pub cache: Cache,
}

impl MemoryController{
    pub fn new(memory_size: usize, cache_size: usize) -> VMResult<Self> {
        Ok(Self {
            main_memory: vec![0; memory_size],
            cache: Cache::new(cache_size)?,
        })
    }

    pub fn reset(&mut self) -> VMResult<()> {
        self.main_memory.fill(0);
        self.cache.reset()?;
        Ok(())
    }

    /// Lit une valeur 64 bits à l'adresse spécifiée
    pub fn read(&mut self, addr: u64) -> VMResult<u64> {
        let addr = addr as usize;
        if addr + 8 > self.main_memory.len() {
            return Err(VMError::MemoryError(format!(
                "Tentative de lecture hors limites à l'adresse 0x{:X}",
                addr
            )));
        }

        // Vérifier d'abord dans le cache
        if let Some(value) = self.cache.read(addr)? {
            return Ok(value);
        }

        // Si pas dans le cache, lire depuis la mémoire principale
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&self.main_memory[addr..addr + 8]);
        let value = u64::from_le_bytes(bytes);

        // Mettre à jour le cache
        self.cache.write(addr, value)?;

        Ok(value)
    }

    /// Écrit une valeur 64 bits à l'adresse spécifiée
    pub fn write(&mut self, addr: u64, value: u64) -> VMResult<()> {
        let addr = addr as usize;
        if addr + 8 > self.main_memory.len() {
            return Err(VMError::MemoryError(format!(
                "Tentative d'écriture hors limites à l'adresse 0x{:X}",
                addr
            )));
        }

        // Mettre à jour le cache
        self.cache.write(addr, value)?;

        // Écrire dans la mémoire principale
        let bytes = value.to_le_bytes();
        self.main_memory[addr..addr + 8].copy_from_slice(&bytes);

        Ok(())
    }

    /// Obtien stats du cache
    pub fn get_cache_stats(&self) -> VMResult<CacheStatistics> {
        // Cloner les statistiques plutôt que de retourner une référence
        Ok(self.cache.get_statistics().clone())
    }

}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_basic_operations() {
        let mut memory = MemoryController::new(1024, 256).unwrap();

        // Test écriture/lecture simple
        memory.write(0, 0x1234_5678_9ABC_DEF0).unwrap();
        assert_eq!(memory.read(0).unwrap(), 0x1234_5678_9ABC_DEF0);

        // Test écriture/lecture à différentes adresses
        memory.write(8, 0xFEDC_BA98_7654_3210).unwrap();
        assert_eq!(memory.read(8).unwrap(), 0xFEDC_BA98_7654_3210);
        assert_eq!(memory.read(0).unwrap(), 0x1234_5678_9ABC_DEF0);
    }

    #[test]
    fn test_memory_bounds() {
        let mut memory = MemoryController::new(16, 256).unwrap();

        // Test écriture hors limites
        assert!(memory.write(16, 0x1234).is_err());

        // Test lecture hors limites
        assert!(memory.read(16).is_err());
    }

    #[test]
    fn test_memory_alignment() {
        let mut memory = MemoryController::new(1024, 256).unwrap();

        // Test écriture/lecture à des adresses alignées
        for addr in (0..32).step_by(8) {
            memory.write(addr, addr as u64).unwrap();
            assert_eq!(memory.read(addr).unwrap(), addr as u64);
        }
    }

    // #[test]
    // fn test_cache_statistics() {
    //     let mut memory = MemoryController::new(1024, 256).unwrap();
    //
    //     // Effectuer quelques accès mémoire
    //     memory.write(0, 42).unwrap();
    //     memory.read(0).unwrap();
    //     memory.read(0).unwrap(); // Devrait être un hit de cache
    //
    //     let stats = memory.get_cache_stats().unwrap();
    //     assert!(stats.hits > 0);
    //     assert!(stats.total_accesses > stats.hits);
    // }
}
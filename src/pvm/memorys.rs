// //src/pvm/memorys.rs
//
// use std::collections::HashMap;
// use crate::pvm::cache_configs::{CacheConfig, ReplacementPolicy, WritePolicy};
// use crate::pvm::cache_stats::CacheStatistics;
// use crate::pvm::caches::Cache;
// use crate::pvm::vm_errors::{VMError, VMResult};
//
// const DEFAULT_MEMORY_SIZE: usize = 1024 * 1024; // 1MB par défaut
//
// pub struct Memory {
//     data: HashMap<u64, u64>,
//     cache: Cache,
// }
//
// pub struct MemoryController {
//     pub main_memory: Vec<u8>,
//     pub cache: Cache,
// }
//
// impl Memory {
//     pub fn new() -> VMResult<Self> {
//         // Crée un L2, puis un L1 branché sur L2
//         let l2_config = CacheConfig::new_l2();
//         let l1_config = CacheConfig::new_l1();
//
//         let l2_cache = Box::new(Cache::new(l2_config, None));
//         let l1_cache = Cache::new(l1_config, Some(l2_cache));
//
//         Ok(Self {
//             data: HashMap::new(),
//             cache: l1_cache,
//         })
//     }
//
//
//     /// Lecture avec le cache:
//     ///  1) cache.read(...) => Ok => renvoie
//     ///  2) si Err => on cherche data.get(...) => si absent => Err
//     pub fn read(&mut self, addr: u64) -> VMResult<u64> {
//         // self.check_alignement(addr, 8)?;
//         match self.cache.read(addr) {
//             Ok(value) => {
//                 // Trouvé dans le cache
//                 Ok(value)
//             }
//             Err(_) => {
//                 // Pas trouvé ni en L1 ni en L2 => on check "data"
//                 self.data
//                     .get(&addr)
//                     .copied()
//                     .ok_or_else(|| VMError::memory_error(&format!(
//                         "Address {:#x} not found", addr
//                     )))
//             }
//         }
//     }
//
//     /// Ecrit d'abord dans le cache.
//     /// Si WriteThrough, on actualise `data`.
//     pub fn write(&mut self, addr: u64, value: u64) -> VMResult<()> {
//         self.cache.write(addr, value)?;
//
//         // Si c'est WriteThrough => on stocke aussi dans data
//         if self.cache.get_write_policy() == WritePolicy::WriteThrough {
//             self.data.insert(addr, value);
//         }
//         Ok(())
//     }
//
//     /// Vide tout: data.clear() + cache.reset() => plus de traces => read(...) -> Err
//     pub fn clear(&mut self) {
//         self.data.clear();
//         // On force un reset complet du cache
//         if let Some(l2_cache) = &mut self.cache.next_level {
//             l2_cache.reset().unwrap_or_default();
//         }
//         self.cache.reset().unwrap_or_default();
//     }
//
//     pub fn get_cache_stats(&self) -> String {
//         self.cache.get_detailed_stats()
//     }
//
//     pub fn check_alignement(&self, addr: u64, size: usize) -> VMResult<()> {
//         if addr % size as u64 != 0 {
//             return Err(VMError::memory_error(&format!(
//                 "Unaligned memory access at address {:#x}", addr
//             )));
//         }
//         Ok(())
//     }
// }
//
// impl MemoryController {
//     pub fn new(memory_size: usize, cache_size: usize) -> VMResult<Self> {
//         // Simple L1 direct, pas de next_level
//         let l1_config = CacheConfig {
//             size: cache_size,
//             lines_size: 64,
//             associativity: 4,
//             write_policy: WritePolicy::WriteThrough,
//             replacement_policy: ReplacementPolicy::LRU,
//         };
//
//         Ok(Self {
//             main_memory: vec![0; memory_size],
//             cache: Cache::new(l1_config, None),
//         })
//     }
//
//     pub fn with_default_size() -> VMResult<Self> {
//         Self::new(DEFAULT_MEMORY_SIZE, DEFAULT_MEMORY_SIZE / 4)
//     }
//
//     /// Remet main_memory à 0 + reset le cache
//     pub fn reset(&mut self) -> VMResult<()> {
//         self.main_memory.fill(0);
//         self.cache.reset()
//     }
//
//
//     /// Lit 8 octets à l’adresse `addr`, en passant par `cache`.
//     ///  1) On tente un `cache.read(...)`
//     ///  2) s'il y a Err => on lit main_memory => on “allocate” => write en cache
//     // pub fn read(&mut self, addr: u64) -> VMResult<u64> {
//     //     let addr_usize = addr as usize;
//     //     self.check_bounds(addr_usize, 8)?;
//     //
//     //     match self.cache.read(addr) {
//     //         Ok(value) => Ok(value),
//     //         Err(_) => {
//     //             // Miss => lire "main_memory"
//     //             let mut bytes = [0u8; 8];
//     //             bytes.copy_from_slice(&self.main_memory[addr_usize..addr_usize + 8]);
//     //             let value = u64::from_le_bytes(bytes);
//     //
//     //             // On "place" la donnée dans le cache => write
//     //             self.cache.write(addr, value)?;
//     //             Ok(value)
//     //         }
//     //     }
//     // }
//
//     pub fn read(&mut self, addr: u64) -> VMResult<u64> {
//         let addr_usize = addr as usize;
//         self.check_bounds(addr_usize, 8)?;
//
//         match self.cache.read(addr) {
//             Ok(value) => Ok(value),
//             Err(_) => {
//                 // Miss => incrémenter le compteur
//                 self.cache.increment_misses();
//
//                 // Lire depuis la mémoire principale
//                 let mut bytes = [0u8; 8];
//                 bytes.copy_from_slice(&self.main_memory[addr_usize..addr_usize + 8]);
//                 let value = u64::from_le_bytes(bytes);
//
//                 // Mettre en cache
//                 self.cache.write(addr, value)?;
//                 Ok(value)
//             }
//         }
//     }
//
//     // Écrit 8 octets. Politique “WriteThrough” => on écrit main_memory aussi
//     pub fn write(&mut self, addr: u64, value: u64) -> VMResult<()> {
//         let addr_usize = addr as usize;
//         self.check_bounds(addr_usize, 8)?;
//
//         // Ecrit dans cache
//         self.cache.write(addr, value)?;
//
//         if self.cache.get_write_policy() == WritePolicy::WriteThrough {
//             let bytes = value.to_le_bytes();
//             self.main_memory[addr_usize..addr_usize + 8].copy_from_slice(&bytes);
//         }
//
//         Ok(())
//     }
//
//     pub fn get_cache_stats(&self) -> VMResult<CacheStatistics> {
//         Ok(self.cache.get_statistics().clone())
//     }
//
//     fn check_bounds(&self, addr: usize, size: usize) -> VMResult<()> {
//         if addr + size > self.main_memory.len() {
//             return Err(VMError::memory_error(&format!(
//                 "Memory access out of bounds at address 0x{:X}",
//                 addr
//             )));
//         }
//         Ok(())
//     }
// }
//
//
// /// -----------------------------------------------------------------------------
// /// ------------------------------ TESTS  ----------------------------------------
// /// -----------------------------------------------------------------------------
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_memory_basic_operations() {
//         let mut memory = MemoryController::new(1024, 256).unwrap();
//
//         memory.write(0, 0x1234_5678_9ABC_DEF0).unwrap();        // 0x1234_5678_9ABC_DEF0 à l'adresse 0
//         assert_eq!(memory.read(0).unwrap(), 0x1234_5678_9ABC_DEF0);
//
//         memory.write(8, 0xFEDC_BA98_7654_3210).unwrap();
//         assert_eq!(memory.read(8).unwrap(), 0xFEDC_BA98_7654_3210); // 0xFEDC_BA98_7654_3210 à l'adresse 8
//         assert_eq!(memory.read(0).unwrap(), 0x1234_5678_9ABC_DEF0);
//     }
//
//     #[test]
//     fn test_memory_bounds() {
//         let mut memory = MemoryController::new(16, 256).unwrap();
//         // on ne peut pas écrire à l'index 16 (out of bounds)
//         assert!(memory.write(16, 0x1234).is_err());
//         // idem en lecture
//         assert!(memory.read(16).is_err());
//     }
//
//     #[test]
//     fn test_memory_alignment() {
//         let mut memory = MemoryController::new(1024, 256).unwrap();
//
//         for addr in (0..32).step_by(8) {
//             memory.write(addr, addr as u64).unwrap();
//             assert_eq!(memory.read(addr).unwrap(), addr as u64);
//         }
//     }
//
//     /// test_memory_cache_coherence:
//     /// 1) On écrit dans memory: 0x1000 => 42
//     /// 2) On lit => 42
//     /// 3) On overwrite => 84 => lit => 84
//     /// 4) .clear() => plus rien
//     /// => read(0x1000) => Err
//     #[test]
//     fn test_memory_cache_coherence() {
//         let mut memory = Memory::new().unwrap();
//
//         // 1) write => 42
//         memory.write(0x1000, 42).unwrap();
//         assert_eq!(memory.read(0x1000).unwrap(), 42);
//
//         // 2) relit => 42
//         assert_eq!(memory.read(0x1000).unwrap(), 42);
//
//         // 3) overwrite => 84 => relit => 84
//         memory.write(0x1000, 84).unwrap();
//         assert_eq!(memory.read(0x1000).unwrap(), 84);
//
//         // 4) clear
//         memory.clear();
//
//         // => on veut un Err
//         assert!(
//             memory.read(0x1000).is_err(),
//             "Après clear(), read(0x1000) devrait être Err()"
//         );
//     }
//
//     /// test_cache_statistics:
//     /// 1) write(0, 42)
//     /// 2) read(0) -> miss
//     /// 3) read(0) -> hit
//     /// => stats.hits>0, total_accesses()>hits
//     #[test]
//     fn test_cache_statistics() {
//         let mut memory = MemoryController::with_default_size().unwrap();
//
//         // 1) On write => c'est un "write" => ça incrémente (write_hits ou write_misses)
//         //    Si la ligne n'est pas en cache => c'est un "miss" ?
//         //    Dans un WriteThrough NoWriteAllocate scenario, on n'ajoute pas la ligne,
//         //    ou si on a un "store miss" => ???
//
//         // 2) On read(0) => 1er read => devrais être un MISS => on rapatrie la valeur "0" ?
//         //    Eh, on a mis 42 => oh c'est main_memory => ???
//
//         memory.write(0, 42).unwrap();
//
//         // 2) 1er read => on s'attend à un "miss"
//         let _ = memory.read(0).unwrap();
//         // 3) 2eme read => "hit"
//
//         let _ = memory.read(0).unwrap();
//
//         let stats = memory.get_cache_stats().unwrap();
//
//         // On veut >=1 hit
//         assert!(
//             stats.hits > 0,
//             "On veut au moins 1 hit sur la 2eme lecture"
//         );
//
//         // total_accesses() > hits => il y a eu un miss
//         assert!(
//             stats.total_accesses() > stats.hits,
//             "Il doit y avoir au moins 1 miss => total_accesses()>hits"
//         );
//     }
// }
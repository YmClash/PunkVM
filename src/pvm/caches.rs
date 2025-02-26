// //src/pvm/caches.rs
//
// use crate::pvm::vm_errors::{VMError, VMResult,};
// use crate::pvm::cache_stats::CacheStatistics;
//
// use rand::Rng;
// use crate::pvm::cache_configs::{CacheConfig, ReplacementPolicy, WritePolicy};
// use crate::pvm::metrics::CacheMetrics;
//
// const CACHE_LINE_SIZE: usize = 64;
//
//
// #[derive(Debug, Clone, Copy)]
// pub enum CacheState {
//     Modified,
//     Exclusive,
//     Shared,
//     Invalid,
// }
//
// impl Default for CacheState {
//     fn default() -> Self {
//         CacheState::Invalid
//     }
// }
//
//
//
// #[derive(Debug, Clone)]
// pub struct CacheLine {
//     tag: u64,
//     data: Vec<u64>,
//     valid: bool,
//     dirty: bool,
//     last_access: u64,
//     state: CacheState,
// }
//
// impl Default for CacheLine {
//     fn default() -> Self {
//         Self {
//             tag: 0,
//             data: vec![0; CACHE_LINE_SIZE],
//             valid: false,
//             dirty: false,
//             last_access: 0,
//             state: CacheState::Invalid,
//         }
//     }
// }
//
// #[derive(Debug, Clone)]
// pub struct Cache {
//     pub config: CacheConfig,
//     pub lines: Vec<Vec<CacheLine>>,
//     pub access_count: u64,
//     pub statistics: CacheStatistics,
//     pub next_level: Option<Box<Cache>>,
// }
//
// impl Cache {
//     pub fn new(config: CacheConfig, next_level: Option<Box<Cache>>) -> Self {
//         let num_sets = config.size / (config.lines_size * config.associativity);
//         let mut lines = Vec::with_capacity(num_sets);
//
//         for _ in 0..num_sets {
//             let mut set = Vec::with_capacity(config.associativity);
//             for _ in 0..config.associativity {
//                 set.push(CacheLine::default());
//             }
//             lines.push(set);
//         }
//
//         Self {
//             config,
//             lines,
//             access_count: 0,
//             statistics: CacheStatistics::default(),
//             next_level,
//         }
//     }
//
//     pub fn reset(&mut self) -> VMResult<()> {
//         for set in &mut self.lines {
//             for line in set {
//                 *line = CacheLine::default();
//             }
//         }
//         self.statistics = CacheStatistics::default();
//         self.access_count = 0;
//         Ok(())
//     }
//     // pub fn reset(&mut self) -> VMResult<()> {
//     //     self.entries.clear();
//     //     self.statistics = CacheStatistics::default();
//     //     if let Some(next_level) = &mut self.next_level {
//     //         next_level.reset()?;
//     //     }
//     //     Ok(())
//     // }
//
//     /// Méthode d'écriture (write) avec la même séparation
//     // pub fn write(&mut self, addr: u64, value: u64) -> Result<(), VMError> {
//     //     let (set_index, tag, offset) = self.decode_address(addr);
//     //
//     //     // Vérifier si la ligne est présente (hit/miss)
//     //     let line_index = self.find_line_index(set_index, tag);
//     //
//     //     match line_index {
//     //         Some(i) => {
//     //             // HIT => incrémenter la stat
//     //             self.statistics.write_hits += 1;
//     //
//     //             let line = &mut self.lines[set_index][i];
//     //             match line.state {
//     //                 // Cas: Already Modified, Exclusive...
//     //                 CacheState::Modified => {
//     //                     line.data[offset] = value;
//     //                 },
//     //                 CacheState::Exclusive => {
//     //                     line.data[offset] = value;
//     //                     line.state = CacheState::Modified;
//     //                 },
//     //                 CacheState::Shared => {
//     //                     // Invalidation potentielle
//     //                     self.invalidate_other_copies(addr)?;
//     //                     line.data[offset] = value;
//     //                     line.state = CacheState::Modified;
//     //                 },
//     //                 CacheState::Invalid => {
//     //                     // Surprenant, mais s'il est valid && invalid -> incohérent, on force un miss
//     //                     return self.handle_write_miss(addr, value, set_index, tag, offset);
//     //                 }
//     //             }
//     //
//     //             // Write-through si besoin
//     //             if self.config.write_policy == WritePolicy::WriteThrough {
//     //                 if let Some(ref mut next) = self.next_level {
//     //                     next.write(addr, value)?;
//     //                 }
//     //             }
//     //         },
//     //         None => {
//     //             // MISS
//     //             self.statistics.write_misses += 1;
//     //             self.handle_write_miss(addr, value, set_index, tag, offset)?;
//     //         }
//     //     }
//     //
//     //     Ok(())
//     // }
//
//     pub fn write(&mut self, addr: u64, value: u64) -> Result<(), VMError> {
//         let (set_index, tag, offset) = self.decode_address(addr);
//
//         // Vérifier si la ligne est présente
//         if let Some(i) = self.find_line_index(set_index, tag) {
//             // HIT
//             self.statistics.write_hits += 1;
//
//             // Pour éviter l'emprunt mutable prolongé, on copie l'index
//             let line_index = i;
//
//             // Selon l'état, on fait un invalidation-other-copies ou non
//             {
//                 // On ne garde PAS la mutable ref ici longtemps
//                 let current_state = self.lines[set_index][line_index].state;
//
//                 match current_state {
//                     CacheState::Shared => {
//                         // On doit invalider avant de re-emprunter
//                         self.invalidate_other_copies(addr)?;
//                     }
//                     _ => {}
//                 }
//             }
//
//             // Maintenant on peut muter la ligne librement
//             let line = &mut self.lines[set_index][line_index];
//             match line.state {
//                 CacheState::Modified => {
//                     // Already dirty => on écrit direct
//                     line.data[offset] = value;
//                 },
//                 CacheState::Exclusive => {
//                     line.data[offset] = value;
//                     line.state = CacheState::Modified;
//                 },
//                 CacheState::Shared => {
//                     // On a déjà invalidé plus haut
//                     line.data[offset] = value;
//                     line.state = CacheState::Modified;
//                 },
//                 CacheState::Invalid => {
//                     // État incohérent => gérer comme un miss
//                     return self.handle_write_miss(addr, value, set_index, tag, offset);
//                 }
//             }
//
//             // Write-through => propager
//             if self.config.write_policy == WritePolicy::WriteThrough {
//                 if let Some(ref mut next) = self.next_level {
//                     next.write(addr, value)?;
//                 }
//             }
//
//         } else {
//             // MISS
//             self.statistics.write_misses += 1;
//             self.handle_write_miss(addr, value, set_index, tag, offset)?;
//         }
//
//         Ok(())
//     }
//
//
//     /// Méthode de lecture (read) qui évite le conflit mutable/immuable sur self.statistics
//     pub fn read(&mut self, addr: u64) -> Result<u64, VMError> {
//         let (set_index, tag, offset) = self.decode_address(addr);
//
//         // Chercher la ligne dans le set (index de la ligne s'il y a un hit)
//         let line_index = self.find_line_index(set_index, tag);
//
//         if let Some(i) = line_index {
//             // C'est un HIT => on peut d'abord incrémenter `self.statistics.hits`
//             self.statistics.hits += 1;
//
//             // Puis emprunter la ligne mutablement si besoin
//             self.access_count += 1;
//             let line = &mut self.lines[set_index][i];
//             line.last_access = self.access_count;  // Mise à jour LRU
//             let value = line.data[offset];
//
//             Ok(value)
//         } else {
//             // MISS
//             self.statistics.misses += 1;
//             self.handle_miss(addr, set_index, tag, offset)
//         }
//     }
//
//     pub fn invalidate_address(&mut self, addr: u64) -> Result<(), VMError> {
//         let (set_index, tag, _) = self.decode_address(addr);
//         if let Some(i) = self.find_line_index(set_index, tag) {
//             let line = &mut self.lines[set_index][i];
//             line.state = CacheState::Invalid;
//             line.valid = false;
//             self.statistics.invalidations += 1;
//         }
//         Ok(())
//     }
//
//
//     pub fn get_statistics(&self) -> &CacheStatistics {
//         &self.statistics
//     }
//
//     pub fn get_detailed_stats(&self) -> String {
//         format!(
//             "Cache Statistics:\n\
//              Hit Rate: {:.2}%\n\
//              Write Back Rate: {:.2}%\n\
//              Hits: {}\n\
//              Misses: {}\n\
//              Write Backs: {}\n\
//              Invalidations: {}\n\
//              Coherence Misses: {}\n\
//              Write Hits: {}\n\
//              Write Misses: {}\n\
//              Evictions: {}\n",
//             self.statistics.hit_rate() * 100.0,
//             self.statistics.write_back_rate() * 100.0,
//             self.statistics.hits,
//             self.statistics.misses,
//             self.statistics.write_backs,
//             self.statistics.invalidations,
//             self.statistics.coherence_misses,
//             self.statistics.write_hits,
//             self.statistics.write_misses,
//             self.statistics.evictions
//         )
//     }
//
//     /// Décoder l'adresse (comme avant)
//     fn decode_address(&self, addr: u64) -> (usize, u64, usize) {
//         let offset_bits = (self.config.lines_size as f64).log2() as u64;
//         let set_bits = ((self.config.size / (self.config.lines_size * self.config.associativity)) as f64).log2() as u64;
//
//         let offset = (addr & ((1 << offset_bits) - 1)) as usize;
//         let set_index = ((addr >> offset_bits) & ((1 << set_bits) - 1)) as usize;
//         let tag = addr >> (offset_bits + set_bits);
//
//         (set_index, tag, offset)
//     }
//
//     /// Cherche l'index (way) de la ligne correspondant à (tag) dans le set `set_index`
//     fn find_line_index(&self, set_index: usize, tag: u64) -> Option<usize> {
//         self.lines[set_index]
//             .iter()
//             .position(|line| line.valid && line.tag == tag)
//     }
//
//     fn find_line_mut(&mut self, set_index: usize, tag: u64) -> Option<&mut CacheLine> {
//         self.lines[set_index]
//             .iter_mut()
//             .find(|line| line.valid && line.tag == tag)
//     }
//
//     /// handle_miss comme avant, sans le risque de double emprunt
//     fn handle_miss(&mut self, addr: u64, set_index: usize, tag: u64, offset: usize) -> Result<u64, VMError> {
//         // Lire la donnée depuis le next level
//         let data = if let Some(ref mut next) = self.next_level {
//             next.read(addr)?
//         } else {
//             return Err(VMError::memory_error("Cache miss in last level"));
//         };
//
//         // Sélectionner un victim
//         let victim_way = self.select_victim(set_index)?;
//
//         // write-back si dirty
//         {
//             let line = &mut self.lines[set_index][victim_way];
//             if line.valid && line.dirty {
//                 self.write_back(set_index, victim_way)?;
//                 self.statistics.write_backs += 1;
//             }
//         }
//
//         {
//             let line = &mut self.lines[set_index][victim_way];
//             line.tag = tag;
//             line.valid = true;
//             line.dirty = false;
//             line.data[offset] = data;
//             line.state = CacheState::Exclusive;
//
//             // Mise à jour last_access
//             self.access_count += 1;
//             line.last_access = self.access_count;
//         }
//
//         Ok(data)
//     }
//
//     fn handle_write_miss(&mut self, addr: u64, value: u64, set_index: usize, tag: u64, offset: usize) -> Result<(), VMError> {
//         let victim_way = self.select_victim(set_index)?;
//
//         let need_writeback: bool;
//         let old_addr: u64;
//         let data_to_writeback: Vec<u64>;
//
//         {
//             let line = &self.lines[set_index][victim_way];
//             if line.valid && line.dirty {
//                 need_writeback = true;
//                 old_addr = self.reconstruct_address(set_index, line.tag);
//                 data_to_writeback = line.data.clone();
//             } else {
//                 need_writeback = false;
//                 old_addr = 0;
//                 data_to_writeback = vec![];
//             }
//         }
//
//         if need_writeback {
//             if let Some(ref mut next) = self.next_level {
//                 for (i, &val) in data_to_writeback.iter().enumerate() {
//                     next.write(old_addr + i as u64, val)?;
//                 }
//             }
//             self.statistics.write_backs += 1;
//         }
//
//         // Eviction si la ligne était déjà valide
//         {
//             let line = &mut self.lines[set_index][victim_way];
//             if line.valid {
//                 self.statistics.evictions += 1;
//             }
//
//             line.tag = tag;
//             line.valid = true;
//             line.dirty = self.config.write_policy == WritePolicy::WriteBack;
//             line.state = CacheState::Modified;
//             line.data[offset] = value;
//
//             // Mise à jour last_access
//             self.access_count += 1;
//             line.last_access = self.access_count;
//         }
//
//         // Si c'est un write-through, on propage
//         if self.config.write_policy == WritePolicy::WriteThrough {
//             if let Some(ref mut next) = self.next_level {
//                 next.write(addr, value)?;
//             }
//         }
//
//         Ok(())
//     }
//
//
//
//
//     fn select_victim(&self, set_index: usize) -> Result<usize, VMError> {
//         match self.config.replacement_policy {
//             ReplacementPolicy::LRU => {
//                 let mut min_access = u64::MAX;
//                 let mut victim = 0;
//                 for (i, line) in self.lines[set_index].iter().enumerate() {
//                     if !line.valid {
//                         return Ok(i);
//                     }
//                     if line.last_access < min_access {
//                         min_access = line.last_access;
//                         victim = i;
//                     }
//                 }
//                 Ok(victim)
//             }
//             ReplacementPolicy::FIFO => {
//                 // Exemple simplifié : on prend "access_count % associativity"
//                 Ok(self.access_count as usize % self.config.associativity)
//             }
//             ReplacementPolicy::Random => {
//                 Ok(rand::thread_rng().gen_range(0..self.config.associativity))
//             }
//         }
//     }
//
//     fn write_back(&mut self, set_index: usize, way: usize) -> Result<(), VMError> {
//         // Cloner les données nécessaires avant le borrow mutable
//         let (addr, values) = {
//             let line = &self.lines[set_index][way];
//             if !line.dirty {
//                 return Ok(());
//             }
//             let addr = self.reconstruct_address(set_index, line.tag);
//             let values = line.data.clone();
//             (addr, values)
//         };
//
//         // Maintenant on peut écrire sans problème de borrow
//         if let Some(ref mut next) = self.next_level {
//             for (offset, value) in values.iter().enumerate() {
//                 next.write(addr + offset as u64, *value)?;
//             }
//         }
//
//         // Marquer la ligne comme non-dirty
//         let line = &mut self.lines[set_index][way];
//         line.dirty = false;
//
//         Ok(())
//     }
//
//     fn reconstruct_address(&self, set_index: usize, tag: u64) -> u64 {
//         let offset_bits = (self.config.lines_size as f64).log2() as u64;
//         let set_bits = ((self.config.size / (self.config.lines_size * self.config.associativity)) as f64).log2() as u64;
//
//         (tag << (offset_bits + set_bits)) | ((set_index as u64) << offset_bits)
//     }
//
//
//     fn invalidate_other_copies(&mut self, addr: u64) -> Result<(), VMError> {
//         self.statistics.invalidations += 1;
//         if let Some(ref mut next) = self.next_level {
//             next.invalidate_address(addr)?;
//         }
//         Ok(())
//     }
//
//
//     fn update_access(&mut self, set_index: usize, tag: u64) {
//         self.access_count += 1;
//         let current_count = self.access_count;
//
//         if let Some(line) = self.find_line_mut(set_index, tag) {
//             line.last_access = current_count;
//         }
//     }
//
//     pub fn get_write_policy(&self) -> WritePolicy {
//         self.config.write_policy
//     }
//
//     pub fn get_replacement_policy(&self) -> ReplacementPolicy {
//         self.config.replacement_policy
//     }
//
//     pub fn get_associativity(&self) -> usize {
//         self.config.associativity
//     }
//
//     pub fn get_line_size(&self) -> usize {
//         self.config.lines_size
//     }
//
//     pub fn get_size(&self) -> usize {
//         self.config.size
//     }
//
//     // pub fn get_metrics(&self) -> CacheMetrics {
//     //     CacheMetrics {
//     //         total_accesses: self.statistics.hits + self.statistics.misses,
//     //         reads: self.statistics.hits,
//     //         writes: self.statistics.write_hits + self.statistics.write_misses,
//     //         cache_hits: self.statistics.hits + self.statistics.write_hits,
//     //         cache_misses: self.statistics.misses + self.statistics.write_misses,
//     //         average_access_time: if self.statistics.total_accesses() > 0 {
//     //             self.statistics.hits as f64 / self.statistics.total_accesses() as f64
//     //         } else {
//     //             0.0
//     //         },
//     //     }
//     // }
//
//     fn update_metrics(&mut self, hit: bool, write: bool) {
//         if write {
//             if hit {
//                 self.statistics.write_hits += 1;
//             } else {
//                 self.statistics.write_misses += 1;
//             }
//         } else {
//             if hit {
//                 self.statistics.hits += 1;
//             } else {
//                 self.statistics.misses += 1;
//             }
//         }
//     }
//
//     pub fn increment_misses(&mut self) {
//         self.statistics.misses += 1;
//     }
//
//
//
//
// }
//
// #[cfg(test)]
// mod tests {
//     use crate::pvm::cache_configs::WritePolicy;
//     use super::*;
//
//     fn create_test_cache() -> Cache {
//         let l2_config = CacheConfig {
//             size: 1024,
//             lines_size: 64,
//             associativity: 8,
//             write_policy: WritePolicy::WriteBack,
//             replacement_policy: ReplacementPolicy::LRU,
//         };
//
//         let l1_config = CacheConfig {
//             size: 256,
//             lines_size: 64,
//             associativity: 4,
//             write_policy: WritePolicy::WriteThrough,
//             replacement_policy: ReplacementPolicy::LRU,
//         };
//
//         let l2_cache = Box::new(Cache::new(l2_config, None));
//         Cache::new(l1_config, Some(l2_cache))
//     }
//
//     #[test]
//     fn test_cache_read_write() {
//         let mut cache = create_test_cache();
//
//         cache.write(0x1000, 42).unwrap();
//         assert_eq!(cache.read(0x1000).unwrap(), 42);
//
//         for i in 0..10 {
//             cache.write(0x2000 + i * 8, i as u64).unwrap();
//         }
//         for i in 0..10 {
//             assert_eq!(cache.read(0x2000 + i * 8).unwrap(), i as u64);
//         }
//     }
//
//     #[test]
//     fn test_cache_replacement() {
//         let mut cache = create_test_cache();
//
//         for i in 0..8 {
//             cache.write(i * 1024, i as u64).unwrap();
//         }
//
//         for i in 0..8 {
//             assert_eq!(cache.read(i * 1024).unwrap(), i as u64);
//         }
//
//         for i in 8..16 {
//             cache.write(i * 1024, i as u64).unwrap();
//         }
//     }
//
//     #[test]
//     fn test_cache_invalidation() {
//         let mut cache = create_test_cache();
//
//         cache.write(0x1000, 42).unwrap();
//         cache.invalidate_address(0x1000).unwrap();
//
//         // assert!(cache.read(0x1000).is_err());
//         assert_eq!(cache.read(0x1000).unwrap(), 42);
//     }
// }

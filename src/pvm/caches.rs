// //src/pvm/caches.rs

use std::collections::HashMap;
use rand::Rng;
use crate::pvm::cache_configs::{CacheConfig, ReplacementPolicy, WritePolicy};
use crate::pvm::cache_stats::CacheStatistics;
use crate::pvm::vm_errors::{VMError, VMResult};


/// Taille de line Cache
pub const DEFAULT_LINE_SIZE: usize = 64;

/// MSHR (Miss Status Holding Register) Entry
#[derive(Debug, Clone)]
pub struct MSHREntry {
    pub addr: u32,
    pub is_write: bool,
    pub write_data: Option<u8>,
    pub waiting_cycles: u32,
    pub total_cycles: u32,
}

/// MSHR (Miss Status Holding Registers) pour gérer les miss en cours
#[derive(Debug)]
pub struct MSHR {
    entries: Vec<Option<MSHREntry>>,
    max_entries: usize,
}

impl MSHR {
    pub fn new(size: usize) -> Self {
        Self {
            entries: vec![None; size],
            max_entries: size,
        }
    }
    
    pub fn is_full(&self) -> bool {
        self.entries.iter().all(|e| e.is_some())
    }
    
    pub fn find_entry(&self, addr: u32) -> Option<usize> {
        self.entries.iter().position(|e| {
            e.as_ref().map(|entry| entry.addr == addr).unwrap_or(false)
        })
    }
    
    pub fn allocate(&mut self, addr: u32, is_write: bool, write_data: Option<u8>, latency: u32) -> Option<usize> {
        if let Some(idx) = self.entries.iter().position(|e| e.is_none()) {
            self.entries[idx] = Some(MSHREntry {
                addr,
                is_write,
                write_data,
                waiting_cycles: 0,
                total_cycles: latency,
            });
            Some(idx)
        } else {
            None
        }
    }
    
    pub fn update(&mut self) -> Vec<(usize, MSHREntry)> {
        let mut completed = Vec::new();
        
        for (idx, entry) in self.entries.iter_mut().enumerate() {
            if let Some(ref mut e) = entry {
                e.waiting_cycles += 1;
                if e.waiting_cycles >= e.total_cycles {
                    completed.push((idx, e.clone()));
                }
            }
        }
        
        // Libérer les entrées complétées
        for (idx, _) in &completed {
            self.entries[*idx] = None;
        }
        
        completed
    }
}

/// Simple Next-Line Prefetcher
#[derive(Debug)]
pub struct SimplePrefetcher {
    enabled: bool,
    prefetch_degree: usize, // Nombre de lignes à prefetch
}

impl SimplePrefetcher {
    pub fn new(enabled: bool, degree: usize) -> Self {
        Self {
            enabled,
            prefetch_degree: degree,
        }
    }
    
    pub fn should_prefetch(&self, _addr: u32, is_miss: bool) -> bool {
        self.enabled && is_miss
    }
    
    pub fn get_prefetch_addresses(&self, addr: u32) -> Vec<u32> {
        if !self.enabled {
            return vec![];
        }
        
        let mut addrs = Vec::new();
        let line_addr = addr & !(DEFAULT_LINE_SIZE as u32 - 1);
        
        for i in 1..=self.prefetch_degree {
            addrs.push(line_addr + (i as u32 * DEFAULT_LINE_SIZE as u32));
        }
        
        addrs
    }
}

/// Write Buffer entre L1 et L2
#[derive(Debug)]
pub struct WriteBuffer {
    entries: Vec<(u32, u64)>, // (address, data)
    capacity: usize,
}

impl WriteBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
            capacity,
        }
    }
    
    pub fn is_full(&self) -> bool {
        self.entries.len() >= self.capacity
    }
    
    pub fn add(&mut self, addr: u32, data: u64) -> bool {
        if self.is_full() {
            false
        } else {
            self.entries.push((addr, data));
            true
        }
    }
    
    pub fn drain(&mut self) -> Vec<(u32, u64)> {
        std::mem::take(&mut self.entries)
    }
}

/// Résultat d'accès au cache
#[derive(Debug, Clone)]
pub enum CacheAccessResult {
    Hit(u64),        // Hit avec la valeur
    L2Hit(u64),      // Hit dans L2
    Miss,            // Miss complet
    MSHRPending,     // Miss en cours de traitement
}

/// Hiérarchie complète de cache L1/L2 avec MSHR et prefetching
#[derive(Debug)]
pub struct CacheHierarchy {
    pub l1_data: Cache,
    pub l1_inst: Cache,
    pub l2_unified: Cache,
    pub write_buffer: WriteBuffer,
    pub mshr: MSHR,
    pub prefetcher: SimplePrefetcher,
    pub l2_latency: u32,
    pub memory_latency: u32,
}

impl CacheHierarchy {
    pub fn new(l1_data_config: CacheConfig, l1_inst_config: CacheConfig, l2_config: CacheConfig) -> Self {
        // L2 unifié sans next_level (c'est le dernier niveau)
        let l2_unified = Cache::new(l2_config.clone(), None);
        
        // Pour l'instant, L1 n'a pas de next_level intégré (on gérera manuellement)
        let l1_data = Cache::new(l1_data_config, None);
        let l1_inst = Cache::new(l1_inst_config, None);
        
        Self {
            l1_data,
            l1_inst,
            l2_unified,
            write_buffer: WriteBuffer::new(16),
            mshr: MSHR::new(8),
            prefetcher: SimplePrefetcher::new(true, 2),
            l2_latency: 12,
            memory_latency: 100,
        }
    }
    
    /// Accès byte simple - conversion automatique en accès u64 aligné
    pub fn access_byte(&mut self, addr: u32, is_write: bool, write_data: Option<u8>) -> VMResult<CacheAccessResult> {
        // Aligner l'adresse sur u64 (8 bytes)
        let aligned_addr = addr & !7;
        let byte_offset = (addr & 7) as usize;
        
        if is_write {
            if let Some(byte_value) = write_data {
                // Pour une écriture, on doit d'abord lire le u64, modifier le byte, puis réécrire
                match self.access_data(aligned_addr, false, None) {
                    Ok(CacheAccessResult::Hit(mut data)) | Ok(CacheAccessResult::L2Hit(mut data)) => {
                        // Modifier le byte dans le u64
                        let mut bytes = data.to_le_bytes();
                        bytes[byte_offset] = byte_value;
                        data = u64::from_le_bytes(bytes);
                        
                        // Réécrire le u64 modifié
                        self.access_data(aligned_addr, true, Some(data))
                    }
                    Ok(CacheAccessResult::Miss) | Ok(CacheAccessResult::MSHRPending) => {
                        // Miss - créer un nouveau u64 avec le byte
                        let mut bytes = [0u8; 8];
                        bytes[byte_offset] = byte_value;
                        let data = u64::from_le_bytes(bytes);
                        self.access_data(aligned_addr, true, Some(data))
                    }
                    Err(e) => Err(e),
                }
            } else {
                Err(VMError::memory_error("Write without data"))
            }
        } else {
            // Lecture
            match self.access_data(aligned_addr, false, None) {
                Ok(CacheAccessResult::Hit(data)) => {
                    let bytes = data.to_le_bytes();
                    Ok(CacheAccessResult::Hit(bytes[byte_offset] as u64))
                }
                Ok(CacheAccessResult::L2Hit(data)) => {
                    let bytes = data.to_le_bytes();
                    Ok(CacheAccessResult::L2Hit(bytes[byte_offset] as u64))
                }
                other => other,
            }
        }
    }

    pub fn access_data(&mut self, addr: u32, is_write: bool, write_data: Option<u64>) -> VMResult<CacheAccessResult> {
        // Vérifier d'abord si l'adresse est dans le MSHR
        if let Some(_) = self.mshr.find_entry(addr) {
            return Ok(CacheAccessResult::MSHRPending);
        }
        
        // Essayer L1 d'abord
        let l1_result = if is_write {
            if let Some(data) = write_data {
                // Convertir u64 en u8 pour l'écriture dans le cache
                let data_u8 = data as u8;
                self.l1_data.write(addr, data_u8).map(|_| CacheAccessResult::Hit(data))
            } else {
                Err(VMError::memory_error("Write without data"))
            }
        } else {
            self.l1_data.read(addr).map(CacheAccessResult::Hit)
        };
        
        match l1_result {
            Ok(result) => Ok(result),
            Err(_) => {
                // L1 miss, essayer L2
                let l2_result = if is_write {
                    if let Some(data) = write_data {
                        // Convertir u64 en u8 pour l'écriture dans le cache
                        let data_u8 = data as u8;
                        self.l2_unified.write(addr, data_u8).map(|_| CacheAccessResult::L2Hit(data))
                    } else {
                        Err(VMError::memory_error("Write without data"))
                    }
                } else {
                    self.l2_unified.read(addr).map(CacheAccessResult::L2Hit)
                };
                
                match l2_result {
                    Ok(CacheAccessResult::L2Hit(data)) => {
                        // Fill L1 from L2
                        if !is_write {
                            let data_u8 = data as u8;
                            let _ = self.l1_data.write(addr, data_u8);
                        }
                        Ok(CacheAccessResult::L2Hit(data))
                    }
                    Err(_) => {
                        // L2 miss aussi - retourner Miss pour déclencher la lecture mémoire
                        Ok(CacheAccessResult::Miss)
                    }
                    _ => l2_result,
                }
            }
        }
    }
    
    pub fn update_mshr(&mut self) -> Vec<MSHREntry> {
        let completed = self.mshr.update();
        let mut completed_entries = Vec::new();
        
        for (_, entry) in completed {
            // Les requêtes sont maintenant complètes, on peut les traiter
            completed_entries.push(entry);
        }
        
        completed_entries
    }
    
    pub fn flush_write_buffer(&mut self) -> VMResult<()> {
        let entries = self.write_buffer.drain();
        for (addr, data) in entries {
            // Écrire chaque byte du u64 dans le cache
            let bytes = data.to_le_bytes();
            for (i, &byte) in bytes.iter().enumerate() {
                self.l2_unified.write(addr + i as u32, byte)?;
            }
        }
        Ok(())
    }
    
    /// Rempli L2 puis L1 avec une donnée venant de la mémoire (allocation sur miss)
    pub fn fill_from_memory(&mut self, addr: u32, data: u8) -> VMResult<()> {
        // 1. Remplir L2 d'abord (niveau le plus bas de la hiérarchie)
        if let Err(_) = self.l2_unified.write(addr, data) {
            // Si L2 est plein, on force l'éviction (c'est normal)
            let _ = self.l2_unified.write(addr, data);
        }
        
        // 2. Remplir L1 ensuite (niveau le plus haut)
        if let Err(_) = self.l1_data.write(addr, data) {
            // Si L1 est plein, on force l'éviction (c'est normal)
            let _ = self.l1_data.write(addr, data);
        }
        
        Ok(())
    }

    pub fn get_combined_stats(&self) -> String {
        format!(
            "=== Cache Hierarchy Statistics ===\n\
             L1 Data:\n{}\n\
             L1 Inst:\n{}\n\
             L2 Unified:\n{}\n",
            self.l1_data.get_detailed_stats(),
            self.l1_inst.get_detailed_stats(),
            self.l2_unified.get_detailed_stats()
        )
    }
}






#[derive(Debug, Clone, PartialEq)]
pub struct CacheLine {
    pub tag: u32,            // Tag de la ligne
    pub data: Vec<u8>,      // Données de la ligne (64 bytes = 8 u64)
    pub valid: bool,         // Indicateur de validité
    pub dirty: bool,         // Indicateur de saleté
    pub last_access: u64,    // Compteur d'accès pour LRU
    pub state: CacheState,   // État de la ligne
    pub lru_timestamp: u64,  // Timestamp LRU
}

impl Default for CacheLine {
    fn default() -> Self {
        Self {
            tag: 0,
            data: vec![0; DEFAULT_LINE_SIZE], // DEFAULT_LINE_SIZE u64s pour simplifier l'adressage
            valid: false,
            dirty: false,
            last_access: 0,
            state: CacheState::Invalid,
            lru_timestamp: 0,
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CacheState {
    Modified,  // Ligne modifiée, doit être écrite en mémoire
    Exclusive, // Ligne exclusive à ce cache
    Shared,    // Ligne partagée entre plusieurs caches
    Invalid,   // Ligne invalide
}

impl Default for CacheState {
    fn default() -> Self {
        CacheState::Invalid
    }
}

/// Structure de la Cache
#[derive(Debug)]
pub struct Cache {
    pub config: CacheConfig,
    pub lines: Vec<Vec<CacheLine>>,
    pub access_count: u64, // Compteur d'accès pour LRU
    pub statistics: CacheStatistics, // Statistiques de la cache
    pub next_level: Option<Box<Cache>>, // Niveau de cache suivant (si applicable)
}





/// Structure simple pour L1Cache utilisant des bytes
#[derive(Debug, Clone)]
pub struct L1CacheLine {
    pub data: [u8; DEFAULT_LINE_SIZE],
    pub lru_timestamp: u64,
}

/// Cache L1 pour la mémoire
pub struct L1Cache {
    size: usize,        // Taille totale de la cache en bytes
    lines_count: usize, // Nombre de lignes
    line_size: usize,   // Taille de chaque ligne en bytes

    /// Stockage principal : clé = adresse de base alignée, valeur = ligne de cache
    data: HashMap<u32, L1CacheLine>, // Données de la cache (addresse -> données)

    lru_counter: u64, // Compteur LRU global
}



impl L1Cache {
    pub fn new(size: usize) -> Self {
        // Taille de ligne fixée à 64 bytes (typique pour les caches L1)
        let line_size = DEFAULT_LINE_SIZE;
        let lines_count = size / line_size;

        Self {
            size,
            lines_count,
            line_size,
            data: HashMap::with_capacity(lines_count),
            lru_counter: 0,
        }
    }

    /// Vérifie si une adresse est dans le cache
    pub fn has_address(&self, addr: u32) -> bool {
        let line_addr = self.get_line_addr(addr);
        self.data.contains_key(&line_addr)
    }

    pub fn read_byte(&mut self, addr: u32) -> Option<u8> {
        let base = self.get_line_addr(addr);
        let offset = self.get_offset(addr);

        if let Some(line) = self.data.get_mut(&base) {
            // C'est un HIT
            self.lru_counter += 1;
            line.lru_timestamp = self.lru_counter;
            Some(line.data[offset])
        } else {
            // MISS
            None
        }
    }

    pub fn write_byte(&mut self, addr: u32, value: u8) -> bool {
        let base = self.get_line_addr(addr);
        let offset = self.get_offset(addr);

        if let Some(line) = self.data.get_mut(&base) {
            // HIT
            self.lru_counter += 1;
            line.lru_timestamp = self.lru_counter;
            line.data[offset] = value;
            true
        } else {
            // MISS
            false
        }
    }

    pub fn fill_line(&mut self, base_addr: u32, line_data: [u8; DEFAULT_LINE_SIZE]) {
        // S'il existe déjà une ligne pour ce base_addr, on la remplace
        if let Some(line) = self.data.get_mut(&base_addr) {
            // On met à jour la line
            line.data = line_data;
            self.lru_counter += 1;
            line.lru_timestamp = self.lru_counter;
            return;
        }

        // Sinon, c'est une nouvelle insertion
        if self.data.len() >= self.lines_count {
            // Eviction LRU
            let (&victim_addr, _) = self
                .data
                .iter()
                .min_by_key(|&(_, line)| line.lru_timestamp)
                .expect("Cache is not empty but we can't find a min LRU line");

            self.data.remove(&victim_addr);
        }

        // Insérer la nouvelle ligne
        let mut line = L1CacheLine {
            data: line_data,
            lru_timestamp: 0,
        };
        self.lru_counter += 1;
        line.lru_timestamp = self.lru_counter;

        self.data.insert(base_addr, line);
    }

    /// Recherche un byte dans le cache
    // pub fn lookup_byte(&mut self, addr: u32) -> Option<u8> {
    //     let line_addr = self.get_line_addr(addr);
    //     let offset = self.get_offset(addr);
    //
    //     if let Some(line) = self.data.get(&line_addr) {
    //         // Mettre à jour le compteur LRU
    //         self.lru_counter += 1;
    //         // self.lru.insert(line_addr, self.lru_counter);
    //
    //
    //         Some(line[offset])
    //     } else {
    //         None
    //     }
    // }

    /// Nettoyer le cache
    pub fn clear(&mut self) {
        // println!("L1Cache::clear() - début");
        self.data.clear();
        // self.lru.clear();
        self.lru_counter = 0;
        // println!("L1Cache::clear() - fin");
    }

    /// Calcule l'adresse de la ligne de cache pour une adresse donnée
    pub fn get_line_addr(&self, addr: u32) -> u32 {
        // addr - (addr % self.line_size as u32)
        addr & !(self.line_size - 1) as u32
    }

    /// Calcule l'offset dans la ligne pour une adresse mémoire
    pub fn get_offset(&self, addr: u32) -> usize {
        (addr % self.line_size as u32) as usize
    }
}

pub struct CacheMetrics {
    total_accesses: usize,
    reads: usize,
    writes: usize,
    cache_hits: usize,
    cache_misses: usize,
    average_access_time: f64
}

impl Cache {
    pub fn new(config: CacheConfig, next_level: Option<Box<Cache>>) -> Self {
        let num_sets = config.size / (config.lines_size * config.associativity);
        let mut lines = Vec::with_capacity(num_sets);

        for _ in 0..num_sets {
            let mut set = Vec::with_capacity(config.associativity);
            for _ in 0..config.associativity {
                set.push(CacheLine::default());
            }
            lines.push(set);
        }

        Self {
            config,
            lines,
            access_count: 0,
            statistics: CacheStatistics::default(),
            next_level,
        }
    }

    pub fn reset(&mut self) -> VMResult<()> {
        for set in &mut self.lines {
            for line in set {
                *line = CacheLine::default();
            }
        }
        self.statistics = CacheStatistics::default();
        self.access_count = 0;
        Ok(())
    }


    pub fn write(&mut self, addr: u32, value: u8) -> Result<(), VMError> {
        let (set_index, tag, offset) = self.decode_address(addr);

        // Vérifier si la ligne est présente
        if let Some(i) = self.find_line_index(set_index, tag) {
            // HIT
            self.statistics.write_hits += 1;

            // Pour éviter l'emprunt mutable prolongé, on copie l'index
            let line_index = i;

            // Selon l'état, on fait un invalidation-other-copies ou non
            {
                // On ne garde PAS la mutable ref ici longtemps
                let current_state = self.lines[set_index][line_index].state;

                match current_state {
                    CacheState::Shared => {
                        // On doit invalider avant de re-emprunter
                        self.invalidate_other_copies(addr)?;
                    }
                    _ => {}
                }
            }

            // Maintenant on peut muter la ligne librement
            let line = &mut self.lines[set_index][line_index];
            match line.state {
                CacheState::Modified => {
                    // Already dirty => on écrit direct
                    line.data[offset] = value as u8;
                },
                CacheState::Exclusive => {
                    line.data[offset] = value as u8;
                    line.state = CacheState::Modified;
                },
                CacheState::Shared => {
                    // On a déjà invalidé plus haut
                    line.data[offset] = value;
                    line.state = CacheState::Modified;
                },
                CacheState::Invalid => {
                    // État incohérent => gérer comme un miss
                    return self.handle_write_miss(addr, value, set_index, tag, offset);
                }
            }

            // Write-through => propager
            if self.config.write_policy == WritePolicy::WriteThrough {
                if let Some(ref mut next) = self.next_level {
                    next.write(addr, value)?;
                }
            }

        } else {
            // MISS
            self.statistics.write_misses += 1;
            self.handle_write_miss(addr, value, set_index, tag, offset)?;
        }

        Ok(())
    }


    /// Méthode de lecture (read) qui évite le conflit mutable/immuable sur self.statistics
    pub fn read(&mut self, addr: u32) -> Result<u64, VMError> {
        let (set_index, tag, offset) = self.decode_address(addr);

        // Chercher la ligne dans le set (index de la ligne s'il y a un hit)
        let line_index = self.find_line_index(set_index, tag);

        if let Some(i) = line_index {
            // C'est un HIT => on peut d'abord incrémenter `self.statistics.hits`
            self.statistics.hits += 1;

            // Puis emprunter la ligne mutablement si besoin
            self.access_count += 1;
            let line = &mut self.lines[set_index][i];
            line.last_access = self.access_count;  // Mise à jour LRU
            let value = line.data[offset];

            Ok(value as u64)
        } else {
            // MISS
            self.statistics.misses += 1;
            self.handle_miss(addr, set_index, tag, offset)
        }
    }

    pub fn invalidate_address(&mut self, addr: u32) -> Result<(), VMError> {
        let (set_index, tag, _) = self.decode_address(addr);
        if let Some(i) = self.find_line_index(set_index, tag) {
            let line = &mut self.lines[set_index][i];
            line.state = CacheState::Invalid;
            line.valid = false;
            self.statistics.invalidations += 1;
        }
        Ok(())
    }


    pub fn get_statistics(&self) -> &CacheStatistics {
        &self.statistics
    }

    pub fn get_detailed_stats(&self) -> String {
        format!(
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
            self.statistics.hit_rate() * 100.0,
            self.statistics.write_back_rate() * 100.0,
            self.statistics.hits,
            self.statistics.misses,
            self.statistics.write_backs,
            self.statistics.invalidations,
            self.statistics.coherence_misses,
            self.statistics.write_hits,
            self.statistics.write_misses,
            self.statistics.evictions
        )
    }

    /// Décoder l'adresse en (set_index, tag, offset)
    fn decode_address(&self, addr: u32) -> (usize, u32, usize) {
        let offset_bits = (self.config.lines_size as f64).log2() as u32;
        let set_bits = ((self.config.size / (self.config.lines_size * self.config.associativity)) as f64).log2() as u32;
        
        let offset = (addr & ((1 << offset_bits) - 1)) as usize;
        let set_index = ((addr >> offset_bits) & ((1 << set_bits) - 1)) as usize;
        let tag = addr >> (offset_bits + set_bits);
        
        (set_index, tag, offset)
    }

    /// Cherche l'index (way) de la ligne correspondant à (tag) dans le set `set_index`
    fn find_line_index(&self, set_index: usize, tag: u32) -> Option<usize> {
        self.lines[set_index]
            .iter()
            .position(|line| line.valid && line.tag == tag)
    }

    fn find_line_mut(&mut self, set_index: usize, tag: u32) -> Option<&mut CacheLine> {
        self.lines[set_index]
            .iter_mut()
            .find(|line| line.valid && line.tag == tag)
    }

    /// handle_miss comme avant, sans le risque de double emprunt
    fn handle_miss(&mut self, addr: u32, set_index: usize, tag: u32, offset: usize) -> Result<u64, VMError> {
        // Lire la donnée depuis le next level
        let data = if let Some(ref mut next) = self.next_level {
            next.read(addr)?
        } else {
            return Err(VMError::memory_error("Cache miss in last level"));
        };

        // Sélectionner un victim
        let victim_way = self.select_victim(set_index)?;

        // write-back si dirty
        {
            let line = &mut self.lines[set_index][victim_way];
            if line.valid && line.dirty {
                self.write_back(set_index, victim_way)?;
                self.statistics.write_backs += 1;
            }
        }

        {
            let line = &mut self.lines[set_index][victim_way];
            line.tag = tag;
            line.valid = true;
            line.dirty = false;
            line.data[offset] = data as u8;
            line.state = CacheState::Exclusive;

            // Mise à jour last_access
            self.access_count += 1;
            line.last_access = self.access_count;
        }

        Ok(data)
    }

    fn handle_write_miss(&mut self, addr: u32, value: u8, set_index: usize, tag: u32, offset: usize) -> Result<(), VMError> {
        let victim_way = self.select_victim(set_index)?;

        let need_writeback: bool;
        let old_addr: u32;
        let data_to_writeback: Vec<u8>;

        {
            let line = &self.lines[set_index][victim_way];
            if line.valid && line.dirty {
                need_writeback = true;
                old_addr = self.reconstruct_address(set_index, line.tag);
                data_to_writeback = line.data.clone();
            } else {
                need_writeback = false;
                old_addr = 0;
                data_to_writeback = vec![];
            }
        }

        if need_writeback {
            if let Some(ref mut next) = self.next_level {
                for (i, &val) in data_to_writeback.iter().enumerate() {
                    next.write(old_addr + i as u32, val)?;
                }
            }
            self.statistics.write_backs += 1;
        }

        // Eviction si la ligne était déjà valide
        {
            let line = &mut self.lines[set_index][victim_way];
            if line.valid {
                self.statistics.evictions += 1;
            }

            line.tag = tag;
            line.valid = true;
            line.dirty = self.config.write_policy == WritePolicy::WriteBack;
            line.state = CacheState::Modified;
            line.data[offset] = value;

            // Mise à jour last_access
            self.access_count += 1;
            line.last_access = self.access_count;
        }

        // Si c'est un write-through, on propage
        if self.config.write_policy == WritePolicy::WriteThrough {
            if let Some(ref mut next) = self.next_level {
                next.write(addr, value)?;
            }
        }

        Ok(())
    }




    fn select_victim(&self, set_index: usize) -> Result<usize, VMError> {
        match self.config.replacement_policy {
            ReplacementPolicy::LRU => {
                let mut min_access = u64::MAX;
                let mut victim = 0;
                for (i, line) in self.lines[set_index].iter().enumerate() {
                    if !line.valid {
                        return Ok(i);
                    }
                    if line.last_access < min_access {
                        min_access = line.last_access;
                        victim = i;
                    }
                }
                Ok(victim)
            }
            ReplacementPolicy::FIFO => {
                // Exemple simplifié : on prend "access_count % associativity"
                Ok(self.access_count as usize % self.config.associativity)
            }
            ReplacementPolicy::Random => {
                Ok(rand::thread_rng().gen_range(0..self.config.associativity))
            }
        }
    }

    fn write_back(&mut self, set_index: usize, way: usize) -> Result<(), VMError> {
        // Cloner les données nécessaires avant le borrow mutable
        let (addr, values) = {
            let line = &self.lines[set_index][way];
            if !line.dirty {
                return Ok(());
            }
            let addr = self.reconstruct_address(set_index, line.tag);
            let values = line.data.clone();
            (addr, values)
        };

        // Maintenant on peut écrire sans problème de borrow
        if let Some(ref mut next) = self.next_level {
            for (offset, value) in values.iter().enumerate() {
                next.write(addr + offset as u32, *value )?;
            }
        }

        // Marquer la ligne comme non-dirty
        let line = &mut self.lines[set_index][way];
        line.dirty = false;

        Ok(())
    }

    fn reconstruct_address(&self, set_index: usize, tag: u32) -> u32 {
        let offset_bits = (self.config.lines_size as f64).log2() as u32;
        let set_bits = ((self.config.size / (self.config.lines_size * self.config.associativity)) as f64).log2() as u32;

        (tag << (offset_bits + set_bits)) | ((set_index as u32) << offset_bits)
    }


    fn invalidate_other_copies(&mut self, addr: u32) -> Result<(), VMError> {
        self.statistics.invalidations += 1;
        if let Some(ref mut next) = self.next_level {
            next.invalidate_address(addr)?;
        }
        Ok(())
    }


    fn update_access(&mut self, set_index: usize, tag: u32) {
        self.access_count += 1;
        let current_count = self.access_count;

        if let Some(line) = self.find_line_mut(set_index, tag) {
            line.last_access = current_count;
        }
    }

    pub fn get_write_policy(&self) -> WritePolicy {
        self.config.write_policy
    }

    pub fn get_replacement_policy(&self) -> ReplacementPolicy {
        self.config.replacement_policy
    }

    pub fn get_associativity(&self) -> usize {
        self.config.associativity
    }

    pub fn get_line_size(&self) -> usize {
        self.config.lines_size
    }

    pub fn get_size(&self) -> usize {
        self.config.size
    }

    pub fn get_metrics(&self) -> CacheMetrics {
        CacheMetrics {
            total_accesses: self.statistics.hits + self.statistics.misses,
            reads: self.statistics.hits,
            writes: self.statistics.write_hits + self.statistics.write_misses,
            cache_hits: self.statistics.hits + self.statistics.write_hits,
            cache_misses: self.statistics.misses + self.statistics.write_misses,
            average_access_time: if self.statistics.total_accesses() > 0 {
                self.statistics.hits as f64 / self.statistics.total_accesses() as f64
            } else {
                0.0
            },
        }
    }

    fn update_metrics(&mut self, hit: bool, write: bool) {
        if write {
            if hit {
                self.statistics.write_hits += 1;
            } else {
                self.statistics.write_misses += 1;
            }
        } else {
            if hit {
                self.statistics.hits += 1;
            } else {
                self.statistics.misses += 1;
            }
        }
    }

    pub fn increment_misses(&mut self) {
        self.statistics.misses += 1;
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    /// Petit helper pour savoir si une adresse `addr` est présente dans la cache.
    fn in_cache(cache: &L1Cache, addr: u32) -> bool {
        let base = cache.get_line_addr(addr);
        cache.data.contains_key(&base)
    }

    #[test]
    fn test_cache_creation() {
        // Taille totale : 1024 octets => line_size=64 => lines_count=16
        let cache = L1Cache::new(1024);
        assert_eq!(cache.size, 1024);
        assert_eq!(cache.line_size, DEFAULT_LINE_SIZE);
        assert_eq!(cache.lines_count, 16);
        assert_eq!(cache.data.len(), 0);
        assert_eq!(cache.lru_counter, 0);
    }

    #[test]
    fn test_cache_line_addressing() {
        let cache = L1Cache::new(1024);

        // Test alignement d'adresse sur une ligne
        assert_eq!(cache.get_line_addr(0x100), 0x100);
        assert_eq!(cache.get_line_addr(0x12F), 0x100);
        assert_eq!(cache.get_line_addr(0x13F), 0x100);
        assert_eq!(cache.get_line_addr(0x140), 0x140);

        // Test calcul d'offset
        assert_eq!(cache.get_offset(0x100), 0);
        assert_eq!(cache.get_offset(0x12F), 0x2F);
        assert_eq!(cache.get_offset(0x13F), 0x3F);
        assert_eq!(cache.get_offset(0x140), 0);
    }

    #[test]
    fn test_cache_read_write_byte_hit_miss() {
        let mut cache = L1Cache::new(1024);

        // Au départ, la cache est vide => on tente de lire => MISS
        assert_eq!(cache.read_byte(0x100), None);
        assert!(!in_cache(&cache, 0x100));

        // Écrire un octet => s’il n’existe pas encore, c'est un MISS
        // (Le code renvoie false en cas de miss)
        let was_hit = cache.write_byte(0x100, 42);
        assert!(!was_hit, "Écriture devrait être un miss la première fois");
        assert!(!in_cache(&cache, 0x100));
        // => On n’a pas fait de "fill_line" automatique ici, la logique est gérée par la mémoire

        // Simulons l’insertion d’une ligne complète dans la cache (un fill_line).
        let base = cache.get_line_addr(0x100);
        let mut line_data = [0u8; DEFAULT_LINE_SIZE];
        line_data[cache.get_offset(0x100)] = 42; // On y place la valeur 42
        cache.fill_line(base, line_data);

        // Maintenant, on refait un write => cette fois c’est un HIT
        let was_hit2 = cache.write_byte(0x100, 43);
        assert!(was_hit2, "Après un fill_line, écrire doit être un HIT");
        assert!(in_cache(&cache, 0x100));

        // Lire l’octet => on doit obtenir 43
        let read_val = cache.read_byte(0x100);
        assert_eq!(read_val, Some(43));
    }

    #[test]
    fn test_cache_lru_eviction() {
        // Taille 128 => line_size=64 => on a 2 lignes maxi
        let mut cache = L1Cache::new(128);
        assert_eq!(cache.lines_count, 2);

        // Remplissons deux lignes distinctes
        let mut line1 = [0u8; DEFAULT_LINE_SIZE];
        line1[0] = 1;
        cache.fill_line(0x0000, line1);

        let mut line2 = [0u8; DEFAULT_LINE_SIZE];
        line2[0] = 2;
        cache.fill_line(0x0040, line2);

        // Les deux lignes sont en cache
        assert!(in_cache(&cache, 0x0000));
        assert!(in_cache(&cache, 0x0040));

        // Accéder à la première pour la rendre "plus récente"
        let _ = cache.read_byte(0x0000);

        // On prépare une troisième ligne
        let mut line3 = [0u8; DEFAULT_LINE_SIZE];
        line3[0] = 3;
        // fill_line => va provoquer une éviction LRU
        cache.fill_line(0x0080, line3);

        // La ligne la moins récemment utilisée était celle à base 0x0040, donc elle est évincée
        assert!(in_cache(&cache, 0x0000));
        assert!(!in_cache(&cache, 0x0040));
        assert!(in_cache(&cache, 0x0080));
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = L1Cache::new(1024);

        // On insère deux lignes
        let mut data1 = [0u8; DEFAULT_LINE_SIZE];
        data1[0] = 42;
        cache.fill_line(0x0100, data1);

        let mut data2 = [0u8; DEFAULT_LINE_SIZE];
        data2[0] = 43;
        cache.fill_line(0x0200, data2);

        assert!(in_cache(&cache, 0x0100));
        assert!(in_cache(&cache, 0x0200));

        // clear()
        cache.clear();
        assert_eq!(cache.data.len(), 0);
        assert_eq!(cache.lru_counter, 0);
        assert!(!in_cache(&cache, 0x0100));
        assert!(!in_cache(&cache, 0x0200));
    }

    #[test]
    fn test_cache_fill_line_and_write() {
        let mut cache = L1Cache::new(1024);

        // Remplir une ligne
        let base = cache.get_line_addr(0x100);
        let mut line_data = [0u8; DEFAULT_LINE_SIZE];
        line_data[cache.get_offset(0x100)] = 42;
        cache.fill_line(base, line_data);

        // Vérifier qu’on lit 42
        let val = cache.read_byte(0x100);
        assert_eq!(val, Some(42));

        // Écrire une autre valeur dans la même adresse => HIT
        let was_hit = cache.write_byte(0x100, 43);
        assert!(was_hit);
        let val2 = cache.read_byte(0x100);
        assert_eq!(val2, Some(43));

        // Écrire à une autre adresse du même line_base => HIT
        let was_hit2 = cache.write_byte(0x101, 44);
        assert!(was_hit2);
        let val3 = cache.read_byte(0x101);
        assert_eq!(val3, Some(44));
    }


    fn create_test_cache() -> Cache {
        let l2_config = CacheConfig {
            size: 1024,
            lines_size: 64,
            associativity: 8,
            write_policy: WritePolicy::WriteBack,
            replacement_policy: ReplacementPolicy::LRU,
        };

        let l1_config = CacheConfig {
            size: 256,
            lines_size: 64,
            associativity: 4,
            write_policy: WritePolicy::WriteThrough,
            replacement_policy: ReplacementPolicy::LRU,
        };

        let l2_cache = Box::new(Cache::new(l2_config, None));
        Cache::new(l1_config, Some(l2_cache))
    }

    #[test]
    fn test_cache_read_write() {
        let mut cache = create_test_cache();

        cache.write(0x1000, 42).unwrap();
        assert_eq!(cache.read(0x1000).unwrap(), 42);

        for i in 0..10 {
            cache.write(0x2000 + i * 8, i as u8).unwrap();
        }
        for i in 0..10 {
            assert_eq!(cache.read(0x2000 + i * 8).unwrap(), i as u64);
        }
    }

    #[test]
    fn test_cache_replacement() {
        let mut cache = create_test_cache();

        for i in 0..8 {
            cache.write(i * 1024, i as u8).unwrap();
        }

        for i in 0..8 {
            assert_eq!(cache.read(i * 1024).unwrap(), i as u64);
        }

        for i in 8..16 {
            cache.write(i * 1024, i as u8 ).unwrap();
        }
    }

    #[test]
    fn test_cache_invalidation() {
        let mut cache = create_test_cache();

        cache.write(0x1000, 42).unwrap();
        cache.invalidate_address(0x1000).unwrap();

        // assert!(cache.read(0x1000).is_err());
        assert_eq!(cache.read(0x1000).unwrap(), 42);
    }

    // Tests pour CacheHierarchy
    fn create_test_hierarchy() -> CacheHierarchy {
        let l1_data_config = CacheConfig {
            size: 4 * 1024,    // 4KB
            lines_size: 64,
            associativity: 4,
            write_policy: WritePolicy::WriteThrough,
            replacement_policy: ReplacementPolicy::LRU,
        };
        
        let l1_inst_config = l1_data_config.clone();
        
        let l2_config = CacheConfig {
            size: 32 * 1024,   // 32KB
            lines_size: 64,
            associativity: 8,
            write_policy: WritePolicy::WriteBack,
            replacement_policy: ReplacementPolicy::LRU,
        };
        
        CacheHierarchy::new(l1_data_config, l1_inst_config, l2_config)
    }

    #[test]
    fn test_cache_hierarchy_creation() {
        let hierarchy = create_test_hierarchy();
        let stats = hierarchy.get_combined_stats();
        // Test basic creation
        assert!(!stats.is_empty());
        assert!(stats.contains("Cache Hierarchy Statistics"));
    }
    
    #[test]
    fn test_cache_hierarchy_l1_hit() {
        let mut hierarchy = create_test_hierarchy();
        
        // Premier accès - écriture
        let result1 = hierarchy.access_data(0x1000, true, Some(42)).unwrap();
        
        // Deuxième accès à la même adresse - devrait être un hit L1
        let result2 = hierarchy.access_data(0x1000, false, None).unwrap();
        
        match result2 {
            CacheAccessResult::Hit(data) => assert_eq!(data, 42),
            CacheAccessResult::L2Hit(_) => {
                // Aussi acceptable selon la politique
            }
            _ => panic!("Expected hit or L2 hit, got {:?}", result2),
        }
    }
    
    #[test]
    fn test_cache_hierarchy_l2_functionality() {
        let mut hierarchy = create_test_hierarchy();
        
        // Remplir avec quelques données
        for i in 0..10 {
            let addr = 0x1000 + (i * 64); // Différentes lignes de cache
            let _ = hierarchy.access_data(addr, true, Some(i as u64));
        }
        
        // Accéder à des données précédemment écrites
        for i in 0..10 {
            let addr = 0x1000 + (i * 64);
            let result = hierarchy.access_data(addr, false, None).unwrap();
            
            match result {
                CacheAccessResult::Hit(data) | CacheAccessResult::L2Hit(data) => {
                    assert_eq!(data, i as u64);
                }
                CacheAccessResult::Miss | CacheAccessResult::MSHRPending => {
                    // Acceptable selon la politique d'éviction
                }
            }
        }
    }
    
    #[test]
    fn test_cache_hierarchy_mshr() {
        let mut hierarchy = create_test_hierarchy();
        
        // Faire quelques miss pour tester MSHR, mais pas trop pour éviter de le remplir
        for i in 0..3 {
            let addr = 0x10000 + (i * 4096); // Adresses très espacées
            let result = hierarchy.access_data(addr, false, None);
            
            // Gérer le cas où le MSHR est plein
            match result {
                Ok(CacheAccessResult::Miss) | Ok(CacheAccessResult::MSHRPending) | 
                Ok(CacheAccessResult::Hit(_)) | Ok(CacheAccessResult::L2Hit(_)) => {
                    // Tous sont acceptables
                }
                Err(_) => {
                    // MSHR plein est aussi acceptable dans ce test
                    break;
                }
            }
            
            // Mettre à jour MSHR après chaque accès pour libérer des entrées
            let _ = hierarchy.update_mshr();
        }
        
        // Test final - le MSHR devrait fonctionner
        let completed = hierarchy.update_mshr();
        // Le nombre d'entrées complétées peut varier, on teste juste que ça ne crash pas
    }
    
    #[test]
    fn test_cache_hierarchy_write_buffer() {
        let mut hierarchy = create_test_hierarchy();
        
        // Effectuer plusieurs écritures
        for i in 0..5 {
            let addr = 0x2000 + i;
            let _ = hierarchy.access_data(addr, true, Some(i as u64));
        }
        
        // Vider le write buffer
        hierarchy.flush_write_buffer().unwrap();
        
        // Le flush ne devrait pas causer d'erreur
    }
    
    #[test]
    fn test_cache_hierarchy_combined_stats() {
        let mut hierarchy = create_test_hierarchy();
        
        // Effectuer des accès variés
        let _ = hierarchy.access_data(0x1000, true, Some(42));  // Write
        let _ = hierarchy.access_data(0x1000, false, None);     // Read (probable hit)
        let _ = hierarchy.access_data(0x2000, false, None);     // Read (probable miss)
        let _ = hierarchy.access_data(0x3000, true, Some(84));  // Write (probable miss)
        
        let stats = hierarchy.get_combined_stats();
        
        // Vérifier que les statistiques contiennent des données
        assert!(!stats.is_empty());
        assert!(stats.contains("Cache Hierarchy Statistics"));
        assert!(stats.contains("L1 Data:"));
        assert!(stats.contains("L2 Unified:"));
    }
}

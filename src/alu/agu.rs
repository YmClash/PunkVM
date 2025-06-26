//! Address Generation Unit (AGU)
//! Unité dédiée pour le calcul des adresses mémoire en parallèle avec l'ALU
//! Supporte tous les modes d'adressage et optimisations avancées

use std::collections::HashMap;

/// Configuration de l'AGU
#[derive(Debug, Clone)]
pub struct AGUConfig {
    pub enable_stride_prediction: bool,
    pub enable_base_cache: bool,
    pub stride_table_size: usize,
    pub base_cache_size: usize,
}

impl Default for AGUConfig {
    fn default() -> Self {
        Self {
            enable_stride_prediction: true,
            enable_base_cache: true,
            stride_table_size: 64,
            base_cache_size: 8,
        }
    }
}

/// Modes d'adressage supportés par l'AGU
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AddressingMode {
    /// Base + Offset
    BaseOffset { base: u8, offset: i32 },
    
    /// Base + Index * Scale + Offset
    BaseIndexScale { 
        base: u8, 
        index: u8, 
        scale: u8,  // 1, 2, 4, 8
        offset: i32 
    },
    
    /// PC-relative
    PCRelative { offset: i32 },
    
    /// Absolute
    Absolute { address: u64 },
    
    /// Stack-relative (SP-based)
    StackRelative { offset: i32 },
    
    /// Segment:Offset (future use)
    Segmented { segment: u8, offset: u32 },
}

/// Prédicteur de stride pour accès séquentiels
#[derive(Debug, Clone)]
pub struct StridePredictor {
    entries: HashMap<u64, StrideEntry>,
    capacity: usize,
    hits: u64,
    misses: u64,
}

#[derive(Debug, Clone)]
struct StrideEntry {
    last_address: u64,
    stride: i64,
    confidence: u8,  // 0-3 saturating counter
    hits: u32,
    last_used: u64,
}

/// Cache d'adresses de base fréquemment utilisées
#[derive(Debug, Clone)]
pub struct BaseAddressCache {
    entries: Vec<(Option<u8>, u64)>,  // (register, value)
    hits: u64,
    misses: u64,
}

/// Statistiques de l'AGU
#[derive(Debug, Clone, Default)]
pub struct AGUStats {
    pub total_calculations: u64,
    pub early_resolutions: u64,
    pub stride_predictions_correct: u64,
    pub stride_predictions_total: u64,
    pub base_cache_hits: u64,
    pub base_cache_misses: u64,
    pub parallel_executions: u64,
    pub average_latency: f64,
}

/// Erreurs de l'AGU
#[derive(Debug)]
pub enum AGUError {
    InvalidRegister,
    InvalidSegment,
    AddressOverflow,
    InvalidScale,
}

/// Address Generation Unit principale
pub struct AGU {
    /// Configuration
    config: AGUConfig,
    
    /// Prédicteur de stride
    stride_predictor: StridePredictor,
    
    /// Cache d'adresses de base
    base_cache: BaseAddressCache,
    
    /// Registres de segment
    segment_registers: [u64; 4],
    
    /// État interne
    last_base: Option<u64>,
    last_address: Option<u64>,
    current_cycle: u64,
    
    /// Statistiques
    stats: AGUStats,
}

impl StridePredictor {
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: HashMap::new(),
            capacity,
            hits: 0,
            misses: 0,
        }
    }
    
    pub fn predict(&mut self, pc: u64, base: u64) -> Option<u64> {
        if let Some(entry) = self.entries.get(&pc) {
            if entry.confidence >= 2 {
                let predicted = (entry.last_address as i64 + entry.stride) as u64;
                return Some(predicted);
            }
        }
        None
    }
    
    pub fn update(&mut self, pc: u64, actual_address: u64, current_cycle: u64) {
        let entry = self.entries.entry(pc).or_insert(StrideEntry {
            last_address: actual_address,
            stride: 0,
            confidence: 0,
            hits: 0,
            last_used: current_cycle,
        });
        
        let new_stride = (actual_address as i64) - (entry.last_address as i64);
        
        if new_stride == entry.stride {
            // Correct prediction
            entry.confidence = entry.confidence.saturating_add(1);
            entry.hits += 1;
            self.hits += 1;
        } else {
            // Wrong prediction
            entry.confidence = entry.confidence.saturating_sub(1);
            if entry.confidence == 0 {
                entry.stride = new_stride;
            }
            self.misses += 1;
        }
        
        entry.last_address = actual_address;
        entry.last_used = current_cycle;
        
        // LRU eviction if over capacity
        if self.entries.len() > self.capacity {
            let oldest_pc = self.entries.iter()
                .min_by_key(|(_, entry)| entry.last_used)
                .map(|(&pc, _)| pc);
            
            if let Some(pc) = oldest_pc {
                self.entries.remove(&pc);
            }
        }
    }
    
    pub fn get_accuracy(&self) -> f64 {
        let total = self.hits + self.misses;
        if total > 0 {
            self.hits as f64 / total as f64
        } else {
            0.0
        }
    }
}

impl BaseAddressCache {
    pub fn new(size: usize) -> Self {
        Self {
            entries: vec![(None, 0); size],
            hits: 0,
            misses: 0,
        }
    }
    
    pub fn lookup(&mut self, reg: u8, value: u64) -> bool {
        let index = (reg as usize) % self.entries.len();
        
        if let Some((cached_reg, cached_val)) = self.entries[index].0.zip(Some(self.entries[index].1)) {
            if cached_reg == reg && cached_val == value {
                self.hits += 1;
                return true;
            }
        }
        
        self.misses += 1;
        self.entries[index] = (Some(reg), value);
        false
    }
    
    pub fn get_hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total > 0 {
            self.hits as f64 / total as f64
        } else {
            0.0
        }
    }
}

impl AGU {
    /// Crée une nouvelle instance de l'AGU
    pub fn new(config: AGUConfig) -> Self {
        Self {
            stride_predictor: StridePredictor::new(config.stride_table_size),
            base_cache: BaseAddressCache::new(config.base_cache_size),
            segment_registers: [0; 4],
            last_base: None,
            last_address: None,
            current_cycle: 0,
            stats: AGUStats::default(),
            config,
        }
    }
    
    /// Calcule une adresse selon le mode d'adressage spécifié
    pub fn calculate_address(
        &mut self,
        mode: AddressingMode,
        registers: &[u64],
        pc: u64,
        sp: u64,
    ) -> Result<u64, AGUError> {
        self.stats.total_calculations += 1;
        
        let address = match mode {
            AddressingMode::BaseOffset { base, offset } => {
                let base_val = registers.get(base as usize)
                    .ok_or(AGUError::InvalidRegister)?;
                    
                // Check base cache
                if self.config.enable_base_cache {
                    self.base_cache.lookup(base, *base_val);
                }
                
                self.add_with_overflow(*base_val, offset as i64)
            }
            
            AddressingMode::BaseIndexScale { base, index, scale, offset } => {
                // Validate scale
                if ![1, 2, 4, 8].contains(&scale) {
                    return Err(AGUError::InvalidScale);
                }
                
                let base_val = registers.get(base as usize)
                    .ok_or(AGUError::InvalidRegister)?;
                let index_val = registers.get(index as usize)
                    .ok_or(AGUError::InvalidRegister)?;
                
                let scaled = index_val.wrapping_mul(scale as u64);
                let with_base = self.add_with_overflow(*base_val, scaled as i64);
                self.add_with_overflow(with_base, offset as i64)
            }
            
            AddressingMode::PCRelative { offset } => {
                self.add_with_overflow(pc, offset as i64)
            }
            
            AddressingMode::Absolute { address } => {
                address
            }
            
            AddressingMode::StackRelative { offset } => {
                self.add_with_overflow(sp, offset as i64)
            }
            
            AddressingMode::Segmented { segment, offset } => {
                let segment_base = self.segment_registers.get(segment as usize)
                    .ok_or(AGUError::InvalidSegment)?;
                segment_base.wrapping_add(offset as u64)
            }
        };
        
        // Update stride predictor if enabled
        if self.config.enable_stride_prediction {
            self.stride_predictor.update(pc, address, self.current_cycle);
        }
        
        self.last_address = Some(address);
        Ok(address)
    }
    
    /// Prédiction d'adresse pour optimiser le pipeline
    pub fn predict_address(
        &mut self,
        pc: u64,
        base_reg: Option<u8>,
        registers: &[u64],
    ) -> Option<u64> {
        if !self.config.enable_stride_prediction {
            return None;
        }
        
        if let Some(base) = base_reg {
            if let Some(&base_val) = registers.get(base as usize) {
                return self.stride_predictor.predict(pc, base_val);
            }
        }
        
        None
    }
    
    /// Addition avec gestion de débordement
    fn add_with_overflow(&self, base: u64, offset: i64) -> u64 {
        if offset >= 0 {
            base.wrapping_add(offset as u64)
        } else {
            base.wrapping_sub((-offset) as u64)
        }
    }
    
    /// Met à jour le cycle actuel
    pub fn update_cycle(&mut self, cycle: u64) {
        self.current_cycle = cycle;
    }
    
    /// Obtient les statistiques de l'AGU
    pub fn get_stats(&self) -> &AGUStats {
        &self.stats
    }
    
    /// Obtient les statistiques du prédicteur de stride
    pub fn get_stride_stats(&self) -> (u64, u64, f64) {
        (self.stride_predictor.hits, self.stride_predictor.misses, self.stride_predictor.get_accuracy())
    }
    
    /// Obtient les statistiques du cache de base
    pub fn get_base_cache_stats(&self) -> (u64, u64, f64) {
        (self.base_cache.hits, self.base_cache.misses, self.base_cache.get_hit_rate())
    }
    
    /// Réinitialise l'AGU
    pub fn reset(&mut self) {
        self.stride_predictor = StridePredictor::new(self.config.stride_table_size);
        self.base_cache = BaseAddressCache::new(self.config.base_cache_size);
        self.segment_registers = [0; 4];
        self.last_base = None;
        self.last_address = None;
        self.current_cycle = 0;
        self.stats = AGUStats::default();
    }
}

// Tests unitaires
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_agu_creation() {
        let agu = AGU::new(AGUConfig::default());
        assert_eq!(agu.stats.total_calculations, 0);
    }
    
    #[test]
    fn test_base_offset_addressing() {
        let mut agu = AGU::new(AGUConfig::default());
        let mut registers = vec![0; 16];
        registers[5] = 0x1000;
        
        let addr = agu.calculate_address(
            AddressingMode::BaseOffset { base: 5, offset: 0x20 },
            &registers, 0, 0
        ).unwrap();
        
        assert_eq!(addr, 0x1020);
        assert_eq!(agu.stats.total_calculations, 1);
    }
    
    #[test]
    fn test_base_index_scale() {
        let mut agu = AGU::new(AGUConfig::default());
        let mut registers = vec![0; 16];
        registers[5] = 0x1000;  // base
        registers[6] = 0x10;    // index
        
        let addr = agu.calculate_address(
            AddressingMode::BaseIndexScale { 
                base: 5, 
                index: 6, 
                scale: 8, 
                offset: 0x5 
            },
            &registers, 0, 0
        ).unwrap();
        
        assert_eq!(addr, 0x1000 + (0x10 * 8) + 0x5);  // 0x1085
    }
    
    #[test]
    fn test_stride_prediction() {
        let mut predictor = StridePredictor::new(16);
        let pc = 0x100;
        
        // Train the predictor
        predictor.update(pc, 0x1000, 1);
        predictor.update(pc, 0x1008, 2);
        predictor.update(pc, 0x1010, 3);
        
        // Should predict next as 0x1018
        let prediction = predictor.predict(pc, 0x1010);
        assert_eq!(prediction, Some(0x1018));
    }
    
    #[test]
    fn test_pc_relative_addressing() {
        let mut agu = AGU::new(AGUConfig::default());
        let registers = vec![0; 16];
        
        let addr = agu.calculate_address(
            AddressingMode::PCRelative { offset: 100 },
            &registers, 0x2000, 0
        ).unwrap();
        
        assert_eq!(addr, 0x2064);  // 0x2000 + 100
    }
    
    #[test]
    fn test_stack_relative_addressing() {
        let mut agu = AGU::new(AGUConfig::default());
        let registers = vec![0; 16];
        
        let addr = agu.calculate_address(
            AddressingMode::StackRelative { offset: -8 },
            &registers, 0, 0xC000
        ).unwrap();
        
        assert_eq!(addr, 0xC000 - 8);
    }
    
    #[test]
    fn test_absolute_addressing() {
        let mut agu = AGU::new(AGUConfig::default());
        let registers = vec![0; 16];
        
        let addr = agu.calculate_address(
            AddressingMode::Absolute { address: 0x5000 },
            &registers, 0, 0
        ).unwrap();
        
        assert_eq!(addr, 0x5000);
    }
    
    #[test]
    fn test_invalid_scale() {
        let mut agu = AGU::new(AGUConfig::default());
        let mut registers = vec![0; 16];
        registers[5] = 0x1000;
        registers[6] = 0x10;
        
        let result = agu.calculate_address(
            AddressingMode::BaseIndexScale { 
                base: 5, 
                index: 6, 
                scale: 3,  // Invalid scale
                offset: 0 
            },
            &registers, 0, 0
        );
        
        assert!(matches!(result, Err(AGUError::InvalidScale)));
    }
    
    #[test]
    fn test_base_cache() {
        let mut cache = BaseAddressCache::new(4);
        
        // First access should be a miss
        assert!(!cache.lookup(5, 0x1000));
        assert_eq!(cache.misses, 1);
        
        // Second access to same reg/value should be a hit
        assert!(cache.lookup(5, 0x1000));
        assert_eq!(cache.hits, 1);
        
        // Different value should be a miss
        assert!(!cache.lookup(5, 0x2000));
        assert_eq!(cache.misses, 2);
    }
    
    #[test]
    fn test_stride_predictor_accuracy() {
        let mut predictor = StridePredictor::new(16);
        let pc = 0x100;
        
        // Train with consistent stride
        for i in 0..5 {
            predictor.update(pc, 0x1000 + (i * 8), i as u64);
        }
        
        // Should have reasonable accuracy after training
        let accuracy = predictor.get_accuracy();
        assert!(accuracy > 0.5);
    }
}

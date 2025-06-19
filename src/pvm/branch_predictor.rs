// src/pvm/branch_prediction.rs

use std::collections::HashMap;
use crate::pipeline::ras::ReturnAddressStack;
use crate::pvm::branch_perceptor::Perceptron;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PredictorType {
    Static,
    Dynamic,
    GShare,
    Gskew,
    Hybrid,
    Tournament,
    Perceptron,
}

#[derive(Debug, Clone,Copy)]
pub struct PredictorConfig {
    pub local_history_bits: usize,
    pub global_history_bits: usize,
    pub btb_size: usize,
    pub ras_size: usize,
}

#[derive(Debug)]
struct GSharePredictor {
    global_history: u16,
    pattern_table: Vec<u8>,
    history_length: usize,
}

impl GSharePredictor {
    pub fn new() -> Self {
        let history_length = 12; // 12 bits d'historique global
        let table_size = 1 << history_length; // 4096 entrées
        Self {
            global_history: 0,
            pattern_table: vec![1; table_size], // Initialiser avec WeaklyNotTaken (1)
            history_length,
        }
    }
    
    pub fn predict_branch(&self, branch_pc: u64) -> bool {
        let index = self.compute_index(branch_pc);
        let counter = self.pattern_table[index];
        // Si le compteur est >= 2 (WeaklyTaken ou StronglyTaken), prédire pris
        counter >= 2
    }
    
    pub fn update_predictor(&mut self, branch_pc: u64, actual_outcome: bool) {
        let index = self.compute_index(branch_pc);
        let counter = self.pattern_table[index];
        
        // Mise à jour du compteur à 2 bits
        self.pattern_table[index] = if actual_outcome {
            // Branch taken: increment counter (max 3)
            counter.saturating_add(1).min(3)
        } else {
            // Branch not taken: decrement counter (min 0)
            counter.saturating_sub(1)
        };
        
        // Mise à jour de l'historique global
        self.global_history = (self.global_history << 1) | (actual_outcome as u16);
        self.global_history &= (1 << self.history_length) - 1; // Garder seulement les bits nécessaires
    }
    
    fn compute_index(&self, branch_pc: u64) -> usize {
        // XOR entre les bits du PC et l'historique global
        let pc_bits = (branch_pc as usize) & ((1 << self.history_length) - 1);
        let history_bits = self.global_history as usize;
        (pc_bits ^ history_bits) & ((1 << self.history_length) - 1)
    }
}






#[derive(Debug, Clone)]
pub struct BranchTargetBuffer {
    pub entries: Vec<BTBEntry>,
    pub size: usize,
    pub current_cycle: u64, // Pour la gestion du LRU

}

/// B
#[derive(Debug, Clone)]
pub struct BTBEntry {
    pub tag: u32,
    pub target: u32,
    pub valid: bool,
    pub confidence: u8, // 2 bits pour la confiance
    pub last_used: u64, // LRU
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BranchPrediction {
    Taken,
    NotTaken,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TwoBitState {
    StronglyNotTaken = 0,
    WeaklyNotTaken = 1,
    WeaklyTaken = 2,
    StronglyTaken = 3,
}

#[derive(Debug)]
pub struct BranchPredictor {
    pub prediction_type: PredictorType,
    pub predictions: HashMap<u64, BranchPrediction>,
    pub two_bit_states: HashMap<u64, TwoBitState>,
    pub metrics: BranchMetrics,
    pub hybrid_predictor: Option<HybridPredictor>,
    pub overriding_predictor: Option<OverridingPredictor>,
    pub btb: Option<BranchTargetBuffer>,
}
#[derive(Debug, Default, Clone)]
pub struct BranchMetrics {
    pub total_branches: usize,
    pub predictions_made: usize,
    pub correct_predictions: usize,
    pub incorrect_predictions: usize,
    pub gshare_accuracy: f64,
    pub branch_target_buffer: usize,
    pub store_load_forwarding: usize,
    pub btb_hits: usize,
    pub btb_misses: usize,
    pub btb_correct_targets: usize,
    pub btb_incorrect_targets: usize,
    // Métriques spécifiques au Perceptron
    pub perceptron_accuracy: f64,
    pub override_count: usize,
    pub override_benefit: usize,
    pub agreement_rate: f64,
}

#[derive(Debug, Clone, )]
struct LocalHistoryEntry {
    history:u16 , // Historique local (16 bits)
    pattern_table: Vec<TwoBitCounter>, // Table de prédiction locale
    last_used: u64, // Timestamp pour LRU
}


#[derive(Debug, Clone) ]
pub struct HybridPredictor {
    // Prédicteur local (pattern par PC)
    local_history: HashMap<u64, LocalHistoryEntry>,

    // Prédicteur global (GShare)
    global_history: u16,  // 16 bits d'historique global
    gshare_table: Vec<TwoBitCounter>,

    // Sélecteur de prédicteur (meta-predictor)
    selector: Vec<TwoBitCounter>,

    // Configuration
    local_history_bits: usize,
    global_history_bits: usize,
    gshare_table_size: usize,
    return_address_stack: ReturnAddressStack  ,      // 16 entrées pour la RAS
    btb: BranchTargetBuffer, // 512 entrées pour la BTB
}


#[derive(Debug)]
pub struct PerceptronPredictor {
    pub perceptrons: Vec<Perceptron>,
    pub global_history: Vec<isize>, // Global Branch History Register (GHR)
    pub local_histories:  HashMap<u64, Vec<isize>>, // Table pour les historiques locaux par PC de branche
}

#[derive(Debug, Clone,Copy)]
pub struct TwoBitCounter {
    state: TwoBitState,
}

#[derive(Debug, Clone)]
pub struct OverridingPredictorStats {
    pub gshare_accuracy: f64,
    pub perceptron_accuracy: f64,
    pub override_rate: f64,
    pub agreement_rate: f64,
    pub misprediction_penalty_full: u64,
    pub misprediction_penalty_override: u64,
}

#[derive(Debug)]
pub struct OverridingPredictor {
    gshare_predictor: GSharePredictor,
    perceptron_predictor: PerceptronPredictor,
    // Cache des prédictions perceptron en cours de calcul (simule le délai)
    perceptron_predictions_cache: HashMap<u64, bool>,
    // Statistiques de performance
    pub misprediction_penalty_full: u64,
    pub misprediction_penalty_override: u64,
    pub perceptron_correct: u64,
    pub perceptron_incorrect: u64,
    pub gshare_correct: u64,
    pub gshare_incorrect: u64,
    pub override_count: u64,
    pub agreement_correct: u64,
    pub agreement_incorrect: u64,
}

impl TwoBitCounter {
    pub fn new() -> Self {
        Self {
            state: TwoBitState::WeaklyNotTaken,
        }
    }
    
    pub fn new_biased(bias: TwoBitState) -> Self {
        Self {
            state: bias,
        }
    }
    
    pub fn predict(&self) -> BranchPrediction {
        match self.state {
            TwoBitState::StronglyNotTaken | TwoBitState::WeaklyNotTaken => BranchPrediction::NotTaken,
            TwoBitState::WeaklyTaken | TwoBitState::StronglyTaken => BranchPrediction::Taken,
        }
    }
    
    pub fn update(&mut self, taken: bool) {
        self.state = match (self.state, taken) {
            (TwoBitState::StronglyNotTaken, true) => TwoBitState::WeaklyNotTaken,
            (TwoBitState::WeaklyNotTaken, true) => TwoBitState::WeaklyTaken,
            (TwoBitState::WeaklyTaken, true) => TwoBitState::StronglyTaken,
            (TwoBitState::StronglyTaken, true) => TwoBitState::StronglyTaken,
            
            (TwoBitState::StronglyTaken, false) => TwoBitState::WeaklyTaken,
            (TwoBitState::WeaklyTaken, false) => TwoBitState::WeaklyNotTaken,
            (TwoBitState::WeaklyNotTaken, false) => TwoBitState::StronglyNotTaken,
            (TwoBitState::StronglyNotTaken, false) => TwoBitState::StronglyNotTaken,
        };
    }
}





impl HybridPredictor {
    pub fn new(local_history_bits: usize, global_history_bits: usize, gshare_table_size: usize) -> Self {
        let selector_size = 1 << 10; // 1K entries for selector
        
        // Initialize GShare table with slight taken bias (research shows conditional branches are taken ~60% of time)
        let mut gshare_table = Vec::with_capacity(gshare_table_size);
        for _ in 0..gshare_table_size {
            gshare_table.push(TwoBitCounter::new_biased(TwoBitState::WeaklyTaken));
        }
        
        // Initialize selector to slightly favor global predictor
        let mut selector = Vec::with_capacity(selector_size);
        for _ in 0..selector_size {
            selector.push(TwoBitCounter::new_biased(TwoBitState::WeaklyTaken));
        }
        
        Self {
            local_history: HashMap::new(),
            global_history: 0,
            gshare_table,
            selector,
            local_history_bits,
            global_history_bits,
            gshare_table_size,
            return_address_stack: ReturnAddressStack::new(16), // 16 entries for RAS
            btb: BranchTargetBuffer::new(512), // 512 entries BTB
        }
    }
    
    pub fn predict(&self, pc: u64) -> BranchPrediction {
        let local_prediction = self.predict_local(pc);
        let gshare_prediction = self.predict_gshare(pc);
        
        // Use selector to choose between local and global
        let selector_index = (pc & 0x3FF) as usize; // 10 bits
        let selector_prediction = self.selector[selector_index].predict();
        
        match selector_prediction {
            BranchPrediction::NotTaken => local_prediction,  // Use local predictor
            BranchPrediction::Taken => gshare_prediction,    // Use global predictor
        }
    }
    
    fn predict_local(&self, pc: u64) -> BranchPrediction {
        if let Some(entry) = self.local_history.get(&pc) {
            let pattern_index = entry.history as usize & ((1 << self.local_history_bits) - 1);
            if pattern_index < entry.pattern_table.len() {
                entry.pattern_table[pattern_index].predict()
            } else {
                BranchPrediction::Taken // Default to taken for new patterns
            }
        } else {
            BranchPrediction::Taken // Default to taken for unseen branches
        }
    }
    
    fn predict_gshare(&self, pc: u64) -> BranchPrediction {
        let index = self.compute_gshare_index(pc);
        self.gshare_table[index].predict()
    }
    
    fn compute_gshare_index(&self, pc: u64) -> usize {
        let pc_bits = pc as usize & ((1 << self.global_history_bits) - 1);
        let history_bits = self.global_history as usize;
        (pc_bits ^ history_bits) & (self.gshare_table_size - 1)
    }
    
    pub fn update(&mut self, pc: u64, taken: bool) {
        let local_prediction = self.predict_local(pc);
        let gshare_prediction = self.predict_gshare(pc);
        
        // Update local predictor
        self.update_local(pc, taken);
        
        // Update global predictor
        self.update_gshare(pc, taken);
        
        // Update selector based on which predictor was more accurate
        let selector_index = (pc & 0x3FF) as usize;
        let local_correct = (local_prediction == BranchPrediction::Taken) == taken;
        let gshare_correct = (gshare_prediction == BranchPrediction::Taken) == taken;
        
        if local_correct && !gshare_correct {
            self.selector[selector_index].update(false); // Favor local
        } else if !local_correct && gshare_correct {
            self.selector[selector_index].update(true);  // Favor global
        }
        // If both correct or both wrong, don't update selector
        
        // Update global history
        self.global_history = (self.global_history << 1) | (taken as u16);
        self.global_history &= (1 << self.global_history_bits) - 1;
    }
    
    fn update_local(&mut self, pc: u64, taken: bool) {
        let entry = self.local_history.entry(pc).or_insert_with(|| {
            let mut pattern_table = Vec::with_capacity(1 << self.local_history_bits);
            for _ in 0..(1 << self.local_history_bits) {
                pattern_table.push(TwoBitCounter::new_biased(TwoBitState::WeaklyTaken));
            }
            LocalHistoryEntry {
                history: 0,
                pattern_table,
                last_used: 0,
            }
        });
        
        let pattern_index = entry.history as usize & ((1 << self.local_history_bits) - 1);
        if pattern_index < entry.pattern_table.len() {
            entry.pattern_table[pattern_index].update(taken);
        }
        
        // Update local history
        entry.history = (entry.history << 1) | (taken as u16);
        entry.history &= (1 << self.local_history_bits) - 1;
    }
    
    fn update_gshare(&mut self, pc: u64, taken: bool) {
        let index = self.compute_gshare_index(pc);
        self.gshare_table[index].update(taken);
    }
}

impl BranchPredictor {
    pub fn new(predictor_type: PredictorType) -> Self {
        let hybrid_predictor = if predictor_type == PredictorType::Hybrid {
            Some(HybridPredictor::new(
                10, // local history bits
                12, // global history bits
                4096, // gshare table size (4K entries)
            ))
        } else {
            None
        };
        
        let overriding_predictor = if predictor_type == PredictorType::Perceptron {
            Some(OverridingPredictor::new())
        } else {
            None
        };
        
        let btb = if matches!(predictor_type, PredictorType::Hybrid | PredictorType::Dynamic | PredictorType::Perceptron) {
            Some(BranchTargetBuffer::new(512)) // 512 entries BTB
        } else {
            None
        };
        
        Self {
            prediction_type: predictor_type,
            predictions: HashMap::new(),
            two_bit_states: HashMap::new(),
            metrics: BranchMetrics::default(),
            hybrid_predictor,
            overriding_predictor,
            btb,
        }
    }
    
    pub fn new_with_config(predictor_type: PredictorType, config: PredictorConfig) -> Self {
        let hybrid_predictor = if predictor_type == PredictorType::Hybrid {
            Some(HybridPredictor::new(
                config.local_history_bits,
                config.global_history_bits,
                1 << config.global_history_bits, // gshare table size
            ))
        } else {
            None
        };
        
        let overriding_predictor = if predictor_type == PredictorType::Perceptron {
            Some(OverridingPredictor::new())
        } else {
            None
        };
        
        let btb = if matches!(predictor_type, PredictorType::Hybrid | PredictorType::Dynamic | PredictorType::Perceptron) {
            Some(BranchTargetBuffer::new(config.btb_size))
        } else {
            None
        };
        
        Self {
            prediction_type: predictor_type,
            predictions: HashMap::new(),
            two_bit_states: HashMap::new(),
            metrics: BranchMetrics::default(),
            hybrid_predictor,
            overriding_predictor,
            btb,
        }
    }

    pub fn predict_target(&mut self, pc: u64) -> Option<u32> {
        if let Some(ref mut btb) = self.btb {
            let target = btb.predict(pc);
            if target.is_some() {
                self.metrics.btb_hits += 1;
            } else {
                self.metrics.btb_misses += 1;
            }
            target
        } else {
            None
        }
    }
    
    pub fn predict(&mut self, pc: u64) -> BranchPrediction {
        self.metrics.predictions_made += 1;

        match self.prediction_type {
            PredictorType::Static => {
                // Toujours NotTaken en statique
                BranchPrediction::NotTaken
            }
            PredictorType::Dynamic => {
                // Lire l'état 2 bits ou init par défaut
                let state = self
                    .two_bit_states
                    .entry(pc)
                    .or_insert(TwoBitState::WeaklyNotTaken);

                match state {
                    TwoBitState::StronglyNotTaken | TwoBitState::WeaklyNotTaken => {
                        BranchPrediction::NotTaken
                    }
                    TwoBitState::WeaklyTaken | TwoBitState::StronglyTaken => {
                        BranchPrediction::Taken
                    }
                }
            }
            PredictorType::GShare => {
                // GSharePredictor
                BranchPrediction::NotTaken

            }
            PredictorType::Gskew => {
                // GSkewPredictor
                BranchPrediction::NotTaken
            }
            PredictorType::Hybrid => {
                if let Some(ref hybrid) = self.hybrid_predictor {
                    hybrid.predict(pc)
                } else {
                    BranchPrediction::NotTaken
                }
            }
            PredictorType::Tournament => {
                // TournamentPredictor
                BranchPrediction::NotTaken
            }
            PredictorType::Perceptron => {
                if let Some(ref mut overriding) = self.overriding_predictor {
                    let prediction = overriding.get_initial_prediction(pc);
                    if prediction {
                        BranchPrediction::Taken
                    } else {
                        BranchPrediction::NotTaken
                    }
                } else {
                    BranchPrediction::NotTaken
                }
            }
        }
    }

    pub fn update_btb(&mut self, pc: u64, target: u32, predicted_target: Option<u32>) {
        if let Some(ref mut btb) = self.btb {
            let correct = predicted_target == Some(target);
            if correct {
                self.metrics.btb_correct_targets += 1;
            } else {
                self.metrics.btb_incorrect_targets += 1;
            }
            btb.update_with_confidence(pc, target, correct);

        }
    }
    
    /// Appelé après avoir eu le résultat (taken ou pas).
    ///  - `prediction` est la valeur donnée par `predict()`
    ///  - `taken` est la vraie issue
    pub fn update(&mut self, pc: u64, taken: bool, prediction: BranchPrediction) {
        self.metrics.total_branches += 1;

        // Correction de la logique de comptage des prédictions
        match (prediction, taken) {
            (BranchPrediction::Taken, true) | (BranchPrediction::NotTaken, false) => {
                self.metrics.correct_predictions += 1;
                println!(
                    "Branch predictor: PC={:X}, prediction correct ({})",
                    pc,
                    if taken { "taken" } else { "not taken" }
                );
            }
            _ => {
                self.metrics.incorrect_predictions += 1;
                println!(
                    "Branch predictor: PC={:X}, prediction INCORRECT (predicted={:?}, actual={})",
                    pc,
                    prediction,
                    if taken { "taken" } else { "not taken" }
                );
            }
        }

        // Mise à jour du prédicteur dynamique
        if self.prediction_type == PredictorType::Dynamic {
            let old_state = self.two_bit_states.get(&pc).cloned();
            self.update_dynamic(pc, taken);
            let new_state = self.two_bit_states.get(&pc).cloned();
            println!(
                "Branch state update: PC={:X}, {:?} -> {:?}",
                pc, old_state, new_state
            );
        }
        
        // Mise à jour du prédicteur hybride
        if self.prediction_type == PredictorType::Hybrid {
            if let Some(ref mut hybrid) = self.hybrid_predictor {
                hybrid.update(pc, taken);
            }
        }
        
        // Mise à jour du prédicteur Perceptron (OverridingPredictor)
        if self.prediction_type == PredictorType::Perceptron {
            if let Some(ref mut overriding) = self.overriding_predictor {
                let needs_flush = overriding.update_and_get_flush_status(pc, taken);
                
                // Mettre à jour les métriques avec les statistiques du perceptron
                let stats = overriding.get_statistics();
                self.metrics.perceptron_accuracy = stats.perceptron_accuracy;
                self.metrics.gshare_accuracy = stats.gshare_accuracy;
                self.metrics.override_count = overriding.override_count as usize;
                self.metrics.agreement_rate = stats.agreement_rate;
                
                // Compter les overrides bénéfiques (cas 2: GShare faux, Perceptron correct)
                if stats.perceptron_accuracy > stats.gshare_accuracy {
                    self.metrics.override_benefit = ((stats.perceptron_accuracy - stats.gshare_accuracy) * 
                                                     self.metrics.total_branches as f64) as usize;
                }
                
                println!(
                    "Perceptron predictor: PC={:X}, taken={}, GShare acc={:.2}%, Perceptron acc={:.2}%, Override rate={:.2}%",
                    pc, taken, stats.gshare_accuracy * 100.0, stats.perceptron_accuracy * 100.0, stats.override_rate * 100.0
                );
            }
        }
    }

    fn predict_dynamic(&mut self, pc: u64) -> BranchPrediction {
        let state = self
            .two_bit_states
            .entry(pc)
            .or_insert(TwoBitState::WeaklyNotTaken);
        match state {
            TwoBitState::StronglyNotTaken | TwoBitState::WeaklyNotTaken => {
                BranchPrediction::NotTaken
            }
            TwoBitState::WeaklyTaken | TwoBitState::StronglyTaken => BranchPrediction::Taken,
        }
    }

    fn update_dynamic(&mut self, pc: u64, taken: bool) {
        let state = self
            .two_bit_states
            .entry(pc)
            .or_insert(TwoBitState::WeaklyNotTaken);
        *state = match (*state, taken) {
            // Comportement standard du prédicteur à 2 bits
            (TwoBitState::StronglyNotTaken, true) => TwoBitState::WeaklyNotTaken,
            (TwoBitState::WeaklyNotTaken, true) => TwoBitState::WeaklyTaken,
            (TwoBitState::WeaklyTaken, true) => TwoBitState::StronglyTaken,
            (TwoBitState::StronglyTaken, true) => TwoBitState::StronglyTaken,

            (TwoBitState::StronglyTaken, false) => TwoBitState::WeaklyTaken,
            (TwoBitState::WeaklyTaken, false) => TwoBitState::WeaklyNotTaken,
            (TwoBitState::WeaklyNotTaken, false) => TwoBitState::StronglyNotTaken,
            (TwoBitState::StronglyNotTaken, false) => TwoBitState::StronglyNotTaken,
        };
    }

    /// Retourne le ratio de prédictions correctes
    pub fn get_accuracy(&self) -> f64 {
        if self.metrics.total_branches == 0 {
            0.0
        } else {
            self.metrics.correct_predictions as f64 / self.metrics.total_branches as f64
        }
    }
    
    pub fn get_btb_hit_rate(&self) -> f64 {
        let total = self.metrics.btb_hits + self.metrics.btb_misses;
        if total == 0 {
            0.0
        } else {
            self.metrics.btb_hits as f64 / total as f64
        }
    }
    
    pub fn get_btb_accuracy(&self) -> f64 {
        let total = self.metrics.btb_correct_targets + self.metrics.btb_incorrect_targets;
        if total == 0 {
            0.0
        } else {
            self.metrics.btb_correct_targets as f64 / total as f64
        }
    }

}



impl BranchTargetBuffer {
    pub fn new(size: usize) -> Self {
        let mut entries = Vec::with_capacity(size);
        for _ in 0..size {
            entries.push(BTBEntry {
                tag: 0,
                target: 0,
                valid: false,
                confidence: 0,
                last_used: 0,
            });
        }
        Self {
            entries,
            size,
            current_cycle: 0,
        }
    }
    
    pub fn predict(&mut self, pc: u64) -> Option<u32> {
        let index = self.get_index(pc);
        let tag = self.get_tag(pc);
        
        let entry = &mut self.entries[index];
        if entry.valid && entry.tag == tag {
            entry.last_used = self.current_cycle;
            self.current_cycle += 1;
            Some(entry.target)
        } else {
            None
        }
    }
    
    pub fn predict_with_confidence(&mut self, pc: u64) -> Option<(u32, u8)> {
        let index = self.get_index(pc);
        let tag = self.get_tag(pc);

        let entry = &mut self.entries[index];
        if entry.valid && entry.tag == tag {
            entry.last_used = self.current_cycle;
            self.current_cycle += 1;
            Some((entry.target, entry.confidence))
        } else {
            None
        }
    }

    fn get_index(&self, pc: u64) -> usize {
        (pc as usize) % self.size
    }
    
    fn get_tag(&self, pc: u64) -> u32 {
        (pc >> 12) as u32
    }

    pub fn update(&mut self, pc: u64, target: u32) {
        let index = self.get_index(pc);
        let tag = self.get_tag(pc);
        
        let entry = &mut self.entries[index];
        entry.tag = tag;
        entry.target = target;
        entry.valid = true;
        entry.confidence = 128;
        entry.last_used = self.current_cycle;
        self.current_cycle += 1;
    }
    
    pub fn update_with_confidence(&mut self, pc: u64, target: u32, correct: bool) {
        let index = self.get_index(pc);
        let tag = self.get_tag(pc);

        let entry = &mut self.entries[index];
        if entry.valid && entry.tag == tag {
            if correct {
                entry.confidence = entry.confidence.saturating_add(1);
            } else {
                entry.confidence = entry.confidence.saturating_sub(10);
            }
            entry.last_used = self.current_cycle;
        } else {
            entry.tag = tag;
            entry.target = target;
            entry.valid = true;
            entry.confidence = if correct { 128 } else { 64 };
            entry.last_used = self.current_cycle;
        }
        self.current_cycle += 1;

    }
    
    pub fn invalidate(&mut self, pc: u64) {
        let index = self.get_index(pc);
        let tag = self.get_tag(pc);
        
        let entry = &mut self.entries[index];
        if entry.tag == tag {
            entry.valid = false;
        }
    }
}




impl PerceptronPredictor {
    pub fn new() -> Self {
        let mut perceptrons = Vec::with_capacity(crate::pvm::branch_perceptor::NUM_PERCEPTRONS);
        for _ in 0..crate::pvm::branch_perceptor::NUM_PERCEPTRONS {
            perceptrons.push(Perceptron::new(crate::pvm::branch_perceptor::TOTAL_HISTORY_LENGTH));
        }
        Self {
            perceptrons,
            // Initialisation de l'historique global (e.g., tous pris)
            global_history: vec![1; crate::pvm::branch_perceptor::GLOBAL_HISTORY_LENGTH],
            local_histories: HashMap::new(),
        }
    }

    // Sélectionne l'indice du perceptron basé sur le PC de branche
    // Une fonction de hachage plus sophistiquée peut être utilisée ici (PC XOR GHR, etc.)
    fn get_perceptron_index(&self, branch_pc: u64) -> usize {
        (branch_pc as usize) % crate::pvm::branch_perceptor::NUM_PERCEPTRONS
    }

    // Génère l'historique combiné (global + local) pour les entrées du perceptron
    fn get_combined_history(&mut self, branch_pc: u64) -> Vec<isize> {
        let mut combined = Vec::with_capacity(crate::pvm::branch_perceptor::TOTAL_HISTORY_LENGTH);

        // Historique global
        combined.extend_from_slice(&self.global_history);

        // Historique local (créer si n'existe pas)
        let local_hist = self.local_histories
            .entry(branch_pc)
            .or_insert_with(|| vec![1; crate::pvm::branch_perceptor::LOCAL_HISTORY_LENGTH]); // Initialiser avec des 1 par défaut

        combined.extend_from_slice(local_hist);
        combined
    }

    // Prédiction
    pub fn predict_branch(&mut self, branch_pc: u64) -> bool {
        let perceptron_idx = self.get_perceptron_index(branch_pc);
        let combined_history = self.get_combined_history(branch_pc); // Obtenir l'historique combiné
        let sum = self.perceptrons[perceptron_idx].predict_sum(&combined_history);

        sum > 0 // Prédire 'pris' si la somme est positive
    }

    // Mise à jour (entraînement)
    pub fn update_predictor(&mut self, branch_pc: u64, actual_outcome: bool) {
        let perceptron_idx = self.get_perceptron_index(branch_pc);
        let actual_val = if actual_outcome { 1 } else { -1 };
        
        // D'abord, calculer l'historique combiné avant d'emprunter le perceptron
        let combined_history = self.get_combined_history(branch_pc);
        
        // Maintenant, emprunter le perceptron pour l'entraînement
        let predicted_sum = self.perceptrons[perceptron_idx].predict_sum(&combined_history);
        self.perceptrons[perceptron_idx].train(&combined_history, actual_val, predicted_sum);

        // Mettre à jour l'historique global
        self.global_history.rotate_left(1);
        self.global_history[crate::pvm::branch_perceptor::GLOBAL_HISTORY_LENGTH - 1] = actual_val;

        // Mettre à jour l'historique local du branchement
        if let Some(local_hist) = self.local_histories.get_mut(&branch_pc) {
            local_hist.rotate_left(1);
            local_hist[crate::pvm::branch_perceptor::LOCAL_HISTORY_LENGTH - 1] = actual_val;
        } else {
            // Cela ne devrait pas arriver si get_combined_history est toujours appelé avant
            // mais par sécurité si cette fonction est appelée directement sans prédiction préalable.
            let mut new_local_hist = vec![1; crate::pvm::branch_perceptor::LOCAL_HISTORY_LENGTH];
            new_local_hist.rotate_left(1);
            new_local_hist[crate::pvm::branch_perceptor::LOCAL_HISTORY_LENGTH - 1] = actual_val;
            self.local_histories.insert(branch_pc, new_local_hist);
        }
    }

    // Réinitialisation de l'état du prédicteur
    pub fn reset(&mut self) {
        for perceptron in &mut self.perceptrons {
            perceptron.weight.fill(0); // Réinitialiser les poids à 0
        }
        self.global_history.fill(1); // Réinitialiser l'historique global à 1
        self.local_histories.clear(); // Effacer l'historique local
    }

}

impl OverridingPredictor {
    pub fn new() -> Self {
        Self {
            gshare_predictor: GSharePredictor::new(),
            perceptron_predictor: PerceptronPredictor::new(),
            perceptron_predictions_cache: HashMap::new(),
            misprediction_penalty_full: 0,
            misprediction_penalty_override: 0,
            perceptron_correct: 0,
            perceptron_incorrect: 0,
            gshare_correct: 0,
            gshare_incorrect: 0,
            override_count: 0,
            agreement_correct: 0,
            agreement_incorrect: 0,
        }
    }

    /// Fonction de prédiction à appeler au moment du Fetch/Decode
    /// Retourne la prédiction initiale (rapide)
    pub fn get_initial_prediction(&mut self, branch_pc: u64) -> bool {
        // Prédiction rapide du GShare
        let gshare_pred = self.gshare_predictor.predict_branch(branch_pc);
        
        // Lancer le calcul du perceptron en arrière-plan (simulé par le cache)
        let perceptron_pred = self.perceptron_predictor.predict_branch(branch_pc);
        self.perceptron_predictions_cache.insert(branch_pc, perceptron_pred);
        
        gshare_pred
    }
    
    /// Récupère la prédiction du perceptron (simule le délai de calcul)
    pub fn get_perceptron_prediction(&mut self, branch_pc: u64) -> Option<bool> {
        // Dans une vraie implémentation, ceci serait disponible après quelques cycles
        self.perceptron_predictions_cache.get(&branch_pc).copied()
    }

    /// Fonction de mise à jour appelée quand le résultat réel est connu
    /// Retourne `true` si un flush complet est nécessaire, `false` sinon.
    pub fn update_and_get_flush_status(
        &mut self,
        branch_pc: u64,
        actual_outcome: bool,
    ) -> bool {
        // Récupérer la prédiction initiale du GShare
        let gshare_predicted = self.gshare_predictor.predict_branch(branch_pc);
        
        // Récupérer la prédiction du perceptron depuis le cache
        let perceptron_predicted = self.perceptron_predictions_cache
            .remove(&branch_pc)
            .unwrap_or_else(|| {
                // Si pas en cache, calculer maintenant (ne devrait pas arriver normalement)
                self.perceptron_predictor.predict_branch(branch_pc)
            });
        
        // Mettre à jour les statistiques individuelles
        if gshare_predicted == actual_outcome {
            self.gshare_correct += 1;
        } else {
            self.gshare_incorrect += 1;
        }
        
        if perceptron_predicted == actual_outcome {
            self.perceptron_correct += 1;
        } else {
            self.perceptron_incorrect += 1;
        }
        
        // Mettre à jour les deux prédicteurs
        self.gshare_predictor.update_predictor(branch_pc, actual_outcome);
        self.perceptron_predictor.update_predictor(branch_pc, actual_outcome);

        // Analyser les quatre cas pour les pénalités et les flushes
        let mut needs_full_flush = false;

        if gshare_predicted == actual_outcome && perceptron_predicted == actual_outcome {
            // Cas 1: Les deux sont corrects et d'accord
            self.agreement_correct += 1;
        } else if gshare_predicted != actual_outcome && perceptron_predicted == actual_outcome {
            // Cas 2: Gshare faux, Perceptron correct - Override bénéfique
            self.misprediction_penalty_override += 1;
            self.override_count += 1;
            needs_full_flush = true;
        } else if gshare_predicted != actual_outcome && perceptron_predicted != actual_outcome {
            // Cas 3 & 4: Les deux sont faux
            if gshare_predicted != perceptron_predicted {
                // Cas 3: Désaccord et les deux sont faux
                self.misprediction_penalty_override += 1;
                self.override_count += 1;
            } else {
                // Cas 4: Accord et les deux sont faux
                self.agreement_incorrect += 1;
            }
            self.misprediction_penalty_full += 1;
            needs_full_flush = true;
        } else {
            // gshare correct, perceptron incorrect (rare)
            self.misprediction_penalty_full += 1;
            needs_full_flush = true;
        }

        needs_full_flush
    }
    
    /// Obtenir les statistiques de performance
    pub fn get_statistics(&self) -> OverridingPredictorStats {
        OverridingPredictorStats {
            gshare_accuracy: if self.gshare_correct + self.gshare_incorrect > 0 {
                self.gshare_correct as f64 / (self.gshare_correct + self.gshare_incorrect) as f64
            } else { 0.0 },
            perceptron_accuracy: if self.perceptron_correct + self.perceptron_incorrect > 0 {
                self.perceptron_correct as f64 / (self.perceptron_correct + self.perceptron_incorrect) as f64
            } else { 0.0 },
            override_rate: if self.gshare_correct + self.gshare_incorrect > 0 {
                self.override_count as f64 / (self.gshare_correct + self.gshare_incorrect) as f64
            } else { 0.0 },
            agreement_rate: if self.agreement_correct + self.agreement_incorrect + self.override_count > 0 {
                (self.agreement_correct + self.agreement_incorrect) as f64 / 
                (self.agreement_correct + self.agreement_incorrect + self.override_count) as f64
            } else { 0.0 },
            misprediction_penalty_full: self.misprediction_penalty_full,
            misprediction_penalty_override: self.misprediction_penalty_override,
        }
    }
}























//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     // Helper function pour les tests
//     fn check_prediction_sequence(
//         predictor: &mut BranchPredictor,
//         pc: u64,
//         sequence: &[(bool, bool)],
//     ) -> bool {
//         let mut all_correct = true;
//         for &(branch_taken, should_predict_taken) in sequence {
//             let prediction = predictor.predict(pc);
//             predictor.update(pc, branch_taken, prediction);
//
//             let predicted_taken = prediction == BranchPrediction::Taken;
//             if predicted_taken != should_predict_taken {
//                 all_correct = false;
//             }
//         }
//         all_correct
//     }
//
//     #[test]
//     fn test_static_predictor() {
//         let mut predictor = BranchPredictor::new(PredictorType::Static);
//
//         // Le prédicteur statique devrait toujours prédire Not Taken
//         assert_eq!(predictor.predict(0), BranchPrediction::NotTaken);
//         assert_eq!(predictor.predict(4), BranchPrediction::NotTaken);
//
//         // Vérifions que les métriques sont correctement mises à jour
//         predictor.update(0, true, BranchPrediction::NotTaken); // Mauvaise prédiction
//         predictor.update(4, false, BranchPrediction::NotTaken); // Bonne prédiction
//
//         assert_eq!(predictor.metrics.total_branches, 2);
//         assert_eq!(predictor.metrics.correct_predictions, 1);
//         assert_eq!(predictor.metrics.incorrect_predictions, 1);
//     }
//
//     #[test]
//     fn test_dynamic_predictor_initial_state() {
//         let mut predictor = BranchPredictor::new(PredictorType::Dynamic);
//
//         // Par défaut, devrait prédire Not Taken (état initial WeaklyNotTaken)
//         assert_eq!(predictor.predict(0), BranchPrediction::NotTaken);
//         assert_eq!(
//             predictor.two_bit_states.get(&0),
//             Some(&TwoBitState::WeaklyNotTaken)
//         );
//     }
//
//     /// Montre comment la transition est plus "conservatrice" pour ce test
//     #[test]
//     fn test_dynamic_predictor_learning() {
//         let mut predictor = BranchPredictor::new(PredictorType::Dynamic);
//         let pc = 0x1000;
//
//         let sequence = vec![
//             (true, false), // Initial WeaklyNotTaken -> WeaklyTaken
//             (true, true),  // WeaklyTaken -> StronglyTaken
//             (true, true),  // Reste StronglyTaken
//             (true, true),  // Reste StronglyTaken
//         ];
//
//         for (i, &(branch_taken, expected_prediction)) in sequence.iter().enumerate() {
//             let prediction = predictor.predict(pc);
//             let predicted_taken = prediction == BranchPrediction::Taken;
//             predictor.update(pc, branch_taken, prediction);
//
//             assert_eq!(
//                 predicted_taken,
//                 expected_prediction,
//                 "Itération {}: prédit {} mais attendait {}",
//                 i + 1,
//                 predicted_taken,
//                 expected_prediction
//             );
//         }
//     }
//
//     #[test]
//     fn test_dynamic_predictor_state_transitions() {
//         let mut predictor = BranchPredictor::new(PredictorType::Dynamic);
//         let pc = 0x2000;
//
//         // Test de transition d'états
//         predictor.update_dynamic(pc, true); // WeaklyNotTaken -> WeaklyTaken
//         assert_eq!(
//             predictor.two_bit_states.get(&pc),
//             Some(&TwoBitState::WeaklyTaken)
//         );
//
//         predictor.update_dynamic(pc, true); // WeaklyTaken -> StronglyTaken
//         assert_eq!(
//             predictor.two_bit_states.get(&pc),
//             Some(&TwoBitState::StronglyTaken)
//         );
//
//         predictor.update_dynamic(pc, false); // StronglyTaken -> WeaklyTaken
//         assert_eq!(
//             predictor.two_bit_states.get(&pc),
//             Some(&TwoBitState::WeaklyTaken)
//         );
//     }
//
//     #[test]
//     fn test_prediction_accuracy() {
//         let mut predictor = BranchPredictor::new(PredictorType::Dynamic);
//         let pc = 0x3000;
//
//         // Initialisation - état WeaklyNotTaken
//         // Suivons les transitions d'état et les prédictions pas à pas
//         let mut transitions = Vec::new();
//
//         // Séquence T-T-F-T
//         let sequence = [
//             // (branchement pris?, état actuel -> nouvel état, prédiction correcte?)
//             (true, "WeaklyNotTaken -> WeaklyTaken", false), // prédit NT, était T
//             (true, "WeaklyTaken -> StronglyTaken", true),   // prédit T, était T
//             (false, "StronglyTaken -> WeaklyTaken", false), // prédit T, était NT
//             (true, "WeaklyTaken -> StronglyTaken", true),   // prédit T, était T
//         ];
//
//         for (i, &(branch_taken, transition, expected_correct)) in sequence.iter().enumerate() {
//             let prediction = predictor.predict(pc);
//             predictor.update(pc, branch_taken, prediction);
//
//             let was_correct = match (prediction, branch_taken) {
//                 (BranchPrediction::Taken, true) | (BranchPrediction::NotTaken, false) => true,
//                 _ => false,
//             };
//
//             transitions.push(format!(
//                 "Étape {}: {} - prédit {}, était {}, {}",
//                 i + 1,
//                 transition,
//                 if prediction == BranchPrediction::Taken {
//                     "T"
//                 } else {
//                     "NT"
//                 },
//                 if branch_taken { "T" } else { "NT" },
//                 if was_correct { "correct" } else { "incorrect" }
//             ));
//
//             assert_eq!(
//                 was_correct,
//                 expected_correct,
//                 "Étape {} : attendait {}, obtenu {}.\nHistorique des transitions:\n{}",
//                 i + 1,
//                 expected_correct,
//                 was_correct,
//                 transitions.join("\n")
//             );
//         }
//
//         // Vérification finale des métriques
//         assert_eq!(
//             predictor.metrics.correct_predictions,
//             2,
//             "Devrait avoir exactement 2 prédictions correctes.\nHistorique des transitions:\n{}",
//             transitions.join("\n")
//         );
//         assert_eq!(
//             predictor.metrics.incorrect_predictions,
//             2,
//             "Devrait avoir exactement 2 prédictions incorrectes.\nHistorique des transitions:\n{}",
//             transitions.join("\n")
//         );
//         assert_eq!(predictor.metrics.total_branches, 4);
//     }
//
//     #[test]
//     fn test_multiple_branches() {
//         let mut predictor = BranchPredictor::new(PredictorType::Dynamic);
//
//         // Test avec deux branches différentes
//         let pc1 = 0x4000;
//         let pc2 = 0x4004;
//
//         // pc1 est toujours pris
//         predictor.predict(pc1);
//         predictor.update(pc1, true, BranchPrediction::NotTaken);
//         predictor.predict(pc1);
//         predictor.update(pc1, true, BranchPrediction::NotTaken);
//
//         // pc2 n'est jamais pris
//         predictor.predict(pc2);
//         predictor.update(pc2, false, BranchPrediction::NotTaken);
//         predictor.predict(pc2);
//         predictor.update(pc2, false, BranchPrediction::NotTaken);
//
//         // Vérifions que les deux branches ont des états différents
//         assert_ne!(
//             predictor.two_bit_states.get(&pc1),
//             predictor.two_bit_states.get(&pc2)
//         );
//     }
//
//     //////////////////
//     // Test pour vérifier le comportement avec une boucle
//     #[test]
//     fn test_dynamic_predictor_loop() {
//         let mut predictor = BranchPredictor::new(PredictorType::Dynamic);
//         let loop_branch = 0x1000;
//
//         // Simule une boucle qui s'exécute 5 fois puis sort
//         let sequence = vec![
//             (true, false), // 1ère itération - prédit NT, était T
//             (true, true),  // 2ème itération - prédit T
//             (true, true),  // 3ème itération - prédit T
//             (true, true),  // 4ème itération - prédit T
//             (true, true),  // 5ème itération - prédit T
//             (false, true), // Sortie de boucle - prédit T, était NT
//         ];
//
//         for (i, &(branch_taken, _)) in sequence.iter().enumerate() {
//             let prediction = predictor.predict(loop_branch);
//             predictor.update(loop_branch, branch_taken, prediction);
//
//             // Vérifie que le prédicteur apprend bien le pattern de la boucle
//             if i >= 2 {
//                 assert_eq!(
//                     prediction,
//                     BranchPrediction::Taken,
//                     "Le prédicteur devrait prédire Taken après 2 itérations"
//                 );
//             }
//         }
//
//         // Vérifie que le prédicteur commence à s'adapter après la sortie de boucle
//         let final_prediction = predictor.predict(loop_branch);
//         assert_eq!(
//             predictor.two_bit_states.get(&loop_branch),
//             Some(&TwoBitState::WeaklyTaken),
//             "L'état devrait être affaibli après une prédiction incorrecte"
//         );
//     }
//
//     // Test pour les branches alternantes (if/else alterné)
//     fn test_alternating_branches() {
//         let mut predictor = BranchPredictor::new(PredictorType::Dynamic);
//         let branch_pc = 0x2000;
//
//         // Séquence alternée T/NT/T/NT
//         let mut total_correct = 0;
//         let sequence_len = 8;
//
//         // Séquence alternée T/NT/T/NT...
//         for i in 0..sequence_len {
//             let branch_taken = i % 2 == 0; // alterne entre true et false
//
//             let prediction = predictor.predict(branch_pc);
//             let was_correct = match (prediction, branch_taken) {
//                 (BranchPrediction::Taken, true) | (BranchPrediction::NotTaken, false) => true,
//                 _ => false,
//             };
//             predictor.update(branch_pc, branch_taken, prediction);
//
//             if was_correct {
//                 total_correct += 1;
//             }
//         }
//
//         let accuracy = total_correct as f64 / sequence_len as f64;
//         assert!(
//             accuracy <= 0.5,
//             "Sur un pattern alterné, l'accuracy devrait être faible (était: {})",
//             accuracy
//         );
//         assert!(accuracy > 0.0, "L'accuracy ne devrait pas être nulle");
//     }
//
//     // Test des transitions rapides
//     #[test]
//     fn test_rapid_state_changes() {
//         let mut predictor = BranchPredictor::new(PredictorType::Dynamic);
//         let pc = 0x3000;
//
//         // Test de changements rapides d'état
//         predictor.update_dynamic(pc, true); // WeaklyNotTaken -> WeaklyTaken
//         predictor.update_dynamic(pc, true); // WeaklyTaken -> StronglyTaken
//         assert_eq!(
//             predictor.two_bit_states.get(&pc),
//             Some(&TwoBitState::StronglyTaken),
//             "Devrait atteindre StronglyTaken après 2 branches prises"
//         );
//
//         predictor.update_dynamic(pc, false); // StronglyTaken -> WeaklyTaken
//         predictor.update_dynamic(pc, false); // WeaklyTaken -> WeaklyNotTaken
//         assert_eq!(
//             predictor.two_bit_states.get(&pc),
//             Some(&TwoBitState::WeaklyNotTaken),
//             "Devrait atteindre WeaklyNotTaken après 2 branches non prises"
//         );
//     }
//
//     // Test de saturation
//     #[test]
//     fn test_saturation_behavior() {
//         let mut predictor = BranchPredictor::new(PredictorType::Dynamic);
//         let pc = 0x4000;
//
//         // Pousse vers StronglyTaken
//         for _ in 0..4 {
//             predictor.update_dynamic(pc, true);
//         }
//         assert_eq!(
//             predictor.two_bit_states.get(&pc),
//             Some(&TwoBitState::StronglyTaken),
//             "Devrait rester en StronglyTaken après multiples branches prises"
//         );
//
//         // Pousse vers StronglyNotTaken
//         for _ in 0..4 {
//             predictor.update_dynamic(pc, false);
//         }
//         assert_eq!(
//             predictor.two_bit_states.get(&pc),
//             Some(&TwoBitState::StronglyNotTaken),
//             "Devrait atteindre et rester en StronglyNotTaken"
//         );
//     }
//
//     // Test de plusieurs branches en parallèle
//     #[test]
//     fn test_multiple_branch_histories() {
//         let mut predictor = BranchPredictor::new(PredictorType::Dynamic);
//         let branch1 = 0x5000;
//         let branch2 = 0x5004;
//         let branch3 = 0x5008;
//
//         // Branch1 : toujours pris
//         for _ in 0..3 {
//             predictor.update_dynamic(branch1, true);
//         }
//
//         // Branch2 : jamais pris
//         for _ in 0..3 {
//             predictor.update_dynamic(branch2, false);
//         }
//
//         // Branch3 : alterné
//         predictor.update_dynamic(branch3, true);
//         predictor.update_dynamic(branch3, false);
//         predictor.update_dynamic(branch3, true);
//
//         // Vérifie que chaque branche a son propre historique
//         assert_eq!(
//             predictor.two_bit_states.get(&branch1),
//             Some(&TwoBitState::StronglyTaken)
//         );
//         assert_eq!(
//             predictor.two_bit_states.get(&branch2),
//             Some(&TwoBitState::StronglyNotTaken)
//         );
//         assert_ne!(
//             predictor.two_bit_states.get(&branch3),
//             predictor.two_bit_states.get(&branch1)
//         );
//     }
// }

// src/pvm/branch_prediction.rs

use std::collections::HashMap;


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

struct GSharePredictor {
    global_history: u16,
    pattern_table: Vec<u8>,
    history_length: usize,
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
}

#[derive(Debug, Clone,Copy)]
pub struct TwoBitCounter {
    state: TwoBitState,
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
        
        Self {
            prediction_type: predictor_type,
            predictions: HashMap::new(),
            two_bit_states: HashMap::new(),
            metrics: BranchMetrics::default(),
            hybrid_predictor,
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
        
        Self {
            prediction_type: predictor_type,
            predictions: HashMap::new(),
            two_bit_states: HashMap::new(),
            metrics: BranchMetrics::default(),
            hybrid_predictor,
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
                // PerceptronPredictor
                BranchPrediction::NotTaken
            }
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

}



impl BranchTargetBuffer {
    // pub fn predict_with_confidence(&mut self, pc: u64) -> Option<(u32, u8)> {
    //     let index = self.get_index(pc);
    //     let tag = self.get_tag(pc);
    //
    //     if let Some(entry) = &self.entries[index] {
    //         if entry.valid && entry.tag == tag {
    //             return Some((entry.target, entry.confidence));
    //         }
    //     }
    //     None
    // }
    //
    // fn get_index(&self, pc: u64) -> usize {
    //     (pc as usize) % self.size
    // }
    //
    // pub fn update_with_confidence(&mut self, pc: u64, target: u32, correct: bool) {
    //     let index = self.get_index(pc);
    //     let tag = self.get_tag(pc);
    //
    //     match &mut self.entries[index] {
    //         Some(entry) if entry.tag == tag => {
    //             entry.target = target;
    //             if correct {
    //                 entry.confidence = (entry.confidence + 1).min(255);
    //             } else {
    //                 entry.confidence = entry.confidence.saturating_sub(10);
    //             }
    //             entry.last_used = self.current_cycle;
    //         },
    //         _ => {
    //             // Nouvelle entrée
    //             self.entries[index] = Some(BTBEntry {
    //                 tag,
    //                 target,
    //                 valid: true,
    //                 confidence: if correct { 128 } else { 64 },
    //                 last_used: self.current_cycle,
    //             });
    //         }
    //     }
    // }
}
























#[cfg(test)]
mod tests {
    use super::*;

    // Helper function pour les tests
    fn check_prediction_sequence(
        predictor: &mut BranchPredictor,
        pc: u64,
        sequence: &[(bool, bool)],
    ) -> bool {
        let mut all_correct = true;
        for &(branch_taken, should_predict_taken) in sequence {
            let prediction = predictor.predict(pc);
            predictor.update(pc, branch_taken, prediction);

            let predicted_taken = prediction == BranchPrediction::Taken;
            if predicted_taken != should_predict_taken {
                all_correct = false;
            }
        }
        all_correct
    }

    #[test]
    fn test_static_predictor() {
        let mut predictor = BranchPredictor::new(PredictorType::Static);

        // Le prédicteur statique devrait toujours prédire Not Taken
        assert_eq!(predictor.predict(0), BranchPrediction::NotTaken);
        assert_eq!(predictor.predict(4), BranchPrediction::NotTaken);

        // Vérifions que les métriques sont correctement mises à jour
        predictor.update(0, true, BranchPrediction::NotTaken); // Mauvaise prédiction
        predictor.update(4, false, BranchPrediction::NotTaken); // Bonne prédiction

        assert_eq!(predictor.metrics.total_branches, 2);
        assert_eq!(predictor.metrics.correct_predictions, 1);
        assert_eq!(predictor.metrics.incorrect_predictions, 1);
    }

    #[test]
    fn test_dynamic_predictor_initial_state() {
        let mut predictor = BranchPredictor::new(PredictorType::Dynamic);

        // Par défaut, devrait prédire Not Taken (état initial WeaklyNotTaken)
        assert_eq!(predictor.predict(0), BranchPrediction::NotTaken);
        assert_eq!(
            predictor.two_bit_states.get(&0),
            Some(&TwoBitState::WeaklyNotTaken)
        );
    }

    /// Montre comment la transition est plus "conservatrice" pour ce test
    #[test]
    fn test_dynamic_predictor_learning() {
        let mut predictor = BranchPredictor::new(PredictorType::Dynamic);
        let pc = 0x1000;

        let sequence = vec![
            (true, false), // Initial WeaklyNotTaken -> WeaklyTaken
            (true, true),  // WeaklyTaken -> StronglyTaken
            (true, true),  // Reste StronglyTaken
            (true, true),  // Reste StronglyTaken
        ];

        for (i, &(branch_taken, expected_prediction)) in sequence.iter().enumerate() {
            let prediction = predictor.predict(pc);
            let predicted_taken = prediction == BranchPrediction::Taken;
            predictor.update(pc, branch_taken, prediction);

            assert_eq!(
                predicted_taken,
                expected_prediction,
                "Itération {}: prédit {} mais attendait {}",
                i + 1,
                predicted_taken,
                expected_prediction
            );
        }
    }

    #[test]
    fn test_dynamic_predictor_state_transitions() {
        let mut predictor = BranchPredictor::new(PredictorType::Dynamic);
        let pc = 0x2000;

        // Test de transition d'états
        predictor.update_dynamic(pc, true); // WeaklyNotTaken -> WeaklyTaken
        assert_eq!(
            predictor.two_bit_states.get(&pc),
            Some(&TwoBitState::WeaklyTaken)
        );

        predictor.update_dynamic(pc, true); // WeaklyTaken -> StronglyTaken
        assert_eq!(
            predictor.two_bit_states.get(&pc),
            Some(&TwoBitState::StronglyTaken)
        );

        predictor.update_dynamic(pc, false); // StronglyTaken -> WeaklyTaken
        assert_eq!(
            predictor.two_bit_states.get(&pc),
            Some(&TwoBitState::WeaklyTaken)
        );
    }

    #[test]
    fn test_prediction_accuracy() {
        let mut predictor = BranchPredictor::new(PredictorType::Dynamic);
        let pc = 0x3000;

        // Initialisation - état WeaklyNotTaken
        // Suivons les transitions d'état et les prédictions pas à pas
        let mut transitions = Vec::new();

        // Séquence T-T-F-T
        let sequence = [
            // (branchement pris?, état actuel -> nouvel état, prédiction correcte?)
            (true, "WeaklyNotTaken -> WeaklyTaken", false), // prédit NT, était T
            (true, "WeaklyTaken -> StronglyTaken", true),   // prédit T, était T
            (false, "StronglyTaken -> WeaklyTaken", false), // prédit T, était NT
            (true, "WeaklyTaken -> StronglyTaken", true),   // prédit T, était T
        ];

        for (i, &(branch_taken, transition, expected_correct)) in sequence.iter().enumerate() {
            let prediction = predictor.predict(pc);
            predictor.update(pc, branch_taken, prediction);

            let was_correct = match (prediction, branch_taken) {
                (BranchPrediction::Taken, true) | (BranchPrediction::NotTaken, false) => true,
                _ => false,
            };

            transitions.push(format!(
                "Étape {}: {} - prédit {}, était {}, {}",
                i + 1,
                transition,
                if prediction == BranchPrediction::Taken {
                    "T"
                } else {
                    "NT"
                },
                if branch_taken { "T" } else { "NT" },
                if was_correct { "correct" } else { "incorrect" }
            ));

            assert_eq!(
                was_correct,
                expected_correct,
                "Étape {} : attendait {}, obtenu {}.\nHistorique des transitions:\n{}",
                i + 1,
                expected_correct,
                was_correct,
                transitions.join("\n")
            );
        }

        // Vérification finale des métriques
        assert_eq!(
            predictor.metrics.correct_predictions,
            2,
            "Devrait avoir exactement 2 prédictions correctes.\nHistorique des transitions:\n{}",
            transitions.join("\n")
        );
        assert_eq!(
            predictor.metrics.incorrect_predictions,
            2,
            "Devrait avoir exactement 2 prédictions incorrectes.\nHistorique des transitions:\n{}",
            transitions.join("\n")
        );
        assert_eq!(predictor.metrics.total_branches, 4);
    }

    #[test]
    fn test_multiple_branches() {
        let mut predictor = BranchPredictor::new(PredictorType::Dynamic);

        // Test avec deux branches différentes
        let pc1 = 0x4000;
        let pc2 = 0x4004;

        // pc1 est toujours pris
        predictor.predict(pc1);
        predictor.update(pc1, true, BranchPrediction::NotTaken);
        predictor.predict(pc1);
        predictor.update(pc1, true, BranchPrediction::NotTaken);

        // pc2 n'est jamais pris
        predictor.predict(pc2);
        predictor.update(pc2, false, BranchPrediction::NotTaken);
        predictor.predict(pc2);
        predictor.update(pc2, false, BranchPrediction::NotTaken);

        // Vérifions que les deux branches ont des états différents
        assert_ne!(
            predictor.two_bit_states.get(&pc1),
            predictor.two_bit_states.get(&pc2)
        );
    }

    //////////////////
    // Test pour vérifier le comportement avec une boucle
    #[test]
    fn test_dynamic_predictor_loop() {
        let mut predictor = BranchPredictor::new(PredictorType::Dynamic);
        let loop_branch = 0x1000;

        // Simule une boucle qui s'exécute 5 fois puis sort
        let sequence = vec![
            (true, false), // 1ère itération - prédit NT, était T
            (true, true),  // 2ème itération - prédit T
            (true, true),  // 3ème itération - prédit T
            (true, true),  // 4ème itération - prédit T
            (true, true),  // 5ème itération - prédit T
            (false, true), // Sortie de boucle - prédit T, était NT
        ];

        for (i, &(branch_taken, _)) in sequence.iter().enumerate() {
            let prediction = predictor.predict(loop_branch);
            predictor.update(loop_branch, branch_taken, prediction);

            // Vérifie que le prédicteur apprend bien le pattern de la boucle
            if i >= 2 {
                assert_eq!(
                    prediction,
                    BranchPrediction::Taken,
                    "Le prédicteur devrait prédire Taken après 2 itérations"
                );
            }
        }

        // Vérifie que le prédicteur commence à s'adapter après la sortie de boucle
        let final_prediction = predictor.predict(loop_branch);
        assert_eq!(
            predictor.two_bit_states.get(&loop_branch),
            Some(&TwoBitState::WeaklyTaken),
            "L'état devrait être affaibli après une prédiction incorrecte"
        );
    }

    // Test pour les branches alternantes (if/else alterné)
    fn test_alternating_branches() {
        let mut predictor = BranchPredictor::new(PredictorType::Dynamic);
        let branch_pc = 0x2000;

        // Séquence alternée T/NT/T/NT
        let mut total_correct = 0;
        let sequence_len = 8;

        // Séquence alternée T/NT/T/NT...
        for i in 0..sequence_len {
            let branch_taken = i % 2 == 0; // alterne entre true et false

            let prediction = predictor.predict(branch_pc);
            let was_correct = match (prediction, branch_taken) {
                (BranchPrediction::Taken, true) | (BranchPrediction::NotTaken, false) => true,
                _ => false,
            };
            predictor.update(branch_pc, branch_taken, prediction);

            if was_correct {
                total_correct += 1;
            }
        }

        let accuracy = total_correct as f64 / sequence_len as f64;
        assert!(
            accuracy <= 0.5,
            "Sur un pattern alterné, l'accuracy devrait être faible (était: {})",
            accuracy
        );
        assert!(accuracy > 0.0, "L'accuracy ne devrait pas être nulle");
    }

    // Test des transitions rapides
    #[test]
    fn test_rapid_state_changes() {
        let mut predictor = BranchPredictor::new(PredictorType::Dynamic);
        let pc = 0x3000;

        // Test de changements rapides d'état
        predictor.update_dynamic(pc, true); // WeaklyNotTaken -> WeaklyTaken
        predictor.update_dynamic(pc, true); // WeaklyTaken -> StronglyTaken
        assert_eq!(
            predictor.two_bit_states.get(&pc),
            Some(&TwoBitState::StronglyTaken),
            "Devrait atteindre StronglyTaken après 2 branches prises"
        );

        predictor.update_dynamic(pc, false); // StronglyTaken -> WeaklyTaken
        predictor.update_dynamic(pc, false); // WeaklyTaken -> WeaklyNotTaken
        assert_eq!(
            predictor.two_bit_states.get(&pc),
            Some(&TwoBitState::WeaklyNotTaken),
            "Devrait atteindre WeaklyNotTaken après 2 branches non prises"
        );
    }

    // Test de saturation
    #[test]
    fn test_saturation_behavior() {
        let mut predictor = BranchPredictor::new(PredictorType::Dynamic);
        let pc = 0x4000;

        // Pousse vers StronglyTaken
        for _ in 0..4 {
            predictor.update_dynamic(pc, true);
        }
        assert_eq!(
            predictor.two_bit_states.get(&pc),
            Some(&TwoBitState::StronglyTaken),
            "Devrait rester en StronglyTaken après multiples branches prises"
        );

        // Pousse vers StronglyNotTaken
        for _ in 0..4 {
            predictor.update_dynamic(pc, false);
        }
        assert_eq!(
            predictor.two_bit_states.get(&pc),
            Some(&TwoBitState::StronglyNotTaken),
            "Devrait atteindre et rester en StronglyNotTaken"
        );
    }

    // Test de plusieurs branches en parallèle
    #[test]
    fn test_multiple_branch_histories() {
        let mut predictor = BranchPredictor::new(PredictorType::Dynamic);
        let branch1 = 0x5000;
        let branch2 = 0x5004;
        let branch3 = 0x5008;

        // Branch1 : toujours pris
        for _ in 0..3 {
            predictor.update_dynamic(branch1, true);
        }

        // Branch2 : jamais pris
        for _ in 0..3 {
            predictor.update_dynamic(branch2, false);
        }

        // Branch3 : alterné
        predictor.update_dynamic(branch3, true);
        predictor.update_dynamic(branch3, false);
        predictor.update_dynamic(branch3, true);

        // Vérifie que chaque branche a son propre historique
        assert_eq!(
            predictor.two_bit_states.get(&branch1),
            Some(&TwoBitState::StronglyTaken)
        );
        assert_eq!(
            predictor.two_bit_states.get(&branch2),
            Some(&TwoBitState::StronglyNotTaken)
        );
        assert_ne!(
            predictor.two_bit_states.get(&branch3),
            predictor.two_bit_states.get(&branch1)
        );
    }
}

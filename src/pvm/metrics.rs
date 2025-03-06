// //src/pvm/metrics.rs
// use std::collections::HashMap;
// use crate::pvm::branch_predictor::BranchMetrics;
// use crate::pvm::forwardings::ForwardingSource;
// use crate::pvm::hazards::HazardType;
// use crate::pvm::instructions::{Address, ArithmeticOp, DecodedInstruction, Instruction, MemoryOp, RegisterId};
// use crate::pvm::memorys::MemoryController;
// use crate::pvm::pipelines::{Pipeline, PipelineStats};
// use crate::pvm::registers::RegisterBank;
//
// #[derive(Debug, Default,Clone)]  // Ajout de Default
// pub struct PipelineMetrics {
//     pub total_cycles: u64,
//     pub total_instructions: u64,
//     pub ipc: f64,
//     pub fetch_metrics: StageMetrics,
//     pub decode_metrics: StageMetrics,
//     pub execute_metrics: StageMetrics,
//     pub memory_metrics_stage: StageMetrics,
//     pub writeback_metrics: StageMetrics,
//     pub hazard_metrics: HazardMetrics,
//     pub forwarding_metrics: ForwardingMetrics,
//     pub memory_metrics: MemoryAccessMetrics,
//     pub cache_metrics: CacheMetrics,
//
//     pub reorder_count: u64,
//     pub bypass_hits: u64,
//     pub fetch_buffer_utilization: f64,
//
//     pub branch_metrics: BranchMetrics,
//     pub flush_count: usize,
//
// }
//
// #[derive(Debug, Default, Clone)]  // Ajout de Default
// pub struct StageMetrics {
//     pub stall_cycles: u64,
//     pub busy_cycles: u64,
//     pub utilization: f64,
// }
//
// #[derive(Debug, Default, Clone)]  // Ajout de Default
// pub struct HazardMetrics {
//     pub total_hazards: u64,
//     pub data_hazards: u64,
//     pub control_hazards: u64,
//     pub structural_hazards: u64,
//     pub load_use_hazards: u64,
//     pub store_load_hazards: u64,
//     pub resolved_by_forwarding: u64,
//
// }
//
// #[derive(Debug, Default, Clone)]  // Ajout de Default
// pub struct ForwardingMetrics {
//     pub total_forwards: u64,
//     pub successful_forwards: u64,
//     pub failed_forwards: u64,
//     pub forward_sources: HashMap<ForwardingSource, u64>,
// }
//
// #[derive(Debug, Default, Clone)]  // Ajout de Default
// pub struct MemoryAccessMetrics {
//     pub total_accesses: u64,
//     pub reads: u64,
//     pub writes: u64,
//     pub cache_hits: u64,
//     pub cache_misses: u64,
//     pub average_access_time: f64,
// }
// #[derive(Debug, Default, Clone)]
// pub struct CacheMetrics {
//     pub total_accesses: u64,
//     pub reads: u64,
//     pub writes: u64,
//     pub cache_hits: u64,
//     pub cache_misses: u64,
//     pub average_access_time: f64,
//
//     // pub reorder_count: u64,
//     // pub bypass_hits: u64,
//     // pub fetch_buffer_utilisation: f64,
// }
//
// impl StageMetrics{
//     pub fn update(&mut self, is_busy: bool) {
//         if is_busy {
//             self.busy_cycles += 1;
//         } else {
//             self.stall_cycles += 1;
//         }
//         self.utilization = if self.busy_cycles + self.stall_cycles > 0 {
//             self.busy_cycles as f64 / (self.busy_cycles + self.stall_cycles) as f64
//         } else {
//             0.0
//         };
//     }
//
// }
//
// impl HazardMetrics {
//     pub fn update(&mut self, stats: &PipelineStats) {
//         self.total_hazards = stats.hazards as u64;
//         self.store_load_hazards = stats.store_load_hazards as u64;
//         self.load_use_hazards = stats.load_use_hazards as u64;
//         self.data_hazards = stats.data_dependency_hazards as u64;
//
//     }
// }
//
//
// impl Pipeline {
//
//     pub fn update_stage_metrics(&mut self) {
//         // Mise à jour des métriques de stage
//         if self.fetch_state.instruction.is_none() {
//             self.metrics.fetch_metrics.stall_cycles += 1;
//         } else {
//             self.metrics.fetch_metrics.busy_cycles += 1;
//         }
//
//         // Mise à jour des métriques d'exécution
//         if self.execute_state.decoded.is_some() {
//             self.metrics.execute_metrics.busy_cycles += 1;
//         }
//
//         // Mise à jour des métriques mémoire
//         if let Some(decoded) = &self.memory_state.decoded {
//             match decoded {
//                 DecodedInstruction::Memory(MemoryOp::Load { .. }) |
//                 DecodedInstruction::Memory(MemoryOp::Store { .. }) => {
//                     self.metrics.memory_metrics.total_accesses += 1;
//                 }
//                 _ => {}
//             }
//         }
//
//         // Calcul de l'IPC
//         if self.metrics.total_cycles > 0 {
//             self.metrics.ipc = self.metrics.total_instructions as f64 / self.metrics.total_cycles as f64;
//         }
//     }
//
//
//     // Méthode pour générer un rapport de performance
//     pub fn generate_performance_report(&self) -> String {
//         let mut report = String::new();
//         report.push_str("\n=== Pipeline Performance Report ===\n");
//
//         // Métriques générales
//         report.push_str(&format!("Total Cycles: {}\n", self.metrics.total_cycles));
//         report.push_str(&format!("Total Instructions: {}\n", self.metrics.total_instructions));
//         report.push_str(&format!("IPC: {:.2}\n", self.metrics.ipc));
//
//         // Métriques de hazards
//         report.push_str("\n=== Hazard Statistics ===\n");
//         report.push_str(&format!("Total Hazards: {}\n", self.metrics.hazard_metrics.total_hazards));
//         report.push_str(&format!("Data Hazards: {}\n", self.metrics.hazard_metrics.data_hazards));
//         report.push_str(&format!("Load-Use Hazards: {}\n", self.metrics.hazard_metrics.load_use_hazards));
//
//         report
//     }
//
//     pub fn update_hazard_metrics(&mut self) {
//         if let Some(hazard) = &self.current_hazard {
//             self.metrics.hazard_metrics.total_hazards += 1;
//             match hazard {
//                 HazardType::DataDependency => self.metrics.hazard_metrics.data_hazards += 1,
//                 HazardType::LoadUse => self.metrics.hazard_metrics.load_use_hazards += 1,
//                 HazardType::StoreLoad => self.metrics.hazard_metrics.store_load_hazards += 1,
//             }
//         }
//     }
//
//     pub fn update_forwarding_metrics(&mut self) {
//         if self.forwarding_attempted {
//             self.metrics.forwarding_metrics.total_forwards += 1;
//             if self.forwarding_successful {
//                 self.metrics.forwarding_metrics.successful_forwards += 1;
//             } else {
//                 self.metrics.forwarding_metrics.failed_forwards += 1;
//             }
//         }
//     }
//
//     fn update_memory_metrics(&mut self) {
//         if self.memory_access_in_progress {
//             self.metrics.memory_metrics.total_accesses += 1;
//             if self.cache_hit {
//                 self.metrics.memory_metrics.cache_hits += 1;
//             } else {
//                 self.metrics.memory_metrics.cache_misses += 1;
//             }
//         }
//     }
//
//     pub fn update_metrics(&mut self) {
//         if self.writeback_state.result.is_some() {
//             self.metrics.total_instructions += 1;
//         }
//
//         // Mise à jour des métriques de hazards
//         self.metrics.hazard_metrics.total_hazards = self.stats.hazards as u64;
//         self.metrics.hazard_metrics.store_load_hazards = self.stats.store_load_hazards as u64;
//         self.metrics.hazard_metrics.load_use_hazards = self.stats.load_use_hazards as u64;
//         self.metrics.hazard_metrics.data_hazards = self.stats.data_dependency_hazards as u64;
//
//         // Calcul de l'IPC
//         if self.metrics.total_cycles > 0 {
//             self.metrics.ipc = self.metrics.total_instructions as f64 / self.metrics.total_cycles as f64;
//         }
//     }
//
//
//     // Helper methods
//     pub fn needs_forwarding(&self, decoded: &DecodedInstruction) -> bool {
//         match decoded {
//             DecodedInstruction::Arithmetic(op) => {
//                 matches!(op,
//                     ArithmeticOp::Add { .. } |
//                     ArithmeticOp::Sub { .. } |
//                     ArithmeticOp::Mul { .. } |
//                     ArithmeticOp::Div { .. }
//                 )
//             }
//             DecodedInstruction::Memory(MemoryOp::Store { .. }) => true,
//             _ => false
//         }
//     }
//
//     pub fn get_source_register(&self, decoded: &DecodedInstruction) -> RegisterId {
//         match decoded {
//             DecodedInstruction::Arithmetic(op) => {
//                 match op {
//                     ArithmeticOp::Add { src1, .. } => *src1,
//                     ArithmeticOp::Sub { src1, .. } => *src1,
//                     ArithmeticOp::Mul { src1, .. } => *src1,
//                     ArithmeticOp::Div { src1, .. } => *src1,
//                 }
//             }
//             DecodedInstruction::Memory(MemoryOp::Store { reg, .. }) => *reg,
//             _ => RegisterId(0), // Default case
//         }
//     }
//
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
//     pub fn get_branch_stats(&self) -> String {
//         let metrics = &self.branch_predictor.metrics;
//         let accuracy = if metrics.total_branches > 0 {
//             (metrics.correct_predictions as f64 / metrics.total_branches as f64) * 100.0
//         } else {
//             0.0
//         };
//
//         format!(
//             "Branch Prediction Stats:\n\
//              Total Branches: {}\n\
//              Correct Predictions: {}\n\
//              Incorrect Predictions: {}\n\
//              Accuracy: {:.2}%\n\
//              Pipeline Flushes: {}\n",
//             metrics.total_branches,
//             metrics.correct_predictions,
//             metrics.incorrect_predictions,
//             accuracy,
//             self.metrics.flush_count
//         )
//     }
//
//
//
// }
//
//
//
// #[cfg(test)]
// mod tests {
//     use crate::pvm::instructions::{Address, Instruction, RegisterId};
//     use crate::pvm::memorys::MemoryController;
//     use crate::pvm::registers::RegisterBank;
//     use super::*;
//
//
//     fn setup_test_env() -> (Pipeline, RegisterBank, MemoryController) {
//         let pipeline = Pipeline::new();
//         let register_bank = RegisterBank::new(8).unwrap();  // 8 registres
//         let memory_controller = MemoryController::new(1024, 256).unwrap(); // 1KB mémoire, 256B cache
//         (pipeline, register_bank, memory_controller)
//     }
//
//     //src/pvm/metrics.rs
//     #[test]
//     fn test_performance_metrics() {
//         let (mut pipeline, mut register_bank, mut memory_controller) = setup_test_env();
//
//         let program = vec![
//             Instruction::LoadImm(RegisterId(0), 42),
//             Instruction::Store(RegisterId(0), Address(100)),
//             Instruction::Load(RegisterId(1), Address(100)),  // Devrait causer un Store-Load hazard
//             Instruction::Add(RegisterId(2), RegisterId(1), RegisterId(0)),  // Devrait causer un Load-Use hazard
//         ];
//
//         pipeline.load_instructions(program).unwrap();
//
//         let mut cycles = 0;
//         while !pipeline.is_empty().unwrap() {
//             pipeline.cycle(&mut register_bank, &mut memory_controller).unwrap();
//             cycles += 1;
//             if cycles > 20 { // Protection contre les boucles infinies
//                 break;
//             }
//         }
//
//         // Vérifications détaillées
//         assert!(pipeline.metrics.total_cycles > 0, "Cycles total devrait être > 0");
//         assert!(pipeline.metrics.total_instructions > 0, "Instructions total devrait être > 0");
//         assert!(pipeline.metrics.hazard_metrics.total_hazards > 0, "Au moins un hazard devrait être détecté");
//         assert!(pipeline.metrics.hazard_metrics.store_load_hazards > 0, "Un Store-Load hazard devrait être détecté");
//
//         // Affichage des métriques pour le débogage
//         println!("Métriques finales:");
//         println!("Total cycles: {}", pipeline.metrics.total_cycles);
//         println!("Total instructions: {}", pipeline.metrics.total_instructions);
//         println!("Total hazards: {}", pipeline.metrics.hazard_metrics.total_hazards);
//         println!("Store-Load hazards: {}", pipeline.metrics.hazard_metrics.store_load_hazards);
//         println!("Load-Use hazards: {}", pipeline.metrics.hazard_metrics.load_use_hazards);
//     }
//
//     #[test]
//     fn test_pipeline_metrics() {
//         let (mut pipeline, mut register_bank, mut memory_controller) = setup_test_env();
//
//         let program = vec![
//             Instruction::LoadImm(RegisterId(0), 42),
//             Instruction::Store(RegisterId(0), Address(100)),
//             Instruction::Load(RegisterId(1), Address(100)),
//             Instruction::Add(RegisterId(2), RegisterId(1), RegisterId(0)),
//         ];
//
//         pipeline.load_instructions(program).unwrap();
//
//         while !pipeline.is_empty().unwrap() {
//             pipeline.cycle(&mut register_bank, &mut memory_controller).unwrap();
//         }
//
//         // Vérification des métriques
//         assert!(pipeline.metrics.total_cycles > 0);
//         assert!(pipeline.metrics.total_instructions > 0);
//         assert!(pipeline.metrics.ipc > 0.0);
//
//         // Affichage du rapport
//         println!("{}", pipeline.generate_performance_report());
//     }
// }
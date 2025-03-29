// //src/pvm/optimizings.rs
// use crate::pvm::instructions::{ArithmeticOp, DecodedInstruction};
// use crate::pvm::pipelines::Pipeline;
// use crate::pvm::vm_errors::VMResult;
//
// impl Pipeline {
//
//     pub fn optimize_instruction_ordering(&mut self) -> VMResult<()> {
//         if self.execute_state.decoded.is_some() {
//             if self.will_cause_stall(&self.execute_state.decoded.as_ref().unwrap()) {
//                 if let Some(next_instr) = self.peek_next_instruction() {
//                     if !self.will_cause_stall(&next_instr) {
//                         self.swap_instructions()?;
//                         self.metrics.reorder_count += 1;
//                     }
//                 }
//             }
//         }
//         Ok(())
//     }
//
//
//     fn will_cause_stall(&self, decoded: &DecodedInstruction) -> bool {
//         match decoded {
//             DecodedInstruction::Memory(_) => true,
//             DecodedInstruction::Arithmetic(op) => {
//                 self.has_pending_dependencies(op)
//             }
//             _ => false
//         }
//     }
//
//     fn optimize_fetch_stage(&mut self) -> VMResult<()> {
//         // Utiliser un fetch buffer pour réduire les stalls
//         if self.fetch_buffer.is_empty() && !self.instruction_buffer.is_empty() {
//             // Précharger plusieurs instructions
//             for _ in 0..4 {  // Fetch width de 4
//                 if let Some(instr) = self.instruction_buffer.pop_front() {
//                     self.fetch_buffer.push_back(instr);
//                 }
//             }
//         }
//
//         // Si le decode stage est libre
//         if self.decode_state.instruction.is_none() {
//             if let Some(instr) = self.fetch_buffer.pop_front() {
//                 self.decode_state.instruction = Some(instr);
//                 self.stats.instructions_fetched += 1;
//             }
//         }
//
//         Ok(())
//     }
//
//     fn optimize_fetch_buffer(&mut self) -> VMResult<()> {
//         if self.fetch_buffer.is_empty() && !self.instruction_buffer.is_empty() {
//             for _ in 0..4 {
//                 if let Some(instr) = self.instruction_buffer.pop_front() {
//                     self.fetch_buffer.push_back(instr);
//                 }
//             }
//         }
//
//         if self.decode_state.instruction.is_none() {
//             if let Some(instr) = self.fetch_buffer.pop_front() {
//                 self.decode_state.instruction = Some(instr);
//                 self.stats.instructions_fetched += 1;
//             }
//         }
//
//         Ok(())
//     }
//
//     pub fn peek_next_instruction(&self) -> Option<DecodedInstruction> {
//         if let Some(instruction) = self.instruction_buffer.front() {
//             self.decode_instruction(instruction).ok()
//         } else {
//             None
//         }
//     }
//
//     pub fn swap_instructions(&mut self) -> VMResult<()> {
//         if let Some(current) = self.execute_state.decoded.take() {
//             if let Some(next) = self.instruction_buffer.pop_front() {
//                 self.instruction_buffer.push_front(current.into()); // Maintenant cela fonctionne
//                 self.execute_state.decoded = Some(next.into());     // Conversion dans l'autre sens
//             }
//         }
//         Ok(())
//     }
//
//     pub fn has_pending_dependencies(&self, op: &ArithmeticOp) -> bool {
//         match op {
//             ArithmeticOp::Add { src1, src2, .. } |
//             ArithmeticOp::Sub { src1, src2, .. } |
//             ArithmeticOp::Mul { src1, src2, .. } |
//             ArithmeticOp::Div { src1, src2, .. } => {
//                 !self.is_register_ready(*src1) || !self.is_register_ready(*src2)
//             }
//         }
//     }
//
// }
//
//
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::pvm::instructions::{Instruction, RegisterId, Address};
//     use crate::pvm::registers::RegisterBank;
//     use crate::pvm::memorys::MemoryController;
//
//     fn setup_test_env() -> (Pipeline, RegisterBank, MemoryController) {
//         let pipeline = Pipeline::new();
//         let register_bank = RegisterBank::new(8).unwrap();  // 8 registres
//         let memory_controller = MemoryController::new(1024, 256).unwrap(); // 1KB mémoire, 256B cache
//         (pipeline, register_bank, memory_controller)
//     }
//
//     fn create_complex_test_program() -> Vec<Instruction> {
//         vec![
//             // Programme complexe pour tester les optimisations
//             // Test 1: Dépendances de données
//             Instruction::LoadImm(RegisterId(0), 10),
//             Instruction::Add(RegisterId(1), RegisterId(0), RegisterId(0)),
//             Instruction::Mul(RegisterId(2), RegisterId(1), RegisterId(0)),
//
//             // Test 2: Accès mémoire avec hazards
//             Instruction::Store(RegisterId(2), Address(100)),
//             Instruction::Load(RegisterId(3), Address(100)),
//
//             // Test 3: Opérations arithmétiques en chaîne
//             Instruction::Add(RegisterId(4), RegisterId(3), RegisterId(1)),
//             Instruction::Sub(RegisterId(5), RegisterId(4), RegisterId(2)),
//
//             // Test 4: Mixture d'opérations
//             Instruction::LoadImm(RegisterId(6), 42),
//             Instruction::Store(RegisterId(6), Address(200)),
//             Instruction::Load(RegisterId(7), Address(200)),
//             Instruction::Mul(RegisterId(0), RegisterId(7), RegisterId(6)),
//         ]
//     }
//
//
//     #[test]
//     fn test_pipeline_performance_optimized() {
//         let (mut pipeline, mut register_bank, mut memory_controller) = setup_test_env();
//
//         // Programme de test avec mix d'instructions
//         let program = create_complex_test_program();
//         pipeline.load_instructions(program).unwrap();
//
//         let mut cycles = 0;
//         let mut total_stalls = 0;
//         let mut total_instructions = 0;
//
//         while !pipeline.is_empty().unwrap() {
//             pipeline.cycle(&mut register_bank, &mut memory_controller).unwrap();
//             cycles += 1;
//             if pipeline.stats.stalls > total_stalls {
//                 total_stalls = pipeline.stats.stalls;
//             }
//             if pipeline.stats.instructions_executed > total_instructions {
//                 total_instructions = pipeline.stats.instructions_executed;
//             }
//         }
//
//         println!("\nPerformance Metrics:");
//         println!("----------------------------------------");
//         println!("Total Cycles: {}", cycles);
//         println!("Total Instructions: {}", total_instructions);
//         println!("IPC: {:.2}", total_instructions as f64 / cycles as f64);
//         println!("Total Stalls: {}", total_stalls);
//         println!("Reorderings: {}", pipeline.metrics.reorder_count);
//         println!("Execute Stage Utilization: {:.2}%",
//                  (pipeline.metrics.execute_metrics.busy_cycles as f64 / cycles as f64) * 100.0);
//         println!("Memory Stage Utilization: {:.2}%",
//                  (pipeline.metrics.memory_metrics.total_accesses as f64 / cycles as f64) * 100.0);
//
//         // Assertions de base pour vérifier le comportement
//         assert!(cycles > 0, "Le pipeline devrait exécuter au moins un cycle");
//         assert!(total_instructions > 0, "Au moins une instruction devrait être exécutée");
//         assert!(pipeline.metrics.ipc > 0.0, "L'IPC devrait être positif");
//
//     }
// }

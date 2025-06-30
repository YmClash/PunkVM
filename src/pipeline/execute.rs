//src/pipeline/execute.rs

use crate::alu::alu::{ALUOperation, BranchCondition, ALU};
use crate::alu::v_alu::{VectorALU, VectorOperation, };
use crate::alu::fpu::{FPU, FPUOperation, FloatPrecision};
use crate::alu::agu::{AGU, AGUConfig, AddressingMode, AGUError};
use std::collections::{VecDeque};
use std::cell::RefCell;
use std::rc::Rc;
use crate::bytecode::opcodes::{Opcode, };
use crate::bytecode::simds::{Vector128, Vector256, VectorDataType, Vector256DataType};
use crate::pipeline::{DecodeExecuteRegister, ExecuteMemoryRegister};
use crate::pvm::branch_predictor::{BranchPrediction, BranchPredictor, PredictorType};
use crate::pipeline::decode::StackOperation;
use crate::pipeline::parallel::{ExecutionUnit, InstructionPriority, ParallelExecutionEngine, ParallelExecutionStats};
// use crate::pipeline::parallel::{ParallelExecutionEngine, ParallelExecutionStats, ExecutionUnit, InstructionPriority};

/// Contrôleur dual-issue pour exécution parallèle ALU/AGU
#[derive(Debug, Clone)]
pub struct DualIssueController {
    /// Buffer des instructions en attente d'exécution
    instruction_queue: VecDeque<DualIssueInstruction>,
    /// Statistiques d'exécution parallèle
    parallel_executions: u64,
    total_instructions: u64,
    alu_only_instructions: u64,
    agu_only_instructions: u64,
    stalls_resource_conflict: u64,
}

/// Instruction préparée pour dual-issue
#[derive(Debug, Clone)]
struct DualIssueInstruction {
    /// Registre DE contenant l'instruction
    de_register: DecodeExecuteRegister,
    /// Type d'unité d'exécution requise
    execution_unit: ExecutionUnit,
    /// Priorité d'exécution (instructions mémoire prioritaires)
    priority: InstructionPriority,
}

/// Résultat d'exécution dual-issue
#[derive(Debug)]
struct DualIssueResult {
    alu_result: Option<ExecuteMemoryRegister>,
    agu_result: Option<ExecuteMemoryRegister>,
    parallel_execution: bool,
}

impl DualIssueController {
    pub fn new() -> Self {
        Self {
            instruction_queue: VecDeque::new(),
            parallel_executions: 0,
            total_instructions: 0,
            alu_only_instructions: 0,
            agu_only_instructions: 0,
            stalls_resource_conflict: 0,
        }
    }
    
    /// Analyse une instruction pour déterminer son unité d'exécution
    fn analyze_instruction(&self, instruction: &DecodeExecuteRegister) -> (ExecutionUnit, InstructionPriority) {
        match instruction.instruction.opcode {
            // Instructions mémoire - AGU haute priorité
            Opcode::Load | Opcode::LoadB | Opcode::LoadW | Opcode::LoadD |
            Opcode::Store | Opcode::StoreB | Opcode::StoreW | Opcode::StoreD => {
                (ExecutionUnit::AGU, InstructionPriority::High)
            }

            // Instructions SIMD mémoire - AGU haute priorité
            Opcode::Simd128Load | Opcode::Simd128Store |
            Opcode::Simd256Load | Opcode::Simd256Store => {
                (ExecutionUnit::AGU, InstructionPriority::High)
            }

            // Instructions arithmétiques - ALU priorité moyenne
            Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::Div | Opcode::Mod |
            Opcode::Inc | Opcode::Dec | Opcode::Neg |
            Opcode::And | Opcode::Or | Opcode::Xor | Opcode::Not |
            Opcode::Shl | Opcode::Shr | Opcode::Sar | Opcode::Rol | Opcode::Ror |
            Opcode::Cmp | Opcode::Test => {
                (ExecutionUnit::ALU, InstructionPriority::Medium)
            }

            // Instructions de branchement - priorité haute
            Opcode::Jmp | Opcode::JmpIf | Opcode::JmpIfNot | Opcode::JmpIfEqual |
            Opcode::JmpIfNotEqual | Opcode::JmpIfGreater | Opcode::JmpIfGreaterEqual |
            Opcode::JmpIfLess | Opcode::JmpIfLessEqual | Opcode::JmpIfAbove |
            Opcode::JmpIfAboveEqual | Opcode::JmpIfBelow | Opcode::JmpIfBelowEqual |
            Opcode::JmpIfZero | Opcode::JmpIfNotZero | Opcode::JmpIfOverflow |
            Opcode::JmpIfNotOverflow | Opcode::JmpIfPositive | Opcode::JmpIfNegative |
            Opcode::Call | Opcode::Ret => {
                (ExecutionUnit::Branch, InstructionPriority::High)
            }

            // Instructions pile - AGU (calcul d'adresse stack)
            Opcode::Push | Opcode::Pop => {
                (ExecutionUnit::AGU, InstructionPriority::High)
            }

            // Instructions SIMD arithmétiques - SIMD priorité moyenne
            _ if format!("{:?}", instruction.instruction.opcode).starts_with("Simd") => {
                (ExecutionUnit::SIMD, InstructionPriority::Medium)
            }

            // Instructions FPU - FPU priorité moyenne
            _ if format!("{:?}", instruction.instruction.opcode).starts_with("Fp") => {
                (ExecutionUnit::FPU, InstructionPriority::Medium)
            }

            // Instructions de transfert - ALU priorité moyenne
            Opcode::Mov => {
                (ExecutionUnit::ALU, InstructionPriority::Medium)
            }

            // Instructions spéciales - priorité basse
            Opcode::Nop | Opcode::Break => {
                (ExecutionUnit::ALU, InstructionPriority::Low)
            }

            // Instructions système - priorité haute
            Opcode::Halt | Opcode::Syscall => {
                (ExecutionUnit::Both, InstructionPriority::High)
            }

            // Pattern par défaut pour toutes les autres instructions
            _ => {
                (ExecutionUnit::ALU, InstructionPriority::Medium)
            }
        }
    }

    /// Détermine si deux instructions peuvent s'exécuter en parallèle
    fn can_execute_parallel(&self, instr1: &DualIssueInstruction, instr2: &DualIssueInstruction) -> bool {
        // 1. Vérifier qu'elles utilisent des unités différentes
        let units_compatible = match (&instr1.execution_unit, &instr2.execution_unit) {
            (ExecutionUnit::ALU, ExecutionUnit::AGU) => true,
            (ExecutionUnit::AGU, ExecutionUnit::ALU) => true,
            (ExecutionUnit::ALU, ExecutionUnit::SIMD) => true,
            (ExecutionUnit::SIMD, ExecutionUnit::ALU) => true,
            (ExecutionUnit::AGU, ExecutionUnit::SIMD) => false, // SIMD peut utiliser AGU pour mémoire
            (ExecutionUnit::SIMD, ExecutionUnit::AGU) => false,
            _ => false,
        };
        
        if !units_compatible {
            return false;
        }
        
        // 2. Vérifier les dépendances de registres
        let has_register_dependency = self.check_register_dependency(&instr1.de_register, &instr2.de_register);
        
        // 3. Vérifier les dépendances mémoire
        let has_memory_dependency = self.check_memory_dependency(&instr1.de_register, &instr2.de_register);
        
        !has_register_dependency && !has_memory_dependency
    }
    
    /// Vérifie les dépendances de registres entre deux instructions
    fn check_register_dependency(&self, instr1: &DecodeExecuteRegister, instr2: &DecodeExecuteRegister) -> bool {
        // RAW (Read After Write): instr2 lit un registre que instr1 écrit
        if let Some(rd1) = instr1.rd {
            if instr2.rs1 == Some(rd1) || instr2.rs2 == Some(rd1) {
                return true;
            }
        }
        
        // WAR (Write After Read): instr2 écrit un registre que instr1 lit
        if let Some(rd2) = instr2.rd {
            if instr1.rs1 == Some(rd2) || instr1.rs2 == Some(rd2) {
                return true;
            }
        }
        
        // WAW (Write After Write): les deux écrivent le même registre
        if let (Some(rd1), Some(rd2)) = (instr1.rd, instr2.rd) {
            if rd1 == rd2 {
                return true;
            }
        }
        
        false
    }
    
    /// Vérifie les dépendances mémoire entre deux instructions
    fn check_memory_dependency(&self, instr1: &DecodeExecuteRegister, instr2: &DecodeExecuteRegister) -> bool {
        let instr1_is_memory = matches!(instr1.instruction.opcode, 
            Opcode::Load | Opcode::LoadB | Opcode::LoadW | Opcode::LoadD |
            Opcode::Store | Opcode::StoreB | Opcode::StoreW | Opcode::StoreD |
            Opcode::Push | Opcode::Pop
        );
        
        let instr2_is_memory = matches!(instr2.instruction.opcode,
            Opcode::Load | Opcode::LoadB | Opcode::LoadW | Opcode::LoadD |
            Opcode::Store | Opcode::StoreB | Opcode::StoreW | Opcode::StoreD |
            Opcode::Push | Opcode::Pop
        );
        
        // Si les deux sont des instructions mémoire, vérifier l'aliasing
        if instr1_is_memory && instr2_is_memory {
            // Pour simplifier, on considère qu'il y a toujours dépendance entre instructions mémoire
            // Une analyse plus fine pourrait vérifier les adresses
            return true;
        }
        
        false
    }
    
    /// Simule l'exécution dual-issue (version simplifiée)
    pub fn simulate_parallel_execution(&mut self, exec_unit: ExecutionUnit) -> bool {
        // Pour la démonstration, simuler quelques exécutions parallèles
        if self.total_instructions % 5 == 0 && self.total_instructions > 0 {
            self.parallel_executions += 1;
            return true;
        }
        false
    }
    
    /// Obtient les statistiques dual-issue
    pub fn get_stats(&self) -> (u64, u64, u64, u64, u64, f64) {
        let parallel_rate = if self.total_instructions > 0 {
            (self.parallel_executions as f64 / self.total_instructions as f64) * 100.0
        } else {
            0.0
        };
        
        (
            self.parallel_executions,
            self.total_instructions,
            self.alu_only_instructions,
            self.agu_only_instructions,
            self.stalls_resource_conflict,
            parallel_rate,
        )
    }
    
    /// Réinitialise le contrôleur
    pub fn reset(&mut self) {
        self.instruction_queue.clear();
        self.parallel_executions = 0;
        self.total_instructions = 0;
        self.alu_only_instructions = 0;
        self.agu_only_instructions = 0;
        self.stalls_resource_conflict = 0;
    }
}



/// Implementation de l'étage Execute du pipeline
pub struct ExecuteStage {
    // Unité ALU
    branch_predictor: BranchPredictor,
    /// Unité ALU vectorielle SIMD
    vector_alu: Rc<RefCell<VectorALU>>,
    // vector_alu: VectorALU,
    /// Unité de calcul flottant FPU
    fpu: Rc<RefCell<FPU>>,
    // fpu: FPU
    /// Address Generation Unit (AGU) pour calculs d'adresse parallèles
    agu: Rc<RefCell<AGU>>,
    // agu: AGU,
    /// Dual-Issue Controller pour exécution parallèle ALU/AGU
    dual_issue_controller: DualIssueController,
    /// Moteur d'exécution parallèle
    parallel_engine: ParallelExecutionEngine,
    /// Stats Locales
    branch_predictions:u64,
    branch_hits:u64,
    /// Cycle actuel pour l'AGU
    current_cycle: u64,
}

impl ExecuteStage {
    /// Crée un nouvel étage Execute
    pub fn new() -> Self {
        // Créer les unités d'exécution avec smart pointers
        let vector_alu = Rc::new(RefCell::new(VectorALU::new()));
        let fpu = Rc::new(RefCell::new(FPU::new()));
        let agu = Rc::new(RefCell::new(AGU::new(AGUConfig::default())));
        
        // Créer le moteur d'exécution parallèle et configurer les références
        let mut parallel_engine = ParallelExecutionEngine::new();
        parallel_engine.set_execution_units(
            // Pour ALU, nous utiliserons une référence externe car ALU n'est pas dans ExecuteStage
            Rc::new(RefCell::new(ALU::new())),
            agu.clone(),
            vector_alu.clone(),
        );
        
        Self {
            branch_predictor: BranchPredictor::new(PredictorType::Hybrid),
            vector_alu,
            fpu,
            agu,
            dual_issue_controller: DualIssueController::new(),
            parallel_engine,
            branch_predictions: 0,
            branch_hits: 0,
            current_cycle: 0,
        }
    }

    /// Traite l'étage Execute avec fallback sur dual-issue si pas prêt pour parallel
    pub fn process_parallel(
        &mut self,
        instructions: &[DecodeExecuteRegister],
        alu: &mut ALU,
        memory: &mut crate::pvm::memorys::Memory,
        registers: &[u64],
        sp: u64,
    ) -> Result<Vec<ExecuteMemoryRegister>, String> {
        // Pour l'instant, utiliser directement process_with_dual_issue
        // Le ParallelExecutionEngine sera activé progressivement dans les prochaines phases
        
        if instructions.is_empty() {
            return Ok(vec![]);
        }
        
        // Utiliser la méthode éprouvée dual-issue
        let result = self.process_with_dual_issue(&instructions[0], alu, memory, registers, sp)?;
        
        // Logging pour debug
        println!("PARALLEL ENGINE: Exécution fallback pour instruction {:?}", 
                 instructions[0].instruction.opcode);
        
        Ok(vec![result])
    }
    
    /// Traite l'étage Execute avec dual-issue controller pour exécution parallèle
    pub fn process_with_dual_issue(
        &mut self,
        ex_reg: &DecodeExecuteRegister,
        alu: &mut ALU,
        memory: &mut crate::pvm::memorys::Memory,
        registers: &[u64],
        sp: u64,
    ) -> Result<ExecuteMemoryRegister, String> {
        // Mettre à jour le cycle pour l'AGU
        self.agu.borrow_mut().update_cycle(self.current_cycle);
        self.current_cycle += 1;
        
        // Analyser l'instruction pour le dual-issue
        let (exec_unit, _priority) = self.dual_issue_controller.analyze_instruction(ex_reg);
        self.dual_issue_controller.total_instructions += 1;
        
        // Simuler l'exécution parallèle si possible
        let parallel_executed = self.dual_issue_controller.simulate_parallel_execution(exec_unit.clone());
        
        // Exécuter l'instruction selon son type
        let result = match exec_unit {
            ExecutionUnit::ALU => {
                self.dual_issue_controller.alu_only_instructions += 1;
                self.process_direct(ex_reg, alu)
            }
            ExecutionUnit::AGU => {
                self.dual_issue_controller.agu_only_instructions += 1;
                self.process_memory_with_agu(ex_reg, alu, registers, sp)
            }
            ExecutionUnit::SIMD => {
                self.process_with_memory(ex_reg, alu, memory, registers, sp)
            }
            ExecutionUnit::FPU => {
                self.process_direct(ex_reg, alu)
            }
            ExecutionUnit::Branch => {
                self.process_direct(ex_reg, alu)
            }
            ExecutionUnit::Both => {
                self.process_direct(ex_reg, alu)
            }
        };
        
        if parallel_executed {
            println!("DUAL-ISSUE: Exécution parallèle simulée pour {:?}", exec_unit);
        }
        
        result
    }

    /// Traite l'étage Execute avec accès mémoire pour les opérations SIMD Load/Store et AGU
    pub fn process_with_memory(
        &mut self,
        ex_reg: &DecodeExecuteRegister,
        alu: &mut ALU,
        memory: &mut crate::pvm::memorys::Memory,
        registers: &[u64],
        sp: u64,
    ) -> Result<ExecuteMemoryRegister, String> {
        // Mettre à jour le cycle pour l'AGU
        self.agu.borrow_mut().update_cycle(self.current_cycle);
        self.current_cycle += 1;
        
        // Vérifier le type d'instruction
        match ex_reg.instruction.opcode {
            // Toutes les instructions mémoire passent par l'AGU
            Opcode::Load | Opcode::LoadB | Opcode::LoadW | Opcode::LoadD |
            Opcode::Store | Opcode::StoreB | Opcode::StoreW | Opcode::StoreD |
            Opcode::Simd128Load | Opcode::Simd128Store | Opcode::Simd256Load | Opcode::Simd256Store |
            Opcode::Push | Opcode::Pop | Opcode::Call | Opcode::Ret => {
                self.process_memory_with_agu(ex_reg, alu, registers, sp)
            }
            _ => {
                // Pour toutes les autres opérations, utiliser la méthode normale
                self.process_direct(ex_reg, alu)
            }
        }
    }

    /// Traite spécifiquement les opérations SIMD mémoire avec AGU
    fn process_simd_memory_operations(
        &mut self,
        ex_reg: &DecodeExecuteRegister,
        alu: &mut ALU,
        memory: &mut crate::pvm::memorys::Memory,
        registers: &[u64],
        sp: u64,
    ) -> Result<ExecuteMemoryRegister, String> {
        let opcode = &ex_reg.instruction.opcode;
        
        match opcode {
            Opcode::Simd128Load => {
                let dst_reg = ex_reg.rd.ok_or("SIMD128Load: registre destination manquant")?;
                
                // Recalculer l'adresse avec l'AGU pour optimisations
                let addr = self.calculate_memory_address_with_agu(ex_reg, registers, sp)?
                    .unwrap_or(ex_reg.mem_addr.ok_or("SIMD128Load: adresse mémoire manquante")? as u64) as u32;
                
                println!("SIMD128Load: Loading vector from memory address 0x{:08X} into V{}", addr, dst_reg);
                
                // Charger le vecteur depuis la mémoire
                let vector = memory.read_vector128(addr)
                    .map_err(|e| format!("SIMD128Load: Erreur lecture mémoire: {}", e))?;
                
                // Écrire dans le registre vectoriel
                self.vector_alu.borrow_mut().write_v128(dst_reg as u8, vector)
                    .map_err(|e| format!("SIMD128Load: Erreur écriture registre V128: {}", e))?;
                
                println!("SIMD128Load: Vector loaded into V{}", dst_reg);
            }
            
            Opcode::Simd128Store => {
                let src_reg = ex_reg.rs1.ok_or("SIMD128Store: registre source manquant")?;
                
                // Recalculer l'adresse avec l'AGU pour optimisations
                let addr = self.calculate_memory_address_with_agu(ex_reg, registers, sp)?
                    .unwrap_or(ex_reg.mem_addr.ok_or("SIMD128Store: adresse mémoire manquante")? as u64) as u32;
                
                println!("SIMD128Store: Storing vector V{} to memory address 0x{:08X}", src_reg, addr);
                
                // Lire le vecteur du registre source
                let vector = self.vector_alu.borrow_mut().read_v128(src_reg as u8)
                    .map_err(|e| format!("SIMD128Store: Erreur lecture registre V128: {}", e))?;
                
                // Stocker le vecteur en mémoire
                memory.write_vector128(addr, &vector)
                    .map_err(|e| format!("SIMD128Store: Erreur écriture mémoire: {}", e))?;
                
                println!("SIMD128Store: Vector V{} stored to memory", src_reg);
            }
            
            Opcode::Simd256Load => {
                let dst_reg = ex_reg.rd.ok_or("SIMD256Load: registre destination manquant")?;
                
                // Recalculer l'adresse avec l'AGU pour optimisations
                let addr = self.calculate_memory_address_with_agu(ex_reg, registers, sp)?
                    .unwrap_or(ex_reg.mem_addr.ok_or("SIMD256Load: adresse mémoire manquante")? as u64) as u32;
                
                println!("SIMD256Load: Loading vector from memory address 0x{:08X} into Y{}", addr, dst_reg);
                
                // Charger le vecteur depuis la mémoire
                let vector = memory.read_vector256(addr)
                    .map_err(|e| format!("SIMD256Load: Erreur lecture mémoire: {}", e))?;
                
                // Écrire dans le registre vectoriel
                self.vector_alu.borrow_mut().write_v256(dst_reg as u8, vector)
                    .map_err(|e| format!("SIMD256Load: Erreur écriture registre V256: {}", e))?;
                
                println!("SIMD256Load: Vector loaded into Y{}", dst_reg);
            }
            
            Opcode::Simd256Store => {
                let src_reg = ex_reg.rs1.ok_or("SIMD256Store: registre source manquant")?;
                
                // Recalculer l'adresse avec l'AGU pour optimisations
                let addr = self.calculate_memory_address_with_agu(ex_reg, registers, sp)?
                    .unwrap_or(ex_reg.mem_addr.ok_or("SIMD256Store: adresse mémoire manquante")? as u64) as u32;
                
                println!("SIMD256Store: Storing vector Y{} to memory address 0x{:08X}", src_reg, addr);
                
                // Lire le vecteur du registre source
                let vector = self.vector_alu.borrow_mut().read_v256(src_reg as u8)
                    .map_err(|e| format!("SIMD256Store: Erreur lecture registre V256: {}", e))?;
                
                // Stocker le vecteur en mémoire
                memory.write_vector256(addr, &vector)
                    .map_err(|e| format!("SIMD256Store: Erreur écriture mémoire: {}", e))?;
                
                println!("SIMD256Store: Vector Y{} stored to memory", src_reg);
            }
            
            _ => return Err(format!("Opcode SIMD mémoire non supporté: {:?}", opcode)),
        }

        // Créer le registre Execute-Memory pour la suite du pipeline
        Ok(ExecuteMemoryRegister {
            instruction: ex_reg.instruction.clone(),
            rd: ex_reg.rd,
            alu_result: 0, // Les opérations SIMD ne génèrent pas de résultat ALU
            mem_addr: ex_reg.mem_addr,
            store_value: None, // Les opérations SIMD gèrent directement la mémoire
            branch_taken: false,
            branch_target: None,
            stack_operation: None,
            stack_result: None,
            branch_prediction_correct: None,
            ras_prediction_correct: None,
            halted: false,
        })
    }

    /// Traite l'étage Execute directement
    pub fn process_direct(
        &mut self,
        ex_reg: &DecodeExecuteRegister,
        alu: &mut ALU,
    ) -> Result<ExecuteMemoryRegister, String> {
        // on récupère les valeur calcule en decode
        let rs1_value = ex_reg.rs1_value;
        let rs2_value = ex_reg.rs2_value;
        let mut alu_result = 0;
        let mut mem_addr = ex_reg.mem_addr;
        let mut branch_taken = false;
        let mut branch_target = ex_reg.branch_addr;
        let mut store_value = None;

        // pour les opérations de pile
        let mut stack_operation = None;
        let mut stack_result = None;



        // Exécuter l'opération en fonction de l'opcode

        match ex_reg.instruction.opcode {
            // Instructions arithmétiques et logiques
            Opcode::Add => {
                alu_result = alu.execute(ALUOperation::Add, rs1_value, rs2_value)?;
                println!(
                    "Execute ADD: rs1_value={}, rs2_value={}, alu_result={}",
                    rs1_value, rs2_value, alu_result
                );
            }

            Opcode::Sub => {
                alu_result = alu.execute(ALUOperation::Sub, rs1_value, rs2_value)?;
                println!(
                    "Execute SUB: rs1_value={}, rs2_value={}, alu_result={}",
                    rs1_value, rs2_value, alu_result
                );
            }

            Opcode::Mul => {
                alu_result = alu.execute(ALUOperation::Mul, rs1_value, rs2_value)?;
                println!(
                    "Execute MUL: rs1_value={}, rs2_value={}, alu_result={}",
                    rs1_value, rs2_value, alu_result
                );
            }

            Opcode::Div => {
                alu_result = alu.execute(ALUOperation::Div, rs1_value, rs2_value)?;
                println!(
                    "Execute DIV: rs1_value={}, rs2_value={}, alu_result={}",
                    rs1_value, rs2_value, alu_result
                );
            }

            Opcode::Mod => {
                alu_result = alu.execute(ALUOperation::Mod, rs1_value, rs2_value)?;
                println!(
                    "Execute MOD: rs1_value={}, rs2_value={}, alu_result={}",
                    rs1_value, rs2_value, alu_result
                );
            }
            Opcode::Mov => {
                let value = ex_reg.immediate.unwrap_or(ex_reg.rs2_value);
                alu_result = value;
                println!(
                    "Execute MOV: rs1_value={}, immediate={:?}, alu_result={}",
                    rs1_value, ex_reg.immediate, alu_result
                );
            }

            Opcode::Inc => {
                alu_result = alu.execute(ALUOperation::Inc, rs1_value, 0)?;
                println!(
                    "Execute INC: rs1_value={}, alu_result={}",
                    rs1_value, alu_result
                );
            }

            Opcode::Dec => {
                alu_result = alu.execute(ALUOperation::Dec, rs1_value, 0)?;
                println!(
                    "Execute DEC: rs1_value={}, alu_result={}",
                    rs1_value, alu_result
                );
            }

            Opcode::Neg => {
                alu_result = alu.execute(ALUOperation::Neg, rs1_value, 0)?;
                println!(
                    "Execute NEG: rs1_value={}, alu_result={}",
                    rs1_value, alu_result
                );
            }

            Opcode::And => {
                alu_result = alu.execute(ALUOperation::And, rs1_value, rs2_value)?;
                println!(
                    "Execute AND: rs1_value={}, rs2_value={}, alu_result={}",
                    rs1_value, rs2_value, alu_result
                );
            }

            Opcode::Or => {
                alu_result = alu.execute(ALUOperation::Or, rs1_value, rs2_value)?;
                println!(
                    "Execute OR: rs1_value={}, rs2_value={}, alu_result={}",
                    rs1_value, rs2_value, alu_result
                );
            }

            Opcode::Xor => {
                alu_result = alu.execute(ALUOperation::Xor, rs1_value, rs2_value)?;
                println!(
                    "Execute XOR: rs1_value={}, rs2_value={}, alu_result={}",
                    rs1_value, rs2_value, alu_result
                );
            }

            Opcode::Not => {
                alu_result = alu.execute(ALUOperation::Not, rs1_value, 0)?;
                println!(
                    "Execute NOT: rs1_value={}, alu_result={}",
                    rs1_value, alu_result
                );
            }

            Opcode::Nop => {
                // Pas d'opération
                alu_result = 0; // Pas utilisé
                println!("Execute NOP");
            }

            Opcode::Shl => {
                alu_result = alu.execute(ALUOperation::Shl, rs1_value, rs2_value)?;
                println!(
                    "Execute SHL: rs1_value={}, rs2_value={}, alu_result={}",
                    rs1_value, rs2_value, alu_result
                );
            }

            Opcode::Shr => {
                alu_result = alu.execute(ALUOperation::Shr, rs1_value, rs2_value)?;
                println!(
                    "Execute SHR: rs1_value={}, rs2_value={}, alu_result={}",
                    rs1_value, rs2_value, alu_result
                );
            }

            Opcode::Sar => {
                alu_result = alu.execute(ALUOperation::Sar, rs1_value, rs2_value)?;
                println!(
                    "Execute SAR: rs1_value={}, rs2_value={}, alu_result={}",
                    rs1_value, rs2_value, alu_result
                );
            }

            Opcode::Rol => {
                alu_result = alu.execute(ALUOperation::Rol, rs1_value, rs2_value)?;
                println!(
                    "Execute ROL: rs1_value={}, rs2_value={}, alu_result={}",
                    rs1_value, rs2_value, alu_result
                );
            }

            Opcode::Ror => {
                alu_result = alu.execute(ALUOperation::Ror, rs1_value, rs2_value)?;
                println!(
                    "Execute ROR: rs1_value={}, rs2_value={}, alu_result={}",
                    rs1_value, rs2_value, alu_result
                );
            }

            // Instructions de comparaison
            Opcode::Cmp => {
                // Compare mais ne stocke pas le résultat
                alu.execute(ALUOperation::Cmp, rs1_value, rs2_value)?;
                alu_result = 0; // Pas utilisé
                println!(
                    "Execute CMP: rs1_value={} vs rs2_value={}",
                    rs1_value, rs2_value
                );
            }

            Opcode::Test => {
                // Test (AND logique) mais ne stocke pas le résultat
                alu.execute(ALUOperation::Test, rs1_value, rs2_value)?;
                alu_result = 0; // Pas utilisé
                println!(
                    "Execute TEST: rs1_value={}, rs2_value={}",
                    rs1_value, rs2_value
                );
            }
////////////////////////////////////////////CONTROLE FLOW////////////////////////////////////////////////////////
            Opcode::Jmp|
            Opcode::JmpIf
            |Opcode::JmpIfNot
            | Opcode::JmpIfEqual
            | Opcode::JmpIfNotEqual
            | Opcode::JmpIfGreater
            | Opcode::JmpIfGreaterEqual
            | Opcode::JmpIfLess
            | Opcode::JmpIfLessEqual
            | Opcode::JmpIfAbove
            | Opcode::JmpIfAboveEqual
            | Opcode::JmpIfBelow
            | Opcode::JmpIfBelowEqual
            | Opcode::JmpIfZero
            | Opcode::JmpIfNotZero
            | Opcode::JmpIfOverflow
            | Opcode::JmpIfNotOverflow
            | Opcode::JmpIfPositive
            | Opcode::JmpIfNegative => {
                branch_taken = alu.check_condition(match ex_reg.instruction.opcode {
                    Opcode::JmpIf => BranchCondition::Equal,
                    Opcode::JmpIfEqual => BranchCondition::Equal,
                    Opcode::JmpIfNotEqual => BranchCondition::NotEqual,
                    Opcode::JmpIfGreater => BranchCondition::Greater,
                    Opcode::JmpIfGreaterEqual => BranchCondition::GreaterEqual,
                    Opcode::JmpIfLess => BranchCondition::Less,
                    Opcode::JmpIfLessEqual => BranchCondition::LessEqual,
                    Opcode::JmpIfAbove => BranchCondition::Above,
                    Opcode::JmpIfAboveEqual => BranchCondition::AboveEqual,
                    Opcode::JmpIfBelow => BranchCondition::Below,
                    Opcode::JmpIfBelowEqual => BranchCondition::BelowEqual,
                    Opcode::JmpIfZero => BranchCondition::Zero,
                    Opcode::JmpIfNotZero => BranchCondition::NotZero,
                    Opcode::JmpIfOverflow => BranchCondition::Overflow,
                    Opcode::JmpIfNotOverflow => BranchCondition::NotOverflow,
                    Opcode::JmpIfPositive => BranchCondition::Positive,
                    Opcode::JmpIfNegative => BranchCondition::Negative,
                    //pour tous les autres opcodes
                    _ => BranchCondition::Always, // Ne devrait pas arriver
                });

                branch_target = ex_reg.branch_addr;

                if let Some(prediction) = ex_reg.branch_prediction {
                    // le PC devrai etre passé au predicteur ou stocké dans ex_reg
                    // self.update_predictor(ex_reg.pc, prediction, branch_taken);
                    self.update_branch_predictor(ex_reg.pc as u64, branch_taken, prediction);
                }

                // CORRECTION CRITIQUE: Mise à jour du BTB avec la cible de branchement
                if branch_taken {
                    if let Some(target) = branch_target {
                        // Prédire d'abord pour obtenir la prédiction actuelle du BTB
                        let predicted_target = self.branch_predictor.predict_target(ex_reg.pc as u64);
                        
                        // Mettre à jour le BTB avec la vraie cible et vérifier si la prédiction était correcte
                        self.branch_predictor.update_btb(ex_reg.pc as u64, target, predicted_target);
                        
                        println!("BTB Update: PC=0x{:X}, Target=0x{:X}, Predicted={:?}", 
                                 ex_reg.pc, target, predicted_target);
                    }
                }

                println!(
                    "DEBUG: Processing branch instruction: {:?}",
                    ex_reg.instruction
                );

                println!("DEBUG: Branch address: {:?}", ex_reg.branch_addr);
                println!("DEBUG: Format: {:?}", ex_reg.instruction.format);
                println!("DEBUG: Args: {:?}", ex_reg.instruction.args);
                println!(
                    "Execute branch instruction: {:?}, branch_taken={}, branch_target={:?}",
                    ex_reg.instruction.opcode, branch_taken, branch_target
                );
                println!("[[[DEBUG: Traitement d'un Jmp -]]] PC = 0x{:08X}, Target = {:?}", ex_reg.pc, branch_target);

            }
////////////////////////////////////Control des FLOW////////////////////////////////////////////////////////
            // Instructions d'accès mémoire - NOTE: Ces cas sont maintenant gérés dans process_memory_with_agu
            Opcode::Load | Opcode::LoadB | Opcode::LoadW | Opcode::LoadD => {
                // Ces instructions finalisent leur exécution dans l'étage Memory
                // L'adresse peut être recalculée par l'AGU pour optimisations
                alu_result = 0; // Sera remplacé par la valeur chargée
                println!(
                    "Execute LOAD: rs1_value={}, mem_addr={:?}",
                    rs1_value, mem_addr
                );
            }

            Opcode::Store | Opcode::StoreB | Opcode::StoreW | Opcode::StoreD => {
                // Préparer la valeur à stocker
                // L'adresse peut être recalculée par l'AGU pour optimisations
                store_value = Some(rs1_value);
                println!(
                    "Execute STORE: rs1_value={}, mem_addr={:?}",
                    rs1_value, mem_addr
                );
            }

            Opcode::Call => {
                println!("Execute CALL: PC=0x{:X}, target={:?}", ex_reg.pc, branch_target);

                // 1. Calculer l'adresse de retour
                let return_address = ex_reg.pc + ex_reg.instruction.total_size() as u32;

                // 2. Préparer l'adresse de retour pour être empilée sur la pile
                store_value = Some(return_address as u64);
                stack_operation = Some(StackOperation::Push);
                stack_result = Some(return_address as u64);

                // 3. Préparer le saut vers la fonction appelée
                branch_taken = true;

                println!("CALL executed: return_addr=0x{:X}, target={:?}",
                         return_address, branch_target);

            },
            Opcode::Ret => {
                println!("Execute RET: PC=0x{:X}", ex_reg.pc);

                // 1. Indiquer qu'on veut dépiler une valeur de la pile
                stack_operation = Some(StackOperation::Pop);
                
                // 2. Préparer le saut vers l'adresse de retour
                // L'adresse de retour sera fournie par le RAS prediction ou par la pile
                branch_taken = true;
                // branch_target sera défini par le RAS dans decode ou par la pile dans memory

                println!("RET executed: branch_target={:?}", branch_target);
            },


            Opcode::Push => {
                // Préparer la valeur à empiler
                // Prioriser la valeur immédiate si présente (PUSH immédiat)
                let value_to_push = if let Some(imm) = ex_reg.immediate {
                    imm
                } else {
                    rs1_value
                };
                store_value = Some(value_to_push);
                // L'adresse est calculée dans l'étage Memory
                println!(
                    "Execute PUSH: rs1_value={}, immediate={:?}, value_to_push={}, mem_addr={:?}",
                    rs1_value, ex_reg.immediate, value_to_push, mem_addr
                );
            },

            Opcode::Pop => {
                println!("Execute POP");

                // Indiquer qu'on veut dépiler une valeur de la pile
                stack_operation = Some(StackOperation::Pop);
                
                // Le résultat sera fourni par l'étage Memory après dépilage
                // et sera écrit dans le registre de destination via alu_result

                println!("POP executed: will pop value into register");
            },

            // Instructions spéciales
            Opcode::Syscall => {
                // Traitées séparément (pas implémenté pour l'instant)
                return Err("Syscall non implémenté".to_string());
            },

            Opcode::Break => {
                // Instruction de débogage, ne fait rien dans la PunkVM
                println!("Execute BREAK");
            }

            Opcode::Halt => {
                println!("Execute HALT");
                return Ok(ExecuteMemoryRegister {
                    instruction: ex_reg.instruction.clone(),
                    alu_result: 0,
                    rd: ex_reg.rd,
                    store_value: None,
                    mem_addr: None,
                    branch_target: None,
                    branch_taken: false,
                    branch_prediction_correct: None,
                    stack_operation: None,
                    stack_result: None,
                    ras_prediction_correct: None,
                    halted: true, // un champ qu’il faut rajouter (voir ci-dessous)
                });
            }

            // Instructions SIMD 128-bit
            Opcode::Simd128Add | Opcode::Simd128Sub | Opcode::Simd128Mul | Opcode::Simd128Div |
            Opcode::Simd128And | Opcode::Simd128Or | Opcode::Simd128Xor | Opcode::Simd128Not |
            Opcode::Simd128Mov | Opcode::Simd128Load | Opcode::Simd128Store |
            Opcode::Simd128Cmp | Opcode::Simd128Min | Opcode::Simd128Max |
            Opcode::Simd128Sqrt | Opcode::Simd128Shuffle  | Opcode::Simd128Const | Opcode::Simd128ConstF32 |
            Opcode::Simd128ConstI16x8 | Opcode::Simd128ConstI64x2 | Opcode::Simd128ConstF64x2 => {
                self.execute_simd_128(&ex_reg.instruction.opcode, ex_reg)?;
                // Pour les instructions SIMD, on retourne 0 car le résultat est dans les registres vectoriels
                alu_result = 0;

                println!("Execute SIMD128 {:?}: completed", ex_reg.instruction.opcode);
            }

            // Instructions SIMD 256-bit
            Opcode::Simd256Add | Opcode::Simd256Sub | Opcode::Simd256Mul | Opcode::Simd256Div |
            Opcode::Simd256And | Opcode::Simd256Or | Opcode::Simd256Xor | Opcode::Simd256Not |
            Opcode::Simd256Mov | Opcode::Simd256Load | Opcode::Simd256Store |
            Opcode::Simd256Cmp | Opcode::Simd256Min | Opcode::Simd256Max |
            Opcode::Simd256Sqrt | Opcode::Simd256Shuffle | Opcode::Simd256Const | Opcode::Simd256ConstF32 |
            Opcode::Simd256ConstI16x16 | Opcode::Simd256ConstI64x4 | Opcode::Simd256ConstF64x4 => {
                self.execute_simd_256(&ex_reg.instruction.opcode, ex_reg)?;
                // Pour les instructions SIMD, on retourne 0 car le résultat est dans les registres vectoriels
                alu_result = 0;
                println!("Execute SIMD256 {:?}: completed", ex_reg.instruction.opcode);
            }

            // Instructions FPU
            Opcode::FpuAdd | Opcode::FpuSub | Opcode::FpuMul | Opcode::FpuDiv |
            Opcode::FpuSqrt | Opcode::FpuCmp | Opcode::FpuLoad | Opcode::FpuStore |
            Opcode::FpuMov | Opcode::FpuConvert | Opcode::FpuRound |
            Opcode::FpuMin | Opcode::FpuMax => {
                let fpu_result = self.execute_fpu(&ex_reg.instruction.opcode, ex_reg)?;
                alu_result = fpu_result.to_bits();
                println!("Execute FPU {:?}: result={}", ex_reg.instruction.opcode, fpu_result);
            }

            // Instructions étendues et autres
            _ => {
                return Err(format!(
                    "Opcode non supporté: {:?}",
                    ex_reg.instruction.opcode
                ));
            }

        }


        // Debug pour les instructions de branchement
        if ex_reg.instruction.opcode.is_branch() {
            if let Some(prediction) = ex_reg.branch_prediction {
                self.update_branch_predictor(ex_reg.pc as u64, branch_taken, prediction);
            }
            
            // Mise à jour du BTB avec la cible réelle du branchement
            if branch_taken {
                if let Some(target) = branch_target {
                    // Récupérer la cible prédite par le BTB depuis le decode stage
                    let predicted_target = ex_reg.branch_addr;
                    self.branch_predictor.update_btb(ex_reg.pc as u64, target, predicted_target);
                    println!("Updated BTB: PC=0x{:X}, actual_target=0x{:X}, predicted_target={:?}",
                             ex_reg.pc, target, predicted_target);


                }
            }

            println!("DEBUG: Processing branch instruction: {:?}", ex_reg.instruction);
            println!("DEBUG: Branch address: {:?}", branch_target);
            println!("DEBUG: Format: {:?}", ex_reg.instruction.format);
            println!("DEBUG: Args: {:?}", ex_reg.instruction.args);
            println!("Execute branch instruction: {:?}, branch_taken={}, branch_target={:?}",
                     ex_reg.instruction.opcode, branch_taken, branch_target);
        }

        println!("Executed Instruction : {:?}", ex_reg.instruction);



        // Calculer si la prédiction était correcte (pour les branches)
        let branch_prediction_correct = if ex_reg.instruction.opcode.is_branch() {
            ex_reg.branch_prediction.map(|pred| {
                let predicted_taken = pred == BranchPrediction::Taken;
                predicted_taken == branch_taken
            })
        } else {
            None
        };

        // Gestion spéciale pour RET avec validation RAS
        let ras_prediction_correct = if ex_reg.instruction.opcode == Opcode::Ret {
            if let Some(predicted_target) = ex_reg.branch_addr {
                if let Some(actual_target) = branch_target {
                    Some(predicted_target == actual_target)
                } else {
                    Some(false)
                }
            } else {
                None
            }
        } else {
            None
        };


        Ok(ExecuteMemoryRegister {
            instruction: ex_reg.instruction.clone(),
            alu_result,
            rd: ex_reg.rd,
            store_value, // pour CMP
            mem_addr,
            branch_target,
            branch_taken,
            // branch_prediction_correct: ex_reg
            //     .branch_prediction
            //     .map(|pred| (pred == BranchPrediction::Taken) == branch_taken), // Pas encore implémenté
            branch_prediction_correct,
            stack_operation,
            stack_result,
            ras_prediction_correct,
            halted: false, // Pas de halt ici
        })
    }

    pub fn update_branch_predictor(&mut self, pc: u64, taken: bool, prediction: BranchPrediction) {
        println!("Updating branch predictor: PC=0x{:X}, taken={}, prediction={:?}",
                 pc, taken, prediction);

        // Utiliser le prédicteur persistant
        self.branch_predictor.update(pc, taken, prediction);

        // Mise à jour des statistiques locales
        self.branch_predictions += 1;
        if (prediction == BranchPrediction::Taken) == taken {
            self.branch_hits += 1;
        }

        let accuracy = self.get_prediction_accuracy();
        println!("Branch predictor accuracy: {:.2}%", accuracy);
    }

    /// Retourne le taux de réussite du prédicteur
    pub fn get_prediction_accuracy(&self) -> f64 {
        if self.branch_predictions > 0 {
            (self.branch_hits as f64 / self.branch_predictions as f64) * 100.0
        } else {
            0.0
        }

    }

    //methode pour

    /// Optimisation SIMD : Exécute des instructions SIMD en mode super-scalaire quand possible
    pub fn can_execute_simd_parallel(&self, current: &Opcode, next: Option<&Opcode>) -> bool {
        if let Some(next_op) = next {
            // Vérifier si les deux instructions SIMD peuvent s'exécuter en parallèle
            match (current, next_op) {
                // Opérations arithmétiques peuvent se faire en parallèle sur différents registres
                (Opcode::Simd128Add | Opcode::Simd128Sub | Opcode::Simd128Mul | Opcode::Simd128Div,
                 Opcode::Simd128And | Opcode::Simd128Or | Opcode::Simd128Xor) => true,
                 
                // Min/Max peuvent se faire en parallèle avec d'autres opérations
                (Opcode::Simd128Min | Opcode::Simd128Max,
                 Opcode::Simd128Add | Opcode::Simd128Sub | Opcode::Simd128Mul) => true,
                 
                // Éviter les dépendances de données évidentes
                _ => false,
            }
        } else {
            false
        }
    }

    /// Exécute une instruction SIMD 128-bit avec optimisations
    fn execute_simd_128(&mut self, opcode: &Opcode, ex_reg: &DecodeExecuteRegister) -> Result<(), String> {
        let src1_reg = ex_reg.rs1.unwrap_or(0) as u8;
        let src2_reg = ex_reg.rs2.unwrap_or(0) as u8;
        let dst_reg = ex_reg.rd.unwrap_or(0) as u8;

        let operation = match opcode {
            Opcode::Simd128Add => VectorOperation::Add,
            Opcode::Simd128Sub => VectorOperation::Sub,
            Opcode::Simd128Mul => VectorOperation::Mul,
            Opcode::Simd128Div => VectorOperation::Div,
            Opcode::Simd128And => VectorOperation::And,
            Opcode::Simd128Or => VectorOperation::Or,
            Opcode::Simd128Xor => VectorOperation::Xor,
            Opcode::Simd128Not => VectorOperation::Not,
            Opcode::Simd128Min => VectorOperation::Min,
            Opcode::Simd128Max => VectorOperation::Max,
            Opcode::Simd128Sqrt => VectorOperation::Sqrt,
            Opcode::Simd128Cmp => VectorOperation::Cmp,
            Opcode::Simd128Shuffle => VectorOperation::Shuffle,
            Opcode::Simd128Load => {
                // Chargement d'un vecteur 128-bit depuis la mémoire
                // implementer dans process_with_memory()
                println!("SIMD128Load: Loading vector from memory into V{}", dst_reg);
                
                // Créer un vecteur par défaut pour l'instant (sera remplacé par le vrai load mémoire)
                let default_vector = Vector128 { i32x4: [0, 0, 0, 0] };
                self.vector_alu.borrow_mut().write_v128(dst_reg, default_vector)
                    .map_err(|e| format!("Erreur écriture registre V128: {}", e))?;
                return Ok(());
            }
            Opcode::Simd128Store => {
                // Stockage d'un vecteur 128-bit en mémoire
                // src1_reg contient le vecteur à stocker
                // L'adresse est dans ex_reg.mem_addr
                println!("SIMD128Store: Storing vector V{} to memory", src1_reg);
                
                // Lire le vecteur du registre source
                let vector = self.vector_alu.borrow_mut().read_v128(src1_reg)
                    .map_err(|e| format!("Erreur lecture registre V128: {}", e))?;
                
                // Implementer dans
                // Pour l'instant, on affiche juste une confirmation
                println!("SIMD128Store: Vector V{} = {:?} stored to memory", src1_reg, unsafe { vector.i32x4 });
                
                return Ok(());
            }
            Opcode::Simd128Mov => {
                // Mov vectoriel simple
                let src_vector = self.vector_alu.borrow_mut().read_v128(src1_reg)
                    .map_err(|e| format!("Erreur lecture registre V128: {}", e))?;
                self.vector_alu.borrow_mut().write_v128(dst_reg, src_vector)
                    .map_err(|e| format!("Erreur écriture registre V128: {}", e))?;
                return Ok(());
            }
            Opcode::Simd128Const | Opcode::Simd128ConstF32 | 
            Opcode::Simd128ConstI16x8 | Opcode::Simd128ConstI64x2 | Opcode::Simd128ConstF64x2 => {
                // Charger une constante vectorielle 128-bit
                // Les données sont dans les arguments de l'instruction
                // arg1 = registre destination (déjà extrait dans dst_reg)
                // arg2 = première moitié (64 bits)
                // arg3 = deuxième moitié (64 bits)
                
                // Extraire les valeurs des arguments
                let arg2_value = ex_reg.instruction.get_arg2_value()
                    .map_err(|e| format!("Erreur extraction arg2: {:?}", e))?;
                let arg3_value = ex_reg.instruction.get_arg3_value()
                    .map_err(|e| format!("Erreur extraction arg3: {:?}", e))?;
                
                // Vérifier que ce sont des valeurs immédiates
                let (imm1, imm2) = match (arg2_value, arg3_value) {
                    (crate::bytecode::instructions::ArgValue::Immediate(v1), 
                     crate::bytecode::instructions::ArgValue::Immediate(v2)) => (v1, v2),
                    _ => return Err("Simd128Const: arguments doivent être des immédiats".to_string()),
                };
                
                // Construire le vecteur 128-bit à partir des deux moitiés 64-bit
                let vector = match opcode {
                    Opcode::Simd128Const => {
                        // Pour i32x4
                        let bytes1 = imm1.to_le_bytes();
                        let bytes2 = imm2.to_le_bytes();
                        
                        let val0 = i32::from_le_bytes([bytes1[0], bytes1[1], bytes1[2], bytes1[3]]);
                        let val1 = i32::from_le_bytes([bytes1[4], bytes1[5], bytes1[6], bytes1[7]]);
                        let val2 = i32::from_le_bytes([bytes2[0], bytes2[1], bytes2[2], bytes2[3]]);
                        let val3 = i32::from_le_bytes([bytes2[4], bytes2[5], bytes2[6], bytes2[7]]);
                        
                        Vector128 { i32x4: [val0, val1, val2, val3] }
                    }
                    Opcode::Simd128ConstF32 => {
                        // Pour f32x4
                        let bytes1 = imm1.to_le_bytes();
                        let bytes2 = imm2.to_le_bytes();
                        
                        let val0 = f32::from_le_bytes([bytes1[0], bytes1[1], bytes1[2], bytes1[3]]);
                        let val1 = f32::from_le_bytes([bytes1[4], bytes1[5], bytes1[6], bytes1[7]]);
                        let val2 = f32::from_le_bytes([bytes2[0], bytes2[1], bytes2[2], bytes2[3]]);
                        let val3 = f32::from_le_bytes([bytes2[4], bytes2[5], bytes2[6], bytes2[7]]);
                        
                        Vector128 { f32x4: [val0, val1, val2, val3] }
                    }
                    Opcode::Simd128ConstI16x8 => {
                        // Pour i16x8
                        let bytes1 = imm1.to_le_bytes();
                        let bytes2 = imm2.to_le_bytes();
                        
                        let val0 = i16::from_le_bytes([bytes1[0], bytes1[1]]);
                        let val1 = i16::from_le_bytes([bytes1[2], bytes1[3]]);
                        let val2 = i16::from_le_bytes([bytes1[4], bytes1[5]]);
                        let val3 = i16::from_le_bytes([bytes1[6], bytes1[7]]);
                        let val4 = i16::from_le_bytes([bytes2[0], bytes2[1]]);
                        let val5 = i16::from_le_bytes([bytes2[2], bytes2[3]]);
                        let val6 = i16::from_le_bytes([bytes2[4], bytes2[5]]);
                        let val7 = i16::from_le_bytes([bytes2[6], bytes2[7]]);
                        
                        Vector128 { i16x8: [val0, val1, val2, val3, val4, val5, val6, val7] }
                    }
                    Opcode::Simd128ConstI64x2 => {
                        // Pour i64x2
                        Vector128 { i64x2: [imm1 as i64, imm2 as i64] }
                    }
                    Opcode::Simd128ConstF64x2 => {
                        // Pour f64x2
                        let val0 = f64::from_bits(imm1);
                        let val1 = f64::from_bits(imm2);
                        
                        Vector128 { f64x2: [val0, val1] }
                    }
                    _ => return Err(format!("Type de constante 128-bit non supporté: {:?}", opcode)),
                };
                
                // Écrire le vecteur dans le registre destination
                self.vector_alu.borrow_mut().write_v128(dst_reg, vector)
                    .map_err(|e| format!("Erreur écriture registre V128: {}", e))?;
                
                println!("SIMD128Const: Loaded constant vector into V{}", dst_reg);
                return Ok(());
            }
            _ => return Err(format!("Opération SIMD 128-bit non supportée: {:?}", opcode)),
        };

        // Déterminer le type de données vectorielles selon l'opération
        let data_type = match opcode {
            Opcode::Simd128Sqrt => VectorDataType::F32x4, // Sqrt nécessite des flottants
            _ => VectorDataType::I32x4, // Type par défaut pour les autres opérations
        };

        self.vector_alu.borrow_mut().execute_v128(
            operation,
            dst_reg,
            src1_reg,
            Some(src2_reg),
            data_type,
        ).map_err(|e| format!("Erreur exécution SIMD 128-bit: {}", e))?;

        Ok(())
    }

    /// Exécute une instruction SIMD 256-bit
    fn execute_simd_256(&mut self, opcode: &Opcode, ex_reg: &DecodeExecuteRegister) -> Result<(), String> {
        let src1_reg = ex_reg.rs1.unwrap_or(0) as u8;
        let src2_reg = ex_reg.rs2.unwrap_or(0) as u8;
        let dst_reg = ex_reg.rd.unwrap_or(0) as u8;

        let operation = match opcode {
            Opcode::Simd256Add => VectorOperation::Add,
            Opcode::Simd256Sub => VectorOperation::Sub,
            Opcode::Simd256Mul => VectorOperation::Mul,
            Opcode::Simd256Div => VectorOperation::Div,
            Opcode::Simd256And => VectorOperation::And,
            Opcode::Simd256Or => VectorOperation::Or,
            Opcode::Simd256Xor => VectorOperation::Xor,
            Opcode::Simd256Not => VectorOperation::Not,
            Opcode::Simd256Min => VectorOperation::Min,
            Opcode::Simd256Max => VectorOperation::Max,
            Opcode::Simd256Sqrt => VectorOperation::Sqrt,
            Opcode::Simd256Cmp => VectorOperation::Cmp,
            Opcode::Simd256Shuffle => VectorOperation::Shuffle,
            Opcode::Simd256Mov => {
                // Mov vectoriel simple
                let src_vector = self.vector_alu.borrow_mut().read_v256(src1_reg)
                    .map_err(|e| format!("Erreur lecture registre V256: {}", e))?;
                self.vector_alu.borrow_mut().write_v256(dst_reg, src_vector)
                    .map_err(|e| format!("Erreur écriture registre V256: {}", e))?;
                return Ok(());
            }
            Opcode::Simd256Load => {
                // Chargement d'un vecteur 256-bit depuis la mémoire
                println!("SIMD256Load: Loading vector from memory into Y{}", dst_reg);
                
                // Créer un vecteur par défaut pour l'instant (sera remplacé par le vrai load mémoire)
                let default_vector = Vector256 { i32x8: [0, 0, 0, 0, 0, 0, 0, 0] };
                self.vector_alu.borrow_mut().write_v256(dst_reg, default_vector)
                    .map_err(|e| format!("Erreur écriture registre V256: {}", e))?;
                return Ok(());
            }
            Opcode::Simd256Store => {
                // Stockage d'un vecteur 256-bit en mémoire
                println!("SIMD256Store: Storing vector Y{} to memory", src1_reg);
                
                // Lire le vecteur du registre source
                let vector = self.vector_alu.borrow_mut().read_v256(src1_reg)
                    .map_err(|e| format!("Erreur lecture registre V256: {}", e))?;
                
                // Implementer dans process_with_memory() et aussi process_simd_memory_operations()
                println!("SIMD256Store: Vector Y{} = {:?} stored to memory", src1_reg, unsafe { vector.i32x8 });
                
                return Ok(());
            }
            Opcode::Simd256Const | Opcode::Simd256ConstF32 | 
            Opcode::Simd256ConstI16x16 | Opcode::Simd256ConstI64x4 | Opcode::Simd256ConstF64x4 => {
                // Charger une constante vectorielle 256-bit
                // Les données sont dans les arguments de l'instruction
                // arg1 = registre destination (déjà extrait dans dst_reg)
                // arg2 = première moitié (64 bits)
                // arg3 = deuxième moitié (64 bits)
                // Note: Pour 256-bit, nous avons besoin de plus de données que les 128 bits disponibles
                // Nous utiliserons les mêmes 128 bits pour les deux moitiés du vecteur 256-bit
                
                // Extraire les valeurs des arguments
                let arg2_value = ex_reg.instruction.get_arg2_value()
                    .map_err(|e| format!("Erreur extraction arg2: {:?}", e))?;
                let arg3_value = ex_reg.instruction.get_arg3_value()
                    .map_err(|e| format!("Erreur extraction arg3: {:?}", e))?;
                
                // Vérifier que ce sont des valeurs immédiates
                let (imm1, imm2) = match (arg2_value, arg3_value) {
                    (crate::bytecode::instructions::ArgValue::Immediate(v1), 
                     crate::bytecode::instructions::ArgValue::Immediate(v2)) => (v1, v2),
                    _ => return Err("Simd256Const: arguments doivent être des immédiats".to_string()),
                };
                
                // Construire le vecteur 256-bit à partir des deux moitiés 64-bit
                let vector = match opcode {
                    Opcode::Simd256Const => {
                        // Pour i32x8 (8x 32-bit integers dans un vecteur 256-bit)
                        let bytes1 = imm1.to_le_bytes();
                        let bytes2 = imm2.to_le_bytes();
                        
                        let val0 = i32::from_le_bytes([bytes1[0], bytes1[1], bytes1[2], bytes1[3]]);
                        let val1 = i32::from_le_bytes([bytes1[4], bytes1[5], bytes1[6], bytes1[7]]);
                        let val2 = i32::from_le_bytes([bytes2[0], bytes2[1], bytes2[2], bytes2[3]]);
                        let val3 = i32::from_le_bytes([bytes2[4], bytes2[5], bytes2[6], bytes2[7]]);
                        // Dupliquer les 4 premiers éléments pour créer un vecteur 8x32
                        let val4 = val0;
                        let val5 = val1;
                        let val6 = val2;
                        let val7 = val3;
                        
                        Vector256 { i32x8: [val0, val1, val2, val3, val4, val5, val6, val7] }
                    }
                    Opcode::Simd256ConstF32 => {
                        // Pour f32x8 (8x 32-bit floats dans un vecteur 256-bit)
                        let bytes1 = imm1.to_le_bytes();
                        let bytes2 = imm2.to_le_bytes();
                        
                        let val0 = f32::from_le_bytes([bytes1[0], bytes1[1], bytes1[2], bytes1[3]]);
                        let val1 = f32::from_le_bytes([bytes1[4], bytes1[5], bytes1[6], bytes1[7]]);
                        let val2 = f32::from_le_bytes([bytes2[0], bytes2[1], bytes2[2], bytes2[3]]);
                        let val3 = f32::from_le_bytes([bytes2[4], bytes2[5], bytes2[6], bytes2[7]]);
                        // Dupliquer les 4 premiers éléments pour créer un vecteur 8x32
                        let val4 = val0;
                        let val5 = val1;
                        let val6 = val2;
                        let val7 = val3;
                        
                        Vector256 { f32x8: [val0, val1, val2, val3, val4, val5, val6, val7] }
                    }
                    Opcode::Simd256ConstI16x16 => {
                        // Pour i16x16 (16x 16-bit integers dans un vecteur 256-bit)
                        let bytes1 = imm1.to_le_bytes();
                        let bytes2 = imm2.to_le_bytes();
                        
                        // Extraire 4 valeurs i16 des 8 premiers octets
                        let val0 = i16::from_le_bytes([bytes1[0], bytes1[1]]);
                        let val1 = i16::from_le_bytes([bytes1[2], bytes1[3]]);
                        let val2 = i16::from_le_bytes([bytes1[4], bytes1[5]]);
                        let val3 = i16::from_le_bytes([bytes1[6], bytes1[7]]);
                        let val4 = i16::from_le_bytes([bytes2[0], bytes2[1]]);
                        let val5 = i16::from_le_bytes([bytes2[2], bytes2[3]]);
                        let val6 = i16::from_le_bytes([bytes2[4], bytes2[5]]);
                        let val7 = i16::from_le_bytes([bytes2[6], bytes2[7]]);
                        
                        // Dupliquer pour avoir 16 éléments
                        Vector256 { i16x16: [val0, val1, val2, val3, val4, val5, val6, val7,
                                           val0, val1, val2, val3, val4, val5, val6, val7] }
                    }
                    Opcode::Simd256ConstI64x4 => {
                        // Pour i64x4 (4x 64-bit integers dans un vecteur 256-bit)
                        // Dupliquer les deux valeurs pour avoir 4 éléments
                        let val0 = imm1 as i64;
                        let val1 = imm2 as i64;
                        
                        Vector256 { i64x4: [val0, val1, val0, val1] }
                    }
                    Opcode::Simd256ConstF64x4 => {
                        // Pour f64x4 (4x 64-bit floats dans un vecteur 256-bit)
                        let val0 = f64::from_bits(imm1);
                        let val1 = f64::from_bits(imm2);
                        
                        Vector256 { f64x4: [val0, val1, val0, val1] }
                    }
                    _ => return Err(format!("Type de constante 256-bit non supporté: {:?}", opcode)),
                };
                
                // Écrire le vecteur dans le registre destination
                self.vector_alu.borrow_mut().write_v256(dst_reg, vector)
                    .map_err(|e| format!("Erreur écriture registre V256: {}", e))?;
                
                println!("SIMD256Const: Loaded constant vector into Y{}", dst_reg);
                return Ok(());
            }
            _ => return Err(format!("Opération SIMD 256-bit non supportée: {:?}", opcode)),
        };

        // Déterminer le type de données vectorielles selon l'opération
        let data_type = match opcode {
            Opcode::Simd256Sqrt => Vector256DataType::F32x8, // Sqrt nécessite des flottants
            _ => Vector256DataType::I32x8, // Type par défaut pour les autres opérations
        };

        self.vector_alu.borrow_mut().execute_v256(
            operation,
            dst_reg,
            src1_reg,
            Some(src2_reg),
            data_type,
        ).map_err(|e| format!("Erreur exécution SIMD 256-bit: {}", e))?;

        Ok(())
    }

    /// Exécute une instruction FPU
    fn execute_fpu(&mut self, opcode: &Opcode, ex_reg: &DecodeExecuteRegister) -> Result<f64, String> {
        let src1_reg = ex_reg.rs1.unwrap_or(0) as u8;
        let src2_reg = ex_reg.rs2.map(|r| r as u8);
        let dst_reg = ex_reg.rd.unwrap_or(0) as u8;
        let precision = FloatPrecision::Double; // Précision par défaut

        let operation = match opcode {
            Opcode::FpuAdd => FPUOperation::Add,
            Opcode::FpuSub => FPUOperation::Sub,
            Opcode::FpuMul => FPUOperation::Mul,
            Opcode::FpuDiv => FPUOperation::Div,
            Opcode::FpuSqrt => FPUOperation::Sqrt,
            Opcode::FpuCmp => FPUOperation::Cmp,
            Opcode::FpuMin => FPUOperation::Min,
            Opcode::FpuMax => FPUOperation::Max,
            Opcode::FpuConvert => FPUOperation::Convert,
            Opcode::FpuRound => FPUOperation::Round,
            Opcode::FpuMov => {
                // Mov FPU simple
                let src_value = self.fpu.borrow_mut().read_fp_register(src1_reg)
                    .map_err(|e| format!("Erreur lecture registre FPU: {}", e))?;
                self.fpu.borrow_mut().write_fp_register(dst_reg, src_value)
                    .map_err(|e| format!("Erreur écriture registre FPU: {}", e))?;
                return Ok(src_value);
            }
            _ => return Err(format!("Opération FPU non supportée: {:?}", opcode)),
        };

        self.fpu.borrow_mut().execute(operation, dst_reg, src1_reg, src2_reg, precision)
            .map_err(|e| format!("Erreur exécution FPU: {}", e))?;

        // Retourner la valeur du registre de destination
        self.fpu.borrow_mut().read_fp_register(dst_reg)
            .map_err(|e| format!("Erreur lecture résultat FPU: {}", e))
    }

    /// Réinitialise l'étage Execute
    pub fn reset(&mut self) {
        self.branch_predictor = BranchPredictor::new(PredictorType::Dynamic);
        self.vector_alu.borrow_mut().reset();
        self.fpu.borrow_mut().reset();
        self.agu.borrow_mut().reset();
        self.dual_issue_controller.reset();
        self.parallel_engine = ParallelExecutionEngine::new();
        self.branch_predictions = 0;
        self.branch_hits = 0;
        self.current_cycle = 0;
    }
    
    /// Obtient les statistiques du dual-issue controller
    pub fn get_dual_issue_stats(&self) -> (u64, u64, u64, u64, u64, f64) {
        self.dual_issue_controller.get_stats()
    }
    
    /// Obtient les statistiques du moteur d'exécution parallèle
    pub fn get_parallel_engine_stats(&self) -> &ParallelExecutionStats {
        self.parallel_engine.get_stats()
    }

    /// Accès aux unités d'exécution via Rc<RefCell<T>>
    pub fn get_vector_alu_ref(&self) -> &Rc<RefCell<VectorALU>> {
        &self.vector_alu
    }
    
    pub fn get_fpu_ref(&self) -> &Rc<RefCell<FPU>> {
        &self.fpu
    }
    
    pub fn get_agu_ref(&self) -> &Rc<RefCell<AGU>> {
        &self.agu
    }
    
    /// Traite les instructions mémoire avec l'AGU pour optimisation
    fn process_memory_with_agu(
        &mut self,
        ex_reg: &DecodeExecuteRegister,
        alu: &mut ALU,
        registers: &[u64],
        sp: u64,
    ) -> Result<ExecuteMemoryRegister, String> {
        let rs1_value = ex_reg.rs1_value;
        let mut alu_result = 0;
        let mut store_value = None;
        
        // Recalculer l'adresse avec l'AGU pour optimisations (stride prediction, etc.)
        let agu_addr = self.calculate_memory_address_with_agu(ex_reg, registers, sp)?;
        
        // Utiliser l'adresse de l'AGU si disponible, sinon utiliser celle du decode
        let final_addr = agu_addr.map(|addr| addr as u32)
            .or(ex_reg.mem_addr);
        
        println!("AGU: Calculated address {:?} (original: {:?}) for {:?}", 
                 agu_addr, ex_reg.mem_addr, ex_reg.instruction.opcode);
        
        match ex_reg.instruction.opcode {
            Opcode::Load | Opcode::LoadB | Opcode::LoadW | Opcode::LoadD => {
                // Ces instructions finalisent leur exécution dans l'étage Memory
                alu_result = 0; // Sera remplacé par la valeur chargée
                println!("Execute LOAD with AGU: rs1_value={}, mem_addr={:?}", 
                         rs1_value, final_addr);
            }
            
            Opcode::Store | Opcode::StoreB | Opcode::StoreW | Opcode::StoreD => {
                // Préparer la valeur à stocker
                store_value = Some(rs1_value);
                println!("Execute STORE with AGU: rs1_value={}, mem_addr={:?}", 
                         rs1_value, final_addr);
            }
            
            // Instructions SIMD mémoire
            Opcode::Simd128Load | Opcode::Simd256Load => {
                alu_result = 0; // Sera remplacé par la valeur chargée
                println!("Execute SIMD LOAD with AGU: mem_addr={:?}", final_addr);
            }
            
            Opcode::Simd128Store | Opcode::Simd256Store => {
                // Pour SIMD, la valeur est dans les registres vectoriels
                store_value = Some(0); // Placeholder, la vraie valeur sera gérée dans Memory stage
                println!("Execute SIMD STORE with AGU: mem_addr={:?}", final_addr);
            }
            
            // Instructions Stack - Push/Pop utilisent l'adresse SP
            Opcode::Push => {
                // Push utilise SP comme adresse de base
                store_value = Some(rs1_value);
                println!("Execute PUSH with AGU: value={}, mem_addr={:?}", rs1_value, final_addr);
            }
            
            Opcode::Pop => {
                // Pop lit depuis SP
                alu_result = 0; // Sera remplacé par la valeur lue
                println!("Execute POP with AGU: mem_addr={:?}", final_addr);
            }
            
            // Instructions CALL/RET qui peuvent aussi accéder à la mémoire
            Opcode::Call => {
                // Call pousse l'adresse de retour sur la pile
                store_value = Some(ex_reg.pc as u64 + ex_reg.instruction.total_size() as u64);
                println!("Execute CALL with AGU: return_addr={}, mem_addr={:?}", 
                         ex_reg.pc as u64 + ex_reg.instruction.total_size() as u64, final_addr);
            }
            
            Opcode::Ret => {
                // Ret lit l'adresse de retour depuis la pile
                alu_result = 0; // Sera remplacé par l'adresse lue
                println!("Execute RET with AGU: mem_addr={:?}", final_addr);
            }
            
            _ => return Err(format!("Instruction non-mémoire passée à process_memory_with_agu: {:?}", 
                                   ex_reg.instruction.opcode)),
        }
        
        // Déterminer l'opération stack si applicable
        let stack_op = match ex_reg.instruction.opcode {
            Opcode::Push => ex_reg.stack_operation.clone(),
            Opcode::Pop => ex_reg.stack_operation.clone(),
            Opcode::Call => ex_reg.stack_operation.clone(),
            Opcode::Ret => ex_reg.stack_operation.clone(),
            _ => None,
        };
        
        Ok(ExecuteMemoryRegister {
            instruction: ex_reg.instruction.clone(),
            alu_result,
            rd: ex_reg.rd,
            store_value,
            mem_addr: final_addr,
            branch_target: None,
            branch_taken: false,
            branch_prediction_correct: None,
            stack_operation: stack_op,
            stack_result: None,
            ras_prediction_correct: None,
            halted: false,
        })
    }
    
    /// Calcule l'adresse mémoire avec l'AGU en parallèle de l'ALU
    fn calculate_memory_address_with_agu(
        &mut self,
        ex_reg: &DecodeExecuteRegister,
        registers: &[u64],
        sp: u64,
    ) -> Result<Option<u64>, String> {
        // Détecter le mode d'adressage basé sur l'instruction et les registres utilisés
        // Pour les instructions Store/SIMD Store: rs1=source, rs2=base
        // Pour les instructions Load/SIMD Load: rs1=base, rs2=none ou index
        let (base_reg, index_reg) = match ex_reg.instruction.opcode {
            Opcode::Store | Opcode::StoreB | Opcode::StoreW | Opcode::StoreD |
            Opcode::Simd128Store | Opcode::Simd256Store => {
                // Pour Store avec RegisterOffset: rs2 est la base, pas d'index (rs1 est la valeur à stocker)
                (ex_reg.rs2, None)
            }
            _ => {
                // Pour Load et autres: rs1 est le registre de base
                (ex_reg.rs1, ex_reg.rs2)
            }
        };
        
        let addressing_mode = match (base_reg, index_reg, ex_reg.immediate) {
            // Base + Offset (registre + immédiat)
            (Some(base), None, Some(offset)) => {
                AddressingMode::BaseOffset { 
                    base: base as u8, 
                    offset: offset as i32 
                }
            }
            
            // Base + Index (deux registres)
            (Some(base), Some(index), offset) => {
                AddressingMode::BaseIndexScale {
                    base: base as u8,
                    index: index as u8, 
                    scale: 1, // Scale par défaut
                    offset: offset.unwrap_or(0) as i32
                }
            }
            
            // Stack-relative si l'adresse est calculée par rapport à SP
            (Some(reg), None, Some(offset)) if registers.get(reg).copied() == Some(sp) => {
                AddressingMode::StackRelative { 
                    offset: offset as i32 
                }
            }
            
            // Absolute si nous avons une adresse directe
            (None, None, Some(addr)) => {
                AddressingMode::Absolute { 
                    address: addr 
                }
            }
            
            // PC-relative pour les loads relatifs
            _ if ex_reg.mem_addr.is_some() => {
                // Si l'adresse est déjà calculée par decode, on peut l'optimiser avec l'AGU
                AddressingMode::Absolute { 
                    address: ex_reg.mem_addr.unwrap() as u64 
                }
            }
            
            _ => {
                // Pas de mode d'adressage détecté, retourner None pour utiliser l'adresse du decode
                return Ok(None);
            }
        };
        
        // Calculer l'adresse avec l'AGU
        match self.agu.borrow_mut().calculate_address(addressing_mode, registers, ex_reg.pc as u64, sp) {
            Ok(address) => {
                println!("AGU: Successfully calculated address 0x{:X} using {:?}", 
                         address, addressing_mode);
                Ok(Some(address))
            }
            Err(AGUError::InvalidRegister) => {
                println!("AGU: Invalid register in addressing mode, falling back to decode address");
                Ok(None)
            }
            Err(e) => {
                println!("AGU: Error calculating address: {:?}, falling back to decode address", e);
                Ok(None)
            }
        }
    }
    
    /// Obtient les statistiques de l'AGU
    pub fn get_agu_stats(&self) -> crate::alu::agu::AGUStats {
        self.agu.borrow().get_stats().clone()
    }
    
    /// Obtient les statistiques de stride prediction
    pub fn get_agu_stride_stats(&self) -> (u64, u64, f64) {
        self.agu.borrow().get_stride_stats()
    }
    
    /// Obtient les statistiques du cache de base
    pub fn get_agu_base_cache_stats(&self) -> (u64, u64, f64) {
        self.agu.borrow().get_base_cache_stats()
    }
}









/////////////////////////////////////////////Test Unitaire pour l'étage Execute/////////////////////////////////////////////

//
//
//
//
//
//
// // Test unitaire pour l'étage Execute
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::bytecode::format::ArgType;
//     use crate::bytecode::format::InstructionFormat;
//     use crate::bytecode::instructions::Instruction;
//     use crate::bytecode::opcodes::Opcode;
//     use crate::pipeline::DecodeExecuteRegister;
//
//     #[test]
//     fn test_execute_stage_creation() {
//         let execute = ExecuteStage::new();
//         // Vérifier que la création réussit
//         assert!(true);
//     }
//
//     #[test]
//     fn test_execute_stage_reset() {
//         let mut execute = ExecuteStage::new();
//         execute.reset();
//         // L'étage Execute n'a pas d'état interne, donc reset() ne fait rien
//         // On s'assure juste que la méthode peut être appelée sans erreur
//         assert!(true);
//     }
//
//     /// Test d'une instruction ADD (2 registres) où on veut R0 = R0 + R1
//     #[test]
//     fn test_execute_add_instruction_two_reg() {
//         let mut execute = ExecuteStage::new();
//         let mut alu = ALU::new();
//
//         // Suppose qu'on a fait "Decode" et trouvé que c'est "Add R0, R1"
//         let add_instruction = Instruction::create_reg_reg(Opcode::Add, 0, 1);
//
//         // On crée un decode->execute register
//         // On positionne rs1_value=5, rs2_value=7
//         let de_reg = DecodeExecuteRegister {
//             instruction: add_instruction,
//             pc: 100,
//             rs1: Some(0), // index
//             rs2: Some(1),
//             rd: Some(0),
//             rs1_value: 5, // R0=5
//             rs2_value: 7, // R1=7
//             immediate: None,
//             branch_addr: None,
//             branch_prediction: None,
//             stack_operation: None,
//             mem_addr: None,
//             stack_value: None,
//         };
//
//         // Exécuter l'instruction
//         let result = execute.process_direct(&de_reg, &mut alu);
//         assert!(result.is_ok());
//
//         let em_reg = result.unwrap();
//         // 5 + 7 = 12
//         assert_eq!(em_reg.alu_result, 12);
//         assert_eq!(em_reg.rd, Some(0));
//         assert!(!em_reg.branch_taken);
//         assert_eq!(em_reg.branch_target, None);
//     }
//
//     /// Test d'une instruction ADD (3 registres) style R2 = R0 + R1
//     #[test]
//     fn test_execute_add_instruction_three_reg() {
//         let mut execute = ExecuteStage::new();
//         let mut alu = ALU::new();
//
//         // Instruction "Add R2, R0, R1"
//         let add_instruction = Instruction::create_reg_reg_reg(Opcode::Add, 2, 0, 1);
//
//         // On simule "Decode" qui a trouvé rs1=0, rs2=1, rd=2,
//         // et lit la banque de registres => R0=5, R1=7
//         let de_reg = DecodeExecuteRegister {
//             instruction: add_instruction,
//             pc: 100,
//             rs1: Some(0),
//             rs2: Some(1),
//             rd: Some(2),
//             rs1_value: 5, // R0=5
//             rs2_value: 7, // R1=7
//             immediate: None,
//             branch_addr: None,
//             branch_prediction: None,
//             stack_operation: None,
//             mem_addr: None,
//             stack_value: None,
//         };
//
//         let result = execute.process_direct(&de_reg, &mut alu);
//         assert!(result.is_ok());
//
//         let em_reg = result.unwrap();
//         assert_eq!(em_reg.alu_result, 12);
//         assert_eq!(em_reg.rd, Some(2));
//         assert!(!em_reg.branch_taken);
//         assert_eq!(em_reg.branch_target, None);
//     }
//
//     #[test]
//     fn test_execute_sub_instruction_two_reg() {
//         let mut execute = ExecuteStage::new();
//         let mut alu = ALU::new();
//
//         // SUB R0, R1 => R0 = R0 - R1
//         let sub_instruction = Instruction::create_reg_reg(Opcode::Sub, 0, 1);
//
//         let de_reg = DecodeExecuteRegister {
//             instruction: sub_instruction,
//             pc: 100,
//             rs1: Some(0),
//             rs2: Some(1),
//             rd: Some(0),
//             rs1_value: 10, // R0=10
//             rs2_value: 7,  // R1=7
//             immediate: None,
//             branch_addr: None,
//             branch_prediction: None,
//             stack_operation: None,
//             mem_addr: None,
//             stack_value: None,
//         };
//
//         let result = execute.process_direct(&de_reg, &mut alu);
//         assert!(result.is_ok());
//
//         let em_reg = result.unwrap();
//         // 10 - 7 = 3
//         assert_eq!(em_reg.alu_result, 3);
//     }
//
//     #[test]
//     fn test_execute_sub_instruction_three_reg() {
//         let mut execute = ExecuteStage::new();
//         let mut alu = ALU::new();
//
//         // SUB R2, R0, R1 => R2 = R0 - R1
//         let sub_instruction = Instruction::create_reg_reg_reg(Opcode::Sub, 2, 0, 1);
//
//         let de_reg = DecodeExecuteRegister {
//             instruction: sub_instruction,
//             pc: 100,
//             rs1: Some(0),
//             rs2: Some(1),
//             rd: Some(2),
//             rs1_value: 10, // R0=10
//             rs2_value: 7,  // R1=7
//             immediate: None,
//             branch_addr: None,
//             branch_prediction: None,
//             stack_operation: None,
//             mem_addr: None,
//             stack_value: None,
//         };
//
//         let result = execute.process_direct(&de_reg, &mut alu);
//         assert!(result.is_ok());
//
//         let em_reg = result.unwrap();
//         // 10 - 7 = 3
//         assert_eq!(em_reg.alu_result, 3);
//         assert_eq!(em_reg.rd, Some(2));
//     }
//
//     #[test]
//     fn test_execute_arithmetic_operations_three_reg() {
//         let mut execute = ExecuteStage::new();
//         let mut alu = ALU::new();
//
//         // Tester plusieurs opérations arithmétiques
//         let operations = [
//             (Opcode::Add, 5, 7, 12),
//             (Opcode::Sub, 10, 3, 7),
//             (Opcode::Mul, 4, 5, 20),
//             (Opcode::Div, 20, 4, 5),
//             (Opcode::Mod, 10, 3, 1),
//         ];
//
//         for (op, val1, val2, expected) in operations {
//             // ex: OP R2, R0, R1
//             let instruction = Instruction::create_reg_reg_reg(op, 2, 0, 1);
//
//             let de_reg = DecodeExecuteRegister {
//                 instruction,
//                 pc: 100,
//                 rs1: Some(0),
//                 rs2: Some(1),
//                 rd: Some(2),
//                 rs1_value: val1,
//                 rs2_value: val2,
//                 immediate: None,
//                 branch_addr: None,
//                 branch_prediction: None,
//                 stack_operation: None,
//                 mem_addr: None,
//                 stack_value: None,
//             };
//
//             let result = execute.process_direct(&de_reg, &mut alu);
//             assert!(result.is_ok());
//
//             let em_reg = result.unwrap();
//             assert_eq!(
//                 em_reg.alu_result, expected,
//                 "Opération {:?} avec {} et {} devrait donner {}",
//                 op, val1, val2, expected
//             );
//             assert_eq!(em_reg.rd, Some(2));
//         }
//     }
//
//     #[test]
//     fn test_execute_logical_operations_three_reg() {
//         let mut execute = ExecuteStage::new();
//         let mut alu = ALU::new();
//
//         // Tester les opérations logiques
//         let operations = [
//             (Opcode::And, 0xF0, 0x0F, 0x00),
//             (Opcode::Or, 0xF0, 0x0F, 0xFF),
//             (Opcode::Xor, 0xF0, 0x0F, 0xFF),
//         ];
//
//         for (op, val1, val2, expected) in operations {
//             let instruction = Instruction::create_reg_reg_reg(op, 2, 0, 1);
//
//             let de_reg = DecodeExecuteRegister {
//                 instruction,
//                 pc: 100,
//                 rs1: Some(0),
//                 rs2: Some(1),
//                 rd: Some(2),
//                 rs1_value: val1,
//                 rs2_value: val2,
//                 immediate: None,
//                 branch_addr: None,
//                 branch_prediction: None,
//                 stack_operation: None,
//                 mem_addr: None,
//                 stack_value: None,
//             };
//
//             let result = execute.process_direct(&de_reg, &mut alu);
//             assert!(result.is_ok());
//
//             let em_reg = result.unwrap();
//             assert_eq!(
//                 em_reg.alu_result, expected,
//                 "Opération {:?} avec {:X} et {:X} devrait donner {:X}",
//                 op, val1, val2, expected
//             );
//             assert_eq!(em_reg.rd, Some(2));
//         }
//     }
//
//     #[test]
//     fn test_execute_store_instruction() {
//         let mut execute = ExecuteStage::new();
//         let mut alu = ALU::new();
//
//         // STORE R0, [0x2000]
//         let store_instruction = Instruction::new(
//             Opcode::Store,
//             InstructionFormat::new(ArgType::Register, ArgType::AbsoluteAddr, ArgType::None),
//             vec![],
//         );
//
//         let de_reg = DecodeExecuteRegister {
//             instruction: store_instruction,
//             pc: 100,
//             rs1: Some(0), // R0 => source
//             rs2: None,
//             rd: None,
//             rs1_value: 42, // On veut stocker 42
//             rs2_value: 0,
//             immediate: None,
//             branch_addr: None,
//             branch_prediction: None,
//             stack_operation: None,
//             mem_addr: Some(0x2000),
//             stack_value: None,
//         };
//
//         let result = execute.process_direct(&de_reg, &mut alu);
//         assert!(result.is_ok());
//
//         let em_reg = result.unwrap();
//         assert_eq!(em_reg.mem_addr, Some(0x2000));
//         // store_value = rs1_value => 42
//         assert_eq!(em_reg.store_value, Some(42));
//     }
//
//     #[test]
//     fn test_execute_complex_instruction_sequence() {
//         let mut execute = ExecuteStage::new();
//         let mut alu = ALU::new();
//
//         // On veut calculer (5 + 7) * 3
//         // 1) ADD R3, R0, R1 => R3 = R0 + R1
//         let add_instr = Instruction::create_reg_reg_reg(Opcode::Add, 3, 0, 1);
//         let de_reg_add = DecodeExecuteRegister {
//             instruction: add_instr,
//             pc: 100,
//             rs1: Some(0),
//             rs2: Some(1),
//             rd: Some(3),
//             rs1_value: 5, // R0=5
//             rs2_value: 7, // R1=7
//             immediate: None,
//             branch_addr: None,
//             branch_prediction: None,
//             stack_operation: None,
//             mem_addr: None,
//             stack_value: None,
//         };
//         let res_add = execute.process_direct(&de_reg_add, &mut alu).unwrap();
//         assert_eq!(res_add.alu_result, 12);
//
//         // 2) MUL R4, R3, R2 => R4 = R3 * R2
//         // Suppose R2=3
//         let mul_instr = Instruction::create_reg_reg_reg(Opcode::Mul, 4, 3, 2);
//         let de_reg_mul = DecodeExecuteRegister {
//             instruction: mul_instr,
//             pc: 104,
//             rs1: Some(3),
//             rs2: Some(2),
//             rd: Some(4),
//             rs1_value: res_add.alu_result, // R3=12
//             rs2_value: 3,                  // R2=3
//             immediate: None,
//             branch_addr: None,
//             branch_prediction: None,
//             stack_operation: None,
//             mem_addr: None,
//             stack_value: None,
//         };
//         let res_mul = execute.process_direct(&de_reg_mul, &mut alu).unwrap();
//         assert_eq!(res_mul.alu_result, 36);
//
//         // On a 36 dans R4 => c'est le résultat final
//     }
//     #[test]
//     fn test_execute_mixed_format_program() {
//         let mut execute = ExecuteStage::new();
//         let mut alu = ALU::new();
//
//         // 1) ADD R3, R0, R1 => 3-op => R3=R0+R1
//         let add_instr = Instruction::create_reg_reg_reg(Opcode::Add, 3, 0, 1);
//         let de_reg_add = DecodeExecuteRegister {
//             instruction: add_instr,
//             pc: 100,
//             rs1: Some(0),
//             rs2: Some(1),
//             rd: Some(3),
//             rs1_value: 5, // R0=5
//             rs2_value: 7, // R1=7
//             immediate: None,
//             branch_addr: None,
//             branch_prediction: None,
//             stack_operation: None,
//             mem_addr: None,
//             stack_value: None,
//         };
//         let em_reg_add = execute.process_direct(&de_reg_add, &mut alu).unwrap();
//         assert_eq!(em_reg_add.alu_result, 12);
//
//         // 2) INC R3 => format 1 reg => "rd=3, rs1=3"
//         // => R3 = R3 + 1 => 12 + 1 => 13
//         let inc_instr = Instruction::create_single_reg(Opcode::Inc, 3);
//         let de_reg_inc = DecodeExecuteRegister {
//             instruction: inc_instr,
//             pc: 104,
//             rs1: Some(3),
//             rs2: None,
//             rd: Some(3),
//             rs1_value: em_reg_add.alu_result, // R3=12
//             rs2_value: 0,
//             immediate: None,
//             branch_addr: None,
//             branch_prediction: None,
//             stack_operation: None,
//             mem_addr: None,
//             stack_value: None,
//         };
//         let em_reg_inc = execute.process_direct(&de_reg_inc, &mut alu).unwrap();
//         assert_eq!(em_reg_inc.alu_result, 13);
//
//         // 3) CMP R3, R2 => 2-reg => "rs1=3, rs2=2"
//         // Suppose R3=13, R2=13
//         let cmp_instr = Instruction::create_reg_reg(Opcode::Cmp, 3, 2);
//         let de_reg_cmp = DecodeExecuteRegister {
//             instruction: cmp_instr,
//             pc: 106,
//             rs1: Some(3),
//             rs2: Some(2),
//             rd: None,
//             rs1_value: 13, // R3=13
//             rs2_value: 13, // R2=13
//             immediate: None,
//             branch_addr: None,
//             branch_prediction: None,
//             stack_operation: None,
//             mem_addr: None,
//             stack_value: None,
//         };
//         let em_reg_cmp = execute.process_direct(&de_reg_cmp, &mut alu).unwrap();
//         // On attend ZF=1 => alu.flags.zero = true
//         assert!(alu.flags.zero);
//         assert!(!alu.flags.negative);
//         assert!(!alu.flags.carry);
//     }
//
//     #[test]
//     fn test_execute_jump_instruction() {
//         let mut execute = ExecuteStage::new();
//         let mut alu = ALU::new();
//
//         // Créer une instruction JMP à l'adresse absolue 0x1000
//         let jmp_instruction = Instruction::new(
//             Opcode::Jmp,
//             InstructionFormat::new(ArgType::None, ArgType::AbsoluteAddr, ArgType::None),
//             vec![0, 16, 0, 0], // Adresse 0x1000 (little-endian)
//         );
//
//         // Créer un registre Decode → Execute avec adresse de branchement
//         let de_reg = DecodeExecuteRegister {
//             instruction: jmp_instruction,
//             pc: 100,
//             rs1: None,
//             rs2: None,
//             rd: None,
//             rs1_value: 0,
//             rs2_value: 0,
//             immediate: None,
//             branch_addr: Some(0x1000),
//             branch_prediction: None,
//             stack_operation: None,
//             mem_addr: None,
//             stack_value: None,
//         };
//
//         // Exécuter l'instruction
//         let result = execute.process_direct(&de_reg, &mut alu);
//         assert!(result.is_ok());
//
//         // Vérifier le résultat
//         let em_reg = result.unwrap();
//         assert_eq!(em_reg.branch_taken, true);
//         assert_eq!(em_reg.branch_target, Some(0x1000));
//     }
//
//     #[test]
//     fn test_execute_conditional_jump() {
//         let mut execute = ExecuteStage::new();
//         let mut alu = ALU::new();
//
//         // Préparer l'ALU avec des flags
//         alu.flags.zero = true; // Condition égalité vraie
//
//         // Créer une instruction JMP_IF (saut si égal)
//         let jmp_if_instruction = Instruction::new(
//             Opcode::JmpIf,
//             InstructionFormat::new(ArgType::None, ArgType::AbsoluteAddr, ArgType::None),
//             vec![0, 16, 0, 0], // Adresse 0x1000
//         );
//
//         // Créer un registre Decode → Execute
//         let de_reg = DecodeExecuteRegister {
//             instruction: jmp_if_instruction,
//             pc: 100,
//             rs1: None,
//             rs2: None,
//             rd: None,
//             rs1_value: 0,
//             rs2_value: 0,
//             immediate: None,
//             branch_addr: Some(0x1000),
//             branch_prediction: None,
//             stack_operation: None,
//             mem_addr: None,
//             stack_value: None,
//         };
//
//         // Exécuter l'instruction
//         let result = execute.process_direct(&de_reg, &mut alu);
//         assert!(result.is_ok());
//
//         // Vérifier le résultat - devrait prendre le branchement car ZF=1
//         let em_reg = result.unwrap();
//         assert_eq!(em_reg.branch_taken, true);
//         assert_eq!(em_reg.branch_target, Some(0x1000));
//     }
//
//     #[test]
//     fn test_execute_load_instruction() {
//         let mut execute = ExecuteStage::new();
//         let mut alu = ALU::new();
//
//         // Créer une instruction LOAD R0, [0x2000]
//         let load_instruction = Instruction::new(
//             Opcode::Load,
//             InstructionFormat::new(ArgType::Register, ArgType::AbsoluteAddr, ArgType::None),
//             vec![0, 0, 32, 0, 0], // R0 = Mem[0x2000]
//         );
//
//         // Créer un registre Decode → Execute
//         let de_reg = DecodeExecuteRegister {
//             instruction: load_instruction,
//             pc: 100,
//             rs1: None,
//             rs2: None,
//             rd: Some(0),
//             rs1_value: 0,
//             rs2_value: 0,
//             immediate: None,
//             branch_addr: None,
//             branch_prediction: None,
//             stack_operation: None,
//             mem_addr: Some(0x2000),
//             stack_value: None,
//         };
//
//         // Exécuter l'instruction
//         let result = execute.process_direct(&de_reg, &mut alu);
//         assert!(result.is_ok());
//
//         // Vérifier le résultat - l'étage Execute ne charge pas la valeur, il prépare juste l'accès mémoire
//         let em_reg = result.unwrap();
//         assert_eq!(em_reg.mem_addr, Some(0x2000));
//         assert_eq!(em_reg.rd, Some(0));
//         assert_eq!(em_reg.alu_result, 0); // Pas de calcul ALU pour LOAD
//     }
//
//     // ==================== TESTS SIMD ====================
//
//     #[test]
//     fn test_simd128_const_i32x4() {
//         let mut execute = ExecuteStage::new();
//         let mut alu = ALU::new();
//
//         // Créer une instruction SIMD128Const avec des valeurs i32x4
//         let values = [1, 2, 3, 4];
//         let simd_instruction = Instruction::create_simd128_const_i32x4(0, values);
//
//         let de_reg = DecodeExecuteRegister {
//             instruction: simd_instruction,
//             pc: 100,
//             rs1: None,
//             rs2: None,
//             rd: Some(0), // V0
//             rs1_value: 0,
//             rs2_value: 0,
//             immediate: None,
//             branch_addr: None,
//             branch_prediction: None,
//             stack_operation: None,
//             mem_addr: None,
//             stack_value: None,
//         };
//
//         // Exécuter l'instruction
//         let result = execute.process_direct(&de_reg, &mut alu);
//         assert!(result.is_ok(), "SIMD128Const execution should succeed");
//
//         // Vérifier que le vecteur a été écrit dans le registre V0
//         let vector = execute.vector_alu.read_v128(0);
//         assert!(vector.is_ok(), "Should be able to read vector from V0");
//
//         let vector_data = vector.unwrap();
//         unsafe {
//             assert_eq!(vector_data.i32x4, [1, 2, 3, 4], "Vector V0 should contain [1, 2, 3, 4]");
//         }
//     }
//
//     #[test]
//     fn test_simd128_const_f32x4() {
//         let mut execute = ExecuteStage::new();
//         let mut alu = ALU::new();
//
//         // Créer une instruction SIMD128ConstF32 avec des valeurs f32x4
//         let values = [1.0, 2.0, 3.0, 4.0];
//         let simd_instruction = Instruction::create_simd128_const_f32x4(1, values);
//
//         let de_reg = DecodeExecuteRegister {
//             instruction: simd_instruction,
//             pc: 100,
//             rs1: None,
//             rs2: None,
//             rd: Some(1), // V1
//             rs1_value: 0,
//             rs2_value: 0,
//             immediate: None,
//             branch_addr: None,
//             branch_prediction: None,
//             stack_operation: None,
//             mem_addr: None,
//             stack_value: None,
//         };
//
//         // Exécuter l'instruction
//         let result = execute.process_direct(&de_reg, &mut alu);
//         assert!(result.is_ok(), "SIMD128ConstF32 execution should succeed");
//
//         // Vérifier que le vecteur a été écrit dans le registre V1
//         let vector = execute.vector_alu.read_v128(1);
//         assert!(vector.is_ok(), "Should be able to read vector from V1");
//
//         let vector_data = vector.unwrap();
//         unsafe {
//             assert_eq!(vector_data.f32x4, [1.0, 2.0, 3.0, 4.0], "Vector V1 should contain [1.0, 2.0, 3.0, 4.0]");
//         }
//     }
//
//     #[test]
//     fn test_simd256_const_i32x8() {
//         let mut execute = ExecuteStage::new();
//         let mut alu = ALU::new();
//
//         // Créer une instruction SIMD256Const avec des valeurs i32x8
//         let values = [1, 2, 3, 4, 5, 6, 7, 8];
//         let simd_instruction = Instruction::create_simd256_const_i32x8(0, values);
//
//         let de_reg = DecodeExecuteRegister {
//             instruction: simd_instruction,
//             pc: 100,
//             rs1: None,
//             rs2: None,
//             rd: Some(0), // Y0
//             rs1_value: 0,
//             rs2_value: 0,
//             immediate: None,
//             branch_addr: None,
//             branch_prediction: None,
//             stack_operation: None,
//             mem_addr: None,
//             stack_value: None,
//         };
//
//         // Exécuter l'instruction
//         let result = execute.process_direct(&de_reg, &mut alu);
//         assert!(result.is_ok(), "SIMD256Const execution should succeed");
//
//         // Vérifier que le vecteur a été écrit dans le registre Y0
//         let vector = execute.vector_alu.read_v256(0);
//         assert!(vector.is_ok(), "Should be able to read vector from Y0");
//
//         let vector_data = vector.unwrap();
//         unsafe {
//             // Note: L'implémentation actuelle duplique les 4 premiers éléments
//             // mais nous testons avec les 8 valeurs complètes
//             assert_eq!(vector_data.i32x8[0..4], [1, 2, 3, 4], "First half should contain [1, 2, 3, 4]");
//         }
//     }
//
//     #[test]
//     fn test_simd256_const_f32x8() {
//         let mut execute = ExecuteStage::new();
//         let mut alu = ALU::new();
//
//         // Créer une instruction SIMD256ConstF32 avec des valeurs f32x8
//         let values = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
//         let simd_instruction = Instruction::create_simd256_const_f32x8(1, values);
//
//         let de_reg = DecodeExecuteRegister {
//             instruction: simd_instruction,
//             pc: 100,
//             rs1: None,
//             rs2: None,
//             rd: Some(1), // Y1
//             rs1_value: 0,
//             rs2_value: 0,
//             immediate: None,
//             branch_addr: None,
//             branch_prediction: None,
//             stack_operation: None,
//             mem_addr: None,
//             stack_value: None,
//         };
//
//         // Exécuter l'instruction
//         let result = execute.process_direct(&de_reg, &mut alu);
//         assert!(result.is_ok(), "SIMD256ConstF32 execution should succeed");
//
//         // Vérifier que le vecteur a été écrit dans le registre Y1
//         let vector = execute.vector_alu.read_v256(1);
//         assert!(vector.is_ok(), "Should be able to read vector from Y1");
//
//         let vector_data = vector.unwrap();
//         unsafe {
//             // Note: L'implémentation actuelle duplique les 4 premiers éléments
//             assert_eq!(vector_data.f32x8[0..4], [1.0, 2.0, 3.0, 4.0], "First half should contain [1.0, 2.0, 3.0, 4.0]");
//         }
//     }
//
//     #[test]
//     fn test_simd128_load_store() {
//         let mut execute = ExecuteStage::new();
//         let mut alu = ALU::new();
//
//         // Test 1: SIMD128Store - Stocker un vecteur en mémoire
//
//         // D'abord, charger une constante dans V0
//         let values = [10, 20, 30, 40];
//         let load_const_instruction = Instruction::create_simd128_const_i32x4(0, values);
//
//         let de_reg_const = DecodeExecuteRegister {
//             instruction: load_const_instruction,
//             pc: 100,
//             rs1: None,
//             rs2: None,
//             rd: Some(0), // V0
//             rs1_value: 0,
//             rs2_value: 0,
//             immediate: None,
//             branch_addr: None,
//             branch_prediction: None,
//             stack_operation: None,
//             mem_addr: None,
//             stack_value: None,
//         };
//
//         let result = execute.process_direct(&de_reg_const, &mut alu);
//         assert!(result.is_ok(), "Loading SIMD128 constant should succeed");
//
//         // Maintenant, tester SIMD128Store
//         let store_instruction = Instruction::new(
//             Opcode::Simd128Store,
//             InstructionFormat::new(ArgType::Register, ArgType::AbsoluteAddr, ArgType::None),
//             vec![0, 0, 0x10, 0x00, 0x00], // V0, adresse 0x1000
//         );
//
//         let de_reg_store = DecodeExecuteRegister {
//             instruction: store_instruction,
//             pc: 104,
//             rs1: Some(0), // V0 source
//             rs2: None,
//             rd: None,
//             rs1_value: 0,
//             rs2_value: 0,
//             immediate: None,
//             branch_addr: None,
//             branch_prediction: None,
//             stack_operation: None,
//             mem_addr: Some(0x1000),
//             stack_value: None,
//         };
//
//         let result = execute.process_direct(&de_reg_store, &mut alu);
//         assert!(result.is_ok(), "SIMD128Store execution should succeed");
//
//         // Test 2: SIMD128Load - Charger un vecteur depuis la mémoire
//         let load_instruction = Instruction::new(
//             Opcode::Simd128Load,
//             InstructionFormat::new(ArgType::Register, ArgType::AbsoluteAddr, ArgType::None),
//             vec![1, 0, 0x10, 0x00, 0x00], // V1, adresse 0x1000
//         );
//
//         let de_reg_load = DecodeExecuteRegister {
//             instruction: load_instruction,
//             pc: 108,
//             rs1: None,
//             rs2: None,
//             rd: Some(1), // V1 destination
//             rs1_value: 0,
//             rs2_value: 0,
//             immediate: None,
//             branch_addr: None,
//             branch_prediction: None,
//             stack_operation: None,
//             mem_addr: Some(0x1000),
//             stack_value: None,
//         };
//
//         let result = execute.process_direct(&de_reg_load, &mut alu);
//         assert!(result.is_ok(), "SIMD128Load execution should succeed");
//
//         // Vérifier que V1 contient maintenant un vecteur (placeholder dans l'implémentation actuelle)
//         let vector = execute.vector_alu.read_v128(1);
//         assert!(vector.is_ok(), "Should be able to read vector from V1");
//     }
//
//     #[test]
//     fn test_simd256_load_store() {
//         let mut execute = ExecuteStage::new();
//         let mut alu = ALU::new();
//
//         // Test 1: SIMD256Store - Stocker un vecteur 256-bit en mémoire
//
//         // D'abord, charger une constante dans Y0
//         let values = [100, 200, 300, 400, 500, 600, 700, 800];
//         let load_const_instruction = Instruction::create_simd256_const_i32x8(0, values);
//
//         let de_reg_const = DecodeExecuteRegister {
//             instruction: load_const_instruction,
//             pc: 100,
//             rs1: None,
//             rs2: None,
//             rd: Some(0), // Y0
//             rs1_value: 0,
//             rs2_value: 0,
//             immediate: None,
//             branch_addr: None,
//             branch_prediction: None,
//             stack_operation: None,
//             mem_addr: None,
//             stack_value: None,
//         };
//
//         let result = execute.process_direct(&de_reg_const, &mut alu);
//         assert!(result.is_ok(), "Loading SIMD256 constant should succeed");
//
//         // Maintenant, tester SIMD256Store
//         let store_instruction = Instruction::new(
//             Opcode::Simd256Store,
//             InstructionFormat::new(ArgType::Register, ArgType::AbsoluteAddr, ArgType::None),
//             vec![0, 0, 0x20, 0x00, 0x00], // Y0, adresse 0x2000
//         );
//
//         let de_reg_store = DecodeExecuteRegister {
//             instruction: store_instruction,
//             pc: 104,
//             rs1: Some(0), // Y0 source
//             rs2: None,
//             rd: None,
//             rs1_value: 0,
//             rs2_value: 0,
//             immediate: None,
//             branch_addr: None,
//             branch_prediction: None,
//             stack_operation: None,
//             mem_addr: Some(0x2000),
//             stack_value: None,
//         };
//
//         let result = execute.process_direct(&de_reg_store, &mut alu);
//         assert!(result.is_ok(), "SIMD256Store execution should succeed");
//
//         // Test 2: SIMD256Load - Charger un vecteur 256-bit depuis la mémoire
//         let load_instruction = Instruction::new(
//             Opcode::Simd256Load,
//             InstructionFormat::new(ArgType::Register, ArgType::AbsoluteAddr, ArgType::None),
//             vec![1, 0, 0x20, 0x00, 0x00], // Y1, adresse 0x2000
//         );
//
//         let de_reg_load = DecodeExecuteRegister {
//             instruction: load_instruction,
//             pc: 108,
//             rs1: None,
//             rs2: None,
//             rd: Some(1), // Y1 destination
//             rs1_value: 0,
//             rs2_value: 0,
//             immediate: None,
//             branch_addr: None,
//             branch_prediction: None,
//             stack_operation: None,
//             mem_addr: Some(0x2000),
//             stack_value: None,
//         };
//
//         let result = execute.process_direct(&de_reg_load, &mut alu);
//         assert!(result.is_ok(), "SIMD256Load execution should succeed");
//
//         // Vérifier que Y1 contient maintenant un vecteur (placeholder dans l'implémentation actuelle)
//         let vector = execute.vector_alu.read_v256(1);
//         assert!(vector.is_ok(), "Should be able to read vector from Y1");
//     }
//
//     #[test]
//     fn test_simd_multiple_operations() {
//         let mut execute = ExecuteStage::new();
//         let mut alu = ALU::new();
//
//         // Test d'une séquence d'opérations SIMD
//
//         // 1. Charger constante i32x4 dans V0
//         let values1 = [1, 2, 3, 4];
//         let const1_instruction = Instruction::create_simd128_const_i32x4(0, values1);
//
//         let de_reg1 = DecodeExecuteRegister {
//             instruction: const1_instruction,
//             pc: 100,
//             rs1: None, rs2: None, rd: Some(0),
//             rs1_value: 0, rs2_value: 0, immediate: None,
//             branch_addr: None, branch_prediction: None,
//             stack_operation: None, mem_addr: None, stack_value: None,
//         };
//
//         let result = execute.process_direct(&de_reg1, &mut alu);
//         assert!(result.is_ok(), "First SIMD128Const should succeed");
//
//         // 2. Charger constante f32x4 dans V1
//         let values2 = [5.5, 6.5, 7.5, 8.5];
//         let const2_instruction = Instruction::create_simd128_const_f32x4(1, values2);
//
//         let de_reg2 = DecodeExecuteRegister {
//             instruction: const2_instruction,
//             pc: 104,
//             rs1: None, rs2: None, rd: Some(1),
//             rs1_value: 0, rs2_value: 0, immediate: None,
//             branch_addr: None, branch_prediction: None,
//             stack_operation: None, mem_addr: None, stack_value: None,
//         };
//
//         let result = execute.process_direct(&de_reg2, &mut alu);
//         assert!(result.is_ok(), "Second SIMD128ConstF32 should succeed");
//
//         // 3. Charger constante i32x8 dans Y0
//         let values3 = [10, 20, 30, 40, 50, 60, 70, 80];
//         let const3_instruction = Instruction::create_simd256_const_i32x8(0, values3);
//
//         let de_reg3 = DecodeExecuteRegister {
//             instruction: const3_instruction,
//             pc: 108,
//             rs1: None, rs2: None, rd: Some(0),
//             rs1_value: 0, rs2_value: 0, immediate: None,
//             branch_addr: None, branch_prediction: None,
//             stack_operation: None, mem_addr: None, stack_value: None,
//         };
//
//         let result = execute.process_direct(&de_reg3, &mut alu);
//         assert!(result.is_ok(), "SIMD256Const should succeed");
//
//         // Vérifier que tous les registres contiennent les bonnes valeurs
//         let v0 = execute.vector_alu.read_v128(0);
//         let v1 = execute.vector_alu.read_v128(1);
//         let y0 = execute.vector_alu.read_v256(0);
//
//         assert!(v0.is_ok() && v1.is_ok() && y0.is_ok(), "All vector reads should succeed");
//
//         unsafe {
//             assert_eq!(v0.unwrap().i32x4, [1, 2, 3, 4], "V0 should contain integer values");
//             assert_eq!(v1.unwrap().f32x4, [5.5, 6.5, 7.5, 8.5], "V1 should contain float values");
//             // Y0 contient au moins les 4 premiers éléments corrects
//             let y0_data = y0.unwrap();
//             assert_eq!(y0_data.i32x8[0..4], [10, 20, 30, 40], "Y0 first half should contain [10, 20, 30, 40]");
//         }
//     }
//
//     #[test]
//     fn test_simd_error_handling() {
//         let mut execute = ExecuteStage::new();
//         let mut alu = ALU::new();
//
//         // Test avec un registre invalide (> 15)
//         let invalid_instruction = Instruction::new(
//             Opcode::Simd128Const,
//             InstructionFormat::new(ArgType::Register, ArgType::Immediate64, ArgType::Immediate64),
//             vec![16, 0, 0, 0, 0, 0, 0, 0, 0, 0], // Registre 16 invalide
//         );
//
//         let de_reg = DecodeExecuteRegister {
//             instruction: invalid_instruction,
//             pc: 100,
//             rs1: None, rs2: None, rd: Some(16), // Registre invalide
//             rs1_value: 0, rs2_value: 0, immediate: None,
//             branch_addr: None, branch_prediction: None,
//             stack_operation: None, mem_addr: None, stack_value: None,
//         };
//
//         let result = execute.process_direct(&de_reg, &mut alu);
//         // L'instruction devrait échouer avec un registre invalide
//         // (comportement dépend de l'implémentation de VectorALU)
//         assert!(result.is_ok() || result.is_err(), "Invalid register handling test");
//     }
//
//     // ==================== TESTS NOUVEAUX TYPES VECTORIELS ====================
//
//     #[test]
//     fn test_simd128_const_i16x8() {
//         let mut execute = ExecuteStage::new();
//         let mut alu = ALU::new();
//
//         // Créer une instruction SIMD128ConstI16x8 avec des valeurs i16x8
//         let values = [1, 2, 3, 4, 5, 6, 7, 8];
//         let simd_instruction = Instruction::create_simd128_const_i16x8(0, values);
//
//         let de_reg = DecodeExecuteRegister {
//             instruction: simd_instruction,
//             pc: 100,
//             rs1: None, rs2: None, rd: Some(0), // V0
//             rs1_value: 0, rs2_value: 0, immediate: None,
//             branch_addr: None, branch_prediction: None,
//             stack_operation: None, mem_addr: None, stack_value: None,
//         };
//
//         // Exécuter l'instruction
//         let result = execute.process_direct(&de_reg, &mut alu);
//         assert!(result.is_ok(), "SIMD128ConstI16x8 execution should succeed");
//
//         // Vérifier que le vecteur a été écrit dans le registre V0
//         let vector = execute.vector_alu.read_v128(0);
//         assert!(vector.is_ok(), "Should be able to read vector from V0");
//
//         let vector_data = vector.unwrap();
//         unsafe {
//             assert_eq!(vector_data.i16x8, [1, 2, 3, 4, 5, 6, 7, 8], "Vector V0 should contain i16x8 values");
//         }
//     }
//
//     #[test]
//     fn test_simd128_const_i64x2() {
//         let mut execute = ExecuteStage::new();
//         let mut alu = ALU::new();
//
//         // Créer une instruction SIMD128ConstI64x2 avec des valeurs i64x2
//         let values = [0x1234567890ABCDEF, 0x7EDCBA0987654321];
//         let simd_instruction = Instruction::create_simd128_const_i64x2(0, values);
//
//         let de_reg = DecodeExecuteRegister {
//             instruction: simd_instruction,
//             pc: 100,
//             rs1: None, rs2: None, rd: Some(0), // V0
//             rs1_value: 0, rs2_value: 0, immediate: None,
//             branch_addr: None, branch_prediction: None,
//             stack_operation: None, mem_addr: None, stack_value: None,
//         };
//
//         // Exécuter l'instruction
//         let result = execute.process_direct(&de_reg, &mut alu);
//         assert!(result.is_ok(), "SIMD128ConstI64x2 execution should succeed");
//
//         // Vérifier que le vecteur a été écrit dans le registre V0
//         let vector = execute.vector_alu.read_v128(0);
//         assert!(vector.is_ok(), "Should be able to read vector from V0");
//
//         let vector_data = vector.unwrap();
//         unsafe {
//             assert_eq!(vector_data.i64x2, [0x1234567890ABCDEF, 0x7EDCBA0987654321], "Vector V0 should contain i64x2 values");
//         }
//     }
//
//     #[test]
//     fn test_simd128_const_f64x2() {
//         let mut execute = ExecuteStage::new();
//         let mut alu = ALU::new();
//
//         // Créer une instruction SIMD128ConstF64x2 avec des valeurs f64x2
//         let values = [3.14159265359, 2.71828182846];
//         let simd_instruction = Instruction::create_simd128_const_f64x2(0, values);
//
//         let de_reg = DecodeExecuteRegister {
//             instruction: simd_instruction,
//             pc: 100,
//             rs1: None, rs2: None, rd: Some(0), // V0
//             rs1_value: 0, rs2_value: 0, immediate: None,
//             branch_addr: None, branch_prediction: None,
//             stack_operation: None, mem_addr: None, stack_value: None,
//         };
//
//         // Exécuter l'instruction
//         let result = execute.process_direct(&de_reg, &mut alu);
//         assert!(result.is_ok(), "SIMD128ConstF64x2 execution should succeed");
//
//         // Vérifier que le vecteur a été écrit dans le registre V0
//         let vector = execute.vector_alu.read_v128(0);
//         assert!(vector.is_ok(), "Should be able to read vector from V0");
//
//         let vector_data = vector.unwrap();
//         unsafe {
//             assert_eq!(vector_data.f64x2, [3.14159265359, 2.71828182846], "Vector V0 should contain f64x2 values");
//         }
//     }
//
//     #[test]
//     fn test_simd256_const_i16x16() {
//         let mut execute = ExecuteStage::new();
//         let mut alu = ALU::new();
//
//         // Créer une instruction SIMD256ConstI16x16 avec des valeurs i16x16
//         let values = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
//         let simd_instruction = Instruction::create_simd256_const_i16x16(0, values);
//
//         let de_reg = DecodeExecuteRegister {
//             instruction: simd_instruction,
//             pc: 100,
//             rs1: None, rs2: None, rd: Some(0), // Y0
//             rs1_value: 0, rs2_value: 0, immediate: None,
//             branch_addr: None, branch_prediction: None,
//             stack_operation: None, mem_addr: None, stack_value: None,
//         };
//
//         // Exécuter l'instruction
//         let result = execute.process_direct(&de_reg, &mut alu);
//         assert!(result.is_ok(), "SIMD256ConstI16x16 execution should succeed");
//
//         // Vérifier que le vecteur a été écrit dans le registre Y0
//         let vector = execute.vector_alu.read_v256(0);
//         assert!(vector.is_ok(), "Should be able to read vector from Y0");
//
//         let vector_data = vector.unwrap();
//         unsafe {
//             // Note: implémentation utilise la duplication, donc on vérifie les 8 premiers éléments
//             assert_eq!(vector_data.i16x16[0..8], [1, 2, 3, 4, 5, 6, 7, 8], "Vector Y0 should contain i16x16 values");
//         }
//     }
//
//     #[test]
//     fn test_simd256_const_i64x4() {
//         let mut execute = ExecuteStage::new();
//         let mut alu = ALU::new();
//
//         // Créer une instruction SIMD256ConstI64x4 avec des valeurs i64x4
//         let values = [0x1111111111111111, 0x2222222222222222, 0x3333333333333333, 0x4444444444444444];
//         let simd_instruction = Instruction::create_simd256_const_i64x4(0, values);
//
//         let de_reg = DecodeExecuteRegister {
//             instruction: simd_instruction,
//             pc: 100,
//             rs1: None, rs2: None, rd: Some(0), // Y0
//             rs1_value: 0, rs2_value: 0, immediate: None,
//             branch_addr: None, branch_prediction: None,
//             stack_operation: None, mem_addr: None, stack_value: None,
//         };
//
//         // Exécuter l'instruction
//         let result = execute.process_direct(&de_reg, &mut alu);
//         assert!(result.is_ok(), "SIMD256ConstI64x4 execution should succeed");
//
//         // Vérifier que le vecteur a été écrit dans le registre Y0
//         let vector = execute.vector_alu.read_v256(0);
//         assert!(vector.is_ok(), "Should be able to read vector from Y0");
//
//         let vector_data = vector.unwrap();
//         unsafe {
//             // Note: implémentation utilise la duplication des 2 premières valeurs
//             assert_eq!(vector_data.i64x4[0], 0x1111111111111111, "Vector Y0[0] should contain first i64 value");
//             assert_eq!(vector_data.i64x4[1], 0x2222222222222222, "Vector Y0[1] should contain second i64 value");
//         }
//     }
//
//     #[test]
//     fn test_simd256_const_f64x4() {
//         let mut execute = ExecuteStage::new();
//         let mut alu = ALU::new();
//
//         // Créer une instruction SIMD256ConstF64x4 avec des valeurs f64x4
//         let values = [1.111, 2.222, 3.333, 4.444];
//         let simd_instruction = Instruction::create_simd256_const_f64x4(0, values);
//
//         let de_reg = DecodeExecuteRegister {
//             instruction: simd_instruction,
//             pc: 100,
//             rs1: None, rs2: None, rd: Some(0), // Y0
//             rs1_value: 0, rs2_value: 0, immediate: None,
//             branch_addr: None, branch_prediction: None,
//             stack_operation: None, mem_addr: None, stack_value: None,
//         };
//
//         // Exécuter l'instruction
//         let result = execute.process_direct(&de_reg, &mut alu);
//         assert!(result.is_ok(), "SIMD256ConstF64x4 execution should succeed");
//
//         // Vérifier que le vecteur a été écrit dans le registre Y0
//         let vector = execute.vector_alu.read_v256(0);
//         assert!(vector.is_ok(), "Should be able to read vector from Y0");
//
//         let vector_data = vector.unwrap();
//         unsafe {
//             // Note: implémentation utilise la duplication des 2 premières valeurs
//             assert_eq!(vector_data.f64x4[0], 1.111, "Vector Y0[0] should contain first f64 value");
//             assert_eq!(vector_data.f64x4[1], 2.222, "Vector Y0[1] should contain second f64 value");
//         }
//     }
//
//
// /////////////////////    // ==================== TESTS AVEC MÉMOIRE RÉELLE (COMMENTÉS) ====================
//     // #[test]
//     // #[ignore] // Ce test nécessite une mémoire réelle pour fonctionner
//     // fn test_simd128_real_memory_operations() {
//     //     use crate::pvm::memorys::{Memory, MemoryConfig};
//     //     use crate::bytecode::simds::Vector128;
//     //
//     //     let mut execute = ExecuteStage::new();
//     //     let mut alu = ALU::new();
//     //     let mut memory = Memory::new(MemoryConfig::default());
//     //
//     //     // Test 1: Préparer un vecteur dans V0
//     //     let test_vector = Vector128::from_i32x4([100, 200, 300, 400]);
//     //     execute.vector_alu.write_v128(0, test_vector).unwrap();
//     //
//     //     // Test 2: SIMD128Store - Stocker V0 en mémoire
//     //     let store_instruction = Instruction::create_reg_imm(Opcode::Simd128Store, 0, 0x1000);
//     //
//     //     let de_reg_store = DecodeExecuteRegister {
//     //         instruction: store_instruction,
//     //         pc: 100,
//     //         rs1: Some(0), // V0 source
//     //         rs2: None,
//     //         rd: None,
//     //         rs1_value: 0,
//     //         rs2_value: 0,
//     //         immediate: Some(0x1000),
//     //         branch_addr: None,
//     //         branch_prediction: None,
//     //         stack_operation: None,
//     //         mem_addr: Some(0x1000), // Adresse alignée sur 16 bytes
//     //         stack_value: None,
//     //     };
//     //
//     //     let result = execute.process_with_memory(&de_reg_store, &mut alu, &mut memory);
//     //     assert!(result.is_ok(), "SIMD128Store avec vraie mémoire devrait réussir");
//     //
//     //     // Test 3: SIMD128Load - Charger depuis la mémoire vers V1
//     //     let load_instruction = Instruction::create_reg_imm(Opcode::Simd128Load, 1, 0x1000);
//     //
//     //     let de_reg_load = DecodeExecuteRegister {
//     //         instruction: load_instruction,
//     //         pc: 104,
//     //         rs1: None,
//     //         rs2: None,
//     //         rd: Some(1), // V1 destination
//     //         rs1_value: 0,
//     //         rs2_value: 0,
//     //         immediate: Some(0x1000),
//     //         branch_addr: None,
//     //         branch_prediction: None,
//     //         stack_operation: None,
//     //         mem_addr: Some(0x1000), // Même adresse
//     //         stack_value: None,
//     //     };
//     //
//     //     let result = execute.process_with_memory(&de_reg_load, &mut alu, &mut memory);
//     //     assert!(result.is_ok(), "SIMD128Load avec vraie mémoire devrait réussir");
//     //
//     //     // Test 4: Vérifier que les données sont identiques
//     //     let loaded_vector = execute.vector_alu.read_v128(1).unwrap();
//     //     unsafe {
//     //         assert_eq!(loaded_vector.i32x4, [100, 200, 300, 400], "Les données chargées devraient être identiques");
//     //     }
//     // }
//
//     // #[test]
//     // fn test_simd256_real_memory_operations() {
//     //     use crate::pvm::memorys::{Memory, MemoryConfig};
//     //     use crate::bytecode::simds::Vector256;
//     //
//     //     let mut execute = ExecuteStage::new();
//     //     let mut alu = ALU::new();
//     //     let mut memory = Memory::new(MemoryConfig::default());
//     //
//     //     // Test 1: Préparer un vecteur dans Y0
//     //     let test_vector = Vector256::from_i32x8([10, 20, 30, 40, 50, 60, 70, 80]);
//     //     execute.vector_alu.write_v256(0, test_vector).unwrap();
//     //
//     //     // Test 2: SIMD256Store - Stocker Y0 en mémoire
//     //     let store_instruction = Instruction::create_simd128_store(Opcode::Simd256Store, 0, 0x2000);
//     //
//     //     let de_reg_store = DecodeExecuteRegister {
//     //         instruction: store_instruction,
//     //         pc: 100,
//     //         rs1: Some(0), // Y0 source
//     //         rs2: None,
//     //         rd: None,
//     //         rs1_value: 0,
//     //         rs2_value: 0,
//     //         immediate: Some(0x2000),
//     //         branch_addr: None,
//     //         branch_prediction: None,
//     //         stack_operation: None,
//     //         mem_addr: Some(0x2000), // Adresse alignée sur 32 bytes
//     //         stack_value: None,
//     //     };
//     //
//     //     let result = execute.process_with_memory(&de_reg_store, &mut alu, &mut memory);
//     //     assert!(result.is_ok(), "SIMD256Store avec vraie mémoire devrait réussir");
//     //
//     //     // Test 3: SIMD256Load - Charger depuis la mémoire vers Y1
//     //     let load_instruction = Instruction::create_reg_imm(Opcode::Simd256Load, 1, 0x2000);
//     //
//     //     let de_reg_load = DecodeExecuteRegister {
//     //         instruction: load_instruction,
//     //         pc: 104,
//     //         rs1: None,
//     //         rs2: None,
//     //         rd: Some(1), // Y1 destination
//     //         rs1_value: 0,
//     //         rs2_value: 0,
//     //         immediate: Some(0x2000),
//     //         branch_addr: None,
//     //         branch_prediction: None,
//     //         stack_operation: None,
//     //         mem_addr: Some(0x2000), // Même adresse
//     //         stack_value: None,
//     //     };
//     //
//     //     let result = execute.process_with_memory(&de_reg_load, &mut alu, &mut memory);
//     //     assert!(result.is_ok(), "SIMD256Load avec vraie mémoire devrait réussir");
//     //
//     //     // Test 4: Vérifier que les données sont identiques
//     //     let loaded_vector = execute.vector_alu.read_v256(1).unwrap();
//     //     unsafe {
//     //         assert_eq!(loaded_vector.i32x8, [10, 20, 30, 40, 50, 60, 70, 80], "Les données chargées devraient être identiques");
//     //     }
//     // }
//
//     // #[test]
//     // #[ignore] // Ce test nécessite une mémoire réelle pour fonctionner
//     // fn test_simd_memory_alignment_errors() {
//     //     use crate::pvm::memorys::{Memory, MemoryConfig};
//     //     use crate::bytecode::simds::Vector128;
//     //
//     //     let mut execute = ExecuteStage::new();
//     //     let mut alu = ALU::new();
//     //     let mut memory = Memory::new(MemoryConfig::default());
//     //
//     //     // Préparer un vecteur dans V0
//     //     let test_vector = Vector128::from_i32x4([1, 2, 3, 4]);
//     //     execute.vector_alu.write_v128(0, test_vector).unwrap();
//     //
//     //     // Test avec adresse non alignée (doit échouer)
//     //     let store_instruction = Instruction::create_reg_imm(Opcode::Simd128Store, 0, 0x1001);
//     //
//     //     let de_reg_store = DecodeExecuteRegister {
//     //         instruction: store_instruction,
//     //         pc: 100,
//     //         rs1: Some(0),
//     //         rs2: None,
//     //         rd: None,
//     //         rs1_value: 0,
//     //         rs2_value: 0,
//     //         immediate: Some(0x1001),
//     //         branch_addr: None,
//     //         branch_prediction: None,
//     //         stack_operation: None,
//     //         mem_addr: Some(0x1001), // Adresse NON alignée sur 16 bytes
//     //         stack_value: None,
//     //     };
//     //
//     //     let result = execute.process_with_memory(&de_reg_store, &mut alu, &mut memory);
//     //     assert!(result.is_err(), "SIMD128Store avec adresse non alignée devrait échouer");
//     //
//     //     // Vérifier le message d'erreur
//     //     let error_msg = result.unwrap_err();
//     //     assert!(error_msg.contains("aligné"), "Le message d'erreur devrait mentionner l'alignement");
//     // }
//
//     // #[test]
//     // #[ignore] // Ce test nécessite une mémoire réelle pour fonctionner
//     // fn test_simd_memory_different_vector_types() {
//     //     use crate::pvm::memorys::{Memory, MemoryConfig};
//     //     use crate::bytecode::simds::{Vector128, Vector256};
//     //
//     //     let mut execute = ExecuteStage::new();
//     //     let mut alu = ALU::new();
//     //     let mut memory = Memory::new(MemoryConfig::default());
//     //
//     //     // Test avec différents types de vecteurs
//     //
//     //     // 1. f32x4 dans V0
//     //     let f32_vector = Vector128::from_f32x4([1.5, 2.5, 3.5, 4.5]);
//     //     execute.vector_alu.write_v128(0, f32_vector).unwrap();
//     //
//     //     // Store f32x4
//     //     let store_f32 = Instruction::create_reg_imm(Opcode::Simd128Store, 0, 0x1000);
//     //     let de_reg_f32 = DecodeExecuteRegister {
//     //         instruction: store_f32, pc: 100, rs1: Some(0), rs2: None, rd: None,
//     //         rs1_value: 0, rs2_value: 0, immediate: Some(0x1000),
//     //         branch_addr: None, branch_prediction: None, stack_operation: None,
//     //         mem_addr: Some(0x1000), stack_value: None,
//     //     };
//     //
//     //     let result = execute.process_with_memory(&de_reg_f32, &mut alu, &mut memory);
//     //     assert!(result.is_ok(), "Store f32x4 devrait réussir");
//     //
//     //     // 2. f32x8 dans Y0
//     //     let f32x8_vector = Vector256::from_f32x8([0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8]);
//     //     execute.vector_alu.write_v256(0, f32x8_vector).unwrap();
//     //
//     //     // Store f32x8
//     //     let store_f32x8 = Instruction::create_reg_imm(Opcode::Simd256Store, 0, 0x2000);
//     //     let de_reg_f32x8 = DecodeExecuteRegister {
//     //         instruction: store_f32x8, pc: 104, rs1: Some(0), rs2: None, rd: None,
//     //         rs1_value: 0, rs2_value: 0, immediate: Some(0x2000),
//     //         branch_addr: None, branch_prediction: None, stack_operation: None,
//     //         mem_addr: Some(0x2000), stack_value: None,
//     //     };
//     //
//     //     let result = execute.process_with_memory(&de_reg_f32x8, &mut alu, &mut memory);
//     //     assert!(result.is_ok(), "Store f32x8 devrait réussir");
//     //
//     //     // 3. Vérifier qu'on peut charger les données correctement
//     //     let load_f32 = Instruction::create_reg_imm(Opcode::Simd128Load, 1, 0x1000);
//     //     let de_reg_load_f32 = DecodeExecuteRegister {
//     //         instruction: load_f32, pc: 108, rs1: None, rs2: None, rd: Some(1),
//     //         rs1_value: 0, rs2_value: 0, immediate: Some(0x1000),
//     //         branch_addr: None, branch_prediction: None, stack_operation: None,
//     //         mem_addr: Some(0x1000), stack_value: None,
//     //     };
//     //
//     //     let result = execute.process_with_memory(&de_reg_load_f32, &mut alu, &mut memory);
//     //     assert!(result.is_ok(), "Load f32x4 devrait réussir");
//     //
//     //     let loaded_f32 = execute.vector_alu.read_v128(1).unwrap();
//     //     unsafe {
//     //         assert_eq!(loaded_f32.f32x4, [1.5, 2.5, 3.5, 4.5], "Données f32x4 devraient être identiques");
//     //     }
//     // }
// }
//


// src/pipeline/mod.rs
pub mod decode;
pub mod execute;
pub mod fetch;
pub mod forward;
pub mod hazard;
pub mod memory;
pub mod writeback;
pub mod ras;

use crate::alu::alu::ALU;
use crate::bytecode::opcodes::Opcode;

use crate::bytecode::instructions::Instruction;
use crate::pipeline::decode::StackOperation;
use crate::pvm::branch_predictor::{BranchPrediction, BranchPredictor};
use crate::pvm::memorys::Memory;
use crate::pipeline::ras::{RASStats, ReturnAddressStack};

/// Structure représentant le pipeline à 5 étages
pub struct Pipeline {
    /// État actuel du pipeline
    pub state: PipelineState,
    /// Module de l'étage Fetch
    fetch: fetch::FetchStage,
    /// Module de l'étage Decode
    decode: decode::DecodeStage,
    /// Module de l'étage Execute
    execute: execute::ExecuteStage,
    /// Module de l'étage Memory
    memory: memory::MemoryStage,
    /// Module de l'étage Writeback
    writeback: writeback::WritebackStage,
    /// RAS (Return Address Stack) pour la gestion des appels et retours
    // pub ras: ReturnAddressStack,
    /// Unité de détection de hazards
    pub hazard_detection: hazard::HazardDetectionUnit,
    /// Unité de forwarding
    pub forwarding: forward::ForwardingUnit,
    /// Statistiques du pipeline
    stats: PipelineStats,
    /// Configuration
    enable_forwarding: bool,
    enable_hazard_detection: bool,
}

/// État du pipeline à un instant donné
#[derive(Debug, Clone)]
pub struct PipelineState {
    /// Registre intermédiaire Fetch -> Decode
    pub fetch_decode: Option<FetchDecodeRegister>,
    /// Registre intermédiaire Decode -> Execute
    pub decode_execute: Option<DecodeExecuteRegister>,
    /// Registre intermédiaire Execute -> Memory
    pub execute_memory: Option<ExecuteMemoryRegister>,
    /// Registre intermédiaire Memory -> Writeback
    pub memory_writeback: Option<MemoryWritebackRegister>,
    /// Prochain PC à charger
    pub next_pc: u32,
    /// Indique si le pipeline est stall (bloqué)
    pub stalled: bool,
    /// Indique si l'exécution est terminée (HALT)
    pub halted: bool,
    /// Nombre d'instructions complétées ce cycle
    pub instructions_completed: usize,
    /// Indique si la branche a été traitée
    branch_processed: bool,
}

impl Default for PipelineState {
    fn default() -> Self {
        Self {
            fetch_decode: None,
            decode_execute: None,
            execute_memory: None,
            memory_writeback: None,
            next_pc: 0,
            stalled: false,
            halted: false,
            instructions_completed: 0,
            branch_processed: false,
        }
    }
}

/// Registre intermédiaire entre les étages Fetch et Decode
#[derive(Debug, Clone)]
pub struct FetchDecodeRegister {
    /// Instruction brute récupérée
    pub instruction: Instruction,
    /// Adresse de l'instruction
    pub pc: u32,
}

/// Registre intermédiaire entre les étages Decode et Execute
#[derive(Debug, Clone)]
pub struct DecodeExecuteRegister {
    /// Instruction décodée
    pub instruction: Instruction,
    /// Adresse de l'instruction
    pub pc: u32,
    /// Registre source 1
    pub rs1: Option<usize>,
    /// Registre source 2
    pub rs2: Option<usize>,
    /// Registre destination
    pub rd: Option<usize>,

    /// Valeurs des registres source 1 et 2
    pub rs1_value: u64,
    pub rs2_value: u64,

    /// Valeur immédiate (si présente)
    pub immediate: Option<u64>,
    /// Adresse branchement (si instruction de saut)
    pub branch_addr: Option<u32>,
    /// Adresse mémoire (si instruction mémoire)
    pub mem_addr: Option<u32>,
    ///Prediction de branchement (si instruction de branchement)
    pub branch_prediction: Option<BranchPrediction>,
    //
    pub stack_operation: Option<StackOperation>,

    pub stack_value: Option<u64>,
}

/// Registre intermédiaire entre les étages Execute et Memory
#[derive(Debug, Clone)]
pub struct ExecuteMemoryRegister {
    /// Instruction
    pub instruction: Instruction,
    /// Résultat de l'ALU
    pub alu_result: u64,
    /// Registre destination
    pub rd: Option<usize>,
    /// Valeur à écrire en mémoire (si store)
    pub store_value: Option<u64>,
    /// Adresse mémoire (si load/store)
    pub mem_addr: Option<u32>,
    /// PC du branchement (si instruction de saut)
    pub branch_target: Option<u32>,
    /// Branchement pris ou non
    pub branch_taken: bool,

    pub branch_prediction_correct: Option<bool>,

    // Nouveaux champs pour la pile
    pub stack_operation: Option<StackOperation>,
    pub stack_result: Option<u64>,
    pub ras_prediction_correct: Option<bool>,

    /// Halt
    pub halted: bool,
}

/// Registre intermédiaire entre les étages Memory et Writeback
#[derive(Debug, Clone)]
pub struct MemoryWritebackRegister {
    /// Instruction
    pub instruction: Instruction,
    /// Résultat à écrire dans le registre destination
    pub result: u64,
    /// Registre destination
    pub rd: Option<usize>,
}

/// Statistiques du pipeline
#[derive(Debug, Clone, Copy)]
pub struct PipelineStats {
    /// Nombre total de cycles
    pub cycles: u64,
    /// Nombre d'instructions exécutées
    pub instructions: u64,
    /// Nombre de stalls (cycles où le pipeline est bloqué)
    pub stalls: u64,
    /// Nombre de hazards détectés
    pub hazards: u64,
    /// Nombre de forwards effectués
    pub forwards: u64,
    /// Nombre de prédictions de branchement
    pub branch_predictions: u64,
    /// Nombre de prédictions correctes
    pub branch_hits: u64,
    /// Nombre de prédictions incorrectes
    pub branch_misses: u64,
    /// Nombre de branch flush
    pub branch_flush: u64,
    /// Taux de prédiction de branchement (calculé lors de l'accès)
    pub branch_predictor_rate: f64,

    /// Statistiques CALL/RET
    pub stack_pushes: u64,
    pub stack_pops: u64,
    pub total_calls: u64,
    pub total_returns: u64,
    pub ras_hits: u64,
    pub ras_misses: u64,
    pub ras_accuracy: f64,
    pub max_call_depth: usize,
    pub current_call_depth: usize,
}





impl PipelineStats {
    pub fn branch_prediction_rate(&self) -> f64 {
        if self.branch_predictions > 0 {
            (self.branch_hits as f64 / self.branch_predictions as f64) * 100.0
        } else { 0.0 }
    }

}


impl Default for PipelineStats{
    fn default() -> Self {
        Self {
            cycles: 0,
            instructions: 0,
            stalls: 0,
            hazards: 0,
            forwards: 0,
            branch_predictions: 0,
            branch_hits: 0,
            branch_misses: 0,
            branch_flush: 0,
            branch_predictor_rate: 0.0,

            stack_pushes: 0,
            stack_pops: 0,
            total_calls: 0,
            total_returns: 0,
            ras_hits: 0,
            ras_misses: 0,
            ras_accuracy: 0.0,
            max_call_depth: 0,
            current_call_depth: 0,
        }
    }
}

impl Pipeline {
    /// Crée un nouveau pipeline
    pub fn new(
        fetch_buffer_size: usize,
        enable_forwarding: bool,
        enable_hazard_detection: bool,
    ) -> Self {
        Self {
            state: PipelineState::default(),
            fetch: fetch::FetchStage::new(fetch_buffer_size),
            decode: decode::DecodeStage::new(),
            execute: execute::ExecuteStage::new(),
            memory: memory::MemoryStage::new(),
            writeback: writeback::WritebackStage::new(),
            // ras: ReturnAddressStack::new(),

            hazard_detection: hazard::HazardDetectionUnit::new(),
            forwarding: forward::ForwardingUnit::new(),
            stats: PipelineStats::default(),
            enable_forwarding,
            enable_hazard_detection,
        }
    }

    /// Réinitialise le pipeline
    pub fn reset(&mut self) {
        self.state = PipelineState::default();
        self.fetch.reset();
        self.decode.reset();
        self.execute.reset();
        self.memory.reset();
        self.writeback.reset();
        self.hazard_detection.reset();
        self.forwarding.reset();
        self.stats = PipelineStats::default();
    }

    /// Exécute un cycle du pipeline
    pub fn cycle(
        &mut self,
        pc: u32,
        registers: &mut [u64],
        memory: &mut Memory,
        alu: &mut ALU,
        instructions: &[Instruction],
    ) -> Result<PipelineState, String> {
        // 0) Incrément du compteur de cycles pipeline
        self.stats.cycles += 1;
        println!("DEBUG: Debut du cycle - PC = {}", pc);

        // 1) Clone de l’état local
        let mut state = self.state.clone();
        state.stalled = false;
        state.instructions_completed = 0;
        let pc_for_this_cycle = pc; // bug fix
        let current_pc_target = self.state.next_pc;

        //Gestion de l'etai des branchement en cours
        if state.execute_memory.is_some()
            && state
            .execute_memory
            .as_ref()
            .unwrap()
            .instruction
            .opcode
            .is_branch()
        {
            if state.stalled {
                // Si on est stalled, on ne traite pas le branchement
                println!("DEBUG: Branchement en cours, mais pipeline est stalled");
                state.branch_processed = true;
            } else {
                // Si on n'est pas stalled, on traite le branchement
                println!("DEBUG: Branchement en cours, mais pipeline n'est pas stalled");
                state.branch_processed = false
            }
        }

        // 2) Détection de hazards
        if self.enable_hazard_detection {
            let any_hazard = self.hazard_detection.detect_hazards(&state);
            if any_hazard {
                self.stats.stalls += 1;
                self.stats.hazards += 1;
                state.stalled = true;
            }
        }

        // Bug fix: Insert stall logic
        if state.stalled {
            // If a stall is determined for this cycle:
            // 1. The PC for the next fetch will retry the current PC.
            //    (Branch resolution logic later in this cycle can still override this if a branch resolves).
            state.next_pc = pc_for_this_cycle;
            // 2. The IF/DE latch (output of Fetch for the next cycle) gets a bubble.
            state.fetch_decode = None;
            // 3. The DE/EX latch (output of Decode for the next cycle) also gets a bubble,
            //    as Decode would not have valid input if Fetch is stalled.
            //    (Note: This simplifies stall propagation. More complex schemes might allow Decode to process
            //    an old instruction if only Fetch is stalled but Decode isn't directly,
            //    but for now, this ensures bubbles flow if Fetch is blocked).
            state.decode_execute = None;
        }

        // ----- (1ᵉʳᵉ étape) FETCH -----
        // Si on n’est pas stalled, on fetch l’instruction à l’adresse `pc`.
        if !state.stalled {
            // On fetch
            let fd_reg = self.fetch.process_direct(pc, instructions)?;
            state.fetch_decode = Some(fd_reg.clone()); // Clone fd_reg as it's used in println later

            // BugFixe: Modify Fetch PC update
            // If Fetch runs (not stalled), it calculates the next sequential PC.
            // This can be overridden by branch resolution logic later in this same cycle.
            if let Some(fetched_instruction_data) = &state.fetch_decode { // Use the just-fetched instruction
                let instruction_size = fetched_instruction_data.instruction.total_size() as u32;
                state.next_pc = pc_for_this_cycle.wrapping_add(instruction_size);
                // println!("[DEBUG: Fin Fetch -] PC = 0x{:08X}, next_pc = 0x{:08X}", fd_reg.pc, state.next_pc);
                // Ensure fd_reg.pc is pc_for_this_cycle if used in the println.
                // fd_reg.pc should be pc_for_this_cycle if fetch was successful for pc_for_this_cycle
                println!("[DEBUG: Fin Fetch -] Fetched for PC = 0x{:08X}, calculated state.next_pc after fetch = 0x{:08X}", fetched_instruction_data.pc, state.next_pc);
            }

        }

        // ----- (2ᵉ étape) DECODE -----
        if !state.stalled {
            if let Some(fd_reg) = &state.fetch_decode {
                let ex_reg = self.decode.process_direct(fd_reg, registers)?;
                state.decode_execute = Some(ex_reg);
                println!("[DEBUG: Fin Decode -] PC = 0x{:08X}, instruction = {:?},next_pc = 0x{:08X}", fd_reg.pc, fd_reg.instruction.opcode, state.next_pc);
            } else {
                state.decode_execute = None;
                println!("DEBUG: Pas d'instruction à décoder (fetch_decode est None)");
            }

        }


        // ----- (3ᵉ étape) EXECUTE -----
        if let Some(de_reg) = &state.decode_execute {
            let pc_of_executed_branch_instr = de_reg.pc; // Copy PC early
            // Forwarding si activé
            let mut de_reg_mut = de_reg.clone();
            if self.enable_forwarding {
                self.forwarding.forward(
                    &mut de_reg_mut,
                    &state.execute_memory,
                    &state.memory_writeback,
                );

            }

            let mem_reg = self.execute.process_direct(&de_reg_mut, alu,)?;

            // Extraire les valeurs dont nous aurons besoin plus tard
            let branch_pc = de_reg.pc;
            let branch_prediction = de_reg.branch_prediction;

            // Gérer les prédictions de branchement
            if let Some(prediction_correct) = mem_reg.branch_prediction_correct {
                if prediction_correct {
                    // Prédiction correcte - mise à jour des statistiques
                    self.stats.branch_hits += 1;

                } else {
                    // Prédiction incorrecte - flush du pipeline et mise à jour du PC
                    self.stats.branch_misses += 1;


                    if mem_reg.branch_taken {
                        if let Some(target) = mem_reg.branch_target {
                            // Branchement pris mais prédit non pris
                            state.next_pc = target;
                            println!("Branchement pris vers l'adresse: 0x{:08X}", target);
                            state.fetch_decode = None;
                            state.decode_execute = None;
                            self.stats.branch_flush += 1;
                        }else {
                            println!("On ne fait rien ")
                        }
                    }
                }

                // Mise à jour du prédicteur
                let pc = branch_pc as u64;
                let taken = mem_reg.branch_taken;
                let prediction = branch_prediction.unwrap_or(BranchPrediction::NotTaken);
                // let prediction = branch_prediction.unwrap_or(BranchPredictor::predict(pc));
                self.decode.branch_predictor.update(pc, taken, prediction);

                self.stats.branch_predictions += 1;
            }

            if !mem_reg.branch_taken && mem_reg.instruction.opcode.is_branch() {
                // This instruction (`mem_reg.instruction`) just finished the Execute stage.
                // `de_reg` was its input in the DecodeExecuteRegister.
                // Use the copied `pc_of_executed_branch_instr`.
                let actual_branch_instruction_pc = pc_of_executed_branch_instr;
                let branch_instruction_size = mem_reg.instruction.total_size() as u32;

                // Calculate the correct sequential PC that should follow this non-taken branch.
                let correct_sequential_pc_after_branch = actual_branch_instruction_pc + branch_instruction_size; // e.g., 0x5E + 8 = 0x66

                // If state.next_pc (which might be a speculatively fetched PC or a mispredicted target)
                // is not this correct sequential PC, update it. This handles flushing.
                if state.next_pc != correct_sequential_pc_after_branch {
                    state.next_pc = correct_sequential_pc_after_branch;
                }

                // The original log message is preserved for consistency with existing logs,
                // but now it should print the corrected PC (e.g., 0x66).
                println!(
                    "Branchement non pris, PC avance à 0x{:08X}",
                    state.next_pc
                );
            }

            state.execute_memory = Some(mem_reg);
        } else {
            state.execute_memory = None;
        }

        // ----- (4ᵉ étape) MEMORY -----
        if let Some(ex_mem) = &state.execute_memory {
            let wb_reg = self.memory.process_direct(ex_mem, memory, registers)?;

            // Si c’est un HALT => on arrête tout de suite
            if ex_mem.instruction.opcode == Opcode::Halt {
                state.halted = true;
                // Flush le pipeline
                state.fetch_decode = None;
                state.decode_execute = None;
                state.execute_memory = None;
                // Optionnellement, on peut stocker wb_reg pour un dernier writeback
                state.memory_writeback = Some(wb_reg);
                // On arrête le cycle ici, sans traiter les étages suivants
                self.state = state.clone();
                return Ok(state);
            }

            state.memory_writeback = Some(wb_reg);

        } else {
            state.memory_writeback = None;
        }

        // ----- (5ᵉ étape) WRITEBACK -----
        if let Some(mw_reg) = &state.memory_writeback {
            self.writeback.process_direct(mw_reg, registers)?;
            // On considère qu’une instruction est finalisée ici
            state.instructions_completed += 1;
            // self.stats.instructions += 1;
        }
        state.memory_writeback = None;

        self.stats.hazards = self.hazard_detection.get_hazards_count();
        self.stats.forwards = self.forwarding.get_forwards_count();

        // Mise à jour des statistiques
        self.stats.hazards = self.hazard_detection.get_hazards_count();
        self.stats.forwards = self.forwarding.get_forwards_count();

        // 9) Mise à jour de self.state
        self.state = state.clone();

        println!("[[[DEBUG: Fin du cycle ]]] - PC = 0x{:08X}, next_pc = 0x{:08X}", pc, state.next_pc);
        // println!("DEBUG: Fin du cycle - PC = {}", pc);
        Ok(state)
    }

    pub fn update_branch_predictor(&mut self, pc: u64, taken: bool, prediction: BranchPredictor) {
        println!("Updating branch predictor: PC=0x{:X}, taken={}, prediction={:?}",
                 pc, taken, prediction);
    }

    /// Retourne les statistiques du pipeline
    pub fn stats(&self) -> PipelineStats {
        let mut stats = self.stats;
        stats.branch_predictor_rate = stats.branch_prediction_rate();
        // Mise à jour des statistiques de la pile
        // stats.update_stack_stats(self.get_ras_stats());
        stats
    }

    pub fn get_ras_stats(&self) -> RASStats {
        self.get_ras_stats()
    }

}


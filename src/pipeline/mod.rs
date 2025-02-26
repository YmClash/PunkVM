mod fetch;
mod decode;
mod execute;
mod memory;
mod writeback;
mod hazard;
mod forward;

use std::io;

use crate::bytecode::opcodes::Opcode;
use crate::alu::ALU;
use crate::bytecode::instructions::Instruction;
use crate::memory::Memory;

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
    /// Unité de détection de hazards
    hazard_detection: hazard::HazardDetectionUnit,
    /// Unité de forwarding
    forwarding: forward::ForwardingUnit,
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
    /// Valeur immédiate (si présente)
    pub immediate: Option<u64>,
    /// Adresse branchement (si instruction de saut)
    pub branch_addr: Option<u32>,
    /// Adresse mémoire (si instruction mémoire)
    pub mem_addr: Option<u32>,
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
#[derive(Debug, Clone, Copy, Default)]
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
        // Incrémenter le compteur de cycles
        self.stats.cycles += 1;

        // 1. Vérifier les hazards et déterminer s'il faut stall le pipeline
        let mut state = self.state.clone();
        state.stalled = false;

        if self.enable_hazard_detection {
            state.stalled = self.hazard_detection.detect_hazards(&state);
            if state.stalled {
                self.stats.stalls += 1;
                self.stats.hazards += 1;
            }
        }

        // 2. Exécuter les étages dans l'ordre inverse pour éviter d'écraser les données

        // Writeback - Écrit les résultats dans les registres
        if let Some(wb_reg) = &state.memory_writeback {
            self.writeback.process(wb_reg, registers)?;
            state.instructions_completed += 1;
            self.stats.instructions += 1;
        }

        // Memory - Accède à la mémoire
        if let Some(mem_reg) = &state.execute_memory {
            let wb_reg = self.memory.process(mem_reg, memory)?;
            state.memory_writeback = Some(wb_reg);

            // Si c'est une instruction HALT, marquer le pipeline comme terminé
            if mem_reg.instruction.opcode == Opcode::Halt {
                state.halted = true;
            }
        } else {
            state.memory_writeback = None;
        }

        // Execute - Exécute l'opération ALU
        if let Some(ex_reg) = &state.decode_execute {
            // Appliquer le forwarding si activé
            let mut ex_reg = ex_reg.clone();
            if self.enable_forwarding {
                self.forwarding.forward(
                    &mut ex_reg,
                    &state.execute_memory,
                    &state.memory_writeback,
                    registers,
                );
            }

            let mem_reg = self.execute.process(&ex_reg, alu)?;

            // Si c'est un branchement pris, mettre à jour le PC
            if mem_reg.branch_taken {
                if let Some(target) = mem_reg.branch_target {
                    state.next_pc = target;

                    // Vider les étages précédents (flush du pipeline)
                    state.fetch_decode = None;
                    state.decode_execute = None;
                }
            }

            state.execute_memory = Some(mem_reg);
        } else {
            state.execute_memory = None;
        }

        // Decode - Décode l'instruction
        if !state.stalled {
            if let Some(fd_reg) = &state.fetch_decode {
                let ex_reg = self.decode.process(fd_reg, registers)?;
                state.decode_execute = Some(ex_reg);
            } else {
                state.decode_execute = None;
            }

            // Fetch - Récupère l'instruction suivante
            let fd_reg = self.fetch.process(pc, instructions)?;
            state.fetch_decode = Some(fd_reg);

            // Par défaut, incrémenter le PC
            if !state.stalled && !state.halted && state.next_pc == pc {
                // Le PC est incrémenté de la taille de l'instruction
                if let Some(fd_reg) = &state.fetch_decode {
                    state.next_pc = pc + fd_reg.instruction.total_size() as u32;
                }
            }
        }

        // 3. Mettre à jour l'état du pipeline
        self.state = state.clone();

        Ok(state)
    }

    /// Retourne les statistiques du pipeline
    pub fn stats(&self) -> PipelineStats {
        self.stats
    }
}

/// Implémentation des étages du pipeline
pub mod stage {
    /// Trait commun pour tous les étages du pipeline
    pub trait PipelineStage {
        /// Type d'entrée de l'étage
        type Input;
        /// Type de sortie de l'étage
        type Output;

        /// Traite une entrée et produit une sortie
        fn process(&mut self, input: &Self::Input) -> Result<Self::Output, String>;

        /// Réinitialise l'état de l'étage
        fn reset(&mut self);
    }
}

use crate::pvm::instructions::{Address, ArithmeticOp, ControlOp, DecodedInstruction, Instruction, MemoryOp, RegisterId};
use crate::pvm::vm_errors::{VMError, VMResult};

use std::collections::VecDeque;
use std::collections::HashMap;
use crate::pvm::caches::CacheStatistics;
use crate::pvm::memorys::MemoryController;
use crate::pvm::registers::RegisterBank;

/// Pipeline d'exécution
// pub struct Pipeline {
//     stages: Vec<Stage>,
//     stalled: bool,
// }

#[derive(Debug)]
enum Stage {
    Fetch,
    Decode,
    Execute,
    Memory,
    Writeback,
}

/// État d'une instruction dans le pipeline
#[derive(Debug, Clone)]
pub struct PipelineState {
    instruction: Option<Instruction>,
    decoded: Option<DecodedInstruction>,
    result: Option<ExecutionResult>,
    memory_result: Option<MemoryResult>,
    destination: Option<RegisterId>,
}


/// Résultat d'exécution
#[derive(Debug, Clone,Copy)]
pub struct ExecutionResult {
    value: i64,
    flags: StatusFlags,
}

/// Résultat d'opération mémoire
#[derive(Debug, Clone,)]
pub struct MemoryResult {
    data: i64,
    address: Address,
}

#[derive(Debug, Clone,Copy)]
pub struct StatusFlags {
    pub zero: bool,
    pub negative: bool,
    pub overflow: bool,
}

impl Default for StatusFlags {
    fn default() -> Self {
        Self {
            zero: false,
            negative: false,
            overflow: false,
        }
    }
}

pub struct HazardUnit {
    last_write_registers: Vec<RegisterId>,
}

impl HazardUnit {
    pub fn new() -> Self {
        Self {
            last_write_registers: Vec::new(),
        }
    }

    pub fn check_hazards(&self, instruction: &Instruction, registers: &RegisterBank) -> bool {
        false // Implémentation basique pour commencer
    }
}

pub struct ForwardingUnit {
    forward_table: HashMap<RegisterId, ExecutionResult>,
}

impl ForwardingUnit {
    pub fn new() -> Self {
        Self {
            forward_table: HashMap::new(),
        }
    }
}


#[derive(Default, Debug)]
pub struct PipelineStats {
    pub cycles: usize,
    pub instructions_loaded: usize,
    pub instructions_fetched: usize,
    pub instructions_decoded: usize,
    pub instructions_executed: usize,
    pub memory_operations: usize,
    pub writebacks: usize,
    pub stalls: usize,
    pub hazards: usize,
}


#[derive(Debug)]
pub struct DetailedStats {
    pub pipeline_stats: PipelineStats,
    pub cache_stats: CacheStatistics,
}


/// Pipeline complet
pub struct Pipeline {
    // États des différents étages
    fetch_state: PipelineState,
    decode_state: PipelineState,
    execute_state: PipelineState,
    memory_state: PipelineState,
    writeback_state: PipelineState,

    // Buffer d'instructions
    instruction_buffer: VecDeque<Instruction>,

    // Détection de hazards
    hazard_unit: HazardUnit,

    // Forwarding
    forwarding_unit: ForwardingUnit,

    // Statistiques
    stats: PipelineStats,
}








impl Default for PipelineState {
    fn default() -> Self {
        Self {
            instruction: None,
            decoded: None,
            result: None,
            memory_result: None,
            destination: None,
        }
    }
}




impl Pipeline {

    pub fn new() -> Self {
        Self {
            fetch_state: PipelineState::default(),
            decode_state: PipelineState::default(),
            execute_state: PipelineState::default(),
            memory_state: PipelineState::default(),
            writeback_state: PipelineState::default(),
            instruction_buffer: VecDeque::new(),
            hazard_unit: HazardUnit::new(),
            forwarding_unit: ForwardingUnit::new(),
            stats: PipelineStats::default(),
        }
    }
    pub fn load_instructions(&mut self, program: Vec<Instruction>) -> VMResult<()> {
        self.instruction_buffer = VecDeque::from(program);
        self.stats.instructions_loaded = self.instruction_buffer.len();
        Ok(())
    }

    pub fn is_empty(&self) -> VMResult<bool> {
        Ok(self.instruction_buffer.is_empty()
            && self.fetch_state.instruction.is_none()
            && self.decode_state.instruction.is_none()
            && self.execute_state.decoded.is_none()
            && self.memory_state.result.is_none()
            && self.writeback_state.result.is_none())
    }

    pub fn should_halt(&self) -> VMResult<bool> {
        // Vérifier si l'instruction en cours est HALT
        if let Some(Instruction::Halt) = &self.decode_state.instruction {
            return Ok(true);
        }
        Ok(false)
    }



    /// Exécute un cycle complet du pipeline
    // pub fn cycle(
    //     &mut self,
    //     registers: &mut RegisterBank,
    //     memory: &mut MemoryController,
    // ) -> VMResult<()> {
    //     // Mise à jour des statistiques
    //     self.stats.cycles += 1;
    //
    //     // Exécution des étages dans l'ordre inverse pour éviter les conflits
    //     self.writeback_stage(registers)?;
    //     self.memory_stage(memory)?;
    //     self.execute_stage()?;
    //     self.decode_stage(registers)?;
    //     self.fetch_stage()?;
    //
    //     Ok(())
    // }

    pub fn cycle(
        &mut self,
        registers: &mut RegisterBank,
        memory: &mut MemoryController,
    ) -> VMResult<()> {
        // Mise à jour des statistiques
        self.stats.cycles += 1;

        // Exécution des étages dans l'ordre inverse pour éviter les conflits
        if let Err(e) = self.writeback_stage(registers) {
            println!("Erreur dans l'étage Writeback: {:?}", e);
            return Err(e);
        }

        if let Err(e) = self.memory_stage(memory) {
            println!("Erreur dans l'étage Memory: {:?}", e);
            return Err(e);
        }

        if let Err(e) = self.execute_stage() {
            println!("Erreur dans l'étage Execute: {:?}", e);
            return Err(e);
        }

        if let Err(e) = self.decode_stage(registers) {
            println!("Erreur dans l'étage Decode: {:?}", e);
            return Err(e);
        }

        if let Err(e) = self.fetch_stage() {
            println!("Erreur dans l'étage Fetch: {:?}", e);
            return Err(e);
        }

        // Afficher l'état du pipeline si nécessaire
        if self.stats.cycles % 10 == 0 {
            println!("Cycle {}: {} instructions exécutées, {} stalls",
                     self.stats.cycles,
                     self.stats.instructions_executed,
                     self.stats.stalls);
        }

        Ok(())
    }

    /// Étage Fetch: Chargement de l'instruction
    fn fetch_stage(&mut self) -> VMResult<()> {
        // Si l'étage suivant est occupé, stall
        if self.decode_state.instruction.is_some() {
            self.stats.stalls += 1;
            return Ok(());
        }

        // Récupérer la prochaine instruction du buffer
        if let Some(instruction) = self.instruction_buffer.pop_front() {
            self.fetch_state.instruction = Some(instruction);
            self.stats.instructions_fetched += 1;
        }

        // Avancer l'état vers l'étage decode
        self.decode_state = self.fetch_state.clone();
        self.fetch_state = PipelineState::default();

        Ok(())
    }

    /// Étage Decode: Décodage de l'instruction
    fn decode_stage(&mut self, registers: &RegisterBank) -> VMResult<()> {
        // Si l'étage suivant est occupé, stall
        if self.execute_state.decoded.is_some() {
            self.stats.stalls += 1;
            return Ok(());
        }

        if let Some(instruction) = &self.decode_state.instruction {
            // Vérifier les hazards de données
            if self.hazard_unit.check_hazards(instruction, registers) {
                self.stats.hazards += 1;
                return Ok(());
            }

            // Décoder l'instruction
            let decoded = self.decode_instruction(instruction)?;
            self.decode_state.decoded = Some(decoded);
            self.stats.instructions_decoded += 1;
        }

        // Avancer l'état vers l'étage execute
        self.execute_state = self.decode_state.clone();
        self.decode_state = PipelineState::default();

        Ok(())
    }

    /// Étage Execute: Exécution de l'instruction
    fn execute_stage(&mut self) -> VMResult<()> {
        // Si l'étage suivant est occupé, stall
        if self.memory_state.result.is_some() {
            self.stats.stalls += 1;
            return Ok(());
        }

        if let Some(decoded) = &self.execute_state.decoded {
            // Exécuter l'instruction
            let result = self.execute_instruction(decoded)?;
            self.execute_state.result = Some(result);
            self.stats.instructions_executed += 1;
        }

        // Avancer l'état vers l'étage memory
        self.memory_state = self.execute_state.clone();
        self.execute_state = PipelineState::default();

        Ok(())
    }

    /// Étage Memory: Accès mémoire
    fn memory_stage(&mut self, memory: &mut MemoryController) -> VMResult<()> {
        // Si l'étage suivant est occupé, stall
        if self.writeback_state.memory_result.is_some() {
            self.stats.stalls += 1;
            return Ok(());
        }

        if let Some(result) = &self.memory_state.result {
            // Effectuer l'opération mémoire si nécessaire
            if let Some(memory_op) = self.get_memory_operation(&self.memory_state) {
                let memory_result = self.execute_memory_operation(memory_op, memory)?;
                self.memory_state.memory_result = Some(memory_result);
                self.stats.memory_operations += 1;
            }
        }

        // Avancer l'état vers l'étage writeback
        self.writeback_state = self.memory_state.clone();
        self.memory_state = PipelineState::default();

        Ok(())
    }

    /// Étage Writeback: Écriture des résultats
    fn writeback_stage(&mut self, registers: &mut RegisterBank) -> VMResult<()> {
        if let Some(dest) = self.writeback_state.destination {
            if let Some(result) = &self.writeback_state.result {
                // Utiliser la nouvelle méthode write_register
                registers.write_register(dest, result.value)?;
                self.stats.writebacks += 1;
            }
        }

        // Mise à jour des flags si nécessaire
        if let Some(result) = &self.writeback_state.result {
            registers.update_flags(result.flags)?;
        }

        // Réinitialiser l'état
        self.writeback_state = PipelineState::default();

        Ok(())
    }

    /// Décode une instruction
    fn decode_instruction(&self, instruction: &Instruction) -> VMResult<DecodedInstruction> {
        match instruction {
            Instruction::Add(dest, src1, src2) => Ok(DecodedInstruction::Arithmetic(
                ArithmeticOp::Add { dest: *dest, src1: *src1, src2: *src2 }
            )),
            // Implémenter les autres cas...
            _ => Err(VMError::InstructionError("Instruction non supportée".into()))
        }
    }

    /// Exécute une instruction décodée
    fn execute_instruction(&self, decoded: &DecodedInstruction) -> VMResult<ExecutionResult> {
        match decoded {
            DecodedInstruction::Arithmetic(op) => self.execute_arithmetic(op),
            DecodedInstruction::Memory(op) => self.execute_memory(op),
            DecodedInstruction::Control(op) => self.execute_control(op),
        }
    }

    fn get_memory_operation(&self, state: &PipelineState) -> Option<MemoryOp> {
        None // Implémentation temporaire
    }

    fn execute_memory_operation(&self, op: MemoryOp, memory: &mut MemoryController) -> VMResult<MemoryResult> {
        // Implémentation temporaire
        Ok(MemoryResult {
            data: 0,
            address: Address(0),
        })
    }

    fn execute_arithmetic(&self, op: &ArithmeticOp) -> VMResult<ExecutionResult> {
        // Implémentation temporaire
        Ok(ExecutionResult {
            value: 0,
            flags: StatusFlags {
                zero: false,
                negative: false,
                overflow: false,
            },
        })
    }

    fn execute_memory(&self, op: &MemoryOp) -> VMResult<ExecutionResult> {
        // Implémentation temporaire
        Ok(ExecutionResult {
            value: 0,
            flags: StatusFlags {
                zero: false,
                negative: false,
                overflow: false,
            },
        })
    }

    fn execute_control(&self, op: &ControlOp) -> VMResult<ExecutionResult> {
        // Implémentation temporaire
        Ok(ExecutionResult {
            value: 0,
            flags: StatusFlags {
                zero: false,
                negative: false,
                overflow: false,
            },
        })
    }


}
//src/pvm/cache_configs.rs
use std::collections::VecDeque;
use crate::pvm::branch_predictor::{BranchPrediction, BranchPredictor, PredictorType};
use crate::pvm::cache_configs::{CacheConfig, CacheSystem, WritePolicy};
use crate::pvm::cache_stats::CacheStatistics;
use crate::pvm::caches::Cache;
use crate::pvm::instructions::{Address, ArithmeticOp, ControlOp, DecodedInstruction, Instruction, MemoryOp, RegisterId};
use crate::pvm::vm_errors::{VMError, VMResult};
use crate::pvm::buffers::{BypassBuffer, FetchBuffer, StoreOperation};
use crate::pvm::memorys::MemoryController;
use crate::pvm::registers::RegisterBank;
use crate::pvm::forwardings::{ForwardingSource, ForwardingUnit};
use crate::pvm::hazards::{HazardResult, HazardType, HazardUnit};
use crate::pvm::metrics::PipelineMetrics;


/// Pipeline d'exécution
// pub struct Pipeline {
//     stages: Vec<Stage>,
//     stalled: bool,
// }


/// Pipeline complet
// #[derive(Debug)]
pub struct Pipeline {
    // États des différents étages
    pub fetch_state: PipelineState,
    pub decode_state: PipelineState,
    pub execute_state: PipelineState,
    pub memory_state: PipelineState,
    pub writeback_state: PipelineState,

    // Buffer d'instructions
    pub instruction_buffer: VecDeque<Instruction>,
    // Détection de hazards
    pub hazard_unit: HazardUnit,
    // Forwarding
    pub forwarding_unit: ForwardingUnit,

    // Statistiques
    pub stats: PipelineStats,
    // Métriques
    pub metrics: PipelineMetrics,
    // Nouveaux champs pour le suivi des métriques
    pub current_hazard: Option<HazardType>,
    pub forwarding_attempted: bool,
    pub forwarding_successful: bool,
    pub memory_access_in_progress: bool,
    pub cache_hit: bool,

    pub fetch_buffer: FetchBuffer,


    // Cache et mémoire
    pub cache_system: CacheSystem,
    pub bypass_buffer: BypassBuffer,
    pub pending_stores: VecDeque<StoreOperation>,

    // Branch predictor
    pub branch_predictor: BranchPredictor,
    pub predicted_branches: VecDeque<(u64, BranchPrediction)>, // Pour le suivi

    // programme counter
    pub pc: u64,



}

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
    pub instruction: Option<Instruction>,
    pub decoded: Option<DecodedInstruction>,
    pub result: Option<ExecutionResult>,
    pub memory_result: Option<MemoryResult>,
    pub destination: Option<RegisterId>,

}
impl PipelineState {
    pub fn get_branch(&self) -> Option<&DecodedInstruction> {
        self.decoded.as_ref().and_then(|decoded| {
            match decoded {
                DecodedInstruction::Control(_) => Some(decoded),
                _ => None
            }
        })
    }
}


/// Résultat d'exécution
#[derive(Debug, Clone,Copy)]
pub struct ExecutionResult {
    pub value: i64,
    pub flags: StatusFlags,
    pub branch_taken: bool,
    pub target_address: Option<u64>,
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
    pub carry: bool,
}

impl Default for StatusFlags {
    fn default() -> Self {
        Self {
            zero: false,
            negative: false,
            overflow: false,
            carry: false,
        }
    }
}

// Ajout d'un impl Default pour ExecutionResult
impl Default for ExecutionResult {
    fn default() -> Self {
        Self {
            value: 0,
            flags: StatusFlags::default(),
            branch_taken: false,
            target_address: None,
        }
    }
}

#[derive(Default, Debug, Clone)]
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



#[derive(Clone, Debug)]
pub struct RegisterDependency {
    reg_id: RegisterId,
    stage: PipelineStage,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PipelineStage {
    Execute,
    Memory,
    Writeback,
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

// Les structs d'état à implémenter
#[derive(Default)]
pub struct FetchState {}

#[derive(Default)]
pub struct DecodeState {}

#[derive(Default)]
pub struct ExecuteState {}

#[derive(Default)]
pub struct MemoryState {}

#[derive(Default)]
pub struct WritebackState {}




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
            metrics: PipelineMetrics::default(),

            current_hazard: None,
            forwarding_attempted: false,
            forwarding_successful: false,
            memory_access_in_progress: false,
            cache_hit: false,
            fetch_buffer: FetchBuffer::new(4), // buffer de taille 4



            // Initialisation du système de cache
            cache_system: CacheSystem::new(),
            bypass_buffer: BypassBuffer::new(32),
            pending_stores: VecDeque::new(),

            // Ajout du branch predictor
            branch_predictor: BranchPredictor::new(PredictorType::Dynamic), // Par défaut on utilise le prédicteur dynamique
            predicted_branches: VecDeque::new(),
            pc: 0,

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
    pub fn cycle(&mut self, registers: &mut RegisterBank, memory: &mut MemoryController) -> VMResult<()> {
        // Phase de fetch avec prédiction
        if let Some(instr) = &self.fetch_state.instruction {
            if instr.is_branch() {
                let prediction = self.branch_predictor.predict(self.pc);
                self.predicted_branches.push_back((self.pc, prediction));

                if prediction == BranchPrediction::Taken {
                    self.pc = instr.get_target_address();
                }
            }
        }

        // Phase d'exécution avec vérification de la prédiction
        if let Some(branch) = self.execute_state.get_branch() {
            // Sauvegarder les valeurs nécessaires avant de modifier self
            let target_addr = branch.get_target_address();
            let taken = branch.is_taken(registers);

            // Récupérer la prédiction
            if let Some((branch_pc, prediction)) = self.predicted_branches.pop_front() {
                self.branch_predictor.update(branch_pc, taken, prediction);

                // Vérifier si la prédiction était incorrecte
                let mispredicted = (taken && prediction == BranchPrediction::NotTaken) ||
                    (!taken && prediction == BranchPrediction::Taken);

                if mispredicted {
                    self.flush_pipeline();
                    self.pc = if taken {
                        target_addr
                    } else {
                        branch_pc + 4
                    };
                }
            }
        }


        self.update_stage_metrics();
        self.stats.cycles += 1;
        self.metrics.total_cycles += 1;

        // Mise à jour des métriques avant l'exécution des étages
        if self.writeback_state.result.is_some() {
            self.metrics.total_instructions += 1;
        }


        self.update_stage_metrics();

        self.writeback_stage(registers)?;
        self.memory_stage(memory)?;
        self.execute_stage(registers)?;
        self.decode_stage(registers)?;
        self.fetch_stage()?;

        // Mise à jour des métriques après l'ex



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
    // fn decode_stage(&mut self, registers: &RegisterBank) -> VMResult<()> {
    //
    //     if self.execute_state.decoded.is_some() {
    //         self.stats.stalls += 1;
    //         return Ok(());
    //     }
    //
    //     if let Some(instruction) = &self.decode_state.instruction {
    //         println!("Décodage de l'instruction: {:?}", instruction);
    //
    //         // Vérifier les hazards avec l'instruction non décodée
    //         if self.hazard_unit.check_hazards(instruction, registers) {
    //             self.stats.hazards += 1;
    //             self.stats.stalls += 1;
    //             return Ok(());
    //         }
    //
    //         match self.decode_instruction(instruction) {
    //             Ok(decoded) => {
    //                 println!("Instruction décodée avec succès: {:?}", decoded);
    //                 self.decode_state.decoded = Some(decoded);
    //                 self.stats.instructions_decoded += 1;
    //             }
    //             Err(e) => {
    //                 println!("Erreur lors du décodage: {:?}", e);
    //                 return Err(e);
    //             }
    //         }
    //     }
    //
    //     self.execute_state = self.decode_state.clone();
    //     self.decode_state = PipelineState::default();
    //
    //     Ok(())
    // }


    fn decode_stage(&mut self, registers: &RegisterBank) -> VMResult<()> {
        // Vérifier si l'étage d'exécution est occupé
        if self.execute_state.decoded.is_some() {
            self.stats.stalls += 1;
            return Ok(());
        }

        // Traiter l'instruction si présente
        if let Some(instruction) = &self.decode_state.instruction {
            println!("Décodage de l'instruction: {:?}", instruction);

            // Vérifier les hazards avec le nouveau système de résultat
            match self.hazard_unit.check_hazards(instruction, registers) {
                HazardResult::None => {
                    // Pas de hazard, continuer le décodage normal
                    match self.decode_instruction(instruction) {
                        Ok(decoded) => {
                            println!("Instruction décodée avec succès: {:?}", decoded);
                            self.decode_state.decoded = Some(decoded);
                            self.stats.instructions_decoded += 1;
                        }
                        Err(e) => {
                            println!("Erreur lors du décodage: {:?}", e);
                            return Err(e);
                        }
                    }
                },
                HazardResult::StoreLoad => {
                    println!("Gestion du Store-Load hazard");
                    self.stats.hazards += 1;
                    self.metrics.hazard_metrics.total_hazards += 1;
                    self.metrics.hazard_metrics.store_load_hazards += 1;
                    self.stats.stalls += 1;
                    return Ok(());
                },
                HazardResult::LoadUse => {
                    println!("Gestion du Load-Use hazard");
                    self.stats.hazards += 1;
                    self.metrics.hazard_metrics.total_hazards += 1;
                    self.metrics.hazard_metrics.load_use_hazards += 1;
                    self.stats.stalls += 1;
                    return Ok(());
                },
                HazardResult::DataDependency => {
                    println!("Gestion de la dépendance de données");
                    self.stats.hazards += 1;
                    self.metrics.hazard_metrics.total_hazards += 1;
                    self.metrics.hazard_metrics.data_hazards += 1;
                    self.stats.stalls += 1;
                    return Ok(());
                }
            }
        }

        // Avancer l'état vers l'étage suivant
        self.execute_state = self.decode_state.clone();
        self.decode_state = PipelineState::default();

        Ok(())
    }

    /// Exécute une instruction avec support du forwarding
    fn execute_stage(&mut self, registers: &RegisterBank) -> VMResult<()> {

        //execute_stage pour mettre à jour les métriques de forwarding
        if let Some(decoded) = &self.execute_state.decoded {
            // Vérifier si le forwarding est nécessaire
            let forwarding_needed = self.needs_forwarding(decoded);
            if forwarding_needed {
                self.metrics.forwarding_metrics.total_forwards += 1;
                // Tenter le forwarding
                if self.try_forward_value(self.get_source_register(decoded)).is_some() {
                    self.metrics.forwarding_metrics.successful_forwards += 1;
                } else {
                    self.metrics.forwarding_metrics.failed_forwards += 1;
                }
            }
        }
        // Vérifier si l'étage suivant est occupé
        if self.memory_state.result.is_some() {
            self.stats.stalls += 1;
            println!("Execute - Stall dû à l'étage mémoire occupé");
            return Ok(());
        }

        // Vérifier les dépendances de données
        if let Some(decoded) = &self.execute_state.decoded {
            if self.check_data_hazards(decoded, registers) {
                self.stats.stalls += 1;
                println!("Execute - Stall dû aux dépendances de données");
                return Ok(());
            }
        }

        let decoded = self.execute_state.decoded.clone();

        if let Some(decoded) = &decoded {
            println!("Execute - Début exécution: {:?}", decoded);

            // Exécuter l'instruction avec forwarding
            let result = match decoded {
                DecodedInstruction::Memory(MemoryOp::Store { reg, .. }) => {
                    // Utiliser le forwarding pour Store
                    let value = self.try_forward_value(*reg)
                        .unwrap_or_else(|| registers.read_register(*reg).unwrap() as i64);

                    println!("Execute - Store: forwarding/registre valeur {} depuis {:?}", value, reg);

                    ExecutionResult {
                        value,
                        flags: StatusFlags::default(),
                        branch_taken: false,
                        target_address: None,
                    }
                },
                DecodedInstruction::Memory(MemoryOp::Load { reg, addr }) => {
                    println!("Execute - Load depuis l'adresse {:?}", addr);
                    ExecutionResult::default()
                },
                _ => self.execute_with_forwarding(decoded, registers)?,
            };

            // Mettre à jour les résultats
            self.execute_state.result = Some(result);

            // Déterminer le registre destination
            self.execute_state.destination = match decoded {
                DecodedInstruction::Arithmetic(op) => match op {
                    ArithmeticOp::Add { dest, .. } => Some(*dest),
                    ArithmeticOp::Sub { dest, .. } => Some(*dest),
                    ArithmeticOp::Mul { dest, .. } => Some(*dest),
                    ArithmeticOp::Div { dest, .. } => Some(*dest),
                },
                DecodedInstruction::Memory(op) => match op {
                    MemoryOp::LoadImm { reg, .. } => Some(*reg),
                    MemoryOp::Load { reg, .. } => Some(*reg),
                    MemoryOp::Store { .. } => None,
                    MemoryOp::Move { dest, .. } => Some(*dest),
                },
                _ => None,
            };

            self.stats.instructions_executed += 1;
        }

        // Propager l'état
        self.memory_state = self.execute_state.clone();
        self.execute_state = PipelineState::default();
        Ok(())
    }


    /// Modifications du memory_stage pour utiliser le cache
    fn memory_stage(&mut self, memory: &mut MemoryController) -> VMResult<()> {
        if let Some(decoded) = &self.memory_state.decoded {
            match decoded {
                DecodedInstruction::Memory(MemoryOp::Store { reg, addr }) => {
                    if let Some(result) = &self.memory_state.result {
                        println!("Memory - Store: écriture de {} à l'adresse {:?}", result.value, addr);

                        // Utiliser d'abord le bypass buffer
                        self.bypass_buffer.push_bypass(addr.0, result.value as u64);

                        // Écrire dans le cache
                        self.cache_system.write(addr.0, result.value as u64)?;

                        self.hazard_unit.clear_hazards();
                    }
                },
                DecodedInstruction::Memory(MemoryOp::Load { reg, addr }) => {
                    // Vérifier d'abord le bypass buffer
                    if let Some(value) = self.bypass_buffer.try_bypass(addr.0) {
                        self.cache_hit = true;
                        println!("Memory - Load: lecture de {} depuis le bypass buffer", value);
                        self.memory_state.result = Some(ExecutionResult {
                            value: value as i64,
                            flags: StatusFlags::default(),
                            branch_taken: false,
                            target_address: None,

                        });
                    } else {
                        // Essayer de lire depuis le cache
                        match self.cache_system.read(addr.0) {
                            Ok(value) => {
                                self.cache_hit = true;
                                println!("Memory - Load: lecture de {} depuis le cache", value);
                                self.memory_state.result = Some(ExecutionResult {
                                    value: value as i64,
                                    flags: StatusFlags::default(),
                                    branch_taken: false,
                                    target_address: None,
                                });
                            },
                            Err(_) => {
                                // Cache miss, lire depuis la mémoire
                                self.cache_hit = false;
                                let value = memory.read(addr.0)?;
                                println!("Memory - Load: lecture de {} depuis la mémoire", value);
                                self.memory_state.result = Some(ExecutionResult {
                                    value: value as i64,
                                    flags: StatusFlags::default(),
                                    branch_taken: false,
                                    target_address: None,
                                });
                            }
                        }
                    }
                    self.hazard_unit.clear_hazards();
                },
                _ => {}
            }
        }

        self.writeback_state = self.memory_state.clone();
        self.memory_state = PipelineState::default();

        Ok(())
    }

    // Ajout de méthodes pour la gestion du cache
    pub fn get_cache_statistics(&self) -> String {
        self.cache_system.get_statistics()
    }

    pub fn flush_caches(&mut self) -> VMResult<()> {
        self.cache_system.reset()?;
        self.bypass_buffer = BypassBuffer::new(32);
        self.pending_stores.clear();
        Ok(())
    }


    /// Étage Writeback: Écriture des résultats
    fn writeback_stage(&mut self, registers: &mut RegisterBank) -> VMResult<()> {
        if let Some(result) = &self.writeback_state.result {
            // Mettre à jour le registre destination si présent
            if let Some(dest) = self.writeback_state.destination {
                println!("Writeback - Écriture dans le registre {:?}: {}", dest, result.value);
                registers.write_register(dest, result.value)?;
                self.stats.writebacks += 1;
            }

            // Mettre à jour les flags
            registers.update_flags(result.flags)?;
            println!("Writeback - Instruction complétée");
        }

        // Réinitialiser l'état
        self.writeback_state = PipelineState::default();
        Ok(())
    }

    /// Décode une instruction
    pub fn decode_instruction(&self, instruction: &Instruction) -> VMResult<DecodedInstruction> {
        match instruction {
            // Instructions arithmétiques
            Instruction::Add(dest, src1, src2) => Ok(DecodedInstruction::Arithmetic(
                ArithmeticOp::Add { dest: *dest, src1: *src1, src2: *src2 }
            )),
            Instruction::Sub(dest, src1, src2) => Ok(DecodedInstruction::Arithmetic(
                ArithmeticOp::Sub { dest: *dest, src1: *src1, src2: *src2 }
            )),
            Instruction::Mul(dest, src1, src2) => Ok(DecodedInstruction::Arithmetic(
                ArithmeticOp::Mul { dest: *dest, src1: *src1, src2: *src2 }
            )),
            Instruction::Div(dest, src1, src2) => Ok(DecodedInstruction::Arithmetic(
                ArithmeticOp::Div { dest: *dest, src1: *src1, src2: *src2 }
            )),

            // Instructions mémoire
            Instruction::Load(reg, addr) => Ok(DecodedInstruction::Memory(
                MemoryOp::Load { reg: *reg, addr: *addr }
            )),
            Instruction::Store(reg, addr) => Ok(DecodedInstruction::Memory(
                MemoryOp::Store { reg: *reg, addr: *addr }
            )),
            Instruction::Move(dest, src) => Ok(DecodedInstruction::Memory(
                MemoryOp::Move { dest: *dest, src: *src }
            )),
            Instruction::LoadImm(reg, value) => Ok(DecodedInstruction::Memory(
                MemoryOp::LoadImm { reg: *reg, value: *value }
            )),

            // Instructions de contrôle
            Instruction::Jump(addr) => Ok(DecodedInstruction::Control(
                ControlOp::Jump { addr: *addr }
            )),
            Instruction::JumpIf(condition, addr) => Ok(DecodedInstruction::Control(
                ControlOp::JumpIf { condition: *condition, addr: *addr }
            )),
            Instruction::Call(addr) => Ok(DecodedInstruction::Control(
                ControlOp::Call { addr: *addr }
            )),
            Instruction::Return => Ok(DecodedInstruction::Control(
                ControlOp::Return
            )),
            Instruction::Nop => Ok(DecodedInstruction::Control(
                ControlOp::Nop
            )),
            Instruction::Halt => Ok(DecodedInstruction::Control(
                ControlOp::Halt
            )),
            // Instruction Compares
            Instruction::Cmp(src1, src2) => Ok(DecodedInstruction::Compare {
                src1: *src1,
                src2: *src2
            }),
        }
    }

     /// Exécute une instruction décodée

    pub fn execute_instruction(&mut self, decoded: &DecodedInstruction, registers: &mut RegisterBank) -> VMResult<ExecutionResult> {
        match decoded {

            DecodedInstruction::Arithmetic(op) => match op {
                ArithmeticOp::Add { dest, src1, src2 } => {
                    let val1 = registers.read_register(*src1)? as i64;
                    let val2 = registers.read_register(*src2)? as i64;
                    let result = val1.wrapping_add(val2);

                    println!("Exécution Add: {} + {} = {}", val1, val2, result);

                    Ok(ExecutionResult {
                        value: result,
                        flags: StatusFlags {
                            zero: result == 0,
                            negative: result < 0,
                            overflow: false,
                            carry: false,
                        },
                        branch_taken: false,
                        target_address: None,
                    })
                },
                ArithmeticOp::Sub { dest, src1, src2 } => {
                    let val1 = registers.read_register(*src1)? as i64;
                    let val2 = registers.read_register(*src2)? as i64;
                    let result = val1.wrapping_sub(val2);

                    println!("Exécution Sub: {} - {} = {}", val1, val2, result);

                    Ok(ExecutionResult {
                        value: result,
                        flags: StatusFlags {
                            zero: result == 0,
                            negative: result < 0,
                            overflow: false,
                            carry: false,
                        },
                        branch_taken: false,
                        target_address: None,
                    })
                },
                ArithmeticOp::Mul { dest, src1, src2 } => {
                    let val1 = registers.read_register(*src1)? as i64;
                    let val2 = registers.read_register(*src2)? as i64;
                    let result = val1.wrapping_mul(val2);

                    println!("Exécution Mul: {} * {} = {}", val1, val2, result);

                    Ok(ExecutionResult {
                        value: result,
                        flags: StatusFlags {
                            zero: result == 0,
                            negative: result < 0,
                            overflow: false,
                            carry: false,
                        },
                        branch_taken: false,
                        target_address: None,
                    })
                },
                ArithmeticOp::Div { dest, src1, src2 } => {
                    let val1 = registers.read_register(*src1)? as i64;
                    let val2 = registers.read_register(*src2)? as i64;

                    if val2 == 0 {
                        return Err(VMError::ArithmeticError("Division par zéro".into()));
                    }

                    let result = val1.wrapping_div(val2);

                    println!("Exécution Div: {} / {} = {}", val1, val2, result);

                    Ok(ExecutionResult {
                        value: result,
                        flags: StatusFlags {
                            zero: result == 0,
                            negative: result < 0,
                            overflow: false,
                            carry: false,
                        },
                        branch_taken: false,
                        target_address: None,

                    })
                },
            },
            DecodedInstruction::Memory(op) => match op {
                MemoryOp::LoadImm { reg, value } => {
                    println!("Exécution LoadImm: registre {:?}, valeur {}", reg, value);
                    Ok(ExecutionResult {
                        value: *value,
                        flags: StatusFlags::default(),
                        branch_taken: false,
                        target_address: None,
                    })
                },
                MemoryOp::Load { reg, addr } => {
                    println!("Exécution Load: registre {:?}, adresse {:?}", reg, addr);
                    Ok(ExecutionResult {
                        value: 0, // La valeur sera chargée dans l'étage memory
                        flags: StatusFlags::default(),
                        branch_taken: false,
                        target_address: None,
                    })
                },
                MemoryOp::Store { reg, addr } => {
                    let value = registers.read_register(*reg)?;
                    println!("Exécution Store: valeur {} vers adresse {:?}", value, addr);
                    Ok(ExecutionResult {
                        value: value as i64,
                        flags: StatusFlags::default(),
                        branch_taken: false,
                        target_address: None,
                    })
                },
                MemoryOp::Move { dest, src } => {
                    let value = registers.read_register(*src)?;
                    println!("Exécution Move: {} de {:?} vers {:?}", value, src, dest);
                    Ok(ExecutionResult {
                        value: value as i64,
                        flags: StatusFlags::default(),
                        branch_taken: false,
                        target_address: None,
                    })
                },
            },
            DecodedInstruction::Compare { src1, src2 } => {
                let val1 = registers.read_register(*src1)?;
                let val2 = registers.read_register(*src2)?;

                let mut flags = registers.get_status_flags_mut();
                flags.zero = val1 == val2;
                flags.negative = val1 < val2;
                flags.overflow = false; // À implémenter si nécessaire
                flags.carry = false;    // À implémenter si nécessaire

                // mettre à jour les flags
                registers.update_flags(flags)?;

                Ok(ExecutionResult {
                    value: 0,
                    flags: flags,
                    branch_taken: false,
                    target_address: None,
                    // written_registers: vec![],
                })

            },
            _ => Ok(ExecutionResult::default())
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
                carry: false,
            },
            branch_taken: false,
            target_address: None,
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
                carry: false,
            },
            branch_taken: false,
            target_address: None,
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
                carry: false,
            },
            branch_taken: false,
            target_address: None,
        })
    }

    fn check_dependencies(&self, decoded: &DecodedInstruction, registers: &RegisterBank) -> bool {
        match decoded {
            DecodedInstruction::Arithmetic(op) => match op {
                ArithmeticOp::Add { src1, src2, .. } => {
                    !self.is_register_ready(*src1) || !self.is_register_ready(*src2)
                },
                // Similaire pour Sub, Mul, Div...
                _ => false,
            },
            DecodedInstruction::Memory(op) => match op {
                MemoryOp::Store { reg, .. } => !self.is_register_ready(*reg),
                _ => false,
            },
            _ => false,
        }
    }
    pub fn is_register_ready(&self, reg: RegisterId) -> bool {
        // Vérifier si le registre n'est pas en train d'être modifié dans les étages précédents
        !self.memory_state.destination.map_or(false, |dest| dest == reg) &&
            !self.execute_state.destination.map_or(false, |dest| dest == reg)
    }

    /// Vérifie si une valeur peut être forwardée pour un registre
    pub fn try_forward_value(&self, reg: RegisterId) -> Option<i64> {
        self.forwarding_unit.get_forwarded_value_optimized(reg)
    }

    /// Met à jour le forwarding après l'exécution
    pub fn update_forwarding(&mut self, dest: RegisterId, result: &ExecutionResult) -> VMResult<()> {
        self.forwarding_unit.register_result(dest, result, ForwardingSource::Execute);
        Ok(())
    }

    /// Exécute une instruction avec support du forwarding
    pub fn execute_with_forwarding(&mut self, decoded: &DecodedInstruction, registers: &RegisterBank) -> VMResult<ExecutionResult> {
        match decoded {
            DecodedInstruction::Arithmetic(op) => {
                match op {
                    ArithmeticOp::Add { dest, src1, src2 } => {
                        // Obtenir les valeurs, soit depuis le forwarding, soit depuis les registres
                        let val1 = self.try_forward_value(*src1)
                            .unwrap_or_else(|| registers.read_register(*src1).unwrap() as i64);
                        let val2 = self.try_forward_value(*src2)
                            .unwrap_or_else(|| registers.read_register(*src2).unwrap() as i64);

                        let result = val1.wrapping_add(val2);
                        let execution_result = ExecutionResult {
                            value: result,
                            flags: StatusFlags {
                                zero: result == 0,
                                negative: result < 0,
                                overflow: false,
                                carry: false,
                            },
                            branch_taken: false,
                            target_address: None,
                        };

                        self.update_forwarding(*dest, &execution_result)?;
                        Ok(execution_result)
                    },
                    ArithmeticOp::Sub { dest, src1, src2 } => {
                        let val1 = self.try_forward_value(*src1)
                            .unwrap_or_else(|| registers.read_register(*src1).unwrap() as i64);
                        let val2 = self.try_forward_value(*src2)
                            .unwrap_or_else(|| registers.read_register(*src2).unwrap() as i64);

                        let result = val1.wrapping_sub(val2);
                        let execution_result = ExecutionResult {
                            value: result,
                            flags: StatusFlags {
                                zero: result == 0,
                                negative: result < 0,
                                overflow: false,
                                carry: false,
                            },
                            branch_taken: false,
                            target_address: None,
                        };

                        self.update_forwarding(*dest, &execution_result)?;
                        Ok(execution_result)
                    },
                    ArithmeticOp::Mul { dest, src1, src2 } => {
                        let val1 = self.try_forward_value(*src1)
                            .unwrap_or_else(|| registers.read_register(*src1).unwrap() as i64);
                        let val2 = self.try_forward_value(*src2)
                            .unwrap_or_else(|| registers.read_register(*src2).unwrap() as i64);

                        let result = val1.wrapping_mul(val2);
                        let execution_result = ExecutionResult {
                            value: result,
                            flags: StatusFlags {
                                zero: result == 0,
                                negative: result < 0,
                                overflow: false,
                                carry: false,
                            },
                            branch_taken: false,
                            target_address: None,
                        };

                        self.update_forwarding(*dest, &execution_result)?;
                        Ok(execution_result)
                    },
                    ArithmeticOp::Div { dest, src1, src2 } => {
                        let val1 = self.try_forward_value(*src1)
                            .unwrap_or_else(|| registers.read_register(*src1).unwrap() as i64);
                        let val2 = self.try_forward_value(*src2)
                            .unwrap_or_else(|| registers.read_register(*src2).unwrap() as i64);

                        // Vérification de la division par zéro
                        if val2 == 0 {
                            return Err(VMError::ArithmeticError("Division par zéro".into()));
                        }

                        let result = val1.wrapping_div(val2);
                        let execution_result = ExecutionResult {
                            value: result,
                            flags: StatusFlags {
                                zero: result == 0,
                                negative: result < 0,
                                overflow: false,
                                carry: false,
                            },
                            branch_taken: false,
                            target_address: None,
                        };

                        self.update_forwarding(*dest, &execution_result)?;
                        Ok(execution_result)
                    }
                }
            },
            DecodedInstruction::Memory(op) => {
                match op {
                    MemoryOp::LoadImm { reg, value } => {
                        let execution_result = ExecutionResult {
                            value: *value,
                            flags: StatusFlags::default(),
                            branch_taken: false,
                            target_address: None,

                        };
                        self.update_forwarding(*reg, &execution_result)?;
                        Ok(execution_result)
                    },
                    MemoryOp::Move { dest, src } => {
                        let val = self.try_forward_value(*src)
                            .unwrap_or_else(|| registers.read_register(*src).unwrap() as i64);

                        let execution_result = ExecutionResult {
                            value: val,
                            flags: StatusFlags::default(),
                            branch_taken: false,
                            target_address: None,
                        };
                        self.update_forwarding(*dest, &execution_result)?;
                        Ok(execution_result)
                    },
                    MemoryOp::Store { reg, addr } => {
                        let value = self.try_forward_value(*reg)
                            .unwrap_or_else(|| registers.read_register(*reg).unwrap() as i64);

                        println!("Execute with forwarding - Store: valeur {} du registre {:?}", value, reg);

                        Ok(ExecutionResult {
                            value,
                            flags: StatusFlags::default(),
                            branch_taken: false,
                            target_address: None,

                        })
                    }

                    _ => Ok(ExecutionResult::default())
                }
            },
            _ => Ok(ExecutionResult::default())
        }
    }

    fn execute_arithmetic_with_forwarding(&mut self, op: &ArithmeticOp, registers: &RegisterBank) -> VMResult<ExecutionResult> {
        match op {
            ArithmeticOp::Add { dest, src1, src2 } => {
                let val1 = self.try_forward_value(*src1)
                    .unwrap_or_else(|| registers.read_register(*src1).unwrap() as i64);
                let val2 = self.try_forward_value(*src2)
                    .unwrap_or_else(|| registers.read_register(*src2).unwrap() as i64);

                let result = ExecutionResult {
                    value: val1.wrapping_add(val2),
                    flags: StatusFlags {
                        zero: val1.wrapping_add(val2) == 0,
                        negative: val1.wrapping_add(val2) < 0,
                        overflow: false,
                        carry: false,
                    },
                    branch_taken: false,
                    target_address: None,
                };

                self.update_forwarding(*dest, &result)?;
                Ok(result)
            }
            ArithmeticOp::Sub { dest, src1, src2 } => {
                let val1 = self.try_forward_value(*src1)
                    .unwrap_or_else(|| registers.read_register(*src1).unwrap() as i64);
                let val2 = self.try_forward_value(*src2)
                    .unwrap_or_else(|| registers.read_register(*src2).unwrap() as i64);

                let result = ExecutionResult {
                    value: val1.wrapping_sub(val2),
                    flags: StatusFlags {
                        zero: val1.wrapping_sub(val2) == 0,
                        negative: val1.wrapping_sub(val2) < 0,
                        overflow: false,
                        carry: false,
                    },
                    branch_taken: false,
                    target_address: None,
                };

                self.update_forwarding(*dest, &result)?;
                Ok(result)
            }
            ArithmeticOp::Mul { dest, src1, src2 } => {
                let val1 = self.try_forward_value(*src1)
                    .unwrap_or_else(|| registers.read_register(*src1).unwrap() as i64);
                let val2 = self.try_forward_value(*src2)
                    .unwrap_or_else(|| registers.read_register(*src2).unwrap() as i64);

                let result = ExecutionResult {
                    value: val1.wrapping_mul(val2),
                    flags: StatusFlags {
                        zero: val1.wrapping_mul(val2) == 0,
                        negative: val1.wrapping_mul(val2) < 0,
                        overflow: false,
                        carry: false,
                    },
                    branch_taken: false,
                    target_address: None,
                };

                self.update_forwarding(*dest, &result)?;
                Ok(result)
            }
            ArithmeticOp::Div { dest, src1, src2 } => {
                let val1 = self.try_forward_value(*src1)
                    .unwrap_or_else(|| registers.read_register(*src1).unwrap() as i64);
                let val2 = self.try_forward_value(*src2)
                    .unwrap_or_else(|| registers.read_register(*src2).unwrap() as i64);

                // Vérification de la division par zéro
                if val2 == 0 {
                    return Err(VMError::ArithmeticError("Division par zéro".into()));
                }

                let result = ExecutionResult {
                    value: val1.wrapping_div(val2),
                    flags: StatusFlags {
                        zero: val1.wrapping_div(val2) == 0,
                        negative: val1.wrapping_div(val2) < 0,
                        overflow: false,
                        carry: false,
                    },
                    branch_taken: false,
                    target_address: None,
                };

                self.update_forwarding(*dest, &result)?;
                Ok(result)
            }

            // Implémenter les autres opérations arithmétiques de manière similaire
            _ => Ok(ExecutionResult::default())
        }
    }



    fn check_data_hazards(&self, decoded: &DecodedInstruction, registers: &RegisterBank) -> bool {
        match decoded {
            DecodedInstruction::Arithmetic(op) => {
                match op {
                    ArithmeticOp::Add { src1, src2, .. } |
                    ArithmeticOp::Sub { src1, src2, .. } |
                    ArithmeticOp::Mul { src1, src2, .. } |
                    ArithmeticOp::Div { src1, src2, .. } => {
                        // Vérifier si les registres sources sont en cours de modification
                        !self.is_register_ready(*src1) || !self.is_register_ready(*src2)
                    }
                }
            },
            DecodedInstruction::Memory(MemoryOp::Store { reg, .. }) => {
                !self.is_register_ready(*reg)
            },
            _ => false
        }
    }

    fn optimize_execute_stage(&mut self, registers: &RegisterBank) -> VMResult<()> {
        // Early exit if no work to do
        if self.execute_state.decoded.is_none() {
            return Ok(());
        }

        // Copier les données nécessaires pour éviter le double emprunt
        let decoded = self.execute_state.decoded.clone();

        if let Some(ref decoded) = decoded {
            if !self.needs_hazard_check(decoded) {
                // Clone la partie nécessaire pour l'exécution rapide
                let fast_path_result = self.execute_fast_path_impl(decoded, registers)?;
                self.execute_state.result = Some(fast_path_result);
                return Ok(());
            }
        }

        // Normal execution path
        // path normal d'exéc
        self.execute_stage(registers)
    }

    fn needs_hazard_check(&self, decoded: &DecodedInstruction) -> bool {
        matches!(decoded,
            DecodedInstruction::Memory(_) |
            DecodedInstruction::Arithmetic(ArithmeticOp::Add { .. })
        )
    }


    // Nouvelle méthode qui implémente la logique d'exécution rapide
    fn execute_fast_path_impl(&self, decoded: &DecodedInstruction, registers: &RegisterBank) -> VMResult<ExecutionResult> {
        // Implémentation de l'exécution rapide
        match decoded {
            DecodedInstruction::Arithmetic(op) => {
                // Logique pour les opérations arithmétiques simples
                self.execute_arithmetic(op)
            }
            // Autres cas...
            _ => Ok(ExecutionResult::default())
        }
    }

    // Garde la méthode originale execute_fast_path mais délègue à impl
    fn execute_fast_path(&mut self, decoded: &DecodedInstruction, registers: &RegisterBank) -> VMResult<()> {
        let result = self.execute_fast_path_impl(decoded, registers)?;
        self.execute_state.result = Some(result);
        Ok(())
    }

    fn flush_pipeline(&mut self) {
        self.fetch_state = PipelineState::default();
        self.decode_state = PipelineState::default();
        self.predicted_branches.clear();
        self.metrics.flush_count += 1;
    }


}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::pvm::instructions::{Instruction, RegisterId, Address};
    use crate::pvm::registers::RegisterBank;
    use crate::pvm::memorys::MemoryController;

    fn setup_test_env() -> (Pipeline, RegisterBank, MemoryController) {
        let pipeline = Pipeline::new();
        let register_bank = RegisterBank::new(8).unwrap();  // 8 registres
        let memory_controller = MemoryController::new(1024, 256).unwrap(); // 1KB mémoire, 256B cache
        (pipeline, register_bank, memory_controller)
    }

    #[test]
    fn test_data_dependency_forwarding() {
        let (mut pipeline, mut register_bank, mut memory_controller) = setup_test_env();

        // Test avec dépendances de données immédiates
        // R0 = 10
        // R1 = R0 + 5  // Dépendance immédiate avec R0
        // R2 = R1 + 3  // Dépendance immédiate avec R1
        let program = vec![
            Instruction::LoadImm(RegisterId(0), 10),
            Instruction::Add(RegisterId(1), RegisterId(0), RegisterId(0)),
            Instruction::Add(RegisterId(2), RegisterId(1), RegisterId(1)),
        ];

        pipeline.load_instructions(program).unwrap();

        // Exécuter jusqu'à ce que le pipeline soit vide
        while !pipeline.is_empty().unwrap() {
            pipeline.cycle(&mut register_bank, &mut memory_controller).unwrap();
        }

        // Vérifier les résultats
        assert_eq!(register_bank.read_register(RegisterId(0)).unwrap(), 10);  // R0 = 10
        assert_eq!(register_bank.read_register(RegisterId(1)).unwrap(), 20);  // R1 = 10 + 10
        assert_eq!(register_bank.read_register(RegisterId(2)).unwrap(), 40);  // R2 = 20 + 20
    }

    #[test]
    fn test_memory_dependency_forwarding() {
        let (mut pipeline, mut register_bank, mut memory_controller) = setup_test_env();

        // Test avec dépendances mémoire
        // R0 = 42
        // Store R0, @100
        // R1 = Load @100     // Dépendance mémoire
        // R2 = R1 + R0       // Dépendance de registre
        let program = vec![
            Instruction::LoadImm(RegisterId(0), 42),
            Instruction::Store(RegisterId(0), Address(100)),
            Instruction::Load(RegisterId(1), Address(100)),
            Instruction::Add(RegisterId(2), RegisterId(1), RegisterId(0)),
        ];

        pipeline.load_instructions(program).unwrap();

        while !pipeline.is_empty().unwrap() {
            pipeline.cycle(&mut register_bank, &mut memory_controller).unwrap();
        }

        assert_eq!(register_bank.read_register(RegisterId(0)).unwrap(), 42);
        assert_eq!(register_bank.read_register(RegisterId(1)).unwrap(), 42);
        assert_eq!(register_bank.read_register(RegisterId(2)).unwrap(), 84);
    }

    #[test]
    fn test_complex_arithmetic_forwarding() {
        let (mut pipeline, mut register_bank, mut memory_controller) = setup_test_env();

        // Test avec une séquence complexe d'opérations arithmétiques
        // R0 = 10
        // R1 = 5
        // R2 = R0 + R1      // 15
        // R3 = R2 * R1      // 75
        // R4 = R3 - R2      // 60
        let program = vec![
            Instruction::LoadImm(RegisterId(0), 10),
            Instruction::LoadImm(RegisterId(1), 5),
            Instruction::Add(RegisterId(2), RegisterId(0), RegisterId(1)),
            Instruction::Mul(RegisterId(3), RegisterId(2), RegisterId(1)),
            Instruction::Sub(RegisterId(4), RegisterId(3), RegisterId(2)),
        ];

        pipeline.load_instructions(program).unwrap();

        // Exécuter le pipeline
        while !pipeline.is_empty().unwrap() {
            pipeline.cycle(&mut register_bank, &mut memory_controller).unwrap();
        }

        // Vérifier les résultats
        assert_eq!(register_bank.read_register(RegisterId(2)).unwrap(), 15);  // 10 + 5
        assert_eq!(register_bank.read_register(RegisterId(3)).unwrap(), 75);  // 15 * 5
        assert_eq!(register_bank.read_register(RegisterId(4)).unwrap(), 60);  // 75 - 15
    }

    #[test]
    fn test_mixed_memory_arithmetic() {
        let (mut pipeline, mut register_bank, mut memory_controller) = setup_test_env();

        // Test combinant opérations mémoire et arithmétiques
        // R0 = 100
        // Store R0, @200
        // R1 = 50
        // R2 = Load @200    // Doit charger 100
        // R3 = R2 + R1      // 150
        // Store R3, @300
        // R4 = Load @300    // Doit charger 150
        let program = vec![
            Instruction::LoadImm(RegisterId(0), 100),
            Instruction::Store(RegisterId(0), Address(200)),
            Instruction::LoadImm(RegisterId(1), 50),
            Instruction::Load(RegisterId(2), Address(200)),
            Instruction::Add(RegisterId(3), RegisterId(2), RegisterId(1)),
            Instruction::Store(RegisterId(3), Address(300)),
            Instruction::Load(RegisterId(4), Address(300)),
        ];

        pipeline.load_instructions(program).unwrap();

        while !pipeline.is_empty().unwrap() {
            pipeline.cycle(&mut register_bank, &mut memory_controller).unwrap();
        }

        assert_eq!(register_bank.read_register(RegisterId(2)).unwrap(), 100);
        assert_eq!(register_bank.read_register(RegisterId(3)).unwrap(), 150);
        assert_eq!(register_bank.read_register(RegisterId(4)).unwrap(), 150);
    }

    #[test]
    fn test_pipeline_stalls() {
        let (mut pipeline, mut register_bank, mut memory_controller) = setup_test_env();

        // Test vérifiant les stalls du pipeline
        // R0 = 10
        // R1 = R0 + 5       // Dépendance avec R0
        // R2 = R1 + 3       // Dépendance avec R1
        // Store R2, @100    // Dépendance avec R2
        // R3 = Load @100    // Dépendance mémoire
        let program = vec![
            Instruction::LoadImm(RegisterId(0), 10),
            Instruction::Add(RegisterId(1), RegisterId(0), RegisterId(0)),
            Instruction::Add(RegisterId(2), RegisterId(1), RegisterId(1)),
            Instruction::Store(RegisterId(2), Address(100)),
            Instruction::Load(RegisterId(3), Address(100)),
        ];

        pipeline.load_instructions(program).unwrap();

        let mut cycles = 0;
        while !pipeline.is_empty().unwrap() {
            pipeline.cycle(&mut register_bank, &mut memory_controller).unwrap();
            cycles += 1;
        }

        // Vérifier les résultats et les statistiques
        assert_eq!(register_bank.read_register(RegisterId(3)).unwrap(), 40);
        assert!(pipeline.stats.stalls > 0, "Le pipeline devrait avoir des stalls");
        println!("Pipeline terminé en {} cycles avec {} stalls",
                 cycles, pipeline.stats.stalls);
    }


}

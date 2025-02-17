use crate::pvm::instructions::{Address, ArithmeticOp, ControlOp, DecodedInstruction, Instruction, MemoryOp, RegisterId};
use crate::pvm::vm_errors::{VMError, VMResult};

use std::collections::VecDeque;
use std::collections::HashMap;
use crate::pvm::caches::CacheStatistics;
use crate::pvm::memorys::MemoryController;
use crate::pvm::registers::RegisterBank;
use crate::pvm::forwardings::ForwardingSource;
use crate::pvm::forwardings::ForwardingUnit;

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
    pub value: i64,
    pub flags: StatusFlags,
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

// Ajout d'un impl Default pour ExecutionResult
impl Default for ExecutionResult {
    fn default() -> Self {
        Self {
            value: 0,
            flags: StatusFlags::default(),
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
}

#[derive(Clone, Debug)]
struct RegisterDependency {
    reg_id: RegisterId,
    stage: PipelineStage,
}

#[derive(Clone, Debug, PartialEq)]
enum PipelineStage {
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
        self.stats.cycles += 1;

        if let Err(e) = self.writeback_stage(registers) {
            println!("Erreur dans l'étage Writeback: {:?}", e);
            return Err(e);
        }

        if let Err(e) = self.memory_stage(memory) {
            println!("Erreur dans l'étage Memory: {:?}", e);
            return Err(e);
        }

        if let Err(e) = self.execute_stage(registers) {  // Ajout du paramètre registers
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
        if self.execute_state.decoded.is_some() {
            self.stats.stalls += 1;
            return Ok(());
        }

        if let Some(instruction) = &self.decode_state.instruction {
            println!("Décodage de l'instruction: {:?}", instruction);

            if self.hazard_unit.check_hazards(instruction, registers) {
                self.stats.hazards += 1;
                return Ok(());
            }

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
        }

        self.execute_state = self.decode_state.clone();
        self.decode_state = PipelineState::default();

        Ok(())
    }

    /// Exécute une instruction avec support du forwarding
    fn execute_stage(&mut self, registers: &RegisterBank) -> VMResult<()> {
        if self.memory_state.result.is_some() {
            self.stats.stalls += 1;
            println!("Execute - Stall dû à l'étage mémoire occupé");
            return Ok(());
        }

        let decoded = self.execute_state.decoded.clone();

        if let Some(decoded) = &decoded {
            println!("Execute - Début exécution: {:?}", decoded);

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
                DecodedInstruction::Control(_) => None,
            };

            // Exécuter l'instruction avec forwarding
            self.execute_state.result = Some(match decoded {
                DecodedInstruction::Memory(MemoryOp::Store { reg, .. }) => {
                    // Utiliser le forwarding ou lire le registre
                    let value = self.try_forward_value(*reg)
                        .unwrap_or_else(|| registers.read_register(*reg).unwrap() as i64);

                    println!("Execute - Store: forwarding/registre valeur {} depuis {:?}", value, reg);

                    ExecutionResult {
                        value,
                        flags: StatusFlags::default(),
                    }
                },
                _ => self.execute_with_forwarding(decoded, registers)?,
            });

            println!("Execute - Résultat: {:?}", self.execute_state.result);
            self.stats.instructions_executed += 1;
        }

        self.memory_state = self.execute_state.clone();
        self.execute_state = PipelineState::default();
        Ok(())
    }
    // fn execute_stage(&mut self, registers: &RegisterBank) -> VMResult<()> {
    //     if self.memory_state.result.is_some() {
    //         self.stats.stalls += 1;
    //         println!("Execute - Stall dû à l'étage mémoire occupé");
    //         return Ok(());
    //     }
    //
    //     // Clone les données nécessaires pour éviter le double emprunt
    //     let decoded = self.execute_state.decoded.clone();
    //
    //     if let Some(decoded) = &decoded {
    //         println!("Execute - Début exécution: {:?}", decoded);
    //
    //         // Définir le registre destination et exécuter l'instruction
    //         self.execute_state.destination = match decoded {
    //             DecodedInstruction::Arithmetic(op) => match op {
    //                 ArithmeticOp::Add { dest, .. } => Some(*dest),
    //                 ArithmeticOp::Sub { dest, .. } => Some(*dest),
    //                 ArithmeticOp::Mul { dest, .. } => Some(*dest),
    //                 ArithmeticOp::Div { dest, .. } => Some(*dest),
    //             },
    //             DecodedInstruction::Memory(op) => match op {
    //                 MemoryOp::LoadImm { reg, .. } => Some(*reg),
    //                 MemoryOp::Load { reg, .. } => Some(*reg),
    //                 MemoryOp::Store { .. } => None,
    //                 MemoryOp::Move { dest, .. } => Some(*dest),
    //             },
    //             DecodedInstruction::Control(_) => None,
    //         };
    //
    //         // Exécuter l'instruction
    //         let result = match decoded {
    //             DecodedInstruction::Memory(MemoryOp::Store { reg, addr }) => {
    //                 let value = registers.read_register(*reg)?;
    //                 ExecutionResult {
    //                     value: value as i64,
    //                     flags: StatusFlags::default(),
    //                 }
    //             },
    //             DecodedInstruction::Memory(MemoryOp::Load { .. }) => {
    //                 ExecutionResult::default()
    //             },
    //             _ => self.execute_with_forwarding(decoded, registers)?,
    //         };
    //
    //         println!("Execute - Résultat: {:?}", result);
    //         self.execute_state.result = Some(result);
    //         self.stats.instructions_executed += 1;
    //     }
    //
    //     // Propager l'état vers l'étage suivant
    //     self.memory_state = self.execute_state.clone();
    //     self.execute_state = PipelineState::default();
    //     Ok(())
    // }

    /// Étage Memory: Accès mémoire
    fn memory_stage(&mut self, memory: &mut MemoryController) -> VMResult<()> {
        if let Some(decoded) = &self.memory_state.decoded {
            match decoded {
                DecodedInstruction::Memory(MemoryOp::Store { reg, addr }) => {
                    if let Some(result) = &self.memory_state.result {
                        println!("Memory - Store: écriture de {} à l'adresse {:?}", result.value, addr);
                        memory.write(addr.0, result.value as u64)?;
                    }
                },
                DecodedInstruction::Memory(MemoryOp::Load { reg, addr }) => {
                    let value = memory.read(addr.0)?;
                    println!("Memory - Load: lecture de {} depuis l'adresse {:?}", value, addr);
                    self.memory_state.result = Some(ExecutionResult {
                        value: value as i64,
                        flags: StatusFlags::default(),
                    });
                },
                _ => {}
            }
        }

        self.writeback_state = self.memory_state.clone();
        self.memory_state = PipelineState::default();
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
    fn decode_instruction(&self, instruction: &Instruction) -> VMResult<DecodedInstruction> {
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
        }
    }

    /// Exécute une instruction décodée
    // fn execute_instruction(&mut self, decoded: &DecodedInstruction, registers: &RegisterBank) -> VMResult<ExecutionResult> {
    //     match decoded {
    //         DecodedInstruction::Arithmetic(op) => match op {
    //             ArithmeticOp::Add { dest, src1, src2 } => {
    //                 let val1 = registers.read_register(*src1)?;
    //                 let val2 = registers.read_register(*src2)?;
    //                 let result = val1.wrapping_add(val2);
    //
    //                 println!("Exécution Add: {} + {} = {}", val1, val2, result);
    //
    //                 Ok(ExecutionResult {
    //                     value: result as i64,
    //                     flags: StatusFlags {
    //                         zero: result == 0,
    //                         negative: (result as i64) < 0,
    //                         overflow: false,
    //                     },
    //                 })
    //             },
    //             // Autres opérations arithmétiques similaires...
    //             _ => Ok(ExecutionResult::default())
    //         },
    //         DecodedInstruction::Memory(op) => match op {
    //             MemoryOp::LoadImm { reg: _, value } => {
    //                 println!("Exécution LoadImm: {}", value);
    //                 Ok(ExecutionResult {
    //                     value: *value,
    //                     flags: StatusFlags::default(),
    //                 })
    //             },
    //             MemoryOp::Store { reg, addr: _ } => {
    //                 let value = registers.read_register(*reg)?;
    //                 println!("Exécution Store: valeur {} depuis registre {:?}", value, reg);
    //                 Ok(ExecutionResult {
    //                     value: value as i64,
    //                     flags: StatusFlags::default(),
    //                 })
    //             },
    //             // Autres opérations mémoire...
    //             _ => Ok(ExecutionResult::default())
    //         },
    //         DecodedInstruction::Control(_) => Ok(ExecutionResult::default())
    //     }
    // }


    fn execute_instruction(&mut self, decoded: &DecodedInstruction, registers: &RegisterBank) -> VMResult<ExecutionResult> {
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
                        },
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
                        },
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
                        },
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
                        },
                    })
                },
            },
            DecodedInstruction::Memory(op) => match op {
                MemoryOp::LoadImm { reg, value } => {
                    println!("Exécution LoadImm: registre {:?}, valeur {}", reg, value);
                    Ok(ExecutionResult {
                        value: *value,
                        flags: StatusFlags::default(),
                    })
                },
                MemoryOp::Load { reg, addr } => {
                    println!("Exécution Load: registre {:?}, adresse {:?}", reg, addr);
                    Ok(ExecutionResult {
                        value: 0, // La valeur sera chargée dans l'étage memory
                        flags: StatusFlags::default(),
                    })
                },
                MemoryOp::Store { reg, addr } => {
                    let value = registers.read_register(*reg)?;
                    println!("Exécution Store: valeur {} vers adresse {:?}", value, addr);
                    Ok(ExecutionResult {
                        value: value as i64,
                        flags: StatusFlags::default(),
                    })
                },
                MemoryOp::Move { dest, src } => {
                    let value = registers.read_register(*src)?;
                    println!("Exécution Move: {} de {:?} vers {:?}", value, src, dest);
                    Ok(ExecutionResult {
                        value: value as i64,
                        flags: StatusFlags::default(),
                    })
                },
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
    fn is_register_ready(&self, reg: RegisterId) -> bool {
        // Vérifier si le registre n'est pas en train d'être modifié dans les étages précédents
        !self.memory_state.destination.map_or(false, |dest| dest == reg) &&
            !self.execute_state.destination.map_or(false, |dest| dest == reg)
    }

    /// Vérifie si une valeur peut être forwardée pour un registre
    pub fn try_forward_value(&self, reg: RegisterId) -> Option<i64> {
        self.forwarding_unit.get_forwarded_value(reg)
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
                            },
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
                            },
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
                            },
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
                            },
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
                    },
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
                    },
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
                    },
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
                    },
                };

                self.update_forwarding(*dest, &result)?;
                Ok(result)
            }

            // Implémenter les autres opérations arithmétiques de manière similaire
            _ => Ok(ExecutionResult::default())
        }
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
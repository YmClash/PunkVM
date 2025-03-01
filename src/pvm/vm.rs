// //src/pvm/vm.rs
use std::path::Path;
use std::io;



use crate::pipeline::{Pipeline};
use crate::alu::alu::ALU;
use crate::bytecode::files::SegmentType::{Code, Data, ReadOnlyData};
use crate::BytecodeFile;
use crate::pvm::memorys::{Memory, MemoryConfig};

/// Configuration de la machine virtuelle
#[derive(Debug, Clone)]
pub struct VMConfig {
    pub memory_size: usize,     // Taille de la mémoire
    pub num_registers: usize,   // Nombre de registres
    pub l1_cache_size: usize,   // Taille du cache L1
    pub store_buffer_size: usize,   // Taille du buffer de stockage
    pub stack_size: usize,      // Taille de la pile
    pub fetch_buffer_size: usize,   // Taille du buffer de fetch
    pub btb_size: usize,    // Taille du BTB (Branch Target Buffer)
    pub ras_size: usize,    // Taaille du RAS (Return Address Stack)
    pub enable_forwarding: bool,    // Active ou désactive le forwarding
    pub enable_hazard_detection: bool, // Active ou désactive la détection de hazards
}

impl Default for VMConfig{
    fn default() -> Self {
        Self {
            memory_size: 1024 * 1024, // 1MB
            num_registers: 16,
            l1_cache_size: 4* 1024, // 4KB
            store_buffer_size: 8,
            stack_size: 64 * 1024, // 64KB
            fetch_buffer_size: 16,
            btb_size: 64,
            ras_size: 8,
            enable_forwarding: true,
            enable_hazard_detection: true,
        }
    }
}

///Etat de la machine virtuelle
#[derive(Debug, PartialEq, Eq)]
pub enum VMState{
    Ready,
    Running,
    Halted,
    Error(String),
}
/// Statistiques d'exécution de la VM
#[derive(Debug, Clone, Copy)]
pub struct VMStats {
    pub cycles: u64,        // Nombre total de cycles exécutés
    pub instructions_executed: u64, // Nombre total d'instructions exécutées
    pub ipc: f64,           // Instructions par cycle (IPC)
    pub stalls: u64,        // Nombre de stalls dans le pipeline
    pub hazards: u64,       // Nombre de hazards détectés
    pub forwards: u64,      // Nombre de forwards effectués
    pub memory_hits: u64,   // Nombre de hits dans le cache mémoire
    pub memory_misses: u64, // Nombre de misses dans le cache mémoire
}

/// Machine virtuelle PunkVM
pub struct PunkVM{
    config: VMConfig,
    state: VMState,
    pipeline: Pipeline,
    alu: ALU,
    memory: Memory,
    pub pc: usize, // Compteur de programme
    pub registers: Vec<u64>, // Registres
    program: Option<BytecodeFile>, // Programme
    cycles: u64, // Nombre de cycles
    instructions_executed: u64, // Nombre d'instructions exécutées

}


impl PunkVM {
    /// Crée une nouvelle instance de PunkVM avec la configuration par défaut
    pub fn new() -> Self {
        Self::with_config(VMConfig::default())
    }

    /// Crée une nouvelle instance de PunkVM avec une configuration personnalisée
    pub fn with_config(config: VMConfig) -> Self {
        let memory_config = MemoryConfig {
            size: config.memory_size,
            l1_cache_size: config.l1_cache_size,
            store_buffer_size: config.store_buffer_size,
        };

        Self {
            config: config.clone(),
            state: VMState::Ready,
            pipeline: Pipeline::new(
                config.fetch_buffer_size,
                config.enable_forwarding,
                config.enable_hazard_detection,
            ),
            alu: ALU::new(),
            memory: Memory::new(memory_config),
            pc: 0,
            registers: vec![0; config.num_registers],
            program: None,
            cycles: 0,
            instructions_executed: 0,
        }
    }

    /// Charge un programme depuis un fichier bytecode
    pub fn load_program<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        let program = BytecodeFile::read_from_file(path)?;
        self.load_program_from_bytecode(program)
    }

    /// Charge un programme depuis une structure BytecodeFile
    pub fn load_program_from_bytecode(&mut self, program: BytecodeFile) -> io::Result<()> {
        // Réinitialiser l'état de la VM
        self.reset();

        // Charger le code en mémoire
        self.load_code_segment(&program)?;

        // Charger les segments de données
        self.load_data_segments(&program)?;

        // Stocker le programme
        self.program = Some(program);

        // Mettre à jour l'état
        self.state = VMState::Ready;

        Ok(())
    }

    /// Exécute le programme chargé jusqu'à la fin ou jusqu'à une erreur
    pub fn run(&mut self) -> Result<(), String> {
        if self.program.is_none() {
            return Err("Aucun programme chargé".to_string());
        }

        self.state = VMState::Running;

        while self.state == VMState::Running {
            match self.step() {
                Ok(_) => {},
                Err(e) => {
                    self.state = VMState::Error(e.to_string());
                    return Err(e);
                }
            }
        }

        if let VMState::Error(ref e) = self.state {
            Err(e.clone())
        } else {
            Ok(())
        }
    }

    /// Exécute un seul cycle du pipeline
    pub fn step(&mut self) -> Result<(), String> {
        if self.state != VMState::Running {
            return Err("La machine virtuelle n'est pas en cours d'exécution".to_string());
        }

        // Cycle du pipeline
        let pipeline_state = self.pipeline.cycle(
            self.pc as u32,
            &mut self.registers,
            &mut self.memory,
            &mut self.alu,
            &self.program.as_ref().unwrap().code,
        )?;

        // Mise à jour du PC
        self.pc = pipeline_state.next_pc as usize;

        // Mise à jour des compteurs
        self.cycles += 1;
        self.instructions_executed += pipeline_state.instructions_completed as u64;

        // Vérifier l'état après le cycle
        if pipeline_state.halted {
            self.state = VMState::Halted;
        }

        Ok(())
    }

    /// Réinitialise la machine virtuelle
    pub fn reset(&mut self) {
        self.pc = 0;
        self.registers = vec![0; self.config.num_registers];
        self.cycles = 0;
        self.instructions_executed = 0;
        self.state = VMState::Ready;
        self.pipeline.reset();
        self.memory.reset();
    }

    /// Retourne les statistiques d'exécution
    pub fn stats(&self) -> VMStats {
        VMStats {
            cycles: self.cycles,
            instructions_executed: self.instructions_executed,
            ipc: if self.cycles > 0 {
                self.instructions_executed as f64 / self.cycles as f64
            } else {
                0.0
            },
            stalls: self.pipeline.stats().stalls,
            hazards: self.pipeline.stats().hazards,
            forwards: self.pipeline.stats().forwards,
            memory_hits: self.memory.stats().hits,
            memory_misses: self.memory.stats().misses,
        }
    }

    /// Retourne l'état actuel de la VM
    pub fn state(&self) -> &VMState {
        &self.state
    }

    /// Charge le segment de code en mémoire
    fn load_code_segment(&mut self, program: &BytecodeFile) -> io::Result<()> {
        // Recherche du segment de code
        let code_segment = program
            .segments
            .iter()
            .find(|s| s.segment_type == Code)
            .ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData,
                "Segment de code manquant dans le programme",
            ))?;

        // Encoder les instructions
        let mut code_bytes = Vec::new();
        for instruction in &program.code {
            code_bytes.extend_from_slice(&instruction.encode());
        }

        // Vérifier que la taille correspond à celle du segment
        if code_bytes.len() != code_segment.size as usize {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Taille du code incohérente: segment={}, encodé={}",
                    code_segment.size, code_bytes.len()
                ),
            ));
        }

        // Charger en mémoire à l'adresse spécifiée
        let load_addr = code_segment.load_addr;
        self.memory.write_block(load_addr, &code_bytes)?;

        Ok(())
    }

    /// Charge les segments de données en mémoire
    fn load_data_segments(&mut self, program: &BytecodeFile) -> io::Result<()> {
        // Segment de données
        if let Some(data_segment) = program
            .segments
            .iter()
            .find(|s| s.segment_type == Data)
        {
            if data_segment.size > 0 {
                self.memory.write_block(data_segment.load_addr, &program.data)?;
            }
        }

        // Segment de données en lecture seule
        if let Some(ro_segment) = program
            .segments
            .iter()
            .find(|s| s.segment_type == ReadOnlyData)
        {
            if ro_segment.size > 0 {
                self.memory.write_block(ro_segment.load_addr, &program.readonly_data)?;
            }
        }

        Ok(())
    }
}










// use crate::pvm::instructions::{DecodedInstruction, InstructionDecoder};
// use crate::pvm::memorys::MemoryController;
// use crate::pvm::registers::RegisterBank;
// use crate::pvm::vm_errors::{VMError, VMResult};
//
// use crate::pvm::instructions::{Instruction, RegisterId};
// use crate::pvm::pipelines::{DetailedStats, Pipeline};
// use crate::pvm::stacks::Stack;
//
// ///configuration de la VM
// #[derive(Debug)]
// pub struct VMConfig {
//     pub memory_size: usize,
//     pub stack_size: usize,
//     pub cache_size: usize,
//     pub register_count: usize,
//     pub optimization_level: OptimizationLevel,
// }
//
// ///niveau d'optimisation
//
// #[derive(Debug,Clone,Copy,PartialEq)]
// pub enum OptimizationLevel {
//     None,
//     Basic,
//     Aggressive,
// }
//
// #[derive(Debug)]
// pub struct VMStatistics {
//     pub instructions_executed: usize,
//     pub cycles: usize,
//     pub cache_hits: usize,
//     pub pipeline_stalls: usize,
// }
//
//
// /// Structure pricipale de la VM
// pub struct  PunkVM {
//     pub config:VMConfig,
//     pub memory_controller: MemoryController,
//     // register_controller: RegisterController,
//     pub stack: Stack,
//     pub register_bank: RegisterBank,
//     pub instruction_decoder: InstructionDecoder,
//     pub pipeline: Pipeline,
// }
//
//
// impl PunkVM {
//     /// Crée une nouvelle instance de la VM avec la configuration spécifiée
//     pub fn new(config: VMConfig) -> VMResult<Self> {
//         // Validation de la configuration
//         if config.memory_size == 0 {
//             return Err(VMError::ConfigError("Taille mémoire invalide".into()));
//         }
//         if config.register_count == 0 {
//             return Err(VMError::ConfigError("Nombre de registres invalide".into()));
//         }
//
//         Ok(Self {
//             memory_controller: MemoryController::new(config.memory_size, config.cache_size)?,
//             register_bank: RegisterBank::new(config.register_count)?,
//             instruction_decoder: InstructionDecoder::new(),
//             pipeline:Pipeline::new(),
//             stack: Stack::new(config.stack_size)?,
//             config,
//         })
//     }
//
//     /// Réinitialise la VM
//     pub fn reset(&mut self) -> VMResult<()> {
//         self.memory_controller.reset()?;
//         self.register_bank.reset()?;
//         Ok(())
//     }
//
//     /// Exécute  les Instructions
//     pub fn execute(&mut self, instruction: Instruction) -> VMResult<()> {
//         let decoded = self.instruction_decoder.decode(instruction)?;
//
//         // match decoded {
//         //     DecodedInstruction::Arithmetic(op) => self.execute_arithmetic(&op),
//         //     DecodedInstruction::Memory(op) => self.execute_memory(&op),
//         //     DecodedInstruction::Control(op) => self.execute_control(&op),
//         //     DecodedInstruction::Branch(op) => self.execute_branch(&op),
//         //     DecodedInstruction::Compare { src1, src2 } => self.execute_compare(src1, src2),
//         // }
//     }
//
//     /// load Program
//     pub fn load_program(&mut self, program: Vec<Instruction>) -> VMResult<()> {
//         println!("Chargement du programme: {} instructions", program.len());
//         //
//         // // Réinitialiser l'état de la VM
//         // self.reset()?;
//         //
//         // // Charger les instructions dans le buffer d'instructions du pipeline
//         // self.pipeline.load_instructions(program.into())?;
//
//         Ok(())
//     }
//
//     /// Run
//     pub fn run(&mut self) -> VMResult<()> {
//         println!("Exécution du programme...");
//         // let mut cycles = 0;
//         //
//         // while !self.pipeline.is_empty()? {
//         //     cycles += 1;
//         //     self.pipeline.cycle(&mut self.register_bank, &mut self.memory_controller)?;
//         //
//         //     if cycles > 1000 { // Sécurité anti-boucle infinie
//         //         return Err(VMError::ExecutionError("Trop de cycles d'exécution".into()));
//         //     }
//         //
//         //     if self.pipeline.should_halt()? {
//         //         println!("Programme terminé par instruction HALT");
//         //         break;
//         //     }
//         // }
//
//         Ok(())
//     }
//
//
//     ///read Register
//     pub fn read_register(&self, reg_id: RegisterId) -> VMResult<i64> {
//         let value = self.register_bank.read_register(reg_id)?;
//         Ok(value as i64)
//     }
//
//     /// get Statistics
//     pub fn get_statistics(&self) -> VMResult<VMStatistics> {
//         Ok(VMStatistics {
//             instructions_executed: self.pipeline.stats.instructions_executed,
//             cycles: self.pipeline.stats.cycles,
//             cache_hits: self.memory_controller.get_cache_stats()?.hits,
//             pipeline_stalls: self.pipeline.stats.stalls,
//         })
//     }
//
//     // Ajouter une méthode pour obtenir les statistiques détaillées
//     pub fn get_detailed_stats(&self) -> VMResult<DetailedStats> {
//         Ok(DetailedStats {
//             pipeline_stats: self.pipeline.stats.clone(),
//             cache_stats: self.memory_controller.get_cache_stats()?,
//         })
//     }
//
//
// }
//
//

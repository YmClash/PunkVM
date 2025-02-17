use crate::pvm::instructions::{DecodedInstruction, InstructionDecoder};
use crate::pvm::memorys::MemoryController;
use crate::pvm::registers::RegisterBank;
use crate::pvm::vm_errors::{VMError, VMResult};

use crate::pvm::instructions::{Instruction, RegisterId};
use crate::pvm::pipelines::{DetailedStats, Pipeline};

///configuration de la VM
#[derive(Debug)]
pub struct VMConfig {
    pub memory_size: usize,
    pub stack_size: usize,
    pub cache_size: usize,
    pub register_count: usize,
    pub optimization_level: OptimizationLevel,
}

///niveau d'optimisation

#[derive(Debug,Clone,Copy,PartialEq)]
pub enum OptimizationLevel {
    None,
    Basic,
    Aggressive,
}

#[derive(Debug)]
pub struct VMStatistics {
    pub instructions_executed: usize,
    pub cycles: usize,
    pub cache_hits: usize,
    pub pipeline_stalls: usize,
}


/// Structure pricipale de la VM
pub struct  PunkVM {
    pub config:VMConfig,
    pub memory_controller: MemoryController,
    // register_controller: RegisterController,
    pub register_bank: RegisterBank,
    pub instruction_decoder: InstructionDecoder,
    pub pipeline: Pipeline,
}


impl PunkVM {
    /// Crée une nouvelle instance de la VM avec la configuration spécifiée
    pub fn new(config: VMConfig) -> VMResult<Self> {
        // Validation de la configuration
        if config.memory_size == 0 {
            return Err(VMError::ConfigError("Taille mémoire invalide".into()));
        }
        if config.register_count == 0 {
            return Err(VMError::ConfigError("Nombre de registres invalide".into()));
        }

        Ok(Self {
            memory_controller: MemoryController::new(config.memory_size, config.cache_size)?,
            register_bank: RegisterBank::new(config.register_count)?,
            instruction_decoder: InstructionDecoder::new(),
            pipeline:Pipeline::new(),
            config,
        })
    }

    /// Réinitialise la VM
    pub fn reset(&mut self) -> VMResult<()> {
        self.memory_controller.reset()?;
        self.register_bank.reset()?;
        Ok(())
    }


    pub fn execute(&mut self, instruction: Instruction) -> VMResult<()> {
        // Décode l'instruction
        let decoded = self.instruction_decoder.decode(instruction)?;

        // Exécute l'instruction décodée
        match decoded {
            DecodedInstruction::Arithmetic(op) => self.execute_arithmetic(&op),
            // DecodedInstruction::Arithmetic(op) => self.execute(op),

            DecodedInstruction::Memory(op) => self.execute_memory(&op),
            DecodedInstruction::Control(op) => self.execute_control(&op),
        }
    }

    /// load Program
    pub fn load_program(&mut self, program: Vec<Instruction>) -> VMResult<()> {
        println!("Chargement du programme: {} instructions", program.len());

        // Réinitialiser l'état de la VM
        self.reset()?;

        // Charger les instructions dans le buffer d'instructions du pipeline
        self.pipeline.load_instructions(program.into())?;

        Ok(())
    }

    /// Run
    pub fn run(&mut self) -> VMResult<()> {
        println!("Exécution du programme...");
        let mut cycles = 0;

        while !self.pipeline.is_empty()? {
            cycles += 1;
            self.pipeline.cycle(&mut self.register_bank, &mut self.memory_controller)?;

            if cycles > 1000 { // Sécurité anti-boucle infinie
                return Err(VMError::ExecutionError("Trop de cycles d'exécution".into()));
            }

            if self.pipeline.should_halt()? {
                println!("Programme terminé par instruction HALT");
                break;
            }
        }

        Ok(())
    }


    ///read Register
    pub fn read_register(&self, reg_id: RegisterId) -> VMResult<i64> {
        let value = self.register_bank.read_register(reg_id)?;
        Ok(value as i64)
    }

    /// get Statistics
    pub fn get_statistics(&self) -> VMResult<VMStatistics> {
        Ok(VMStatistics {
            instructions_executed: self.pipeline.stats.instructions_executed,
            cycles: self.pipeline.stats.cycles,
            cache_hits: self.memory_controller.get_cache_stats()?.hits,
            pipeline_stalls: self.pipeline.stats.stalls,
        })
    }

    // Ajouter une méthode pour obtenir les statistiques détaillées
    pub fn get_detailed_stats(&self) -> VMResult<DetailedStats> {
        Ok(DetailedStats {
            pipeline_stats: self.pipeline.stats.clone(),
            cache_stats: self.memory_controller.get_cache_stats()?,
        })
    }




}

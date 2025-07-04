//src/pvm/vm.rs
use std::path::Path;

use crate::alu::alu::ALU;
use crate::alu::agu::AGUStats;
use crate::bytecode::files::SegmentType::{Code, Data, ReadOnlyData};
use crate::debug::{PipelineTracer, TracerConfig};
use crate::pipeline::Pipeline;
use crate::pvm::memorys::{Memory, MemoryConfig};
use crate::pvm::vm_errors::{VMError, VMResult};
use crate::BytecodeFile;
use crate::pipeline::ras::RASStats;
use crate::pvm::stacks::StackStats;




/// Configuration de la machine virtuelle
#[derive(Debug, Clone, Copy)]
pub struct VMConfig {
    pub memory_size: usize,            // Taille de la mémoire
    pub num_registers: usize,          // Nombre de registres
    pub l1_cache_size: usize,          // Taille du cache L1
    pub l2_cache_size: usize,          // Taille du cache L2
    pub store_buffer_size: usize,      // Taille du buffer de stockage
    pub stack_size: usize,             // Taille de la pile
    pub stack_base: u32,               // Base de la pile
    pub fetch_buffer_size: usize,      // Taille du buffer de fetch

    pub btb_size: usize,               // Taille du BTB (Branch Target Buffer)
    pub ras_size: usize,               // Taille du RAS (Return Address Stack)

    pub enable_forwarding: bool,       // Active ou désactive le forwarding
    pub enable_hazard_detection: bool, // Active ou désactive la détection de hazards
    pub enable_tracing: bool,          // Active ou désactive le traçage
}

impl Default for VMConfig {
    fn default() -> Self {
        Self {
            memory_size: 1024 * 1024, // 1MB
            num_registers: 19, // 16 general + SP(16) + BP(17) + RA(18)
            l1_cache_size: 64 * 1024, // 64KB
            l2_cache_size: 256 * 1024, // 256KB
            store_buffer_size: 8,
            stack_size: 64 * 1024, // 64KB
            stack_base: 0xFF000000,
            fetch_buffer_size: 16,
            btb_size: 64,
            ras_size: 8,
            enable_forwarding: true,
            enable_hazard_detection: true,
            enable_tracing: true,
        }
    }
}

///Etat de la machine virtuelle
#[derive(Debug, PartialEq, Eq)]
pub enum VMState {
    Ready,
    Running,
    Halted,
    Error(String),
}
/// Statistiques d'exécution de la VM
#[derive(Debug, Clone, Copy)]
pub struct VMStats {
    pub cycles: u64,                 // Nombre total de cycles exécutés
    pub instructions_executed: u64,  // Nombre total d'instructions exécutées
    pub ipc: f64,                    // Instructions par cycle (IPC)
    pub stalls: u64,                 // Nombre de stalls dans le pipeline
    pub hazards: u64,                // Nombre de hazards détectés (vrais hazards causant des stalls)
    pub data_dependencies: u64,      // Nombre de dépendances de données détectées
    pub forwards: u64,               // Nombre de forwards effectués
    pub potential_forwards: u64,     // Nombre de forwards potentiels détectés
    
    // Statistiques Store-Load forwarding
    pub store_load_forwards: u64,    // Nombre de Store-Load forwards effectués
    pub store_load_attempts: u64,    // Nombre de tentatives de Store-Load forwarding
    
    // Statistiques hiérarchie de cache
    pub l1_data_hits: u64,          // Nombre de hits dans le cache L1 data
    pub l1_data_misses: u64,        // Nombre de misses dans le cache L1 data
    pub l1_inst_hits: u64,          // Nombre de hits dans le cache L1 instruction
    pub l1_inst_misses: u64,        // Nombre de misses dans le cache L1 instruction
    pub l2_hits: u64,               // Nombre de hits dans le cache L2
    pub l2_misses: u64,             // Nombre de misses dans le cache L2
    pub l2_writebacks: u64,         // Nombre de write-backs L2
    pub l2_prefetch_hits: u64,      // Nombre de hits de prefetch
    pub memory_accesses: u64,       // Nombre d'accès à la mémoire principale
    pub average_memory_latency: f64, // Latence moyenne mémoire
    pub branch_flush: u64,           // Nombre de flushes de branchements
    pub branch_predictor: u64,       // Nombre de prédictions de branchements
    pub branch_prediction_rate: f64, // Taux de prédiction de branchements
    
    // Statistiques BTB (Branch Target Buffer)
    pub btb_hits: u64,               // Nombre de hits dans le BTB
    pub btb_misses: u64,             // Nombre de misses dans le BTB
    pub btb_hit_rate: f64,           // Taux de hits du BTB
    pub btb_correct_targets: u64,    // Nombre de cibles correctes prédites par le BTB
    pub btb_incorrect_targets: u64,  // Nombre de cibles incorrectes prédites par le BTB
    pub btb_accuracy: f64,           // Précision du BTB
    
    // Statistiques de la pile Stack
    pub stack_pushes: u64,           // Nombre de pushs dans la pile
    pub stack_pops: u64,             // Nombre de pops dans la pile
    pub stack_hits: u64,             // Nombre de hits dans la pile
    pub stack_misses: u64,           // Nombre de misses dans la pile
    pub stack_accuracy: f64, // Précision de la pile
    pub stack_current_depth: usize,  // Profondeur actuelle de la pile
    pub stack_max_depth: usize,      // Profondeur maximale de la pile

    // Statistiques SIMD
    pub simd128_ops: u64,            // Nombre d'opérations SIMD 128-bit
    pub simd256_ops: u64,            // Nombre d'opérations SIMD 256-bit
    pub simd_total_cycles: u64,      // Total de cycles SIMD
    pub simd_ops_per_cycle: f64,     // Opérations SIMD par cycle
    pub simd_parallel_ops: u64,      // Opérations SIMD parallélisées
    
    // Cache d'opérations SIMD
    pub simd_cache_hits: u64,        // Hits dans le cache d'opérations SIMD
    pub simd_cache_misses: u64,      // Misses dans le cache d'opérations SIMD
    pub simd_cache_hit_rate: f64,    // Taux de réussite du cache SIMD
    
    // Statistiques AGU (Address Generation Unit)
    pub agu_total_calculations: u64,     // Nombre total de calculs d'adresse AGU
    pub agu_early_resolutions: u64,      // Résolutions d'adresse anticipées
    pub agu_stride_predictions_correct: u64, // Prédictions de stride correctes
    pub agu_stride_predictions_total: u64,   // Total des prédictions de stride
    pub agu_stride_accuracy: f64,        // Précision du stride predictor
    pub agu_base_cache_hits: u64,       // Hits dans le cache d'adresses de base
    pub agu_base_cache_misses: u64,     // Misses dans le cache d'adresses de base
    pub agu_base_cache_hit_rate: f64,   // Taux de réussite du cache de base
    pub agu_parallel_executions: u64,   // Exécutions parallèles AGU/ALU
    pub agu_average_latency: f64,       // Latence moyenne des calculs AGU
    
    // Statistiques Dual-Issue Controller
    pub dual_issue_parallel_executions: u64,  // Nombre d'exécutions parallèles dual-issue
    pub dual_issue_total_instructions: u64,   // Total d'instructions traitées par dual-issue
    pub dual_issue_alu_only: u64,            // Instructions exécutées uniquement sur ALU
    pub dual_issue_agu_only: u64,            // Instructions exécutées uniquement sur AGU
    pub dual_issue_resource_conflicts: u64,   // Conflits de ressources détectés
    pub dual_issue_parallel_rate: f64,       // Taux d'exécution parallèle (%)

    // Statistiques Parallel Execution Engine
    pub parallel_engine_total_instructions: u64,   // Instructions totales traitées par le moteur parallèle
    pub parallel_engine_parallel_executions: u64,  // Exécutions parallèles réelles
    pub parallel_engine_alu_instructions: u64,     // Instructions ALU exécutées
    pub parallel_engine_agu_instructions: u64,     // Instructions AGU exécutées  
    pub parallel_engine_simd_instructions: u64,    // Instructions SIMD exécutées
    
    // Analyse des dépendances
    pub parallel_engine_raw_dependencies: u64,     // Dépendances Read After Write
    pub parallel_engine_war_dependencies: u64,     // Dépendances Write After Read
    pub parallel_engine_waw_dependencies: u64,     // Dépendances Write After Write
    pub parallel_engine_dependency_stalls: u64,    // Stalls causés par les dépendances
    pub parallel_engine_resource_conflicts: u64,   // Conflits de ressources
    
    // Utilisation des unités d'exécution
    pub parallel_engine_alu_utilization: f64,      // Utilisation de l'ALU (%)
    pub parallel_engine_agu_utilization: f64,      // Utilisation de l'AGU (%)
    pub parallel_engine_average_queue_depth: f64,  // Profondeur moyenne des queues
    pub parallel_engine_parallel_rate: f64,        // Taux d'exécution parallèle (%)

}

/// Machine virtuelle PunkVM
pub struct PunkVM {
    pub config: VMConfig,
    pub state: VMState,
    pipeline: Pipeline,
    alu: ALU,
    pub memory: Memory,
    pub pc: usize,                     // Compteur de programme
    pub registers: Vec<u64>,           // Registres
    pub program: Option<BytecodeFile>, // Programme
    cycles: u64,                       // Nombre de cycles
    instructions_executed: u64,        // Nombre d'instructions exécutées
    pub tracer: Option<PipelineTracer>,    // Tracer pour le débogage
    pub stack_stats: StackStats,       // Statistiques de la pile

}

impl PunkVM {
    /// Crée une nouvelle instance de PunkVM avec la configuration par défaut
    pub fn new() -> Self {
        Self::with_config(VMConfig::default())
    }

    // Crée une nouvelle instance de PunkVM avec une configuration personnalisée
    pub fn with_config(config: VMConfig) -> Self {
        let memory_config = MemoryConfig {
            size: config.memory_size,
            l1_cache_size: config.l1_cache_size,
            l2_cache_size: config.l2_cache_size,
            store_buffer_size: config.store_buffer_size,
        };

        Self {
            config, // Pas besoin de cloner, car VMConfig implémente Copy
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
            tracer: None, // Pas de traçage par défaut
            stack_stats: StackStats::new(), // Initialiser les statistiques de pile
        }
    }

    // Active le traçage
    pub fn enable_tracing(&mut self, config: TracerConfig) {
        if self.config.enable_tracing {
            println!("Tracing is enabled");
            // self.tracer = Some(PipelineTracer::new(config));
            // self.tracer = Some(PipelineTracer::new(Default::default()));
            self.tracer = Some(PipelineTracer::new(config));
        } else {
            println!("Tracing is disabled");
        }
    }

    /// Charge un programme depuis un fichier bytecode
    pub fn load_program<P: AsRef<Path>>(&mut self, path: P) -> VMResult<()> /*io::Result<()>*/ {
        // let program = BytecodeFile::read_from_file(path)?;
        // self.load_program_from_bytecode(program)
        let program = BytecodeFile::read_from_file(path).map_err(|io_err| {
            VMError::execution_error(&format!(
                "Impossible de lire le fichier bytecode: {}",
                io_err
            ))
        })?;
        self.load_program_from_bytecode(program)
    }

    /// Charge un programme depuis une structure BytecodeFile
    pub fn load_program_from_bytecode(&mut self, program: BytecodeFile) -> VMResult<()> {
        // Réinitialiser l'état de la VM avant de charger
        self.reset();

        // Charger le code en mémoire
        self.load_code_segment(&program)?;

        // Charger les segments de données
        self.load_data_segments(&program)?;

        // Stocker le BytecodeFile
        self.program = Some(program);

        // Mettre l'état en Ready
        self.state = VMState::Ready;
        Ok(())
    }

    /// Exécute le programme chargé jusqu'à la fin ou jusqu'à une erreur
    pub fn run(&mut self) -> VMResult<()> {
        if self.program.is_none() {
            return Err(VMError::execution_error("Aucun programme chargé"));
        }

        self.state = VMState::Running;

        loop {
            if self.state != VMState::Running {
                break;
            }

            // Capturer et logguer toute erreur du pipeline
            let pipeline_result = self.pipeline.cycle(
                self.pc as u32,
                &mut self.registers,
                &mut self.memory,
                &mut self.alu,
                &self.program.as_ref().unwrap().code,
            );

            match pipeline_result {
                Ok(pipeline_state) => {
                    self.pc = pipeline_state.next_pc as usize;
                    self.cycles += 1;
                    self.instructions_executed += pipeline_state.instructions_completed as u64;

                    // Si le pipeline signale qu'il est halted => on arrête
                    if pipeline_state.halted {
                        self.state = VMState::Halted;
                        break;
                    }
                }
                Err(err) => {
                    // Si l'erreur est due à HALT, convertir en VMState::Halted
                    if self.state == VMState::Halted {
                        break;
                    } else {
                        // Sinon propager l'erreur
                        let vm_err = VMError::execution_error(&format!("Erreur pipeline: {}", err));
                        self.state = VMState::Error(vm_err.to_string());
                        return Err(vm_err);
                    }
                }
            }
        }

        // Si on sort de la boucle, c'est soit Halted, soit Error
        match &self.state {
            VMState::Halted => Ok(()),
            VMState::Error(msg) => Err(VMError::execution_error(msg)),
            _ => Ok(()),
        }
    }

    /// Exécute un seul cycle du pipeline
    pub fn step(&mut self) -> VMResult<()> {
        if self.state != VMState::Running {
            return Err(VMError::execution_error(
                "La VM n'est pas en cours d'exécution",
            ));
        }

        // Ici, on va commencer à tracer l'état du pipeline
        // Mise à jour du compteur de cycles du traceur avant l'exécution
        if let Some(tracer) = &mut self.tracer{
            tracer.start_cycle(self.cycles);
        }

        // Exécution d'un cycle pipeline
        let program_code = &self.program.as_ref().unwrap().code;
        let pipeline_state = self
            .pipeline
            .cycle(
                self.pc as u32,
                &mut self.registers,
                &mut self.memory,
                &mut self.alu,
                program_code,
            )
            .map_err(|pipe_err| {
                VMError::execution_error(&format!("Erreur pipeline: {}", pipe_err))
            })?;


        // Ici on va commencer à tracer l'état du pipeline
        // Tracage de l'état du pipeline
        if let Some(tracer) = &mut self.tracer {
            tracer.trace_pipeline_state(&pipeline_state, &self.registers);
        }

        // Mise à jour du PC
        self.pc = pipeline_state.next_pc as usize;

        // Mise à jour compteurs
        self.cycles += 1;
        self.instructions_executed += pipeline_state.instructions_completed as u64;


        // Vérifier s'il y a un halt
        if pipeline_state.halted {
            self.state = VMState::Halted;

            //genere un rapport de synthese si le trace est active
            if let Some(tracer) = &self.tracer {
                println!("\n{}", tracer.generate_summary())
            }
        }

        Ok(())
    }

    // /// Réinitialise la machine virtuelle
    pub fn reset(&mut self) {
        println!("PunkVM::reset() - début");
        self.pc = 0;
        self.registers = vec![0; self.config.num_registers];
        self.cycles = 0;
        self.instructions_executed = 0;
        self.state = VMState::Ready;
        self.pipeline.reset();

        self.memory.reset();
        self.stack_stats.reset();
        
        // Initialiser automatiquement la stack
        self.init_stack();
        
        println!("Fin de Reinitialisation");
    }

    /// Retourne les statistiques d'exécution
    pub fn stats(&self) -> VMStats {
        // let ras_stats = self.ras.get_ras_stats();
        let (mem_pushes, mem_pops, mem_overflow, mem_underflow) = self.pipeline.get_memory_stack_stats();
        let (btb_hits, btb_misses, btb_hit_rate, btb_correct_targets, btb_incorrect_targets, btb_accuracy) = self.pipeline.get_btb_stats();

        VMStats {
            cycles: self.cycles,
            instructions_executed: self.instructions_executed,
            ipc: if self.cycles > 0 {
                self.instructions_executed as f64 / self.cycles as f64
            } else {
                0.0
            },
            stalls: self.pipeline.stats().stalls,
            // hazards: self.pipeline.stats().hazards,
            hazards: self.pipeline.hazard_detection.get_hazards_count(),
            data_dependencies: self.pipeline.hazard_detection.get_data_dependencies_count(),
            // forwards: self.pipeline.stats().forwards,
            forwards: self.pipeline.forwarding.get_forwards_count(),
            potential_forwards: self.pipeline.hazard_detection.get_potential_forwards_count(),
            
            // Store-Load forwarding statistics
            store_load_forwards: self.pipeline.stats().store_load_forwards,
            store_load_attempts: self.pipeline.stats().store_load_attempts,
            
            l1_data_hits: self.memory.stats().l1_hits,
            l1_data_misses: self.memory.stats().l1_misses,
            l1_inst_hits: 0, // Pour l'instant, on track seulement data
            l1_inst_misses: 0,
            l2_hits: self.memory.stats().l2_hits,
            l2_misses: self.memory.stats().l2_misses,
            l2_writebacks: 0, // À implémenter plus tard
            l2_prefetch_hits: 0, // À implémenter plus tard
            memory_accesses: self.memory.stats().l1_misses + self.memory.stats().l2_misses,
            average_memory_latency: 0.0, // À calculer plus tard
            branch_flush: self.pipeline.stats().branch_flush,
            branch_predictor: self.pipeline.stats().branch_predictions,
            branch_prediction_rate: self.pipeline.stats().branch_predictor_rate,
            
            // Statistiques BTB
            btb_hits,
            btb_misses,
            btb_hit_rate,
            btb_correct_targets,
            btb_incorrect_targets,
            btb_accuracy,

            stack_pushes: mem_pushes,
            stack_pops: mem_pops,
            stack_hits: 0, // Pour l'instant, pas de notion de hits/misses pour la pile principale
            stack_misses: mem_overflow + mem_underflow,
            stack_accuracy: 0.0,
            stack_current_depth: self.stack_stats.current_depth,
            stack_max_depth: self.stack_stats.max_depth,

            // Statistiques SIMD récupérées du VectorALU
            simd128_ops: self.get_vector_alu().borrow().get_simd_stats().simd128_ops,
            simd256_ops: self.get_vector_alu().borrow().get_simd_stats().simd256_ops,
            simd_total_cycles: self.get_vector_alu().borrow().get_simd_stats().total_simd_cycles,
            simd_ops_per_cycle: self.get_vector_alu().borrow().get_simd_stats().simd_ops_per_cycle,
            simd_parallel_ops: self.get_vector_alu().borrow().get_simd_stats().parallel_ops,
            
            // Statistiques du cache d'opérations SIMD
            simd_cache_hits: self.get_vector_alu().borrow().get_cache_stats().0,
            simd_cache_misses: self.get_vector_alu().borrow().get_cache_stats().1,
            simd_cache_hit_rate: self.get_vector_alu().borrow().get_cache_stats().2,
            
            // Statistiques AGU récupérées de l'ExecuteStage
            agu_total_calculations: self.get_agu_stats().total_calculations,
            agu_early_resolutions: self.get_agu_stats().early_resolutions,
            agu_stride_predictions_correct: self.get_agu_stats().stride_predictions_correct,
            agu_stride_predictions_total: self.get_agu_stats().stride_predictions_total,
            agu_stride_accuracy: if self.get_agu_stats().stride_predictions_total > 0 {
                self.get_agu_stats().stride_predictions_correct as f64 / self.get_agu_stats().stride_predictions_total as f64
            } else { 0.0 },
            agu_base_cache_hits: self.get_agu_stats().base_cache_hits,
            agu_base_cache_misses: self.get_agu_stats().base_cache_misses,
            agu_base_cache_hit_rate: if (self.get_agu_stats().base_cache_hits + self.get_agu_stats().base_cache_misses) > 0 {
                self.get_agu_stats().base_cache_hits as f64 / (self.get_agu_stats().base_cache_hits + self.get_agu_stats().base_cache_misses) as f64
            } else { 0.0 },
            agu_parallel_executions: self.get_agu_stats().parallel_executions,
            agu_average_latency: self.get_agu_stats().average_latency,
            
            // Statistiques Dual-Issue Controller
            dual_issue_parallel_executions: {
                let dual_stats = self.get_dual_issue_stats();
                dual_stats.0
            },
            dual_issue_total_instructions: {
                let dual_stats = self.get_dual_issue_stats();
                dual_stats.1
            },
            dual_issue_alu_only: {
                let dual_stats = self.get_dual_issue_stats();
                dual_stats.2
            },
            dual_issue_agu_only: {
                let dual_stats = self.get_dual_issue_stats();
                dual_stats.3
            },
            dual_issue_resource_conflicts: {
                let dual_stats = self.get_dual_issue_stats();
                dual_stats.4
            },
            dual_issue_parallel_rate: {
                let dual_stats = self.get_dual_issue_stats();
                dual_stats.5
            },
            
            // Statistiques Parallel Execution Engine
            parallel_engine_total_instructions: {
                let parallel_stats = self.get_parallel_engine_stats();
                parallel_stats.total_instructions
            },
            parallel_engine_parallel_executions: {
                let parallel_stats = self.get_parallel_engine_stats();
                parallel_stats.parallel_executions
            },
            parallel_engine_alu_instructions: {
                let parallel_stats = self.get_parallel_engine_stats();
                parallel_stats.alu_instructions
            },
            parallel_engine_agu_instructions: {
                let parallel_stats = self.get_parallel_engine_stats();
                parallel_stats.agu_instructions
            },
            parallel_engine_simd_instructions: {
                let parallel_stats = self.get_parallel_engine_stats();
                parallel_stats.simd_instructions
            },
            parallel_engine_raw_dependencies: {
                let parallel_stats = self.get_parallel_engine_stats();
                parallel_stats.raw_dependencies
            },
            parallel_engine_war_dependencies: {
                let parallel_stats = self.get_parallel_engine_stats();
                parallel_stats.war_dependencies
            },
            parallel_engine_waw_dependencies: {
                let parallel_stats = self.get_parallel_engine_stats();
                parallel_stats.waw_dependencies
            },
            parallel_engine_dependency_stalls: {
                let parallel_stats = self.get_parallel_engine_stats();
                parallel_stats.dependency_stalls
            },
            parallel_engine_resource_conflicts: {
                let parallel_stats = self.get_parallel_engine_stats();
                parallel_stats.resource_conflicts
            },
            parallel_engine_alu_utilization: {
                let parallel_stats = self.get_parallel_engine_stats();
                parallel_stats.alu_utilization
            },
            parallel_engine_agu_utilization: {
                let parallel_stats = self.get_parallel_engine_stats();
                parallel_stats.agu_utilization
            },
            parallel_engine_average_queue_depth: {
                let parallel_stats = self.get_parallel_engine_stats();
                parallel_stats.average_queue_depth
            },
            parallel_engine_parallel_rate: {
                let parallel_stats = self.get_parallel_engine_stats();
                if parallel_stats.total_instructions > 0 {
                    (parallel_stats.parallel_executions as f64 / parallel_stats.total_instructions as f64) * 100.0
                } else {
                    0.0
                }
            },
        }
    }


    pub fn get_ras_stats(&self) -> RASStats {
        self.get_ras_stats()
    }

    /// Retourne une référence au VectorALU pour accéder aux registres vectoriels
    pub fn get_vector_alu(&self) -> &std::rc::Rc<std::cell::RefCell<crate::alu::v_alu::VectorALU>> {
        self.pipeline.get_execute_stage().get_vector_alu_ref()
    }

    // get_vector_alu_mut supprimée - utiliser get_vector_alu().borrow_mut() à la place

    /// Retourne les statistiques de l'AGU
    pub fn get_agu_stats(&self) -> AGUStats {
        self.pipeline.get_execute_stage().get_agu_stats()
    }
    
    /// Retourne les statistiques du dual-issue controller
    pub fn get_dual_issue_stats(&self) -> (u64, u64, u64, u64, u64, f64) {
        self.pipeline.get_execute_stage().get_dual_issue_stats()
    }
    
    /// Retourne les statistiques du parallel execution engine
    pub fn get_parallel_engine_stats(&self) -> &crate::pipeline::parallel::ParallelExecutionStats {
        self.pipeline.get_execute_stage().get_parallel_engine_stats()
    }

    /// Retourne l'état actuel de la VM
    pub fn state(&self) -> &VMState {
        &self.state
    }

    /// Charge le segment de code en mémoire
    fn load_code_segment(&mut self, program: &BytecodeFile) -> VMResult<()> {
        let code_segment = program
            .segments
            .iter()
            .find(|s| s.segment_type == Code)
            .ok_or_else(|| VMError::memory_error("Segment de code manquant"))?;

        // Encoder les instructions
        let mut code_bytes = Vec::new();
        for instr in &program.code {
            code_bytes.extend_from_slice(&instr.encode());
        }

        // Vérifier la cohérence de taille
        if code_bytes.len() != code_segment.size as usize {
            return Err(VMError::memory_error(&format!(
                "Taille du code incohérente: segment={}, encodé={}",
                code_segment.size,
                code_bytes.len(),
            )));
        }

        // Écrire en mémoire à l'adresse spécifiée
        self.memory
            .write_block(code_segment.load_addr, &code_bytes)
            .map_err(|_| VMError::memory_error("Échec d'écriture du code en mémoire"))?;

        Ok(())
    }

    /// Charge les segments de données en mémoire
    /// Charge les segments de données en mémoire (Data et ReadOnlyData)
    fn load_data_segments(&mut self, program: &BytecodeFile) -> VMResult<()> {
        // Data
        if let Some(data_seg) = program.segments.iter().find(|s| s.segment_type == Data) {
            if data_seg.size > 0 {
                self.memory
                    .write_block(data_seg.load_addr, &program.data)
                    .map_err(|_| VMError::memory_error("Échec d'écriture du segment Data"))?;
            }
        }

        // Read-Only Data
        if let Some(ro_seg) = program
            .segments
            .iter()
            .find(|s| s.segment_type == ReadOnlyData)
        {
            if ro_seg.size > 0 {
                self.memory
                    .write_block(ro_seg.load_addr, &program.readonly_data)
                    .map_err(|_| {
                        VMError::memory_error("Échec d'écriture du segment ReadOnlyData")
                    })?;
            }
        }

        Ok(())
    }

    // Exporter les traces dans un  fichier CSV
    pub fn export_traces_to_csv(&self, file_path: &str) -> VMResult<()> {
        if let Some(tracer) = &self.tracer {
            tracer.export_to_csv(file_path)
        } else {
            Err(VMError::execution_error("Le traçage n'est pas activé"))
        }
    }
}





//
// #[cfg(test)]
// mod tests {
//     use super::*; // Importe PunkVM, VMConfig, etc. de vm.rs
//     use crate::bytecode::files::BytecodeFile;
//     use crate::bytecode::files::{SegmentMetadata, SegmentType};
//     use crate::bytecode::format::InstructionFormat;
//     use crate::bytecode::instructions::Instruction;
//     use crate::bytecode::opcodes::Opcode;
//     use crate::pvm::vm_errors::VMError;
//     use std::fs::File;
//     use std::io::Write;
//     use std::path::PathBuf;
//     use tempfile::tempdir;
//
//     // ----------------------------------------------------------------
//     //      Tests de base sur la config et la création de VM
//     // ----------------------------------------------------------------
//
//     #[test]
//     fn test_vm_config_default() {
//         let config = VMConfig::default();
//         assert_eq!(config.memory_size, 1024 * 1024);
//         assert_eq!(config.num_registers, 16);
//         assert_eq!(config.l1_cache_size, 4 * 1024);
//         assert!(config.enable_forwarding);
//         assert!(config.enable_hazard_detection);
//     }
//
//     #[test]
//     fn test_vm_creation() {
//         let vm = PunkVM::new();
//         assert_eq!(*vm.state(), VMState::Ready);
//         assert_eq!(vm.pc, 0);
//         assert_eq!(vm.registers.len(), 16);
//
//         // Test avec une config perso
//         let config = VMConfig {
//             num_registers: 32,
//             ..VMConfig::default()
//         };
//         let vm2 = PunkVM::with_config(config);
//         assert_eq!(vm2.registers.len(), 32);
//     }
//
//     #[test]
//     fn test_vm_stats_initial() {
//         let vm = PunkVM::new();
//         let stats = vm.stats();
//
//         // Vérifier les statistiques initiales
//         assert_eq!(stats.cycles, 0);
//         assert_eq!(stats.instructions_executed, 0);
//         assert_eq!(stats.ipc, 0.0);
//         assert_eq!(stats.stalls, 0);
//         assert_eq!(stats.hazards, 0);
//         assert_eq!(stats.forwards, 0);
//         assert_eq!(stats.memory_hits, 0);
//         assert_eq!(stats.memory_misses, 0);
//     }
//
//     // ----------------------------------------------------------------
//     //      Tests autour du chargement de programme
//     // ----------------------------------------------------------------
//
//     #[test]
//     fn test_vm_load_program_no_file() {
//         let mut vm = PunkVM::new();
//
//         // Charger un fichier inexistant => doit échouer
//         let result = vm.load_program("nonexistent_file.punk");
//         assert!(result.is_err());
//
//         // Vérifier que l'erreur renvoyée correspond à une ExecutionError, par exemple
//         if let Err(e) = result {
//             match e {
//                 VMError::ExecutionError(msg) => {
//                     // On peut vérifier le contenu du message si besoin
//                     assert!(msg.contains("Impossible de lire le fichier bytecode"));
//                 }
//                 _ => panic!("Attendu VMError::ExecutionError, obtenu: {:?}", e),
//             }
//         }
//     }
//
//     #[test]
//     fn test_vm_load_program_from_empty_bytecode() {
//         // Programme totalement vide
//         let program = BytecodeFile::new();
//         let mut vm = PunkVM::new();
//         let res = vm.load_program_from_bytecode(program);
//
//         // Ça peut réussir ou échouer selon ta logique de validation,
//         // mais au moins on veut être sûr que ça ne panique pas.
//         assert!(res.is_ok() || res.is_err());
//     }
//
//     // Modification du test pour créer correctement un programme bytecode minimal (HALT)
//     #[test]
//     fn test_vm_load_program_from_bytecode_halt() {
//         // Créer un programme bytecode minimal
//         let mut program = BytecodeFile::new();
//
//         // Ajouter instruction HALT
//         let halt_instr = Instruction::create_no_args(Opcode::Halt);
//         let encoded_size = halt_instr.total_size() as u32;
//         program.add_instruction(halt_instr);
//
//         // Créer le segment CODE
//         program.segments = vec![SegmentMetadata::new(SegmentType::Code, 0, encoded_size, 0)];
//
//         // Charger le programme
//         let mut vm = PunkVM::new();
//         let result = vm.load_program_from_bytecode(program);
//         assert!(
//             result.is_ok(),
//             "Chargement d'un programme minimal 'Halt' doit réussir"
//         );
//     }
//
//     // ----------------------------------------------------------------
//     //      Tests d'exécution
//     // ----------------------------------------------------------------
//
//     #[test]
//     fn test_vm_run_no_program() {
//         let mut vm = PunkVM::new();
//         let result = vm.run();
//         // Doit renvoyer Err car pas de programme
//         assert!(result.is_err());
//         if let Err(e) = result {
//             match e {
//                 VMError::ExecutionError(msg) => {
//                     assert!(msg.contains("Aucun programme chargé"));
//                 }
//                 _ => panic!("Attendu ExecutionError, obtenu: {:?}", e),
//             }
//         }
//     }

    // #[test]
    // fn test_vm_step_not_running() {
    //     let mut vm = PunkVM::new();
    //     // Appeler step() alors que la VM est en state=Ready => erreur
    //     let result = vm.step();
    //     assert!(result.is_err());
    //     if let Err(e) = result {
    //         match e {
    //             VMError::ExecutionError(msg) => {
    //                 assert!(msg.contains("n'est pas en cours d'exécution"));
    //             }
    //             _ => panic!("Attendu ExecutionError, obtenu: {:?}", e),
    //         }
    //     }
    // }

    // Petit test complet : on crée un programme HALT, on l’exécute, puis on regarde le state=Halted
    // #[test]
    // fn test_vm_run_halt_program() {
    //     // Construire un programme minimal contenant un HALT
    //     let mut program = BytecodeFile::new();
    //     let halt_instr = Instruction::create_no_args(Opcode::Halt);
    //     let encoded_size = halt_instr.total_size() as u32;
    //     assert_eq!(encoded_size, 4, "La taille de l'instruction HALT doit être de 4 octets");
    //     program.add_instruction(halt_instr);
    //
    //     // Segment code
    //     program.segments = vec![SegmentMetadata::new(Code, 0, encoded_size , 0)];
    //
    //     // Charger et exécuter
    //     let mut vm = PunkVM::new();
    //     vm.load_program_from_bytecode(program).unwrap();
    //     let result = vm.run();
    //
    //     // On s'attend à OK, et state=Halted
    //     assert!(result.is_ok());
    //     assert_eq!(*vm.state(), VMState::Halted);
    //
    //     // Vérifier stats
    //     // let stats = vm.stats();
    //     // assert_eq!(stats.cycles, 2, "Une instruction = un cycle (selon ton pipeline ?)");
    //     // assert_eq!(stats.instructions_executed, 1, "Halt est 1 instruction exécutée");
    // }

    // ----------------------------------------------------------------
    //      Tests de reset
    // ----------------------------------------------------------------
    //
    // #[test]
    // fn test_vm_reset_minimal() {
    //     let mut vm = PunkVM::new();
    //     vm.reset();
    //     // Si aucune panique, test OK
    //     assert_eq!(*vm.state(), VMState::Ready);
    //     assert_eq!(vm.pc, 0);
    //     assert_eq!(vm.cycles, 0);
    //     assert_eq!(vm.instructions_executed, 0);
    // }
    //
    // #[test]
    // fn test_vm_reset_apres_modification() {
    //     let mut vm = PunkVM::new();
    //
    //     // Modifier quelques champs
    //     vm.pc = 123;
    //     vm.registers[0] = 42;
    //     vm.cycles = 10;
    //     vm.instructions_executed = 5;
    //
    //     // reset
    //     vm.reset();
    //
    //     // Vérifier qu'on est bien revenu à 0
    //     assert_eq!(vm.pc, 0);
    //     assert_eq!(vm.registers[0], 0);
    //     assert_eq!(vm.cycles, 0);
    //     assert_eq!(vm.instructions_executed, 0);
    //     assert_eq!(*vm.state(), VMState::Ready);
    // }

    // ----------------------------------------------------------------
    //      Tests sur l'écriture de fichier program (facultatifs)
    // ----------------------------------------------------------------

//     // Petite fonction utilitaire pour créer un fichier .punk minimal
//     fn create_test_program_file() -> (PathBuf, tempfile::TempDir) {
//         // Créer un répertoire temporaire
//         let dir = tempdir().unwrap();
//         let file_path = dir.path().join("test_program.punk");
//
//         // Créer un fichier
//         let mut file = File::create(&file_path).unwrap();
//
//         // Écrire la signature PUNK + version
//         file.write_all(&[0x50, 0x55, 0x4E, 0x4B]).unwrap(); // 'P','U','N','K'
//         file.write_all(&[0x00, 0x01, 0x00, 0x00]).unwrap(); // version 0.1.0.0
//
//         // On pourrait simuler un header minimal, etc.
//
//         file.flush().unwrap();
//         (file_path, dir)
//     }
//
//     #[test]
//     fn test_vm_load_program_from_disk() {
//         let mut vm = PunkVM::new();
//
//         let (file_path, _temp_dir) = create_test_program_file();
//
//         let result = vm.load_program(&file_path);
//         // Selon le contenu minimal, ça peut échouer ou réussir,
//         // on vérifie juste qu'on n'obtient pas de crash
//         assert!(result.is_ok() || result.is_err());
//     }
//
//     // ----------------------------------------------------------------
//     //      Tests additionnels d'intégration (facultatif)
//     // ----------------------------------------------------------------
//
//     #[test]
//     fn test_vm_execution_mock_instructions() {
//         // On va créer un petit programme :
//         //   MOV R0, #10 ; HALT
//         // (ou quelque chose d'approchant, selon ton encodeur)
//         //
//         // On simule juste : R0 = 10, puis HALT
//         let mut program = BytecodeFile::new();
//
//         // On prépare deux instructions
//         // 1) MOV => pas forcément implémenté en tant qu'opcode distinct,
//         //    on va feinter :
//         let mov_instr = Instruction::create_reg_imm8(Opcode::Mov, 0, 10);
//         let halt_instr = Instruction::create_no_args(Opcode::Halt);
//
//         // On calcule la taille totale
//         let size_mov = mov_instr.total_size();
//         let size_halt = halt_instr.total_size();
//         let total_size = (size_mov + size_halt) as u32;
//
//         program.add_instruction(mov_instr);
//         program.add_instruction(halt_instr);
//
//         // Segment code
//         program.segments = vec![SegmentMetadata::new(SegmentType::Code, 0, total_size, 0)];
//
//         // Charger + exécuter
//         let mut vm = PunkVM::new();
//         vm.load_program_from_bytecode(program).unwrap();
//         let run_res = vm.run();
//         assert!(run_res.is_ok());
//
//         // Une fois fini, on s'attend à ce que R0 = 10
//         // (si ton pipeline exécute réellement l'instruction Mov => R0=10)
//         // Sinon, tu ajusteras en fonction de ta sémantique.
//         assert_eq!(vm.registers[0], 10);
//         assert_eq!(*vm.state(), VMState::Halted);
//     }
//
//     #[test]
//     fn test_halt_size() {
//         let halt_instr = Instruction::create_no_args(Opcode::Halt);
//         let encoded_size = halt_instr.total_size() as u32; // 4
//         assert_eq!(
//             encoded_size, 4,
//             "La taille de l'instruction HALT doit être de 4 octet"
//         );
//     }
//
//     #[test]
//     fn test_vm_execution_mock_instructions_2() {
//         // On va créer un petit programme :
//         //   MOV R0, #10 ; HALT
//         let mut program = BytecodeFile::new();
//
//         // On prépare deux instructions
//         let mov_instr = Instruction::create_reg_imm8(Opcode::Mov, 0, 10);
//         let halt_instr = Instruction::create_no_args(Opcode::Halt);
//
//         // On calcule la taille totale
//         let size_mov = mov_instr.total_size();
//         let size_halt = halt_instr.total_size();
//         let total_size = (size_mov + size_halt) as u32;
//
//         program.add_instruction(mov_instr);
//         program.add_instruction(halt_instr);
//
//         // Segment code
//         program.segments = vec![SegmentMetadata::new(SegmentType::Code, 0, total_size, 0)];
//
//         // Charger + exécuter
//         let mut vm = PunkVM::new();
//         vm.load_program_from_bytecode(program).unwrap();
//
//         // Exécuter en gérant le cas où HALT déclenche une erreur contrôlée
//         let run_res = vm.run();
//
//         // Vérifier que soit l'exécution a réussi, soit l'erreur est liée au HALT
//         if run_res.is_err() {
//             let err = run_res.unwrap_err();
//             match err {
//                 VMError::ExecutionError(msg) => {
//                     // Si l'erreur contient "HALT", c'est une erreur contrôlée et acceptable
//                     // assert!(msg.contains("HALT") || msg.contains("halt"),
//                     //         "L'erreur d'exécution devrait être liée à HALT: {}", msg);
//                 }
//                 _ => panic!("Erreur inattendue: {:?}", err),
//             }
//         }
//
//         // Une fois fini, on s'attend à ce que R0 = 10 (confirmant que MOV a bien été exécuté)
//         assert_eq!(vm.registers[0], 10);
//
//         // L'état peut être soit Halted (si HALT est géré comme un signal),
//         // soit Error (si HALT est géré comme une erreur contrôlée)
//         assert!(
//             *vm.state() == VMState::Halted
//                 || matches!(*vm.state(), VMState::Error(ref msg) if msg.contains("HALT") || msg.contains("halt")),
//             "État VM attendu: Halted ou Error(HALT), obtenu: {:?}",
//             vm.state()
//         );
//     }
//
//     #[test]
//     fn test_vm_run_halt_program() {
//         // Construire un programme minimal contenant un HALT
//         let mut program = BytecodeFile::new();
//         let halt_instr = Instruction::create_no_args(Opcode::Halt);
//         let encoded_size = halt_instr.total_size() as u32;
//         assert_eq!(
//             encoded_size, 4,
//             "La taille de l'instruction HALT doit être de 4 octets"
//         );
//         program.add_instruction(halt_instr);
//
//         // Segment code
//         program.segments = vec![SegmentMetadata::new(Code, 0, encoded_size, 0)];
//
//         // Charger et exécuter
//         let mut vm = PunkVM::new();
//         vm.load_program_from_bytecode(program).unwrap();
//         let result = vm.run();
//
//         // Vérifier que soit l'exécution a réussi, soit l'erreur est liée au HALT
//         if result.is_err() {
//             let err = result.unwrap_err();
//             match err {
//                 VMError::ExecutionError(msg) => {
//                     // Si l'erreur contient "HALT", c'est une erreur contrôlée et acceptable
//                     assert!(
//                         msg.contains("HALT") || msg.contains("halt"),
//                         "L'erreur d'exécution devrait être liée à HALT: {}",
//                         msg
//                     );
//                 }
//                 _ => panic!("Erreur inattendue: {:?}", err),
//             }
//         }
//
//         // L'état peut être soit Halted (si HALT est géré comme un signal),
//         // soit Error (si HALT est géré comme une erreur contrôlée)
//         assert!(
//             *vm.state() == VMState::Halted
//                 || matches!(*vm.state(), VMState::Error(ref msg) if msg.contains("HALT") || msg.contains("halt")),
//             "État VM attendu: Halted ou Error(HALT), obtenu: {:?}",
//             vm.state()
//         );
//
//         // Vérifier stats - plus flexible par rapport au nombre de cycles nécessaires
//         let stats = vm.stats();
//         assert!(stats.cycles > 0, "Au moins un cycle devrait être exécuté");
//
//         // On peut être plus souple sur le nombre d'instructions exécutées aussi,
//         // selon la façon dont HALT est compté dans les stats
//         assert!(
//             stats.instructions_executed >= 0,
//             "Les instructions exécutées devraient être comptabilisées"
//         );
//     }
// }

// Test unitaire pour la VM

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

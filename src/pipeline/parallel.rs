//src/pipeline/parallel.rs

use std::collections::{VecDeque, HashMap};
use std::cell::RefCell;
use std::rc::Rc;

use crate::alu::alu::ALU;
use crate::alu::agu::AGU;
use crate::alu::v_alu::VectorALU;
use crate::pipeline::{DecodeExecuteRegister, ExecuteMemoryRegister};
use crate::bytecode::opcodes::Opcode;

/// Types d'unités d'exécution
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionUnit {
    ALU,        // Instructions arithmétiques/logiques
    AGU,        // Instructions d'adresse/mémoire
    Both,       // Instructions complexes nécessitant les deux
    SIMD,       // Instructions vectorielles
    FPU,        // Instructions flottantes
    Branch,     // Instructions de branchement
}

/// Priorité des instructions
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum InstructionPriority {
    High,       // Load/Store, branches
    Medium,     // Instructions arithmétiques
    Low,        // NOP, autres
}

/// Types de dépendances
#[derive(Debug, Clone, PartialEq)]
pub enum DependencyType {
    RAW, // Read After Write
    WAR, // Write After Read  
    WAW, // Write After Write
}

/// Dépendance de registre
#[derive(Debug, Clone)]
pub struct RegisterDependency {
    pub register: usize,
    pub dep_type: DependencyType,
    pub producer_age: u64,
}

/// Dépendance mémoire
#[derive(Debug, Clone)]
pub struct MemoryDependency {
    pub producer: u64,
    pub consumer: u64,
    pub address: Option<u64>,
}

/// Paquet d'exécution avec métadonnées
#[derive(Debug, Clone)]
pub struct ExecutionPacket {
    pub instruction: DecodeExecuteRegister,
    pub age: u64,
    pub priority: InstructionPriority,
    pub dependencies: Vec<RegisterDependency>,
    pub ready: bool,
}

/// Résultat en attente de completion
#[derive(Debug)]
pub struct PendingResult {
    pub result: ExecuteMemoryRegister,
    pub ready_cycle: u64,
    pub execution_unit: ExecutionUnit,
}

/// Statistiques d'exécution parallèle
#[derive(Debug, Clone, Default)]
pub struct ParallelExecutionStats {
    pub total_instructions: u64,
    pub parallel_executions: u64,
    pub alu_instructions: u64,
    pub agu_instructions: u64,
    pub simd_instructions: u64,
    
    pub dependency_stalls: u64,
    pub resource_conflicts: u64,
    
    pub alu_utilization: f64,
    pub agu_utilization: f64,
    pub average_queue_depth: f64,
    
    pub raw_dependencies: u64,
    pub war_dependencies: u64,
    pub waw_dependencies: u64,
}

/// Analyseur de dépendances entre instructions
#[derive(Debug)]
pub struct DependencyAnalyzer {
    /// Producteurs de registres (dernière instruction qui écrit)
    register_producers: HashMap<usize, u64>, // register -> instruction age
    /// Consommateurs de registres (instructions qui lisent)
    register_consumers: HashMap<usize, Vec<u64>>, // register -> [instruction ages]
    /// Dépendances mémoire
    memory_dependencies: Vec<MemoryDependency>,
    /// Compteur d'âge pour les instructions
    instruction_age_counter: u64,
}

impl DependencyAnalyzer {
    pub fn new() -> Self {
        Self {
            register_producers: HashMap::new(),
            register_consumers: HashMap::new(),
            memory_dependencies: Vec::new(),
            instruction_age_counter: 0,
        }
    }
    
    /// Analyse les dépendances pour une instruction
    pub fn analyze_instruction(&mut self, instruction: &DecodeExecuteRegister) -> Vec<RegisterDependency> {
        let mut dependencies = Vec::new();
        let current_age = self.instruction_age_counter;
        
        // Vérifier les dépendances RAW (Read After Write)
        if let Some(rs1) = instruction.rs1 {
            if let Some(&producer_age) = self.register_producers.get(&rs1) {
                dependencies.push(RegisterDependency {
                    register: rs1,
                    dep_type: DependencyType::RAW,
                    producer_age,
                });
            }
        }
        
        if let Some(rs2) = instruction.rs2 {
            if let Some(&producer_age) = self.register_producers.get(&rs2) {
                dependencies.push(RegisterDependency {
                    register: rs2,
                    dep_type: DependencyType::RAW,
                    producer_age,
                });
            }
        }
        
        // Mettre à jour les producteurs/consommateurs
        if let Some(rd) = instruction.rd {
            // WAW check
            if let Some(&producer_age) = self.register_producers.get(&rd) {
                dependencies.push(RegisterDependency {
                    register: rd,
                    dep_type: DependencyType::WAW,
                    producer_age,
                });
            }
            
            // Mettre à jour le producteur
            self.register_producers.insert(rd, current_age);
        }
        
        // Ajouter comme consommateur
        if let Some(rs1) = instruction.rs1 {
            self.register_consumers.entry(rs1).or_insert_with(Vec::new).push(current_age);
        }
        if let Some(rs2) = instruction.rs2 {
            self.register_consumers.entry(rs2).or_insert_with(Vec::new).push(current_age);
        }
        
        self.instruction_age_counter += 1;
        dependencies
    }
    
    /// Réinitialise l'analyseur
    pub fn reset(&mut self) {
        self.register_producers.clear();
        self.register_consumers.clear();
        self.memory_dependencies.clear();
        self.instruction_age_counter = 0;
    }
}

/// Moteur d'exécution parallèle pour architecture superscalaire
pub struct ParallelExecutionEngine {
    /// Files d'instructions par unité d'exécution
    alu_queue: VecDeque<ExecutionPacket>,
    agu_queue: VecDeque<ExecutionPacket>,
    simd_queue: VecDeque<ExecutionPacket>,
    
    /// État des unités d'exécution
    alu_busy: bool,
    agu_busy: bool,
    simd_busy: bool,
    
    /// Latences restantes pour chaque unité
    alu_cycles_remaining: u32,
    agu_cycles_remaining: u32,
    simd_cycles_remaining: u32,
    
    /// Résultats en attente
    pending_results: VecDeque<PendingResult>,
    
    /// Analyseur de dépendances
    dependency_analyzer: DependencyAnalyzer,
    
    /// Statistiques d'exécution
    stats: ParallelExecutionStats,
    
    /// Cycle actuel
    current_cycle: u64,
    
    /// Références aux unités d'exécution (utilisation de smart pointers)
    alu_ref: Option<Rc<RefCell<ALU>>>,
    agu_ref: Option<Rc<RefCell<AGU>>>,
    vector_alu_ref: Option<Rc<RefCell<VectorALU>>>,
}

impl ParallelExecutionEngine {
    pub fn new() -> Self {
        Self {
            alu_queue: VecDeque::new(),
            agu_queue: VecDeque::new(),
            simd_queue: VecDeque::new(),
            
            alu_busy: false,
            agu_busy: false,
            simd_busy: false,
            
            alu_cycles_remaining: 0,
            agu_cycles_remaining: 0,
            simd_cycles_remaining: 0,
            
            pending_results: VecDeque::new(),
            dependency_analyzer: DependencyAnalyzer::new(),
            stats: ParallelExecutionStats::default(),
            current_cycle: 0,
            
            alu_ref: None,
            agu_ref: None,
            vector_alu_ref: None,
        }
    }
    
    /// Configure les références aux unités d'exécution
    pub fn set_execution_units(
        &mut self,
        alu: Rc<RefCell<ALU>>,
        agu: Rc<RefCell<AGU>>,
        vector_alu: Rc<RefCell<VectorALU>>,
    ) {
        self.alu_ref = Some(alu);
        self.agu_ref = Some(agu);
        self.vector_alu_ref = Some(vector_alu);
    }
    
    /// Avance d'un cycle et met à jour l'état des unités
    pub fn advance_cycle(&mut self) {
        self.current_cycle += 1;
        
        // Décrémenter les latences
        if self.alu_cycles_remaining > 0 {
            self.alu_cycles_remaining -= 1;
            if self.alu_cycles_remaining == 0 {
                self.alu_busy = false;
            }
        }
        
        if self.agu_cycles_remaining > 0 {
            self.agu_cycles_remaining -= 1;
            if self.agu_cycles_remaining == 0 {
                self.agu_busy = false;
            }
        }
        
        if self.simd_cycles_remaining > 0 {
            self.simd_cycles_remaining -= 1;
            if self.simd_cycles_remaining == 0 {
                self.simd_busy = false;
            }
        }
    }
    
    /// Analyse une instruction pour déterminer son unité d'exécution et priorité
    pub fn analyze_instruction(instruction: &DecodeExecuteRegister) -> (ExecutionUnit, InstructionPriority) {
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
            
            // Instructions SIMD arithmétiques - SIMD priorité moyenne  
            Opcode::Simd128Add | Opcode::Simd128Sub | Opcode::Simd128Mul | Opcode::Simd128Div |
            Opcode::Simd128And | Opcode::Simd128Or | Opcode::Simd128Xor | Opcode::Simd128Not |
            Opcode::Simd256Add | Opcode::Simd256Sub | Opcode::Simd256Mul | Opcode::Simd256Div => {
                (ExecutionUnit::SIMD, InstructionPriority::Medium)
            }
            
            // Instructions de branchement - ALU haute priorité
            Opcode::Jmp | Opcode::JmpIfEqual | Opcode::JmpIfNotEqual | 
            Opcode::JmpIfGreater | Opcode::JmpIfLess | Opcode::JmpIfGreaterEqual | 
            Opcode::JmpIfLessEqual | Opcode::JmpIfZero | Opcode::JmpIfNotZero => {
                (ExecutionUnit::Branch, InstructionPriority::High)
            }
            
            // Instructions de pile - AGU priorité moyenne
            Opcode::Push | Opcode::Pop | Opcode::Call | Opcode::Ret => {
                (ExecutionUnit::AGU, InstructionPriority::Medium)
            }
            
            // Autres instructions - ALU priorité basse
            _ => (ExecutionUnit::ALU, InstructionPriority::Low)
        }
    }
    
    /// Ajoute une instruction dans la queue appropriée
    pub fn enqueue_instruction(
        &mut self,
        instruction: DecodeExecuteRegister,
        exec_unit: ExecutionUnit,
        priority: InstructionPriority,
    ) -> Result<(), String> {
        // Analyser les dépendances
        let dependencies = self.dependency_analyzer.analyze_instruction(&instruction);
        
        let packet = ExecutionPacket {
            instruction,
            age: self.dependency_analyzer.instruction_age_counter,
            priority,
            dependencies,
            ready: false, // Sera mis à jour par check_dependencies
        };
        
        // Ajouter à la queue appropriée
        match exec_unit {
            ExecutionUnit::ALU | ExecutionUnit::Branch | ExecutionUnit::FPU => {
                self.alu_queue.push_back(packet);
            }
            ExecutionUnit::AGU => {
                self.agu_queue.push_back(packet);
            }
            ExecutionUnit::SIMD => {
                self.simd_queue.push_back(packet);
            }
            ExecutionUnit::Both => {
                // Pour Both, utiliser ALU par défaut
                self.alu_queue.push_back(packet);
            }
        }
        
        self.stats.total_instructions += 1;
        Ok(())
    }



    
    /// Vérifie et met à jour l'état ready des instructions
    pub fn update_ready_status(&mut self) {
        // Mettre à jour ALU queue
        for packet in &mut self.alu_queue {
            packet.ready = Self::check_dependencies_resolved(&packet.dependencies);
        }
        
        // Mettre à jour AGU queue
        for packet in &mut self.agu_queue {
            packet.ready = Self::check_dependencies_resolved(&packet.dependencies);
        }
        
        // Mettre à jour SIMD queue
        for packet in &mut self.simd_queue {
            packet.ready = Self::check_dependencies_resolved(&packet.dependencies);
        }
    }
    
    /// Vérifie si toutes les dépendances sont résolues
    fn check_dependencies_resolved(dependencies: &[RegisterDependency]) -> bool {
        // Pour l'instant, version simplifiée : toujours prêt
        // TODO: Implémenter la vraie vérification avec scoreboard
        dependencies.is_empty()
    }
    
    /// Récupère la prochaine instruction prête de la queue
    fn get_ready_instruction(queue: &mut VecDeque<ExecutionPacket>) -> Option<ExecutionPacket> {
        // Chercher la première instruction prête avec la plus haute priorité
        let mut best_idx = None;
        let mut best_priority = InstructionPriority::Low;
        
        for (idx, packet) in queue.iter().enumerate() {
            if packet.ready && packet.priority >= best_priority {
                best_idx = Some(idx);
                best_priority = packet.priority;
            }
        }
        
        best_idx.and_then(|idx| queue.remove(idx))
    }
    
    /// Exécute les instructions prêtes en parallèle
    pub fn execute_ready_instructions(&mut self) -> Vec<ExecuteMemoryRegister> {
        let mut results = Vec::new();
        
        // D'abord collecter les résultats prêts
        while let Some(pending) = self.pending_results.front() {
            if pending.ready_cycle <= self.current_cycle {
                if let Some(pending) = self.pending_results.pop_front() {
                    results.push(pending.result);
                    
                    // Mettre à jour les statistiques
                    match pending.execution_unit {
                        ExecutionUnit::ALU => self.stats.alu_instructions += 1,
                        ExecutionUnit::AGU => self.stats.agu_instructions += 1,
                        ExecutionUnit::SIMD => self.stats.simd_instructions += 1,
                        _ => {}
                    }
                }
            } else {
                break;
            }
        }
        
        // Exécuter nouvelles instructions si unités disponibles
        let mut executed_count = 0;
        
        // Tenter d'exécuter sur ALU
        if !self.alu_busy {
            if let Some(packet) = Self::get_ready_instruction(&mut self.alu_queue) {
                // TODO: Vraie exécution avec ALU
                println!("PARALLEL: Exécution ALU pour instruction {:?}", packet.instruction.instruction.opcode);
                
                // Simuler l'exécution pour l'instant
                let result = self.create_dummy_result(&packet, ExecutionUnit::ALU);
                self.pending_results.push_back(PendingResult {
                    result,
                    ready_cycle: self.current_cycle + 1,
                    execution_unit: ExecutionUnit::ALU,
                });
                
                self.alu_busy = true;
                self.alu_cycles_remaining = 1;
                executed_count += 1;
            }
        }
        
        // Tenter d'exécuter sur AGU
        if !self.agu_busy {
            if let Some(packet) = Self::get_ready_instruction(&mut self.agu_queue) {
                // TODO: Vraie exécution avec AGU
                println!("PARALLEL: Exécution AGU pour instruction {:?}", packet.instruction.instruction.opcode);
                
                // Simuler l'exécution pour l'instant
                let result = self.create_dummy_result(&packet, ExecutionUnit::AGU);
                self.pending_results.push_back(PendingResult {
                    result,
                    ready_cycle: self.current_cycle + 1,
                    execution_unit: ExecutionUnit::AGU,
                });
                
                self.agu_busy = true;
                self.agu_cycles_remaining = 1;
                executed_count += 1;
            }
        }
        
        // Mettre à jour statistiques d'exécution parallèle
        if executed_count > 1 {
            self.stats.parallel_executions += 1;
            println!("PARALLEL EXECUTION: {} instructions exécutées en parallèle", executed_count);
        }
        
        results
    }
    
    /// Crée un résultat factice pour la simulation
    fn create_dummy_result(&self, packet: &ExecutionPacket, exec_unit: ExecutionUnit) -> ExecuteMemoryRegister {
        ExecuteMemoryRegister {
            instruction: packet.instruction.instruction.clone(),
            alu_result: 0,
            rd: packet.instruction.rd,
            store_value: None,
            mem_addr: packet.instruction.mem_addr,
            branch_target: None,
            branch_taken: false,
            branch_prediction_correct: None,
            stack_operation: None,
            stack_result: None,
            ras_prediction_correct: None,
            halted: false,
        }
    }
    
    /// Obtient les statistiques d'exécution
    pub fn get_stats(&self) -> &ParallelExecutionStats {
        &self.stats
    }
    
    /// Réinitialise le moteur
    pub fn reset(&mut self) {
        self.alu_queue.clear();
        self.agu_queue.clear();
        self.simd_queue.clear();
        
        self.alu_busy = false;
        self.agu_busy = false;
        self.simd_busy = false;
        
        self.alu_cycles_remaining = 0;
        self.agu_cycles_remaining = 0;
        self.simd_cycles_remaining = 0;
        
        self.pending_results.clear();
        self.dependency_analyzer.reset();
        self.stats = ParallelExecutionStats::default();
        self.current_cycle = 0;
    }
}
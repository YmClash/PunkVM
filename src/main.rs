use crate::pvm::instructions::{Address, Instruction, RegisterId};
use crate::pvm::memorys::MemoryController;
use crate::pvm::pipelines::Pipeline;
use crate::pvm::registers::RegisterBank;
use crate::pvm::vm::{OptimizationLevel, VMConfig, PunkVM};
use crate::pvm::vm_errors::VMResult;

// Module declarations
pub mod pvm;

fn main()/* -> VMResult<()>*/ {

    println!("\n");
    println!("========================================");
    println!("===PunkVM=Performance=Analysis=Suite===\n");
    println!("===============Punk=VM==================");

    // Création de l'environnement
    let mut pipeline = Pipeline::new();
    let mut register_bank = RegisterBank::new(8).unwrap();
    let mut memory_controller = MemoryController::new(1024, 256).unwrap();

    // Test Suite 1: Analyse des Branches
    println!("\n=== Test Suite 1: Analyse des Branches ===");
    let branch_test = vec![
        // Programme avec branchements répétitifs (style boucle)
        Instruction::LoadImm(RegisterId(0), 0),    // Compteur
        Instruction::LoadImm(RegisterId(1), 5),    // Limite
        // Début de boucle
        Instruction::Cmp(RegisterId(0), RegisterId(1)),
        Instruction::JumpIf(RegisterId(0), Address(8)), // Si registre 0 != 0, saut
        Instruction::Add(RegisterId(0), RegisterId(0), RegisterId(2)), // Incrémente compteur
        Instruction::Jump(Address(2)),             // Retour au début de la boucle
        // Fin de boucle
        Instruction::Store(RegisterId(0), Address(100)), // Stocke le résultat
    ];
    execute_and_report(&mut pipeline, &mut register_bank, &mut memory_controller,
                       branch_test, "Test des Branchements");

    // Test Suite 2: Analyse des Hazards
    println!("\n=== Test Suite 2: Analyse des Hazards ===");
    let hazard_test = vec![
        // Programme avec différents types de hazards
        Instruction::LoadImm(RegisterId(0), 42),
        Instruction::Store(RegisterId(0), Address(200)),
        Instruction::Load(RegisterId(1), Address(200)),     // RAW hazard
        Instruction::Add(RegisterId(2), RegisterId(1), RegisterId(0)), // Data dependency
        Instruction::Store(RegisterId(2), Address(300)),    // WAW hazard
        Instruction::Load(RegisterId(3), Address(300)),     // RAW hazard
    ];
    execute_and_report(&mut pipeline, &mut register_bank, &mut memory_controller,
                       hazard_test, "Test des Hazards");

    // Test Suite 3: Analyse du Cache
    println!("\n=== Test Suite 3: Analyse du Cache ===");
    let cache_test = vec![
        // Programme pour tester le cache
        Instruction::LoadImm(RegisterId(0), 100),
        Instruction::Store(RegisterId(0), Address(0)),      // Premier accès
        Instruction::Load(RegisterId(1), Address(0)),       // Hit probable
        Instruction::Store(RegisterId(0), Address(256)),    // Miss possible
        Instruction::Load(RegisterId(2), Address(256)),     // Hit probable
        Instruction::Load(RegisterId(3), Address(0)),       // Test de remplacement
    ];
    execute_and_report(&mut pipeline, &mut register_bank, &mut memory_controller,
                       cache_test, "Test du Cache");

    // Test Suite 4: Programme Complexe
    println!("\n=== Test Suite 4: Programme Complexe ===");
    let complex_test = vec![
        // Mélange de toutes les caractéristiques
        Instruction::LoadImm(RegisterId(0), 1),
        Instruction::LoadImm(RegisterId(1), 10),
        // Boucle avec accès mémoire et calculs
        Instruction::Cmp(RegisterId(0), RegisterId(1)),
        Instruction::JumpIf(RegisterId(0), Address(12)), // Utilise R0 comme condition
        Instruction::Store(RegisterId(0), Address(400)),
        Instruction::Load(RegisterId(2), Address(400)),
        Instruction::Add(RegisterId(2), RegisterId(2), RegisterId(0)),
        Instruction::Store(RegisterId(2), Address(500)),
        Instruction::Add(RegisterId(0), RegisterId(0), RegisterId(3)),
        Instruction::Jump(Address(2)),
        // Fin de boucle
    ];
    execute_and_report(&mut pipeline, &mut register_bank, &mut memory_controller,
                       complex_test, "Test Complexe");

    println!("\n=== Rapport Global de Performance ===");
    print_global_metrics(&pipeline);
}

fn execute_and_report(
    pipeline: &mut Pipeline,
    register_bank: &mut RegisterBank,
    memory_controller: &mut MemoryController,
    program: Vec<Instruction>,
    description: &str
) {
    pipeline.load_instructions(program).unwrap();

    println!("\nExécution: {}", description);
    println!("----------------------------------------");

    let mut cycles = 0;
    while !pipeline.is_empty().unwrap() {
        pipeline.cycle(register_bank, memory_controller).unwrap();
        cycles += 1;
    }

    // Métriques du Pipeline
    println!("Cycles: {}", cycles);
    println!("Instructions: {}", pipeline.stats.instructions_executed);
    println!("IPC: {:.2}", pipeline.stats.instructions_executed as f64 / cycles as f64);

    // Métriques de Branchement
    let branch_stats = pipeline.get_branch_stats();
    println!("\nMétriques de Branchement:");
    println!("{}", branch_stats);

    // Métriques des Hazards
    println!("\nMétriques des Hazards:");
    println!("Total Hazards: {}", pipeline.metrics.hazard_metrics.total_hazards);
    println!("Data Hazards: {}", pipeline.metrics.hazard_metrics.data_hazards);
    println!("Load-Use Hazards: {}", pipeline.metrics.hazard_metrics.load_use_hazards);
    println!("Store-Load Hazards: {}", pipeline.metrics.hazard_metrics.store_load_hazards);

    // Métriques de Forwarding
    println!("\nMétriques de Forwarding:");
    println!("Forwards Réussis: {}", pipeline.metrics.forwarding_metrics.successful_forwards);
    println!("Forwards Échoués: {}", pipeline.metrics.forwarding_metrics.failed_forwards);

    // Métriques de Cache
    let total_accesses = pipeline.metrics.cache_metrics.total_accesses;
    if total_accesses > 0 {
        let hit_rate = (pipeline.metrics.cache_metrics.cache_hits as f64 / total_accesses as f64) * 100.0;
        println!("\nMétriques de Cache:");
        println!("Accès Total: {}", total_accesses);
        println!("Cache Hits: {}", pipeline.metrics.cache_metrics.cache_hits);
        println!("Hit Rate: {:.2}%", hit_rate);
    }

    // Utilisation des Ressources
    println!("\nUtilisation des Ressources:");
    println!("Execute Stage: {:.2}%",
             (pipeline.metrics.execute_metrics.busy_cycles as f64 / cycles as f64) * 100.0);
    println!("Memory Stage: {:.2}%",
             (pipeline.metrics.memory_metrics.total_accesses as f64 / cycles as f64) * 100.0);
    println!("----------------------------------------\n");
}

fn print_global_metrics(pipeline: &Pipeline) {
    println!("Statistiques Globales:");
    println!("----------------------------------------");
    println!("Total Instructions: {}", pipeline.metrics.total_instructions);
    println!("Total Cycles: {}", pipeline.metrics.total_cycles);
    println!("IPC Global: {:.2}", pipeline.metrics.ipc);
    println!("Total Branch Mispredictions: {}",
             pipeline.branch_predictor.metrics.incorrect_predictions);
    println!("Branch Prediction Accuracy: {:.2}%",
             (pipeline.branch_predictor.metrics.correct_predictions as f64 /
                 pipeline.branch_predictor.metrics.total_branches as f64) * 100.0);
    println!("Pipeline Flushes: {}", pipeline.metrics.flush_count);
    println!("\nPerformance Summary:");
    println!("{}", pipeline.generate_performance_report());
    println!("========================================");
    println!("=============Punk=VM=by==YmC============");
    println!("========================================\n");
}
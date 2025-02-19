use crate::pvm::instructions::{Address, Instruction, RegisterId};
use crate::pvm::memorys::MemoryController;
use crate::pvm::pipelines::Pipeline;
use crate::pvm::registers::RegisterBank;
use crate::pvm::vm::{OptimizationLevel, VMConfig, PunkVM};
use crate::pvm::vm_errors::VMResult;

// Module declarations
pub mod pvm;

fn main()/* -> VMResult<()>*/ {


//    Configuration de base de la VM
//     let config = VMConfig {
//         memory_size: 1024,    // 1 KB de mémoire
//         cache_size: 256,      // 256 bytes de cache
//         register_count: 16,   // 16 registres généraux
//         optimization_level: OptimizationLevel::Basic,
//         stack_size: 1024,     // 1 KB de pile
//     };
//
//   //  Création de la VM
//     println!("Initialisation de la PunkVM...");
//     let mut vm = PunkVM::new(config)?;
//



    // Création de l'environnement
    let mut pipeline = Pipeline::new();
    let mut register_bank = RegisterBank::new(8).unwrap();
    let mut memory_controller = MemoryController::new(1024, 256).unwrap();



    // /////////////////////////////////////
//     // Programme de test simple
//     let test_program = vec![
//         // 1. Charger des valeurs dans les registres
//         Instruction::LoadImm(RegisterId(0), 42),      // R0 = 42
//         Instruction::LoadImm(RegisterId(1), 58),      // R1 = 58
//
//         // 2. Effectuer une addition
//         Instruction::Add(RegisterId(2), RegisterId(0), RegisterId(1)),  // R2 = R0 + R1
//
//         // 3. Stocker le résultat en mémoire
//         Instruction::Store(RegisterId(2), Address(0x100)),  // MEM[0x100] = R2
//
//         // 4. Recharger la valeur depuis la mémoire
//         Instruction::Load(RegisterId(3), Address(0x100)),   // R3 = MEM[0x100]
//
//         //test_mixed_memory_arithmetic
//         // Instruction::LoadImm(RegisterId(0), 100),
//         // Instruction::Store(RegisterId(0), Address(200)),
//         // Instruction::LoadImm(RegisterId(1), 50),
//         // Instruction::Load(RegisterId(2), Address(200)),
//         // Instruction::Add(RegisterId(3), RegisterId(2), RegisterId(1)),
//         // Instruction::Store(RegisterId(3), Address(300)),
//         // Instruction::Load(RegisterId(4), Address(300)),
//
//         //test_complex_arithmetic_forwarding
//         // Instruction::LoadImm(RegisterId(0), 10),
//         // Instruction::LoadImm(RegisterId(1), 5),
//         // Instruction::Add(RegisterId(2), RegisterId(0), RegisterId(1)),
//         // Instruction::Mul(RegisterId(3), RegisterId(2), RegisterId(1)),
//         // Instruction::Sub(RegisterId(4), RegisterId(3), RegisterId(2)),
//
//
//     ];
//
//     let test_program_1 = vec![
//
//         // Test avec dépendances mémoire
//         // R0 = 42
//         // Store R0, @100
//         // R1 = Load @100     // Dépendance mémoire
//         // R2 = R1 + R0       // Dépendance de registre
//         Instruction::LoadImm(RegisterId(0), 42),
//         Instruction::Store(RegisterId(0), Address(100)),
//         Instruction::Load(RegisterId(1), Address(100)),
//         Instruction::Add(RegisterId(2), RegisterId(1), RegisterId(0)),
//
//
//
//         // Instruction::LoadImm(RegisterId(0), 100),
//         // Instruction::Store(RegisterId(0), Address(200)),
//         // Instruction::LoadImm(RegisterId(1), 50),
//         // Instruction::Load(RegisterId(2), Address(200)),
//         // Instruction::Add(RegisterId(3), RegisterId(2), RegisterId(1)),
//         // Instruction::Store(RegisterId(3), Address(300)),
//         // Instruction::Load(RegisterId(4), Address(300)),
//         //
//
//         // Instruction::LoadImm(RegisterId(0), 10),
//         // Instruction::LoadImm(RegisterId(1), 5),
//         // Instruction::Add(RegisterId(2), RegisterId(0), RegisterId(1)),
//         // Instruction::Mul(RegisterId(3), RegisterId(2), RegisterId(1)),
//         // Instruction::Sub(RegisterId(4), RegisterId(3), RegisterId(2)),
//         //
//
//
//         //test mixed memory arithmetic
//
//         // Test combinant opérations mémoire et arithmétiques
//         // R0 = 100
//         // Store R0, @200
//         // R1 = 50
//         // R2 = Load @200    // Doit charger 100
//         // R3 = R2 + R1      // 150
//         // Store R3, @300
//         // R4 = Load @300    // Doit charger 150
//         // Instruction::LoadImm(RegisterId(0), 100),
//         // Instruction::Store(RegisterId(0), Address(200)),
//         // Instruction::LoadImm(RegisterId(1), 50),
//         // Instruction::Load(RegisterId(2), Address(200)),
//         // Instruction::Add(RegisterId(3), RegisterId(2), RegisterId(1)),
//         // Instruction::Store(RegisterId(3), Address(300)),
//         // Instruction::Load(RegisterId(4), Address(300)),
//
//         // //test pipeline stalls
//         // // Test vérifiant les stalls du pipeline
//         // // R0 = 10
//         // // R1 = R0 + 5       // Dépendance avec R0
//         // // R2 = R1 + 3       // Dépendance avec R1
//         // // Store R2, @100    // Dépendance avec R2
//         // // R3 = Load @100    // Dépendance mémoire
//         //
//         // Instruction::LoadImm(RegisterId(0), 10),
//         // Instruction::Add(RegisterId(1), RegisterId(0), RegisterId(0)),
//         // Instruction::Add(RegisterId(2), RegisterId(1), RegisterId(1)),
//         // Instruction::Store(RegisterId(2), Address(100)),
//         // Instruction::Load(RegisterId(3), Address(100)),
//
//
//     ];
//
//
//
//
//
//
//     // Exécution du programme de test
//     println!("\nDémarrage du programme de test...");
//     println!("-----------Punk-VM-----------------\n");
//
//
//
//
//     // vm.load_program(test_program)?;
//     // vm.run()?;
//
//     println!("\nTest terminé !");
//     println!("État final de la VM:");
//
//     // println!("\nTest Finish!");
//     // println!("Final State of the VM:");
//     print_vm_state(&vm);
//     print_vm_stats(&vm);
//
//
//     Ok(())
///////////////////////////////////////////////////////////////////////////////



    println!("=== PunkVM Performance Analysis ===\n");
    println!("-----------Punk-VM-----------------\n");

    // Création de l'environnement
    let mut pipeline = Pipeline::new();
    let mut register_bank = RegisterBank::new(8).unwrap();
    let mut memory_controller = MemoryController::new(1024, 256).unwrap();

    // Test 1: Programme avec dépendances de données
    println!("Test 1: Programme avec dépendances de données");
    let program1 = vec![
        Instruction::LoadImm(RegisterId(0), 42),          // R0 = 42
        Instruction::Add(RegisterId(1), RegisterId(0), RegisterId(0)),  // R1 = R0 + R0
        Instruction::Add(RegisterId(2), RegisterId(1), RegisterId(0)),  // R2 = R1 + R0
        Instruction::Store(RegisterId(2), Address(100)),  // Mem[100] = R2
    ];

    execute_and_report(&mut pipeline, &mut register_bank, &mut memory_controller, program1, "Programme avec dépendances");

    // Test 2: Programme avec hazards mémoire
    println!("\nTest 2: Programme avec hazards mémoire");
    let program2 = vec![
        Instruction::LoadImm(RegisterId(0), 100),
        Instruction::Store(RegisterId(0), Address(200)),
        Instruction::Load(RegisterId(1), Address(200)),   // Load-After-Store hazard
        Instruction::Add(RegisterId(2), RegisterId(1), RegisterId(1)),  // Load-Use hazard
    ];

    execute_and_report(&mut pipeline, &mut register_bank, &mut memory_controller, program2, "Programme avec hazards mémoire");

    // Test 3: Programme avec forwarding intensif
    println!("\nTest 3: Programme avec forwarding intensif");
    let program3 = vec![
        Instruction::LoadImm(RegisterId(0), 1),
        Instruction::Add(RegisterId(1), RegisterId(0), RegisterId(0)), // Forward from LoadImm
        Instruction::Add(RegisterId(2), RegisterId(1), RegisterId(1)), // Forward from Add
        Instruction::Add(RegisterId(3), RegisterId(2), RegisterId(2)), // Forward from Add
    ];

    execute_and_report(&mut pipeline, &mut register_bank, &mut memory_controller, program3, "Programme avec forwarding intensif");

    // Test 4: Programme complexe pour les optimisations
    println!("\nTest 4: Programme complexe pour les optimisations");
    let program3 = vec![
        // Programme complexe pour tester les optimisations
        // Test 1: Dépendances de données
        Instruction::LoadImm(RegisterId(0), 10),
        Instruction::Add(RegisterId(1), RegisterId(0), RegisterId(0)),
        Instruction::Mul(RegisterId(2), RegisterId(1), RegisterId(0)),

        // Test 2: Accès mémoire avec hazards
        Instruction::Store(RegisterId(2), Address(100)),
        Instruction::Load(RegisterId(3), Address(100)),

        // Test 3: Opérations arithmétiques en chaîne
        Instruction::Add(RegisterId(4), RegisterId(3), RegisterId(1)),
        Instruction::Sub(RegisterId(5), RegisterId(4), RegisterId(2)),

        // Test 4: Mixture d'opérations
        Instruction::LoadImm(RegisterId(6), 42),
        Instruction::Store(RegisterId(6), Address(200)),
        Instruction::Load(RegisterId(7), Address(200)),
        Instruction::Mul(RegisterId(0), RegisterId(7), RegisterId(6)),
    ];

    execute_and_report(&mut pipeline, &mut register_bank, &mut memory_controller, program3, "Programme complexe pour les optimisations");





}

fn print_vm_state(vm: &PunkVM) {
    println!("  Registres:");
    for i in 0..4 {  // Afficher seulement les 4 premiers registres pour la clarté
        if let Ok(value) = vm.read_register(RegisterId(i)) {
            println!("    R{}: {}", i, value);
        }
    }
    println!();
}

fn print_vm_stats(vm: &PunkVM) {
    if let Ok(stats) = vm.get_statistics() {
        println!("Statistiques d'exécution:");
        println!("  Instructions exécutées: {}", stats.instructions_executed);
        println!("  Cycles total: {}", stats.cycles);
        println!("  Cache hits: {}", stats.cache_hits);
        println!("  Pipeline stalls: {}", stats.pipeline_stalls);
    }
}
fn execute_and_report(
    pipeline: &mut Pipeline,
    register_bank: &mut RegisterBank,
    memory_controller: &mut MemoryController,
    program: Vec<Instruction>,
    description: &str
) {
    pipeline.load_instructions(program).unwrap();

    println!("Exécution de: {}", description);
    let mut cycles = 0;

    // Exécution du programme
    while !pipeline.is_empty().unwrap() {
        pipeline.cycle(register_bank, memory_controller).unwrap();
        cycles += 1;
    }

    // Affichage des résultats détaillés
    println!("\nRésultats détaillés:");
    println!("----------------------------------------");
    println!("Cycles total: {}", cycles);
    println!("Instructions total: {}", pipeline.stats.instructions_executed);
    println!("IPC (Instructions par cycle): {:.2}",
             pipeline.stats.instructions_executed as f64 / cycles as f64);
    println!("Stalls total: {}", pipeline.stats.stalls);
    println!("Hazards détectés: {}", pipeline.stats.hazards);

    // Métriques de pipeline
    println!("\nMétriques du pipeline:");
    println!("----------------------------------------");
    println!("Fetch stalls: {}", pipeline.metrics.fetch_metrics.stall_cycles);
    println!("Execute busy cycles: {}", pipeline.metrics.execute_metrics.busy_cycles);
    println!("Memory accesses: {}", pipeline.metrics.memory_metrics.total_accesses);

    // Métriques de forwarding
    println!("\nMétriques de forwarding:");
    println!("----------------------------------------");
    println!("Forwards réussis: {}", pipeline.metrics.forwarding_metrics.successful_forwards);
    println!("Forwards échoués: {}", pipeline.metrics.forwarding_metrics.failed_forwards);

    // Métriques de cache
    println!("\nMétriques de cache:");
    println!("----------------------------------------");
    println!("Cache hits: {}", pipeline.metrics.cache_metrics.cache_hits);
    println!("Cache misses: {}", pipeline.metrics.cache_metrics.cache_misses);
    if pipeline.metrics.cache_metrics.total_accesses > 0 {
        let hit_rate = (pipeline.metrics.cache_metrics.cache_hits as f64 /
            pipeline.metrics.cache_metrics.total_accesses as f64) * 100.0;
        println!("Hit rate: {:.2}%", hit_rate);
    }


    println!("\nPerformance Metrics:");
    println!("----------------------------------------");
    println!("Total Cycles: {}", cycles);
    println!("Total Instructions: {}", pipeline.stats.instructions_executed);
    println!("IPC: {:.2}", pipeline.stats.instructions_executed as f64 / cycles as f64);
    println!("Total Stalls: {}", pipeline.stats.stalls);
    println!("Reorderings: {}", pipeline.metrics.reorder_count);
    println!("Execute Stage Utilization: {:.2}%",
             (pipeline.metrics.execute_metrics.busy_cycles as f64 / cycles as f64) * 100.0);
    println!("Memory Stage Utilization: {:.2}%",
             (pipeline.metrics.memory_metrics.total_accesses as f64 / cycles as f64) * 100.0);

    println!("\nRapport de performance complet:");
    println!("----------------------------------------");
    println!("{}", pipeline.generate_performance_report());
    println!("========================================");
    println!("=============Punk=VM=by==YmC============");
    println!("========================================\n");

}
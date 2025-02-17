use crate::pvm::instructions::{Address, Instruction, RegisterId};
use crate::pvm::vm::{OptimizationLevel, VMConfig, PunkVM};
use crate::pvm::vm_errors::VMResult;

// Module declarations
pub mod pvm;

fn main() -> VMResult<()> {
    // Configuration de base de la VM
    let config = VMConfig {
        memory_size: 1024,    // 1 KB de mémoire
        cache_size: 256,      // 256 bytes de cache
        register_count: 16,   // 16 registres généraux
        optimization_level: OptimizationLevel::Basic,
        stack_size: 1024,     // 1 KB de pile
    };

    // Création de la VM
    println!("Initialisation de la PunkVM...");
    let mut vm = PunkVM::new(config)?;

    // Programme de test simple
    let test_program = vec![
        // 1. Charger des valeurs dans les registres
        Instruction::LoadImm(RegisterId(0), 42),      // R0 = 42
        Instruction::LoadImm(RegisterId(1), 58),      // R1 = 58

        // 2. Effectuer une addition
        Instruction::Add(RegisterId(2), RegisterId(0), RegisterId(1)),  // R2 = R0 + R1

        // 3. Stocker le résultat en mémoire
        Instruction::Store(RegisterId(2), Address(0x100)),  // MEM[0x100] = R2

        // 4. Recharger la valeur depuis la mémoire
        Instruction::Load(RegisterId(3), Address(0x100)),   // R3 = MEM[0x100]
    ];

    // Exécution du programme de test
    println!("\nDémarrage du programme de test...");
    println!("-----------Punk-VM-----------------\n");




    vm.load_program(test_program)?;
    vm.run()?;

    // println!("\nTest terminé !");
    // println!("État final de la VM:");

    println!("\nTest Finish!");
    println!("Final State of the VM:");
    print_vm_state(&vm);
    print_vm_stats(&vm);

    Ok(())
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

// fn print_vm_stats(vm: &PunkVM) {
//     if let Ok(stats) = vm.get_statistics() {
//         println!("Statistiques d'exécution:");
//         println!("  Instructions exécutées: {}", stats.instructions_executed);
//         println!("  Cycles total: {}", stats.cycles);
//         println!("  Cache hits: {}", stats.cache_hits);
//         println!("  Pipeline stalls: {}", stats.pipeline_stalls);
//     }
// }

fn print_vm_stats(vm: &PunkVM) {
    if let Ok(stats) = vm.get_statistics() {
        println!("  Execution statistics::");
        println!("  Instructions executed: {}", stats.instructions_executed);
        println!("  Total cycles: {}", stats.cycles);
        println!("  Cache hits: {}", stats.cache_hits);
        println!("  Pipeline stalls: {}", stats.pipeline_stalls);
    }
}
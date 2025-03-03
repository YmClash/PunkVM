// examples/simple_vm.rs


use PunkVM::bytecode::files::{BytecodeVersion, SegmentMetadata, SegmentType};
use PunkVM::bytecode::format::{ArgType, InstructionFormat};
use PunkVM::bytecode::instructions::Instruction;
use PunkVM::bytecode::opcodes::Opcode;
use PunkVM::BytecodeFile;
use PunkVM::pvm::vm::{VMConfig, VMState};
// use crate::pvm::vm::PunkVM;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Création d'un programme simple qui calcule la somme des nombres de 1 à 10
    // println!("Création du programme de test...");
    // let bytecode = create_minimal_test_program()?;
    //
    // // Configuration de la VM
    // let config = VMConfig {
    //     memory_size: 64 * 1024,       // 64 KB de mémoire
    //     num_registers: 16,            // 16 registres généraux
    //     l1_cache_size: 1024,          // 1 KB de cache L1
    //     store_buffer_size: 8,         // 8 entrées dans le store buffer
    //     stack_size: 4 * 1024,         // 4 KB de pile
    //     fetch_buffer_size: 8,         // 8 instructions dans le buffer de fetch
    //     btb_size: 16,                 // 16 entrées dans la BTB
    //     ras_size: 4,                  // 4 entrées dans le RAS
    //     enable_forwarding: true,      // Activer le forwarding
    //     enable_hazard_detection: true, // Activer la détection de hazards
    // };
    //
    //
    let mut vm = PunkVM::pvm::vm::PunkVM::with_config(Default::default());

    println!("\n===== TEST DE PUNKVM =====\n");

    // Créer une VM avec la configuration par défaut
    println!("Initialisation de la VM...");
    // let mut vm = PunkVM::new();

    // Créer un programme bytecode
    println!("Création du programme de test...");
    let mut program = BytecodeFile::new();


    // Ajouter des instructions variées pour tester différents aspects de la VM
    println!("Ajout des instructions...");

    // NOP - ne fait rien
    program.add_instruction(Instruction::create_no_args(Opcode::Nop));

    // LOAD R0, 5 - charge valeur immédiate dans R0
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Load, 0, 5));

    // LOAD R1, 7 - charge valeur immédiate dans R1
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Load, 1, 7));

    // ADD R2, R0 - R2 = R2 + R0
    program.add_instruction(Instruction::create_reg_reg(Opcode::Add, 2, 0));

    // ADD R2, R1 - R2 = R2 + R1
    program.add_instruction(Instruction::create_reg_reg(Opcode::Add, 2, 1));

    // LOAD R3, 0x1000 - adresse mémoire pour test
    program.add_instruction(Instruction::create_reg_imm16(Opcode::Load, 3, 0x1000));

    // STORE R2, [R3] - stocke R2 à l'adresse dans R3
    let store_instruction = Instruction::new(
        Opcode::Store,
        InstructionFormat::new(ArgType::Register, ArgType::Register),
        vec![2, 3] // Utiliser R2 comme source et R3 comme adresse
    );
    program.add_instruction(store_instruction);

    // LOAD R4, [R3] - charge depuis l'adresse dans R3
    let load_instruction = Instruction::new(
        Opcode::Load,
        InstructionFormat::new(ArgType::Register, ArgType::Register),
        vec![4, 3] // Destination R4, adresse R3
    );
    program.add_instruction(load_instruction);

    // HALT - termine l'exécution
    program.add_instruction(Instruction::create_no_args(Opcode::Halt));

    // Définir les segments et métadonnées
    let code_size = program.code.iter().map(|instr| instr.total_size()).sum::<usize>() as u32;
    program.segments = vec![
        SegmentMetadata::new(SegmentType::Code, 0, code_size, 0)
    ];
    program.data = Vec::new();
    program.readonly_data = Vec::new();

    // Charger le programme dans la VM
    println!("Chargement du programme...");
    match vm.load_program_from_bytecode(program) {
        Ok(_) => println!("Programme chargé avec succès"),
        Err(e) => {
            println!("Erreur lors du chargement du programme: {}", e);
            return Ok(());
        }
    }

    // Exécuter la VM en mode pas à pas pour des statistiques détaillées
    println!("\n===== EXÉCUTION DU PROGRAMME =====\n");

    // Configurer un nombre maximum de cycles pour éviter une boucle infinie
    let max_cycles = 30;
    vm.state = VMState::Running;

    for cycle in 0..max_cycles {
        // Afficher l'état avant le cycle
        println!("Cycle {}: PC = 0x{:X}", cycle, vm.pc);
        println!("  Registres: R0={}, R1={}, R2={}, R3={}, R4={}",
                 vm.registers[0], vm.registers[1], vm.registers[2],
                 vm.registers[3], vm.registers[4]);

        // Exécuter un cycle
        match vm.step() {
            Ok(_) => {},
            Err(e) => {
                println!("Erreur lors de l'exécution: {}", e);
                break;
            }
        }

        // Vérifier si l'exécution est terminée
        if vm.state() == &VMState::Halted {
            println!("Programme terminé au cycle {}", cycle);
            break;
        }

        // Afficher les statistiques intermédiaires
        let stats = vm.stats();
        println!("  Instructions exécutées: {}", stats.instructions_executed);
        println!("  IPC: {:.2}", stats.ipc);
        println!();
    }

    // Afficher les résultats
    println!("\n===== ÉTAT FINAL =====\n");
    println!("Registres:");
    println!("  R0 = {} (devrait être 5)", vm.registers[0]);
    println!("  R1 = {} (devrait être 7)", vm.registers[1]);
    println!("  R2 = {} (devrait être 12 = 5+7)", vm.registers[2]);
    println!("  R3 = {} (devrait être 0x1000 = 4096)", vm.registers[3]);
    println!("  R4 = {} (devrait être 12, chargé depuis l'adresse 0x1000)", vm.registers[4]);

    // Vérifier la mémoire à l'adresse 0x1000
    let memory_value = vm.memory.read_qword(0x1000).unwrap_or(0);
    println!("\nMémoire:");
    println!("  [0x1000] = {} (devrait être 12)", memory_value);

    // Afficher les statistiques d'exécution
    let stats = vm.stats();
    println!("\n===== STATISTIQUES D'EXÉCUTION =====\n");
    println!("Cycles: {}", stats.cycles);
    println!("Instructions exécutées: {}", stats.instructions_executed);
    println!("IPC (Instructions Par Cycle): {:.2}", stats.ipc);
    println!("Stalls: {}", stats.stalls);
    println!("Hazards: {}", stats.hazards);
    println!("Forwards: {}", stats.forwards);
    println!("Cache hits: {}", stats.memory_hits);
    println!("Cache misses: {}", stats.memory_misses);

    // Évaluation des performances
    println!("\n===== ÉVALUATION DES PERFORMANCES =====\n");

    // Taux de hits du cache
    let cache_hit_rate = if stats.memory_hits + stats.memory_misses > 0 {
        stats.memory_hits as f64 / (stats.memory_hits + stats.memory_misses) as f64 * 100.0
    } else {
        0.0
    };
    println!("Taux de hits cache: {:.2}%", cache_hit_rate);

    // Taux de stalls
    let stall_rate = if stats.cycles > 0 {
        stats.stalls as f64 / stats.cycles as f64 * 100.0
    } else {
        0.0
    };
    println!("Taux de stalls: {:.2}%", stall_rate);

    // Efficacité du forwarding
    let forwarding_efficiency = if stats.hazards > 0 {
        stats.forwards as f64 / stats.hazards as f64 * 100.0
    } else {
        0.0
    };
    println!("Efficacité du forwarding: {:.2}%", forwarding_efficiency);

    println!("\n===== TEST TERMINÉ =====\n");
    Ok(())
}
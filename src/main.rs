//src/main.rs
// Utilisez "crate::" au lieu de "PunkVM::" pour les imports internes
// extern crate PunkVM;

use std::time::Instant;
use PunkVM::bytecode::files::{BytecodeVersion, SegmentMetadata, SegmentType, BytecodeFile};
use PunkVM::bytecode::format::{ArgType, InstructionFormat};
use PunkVM::bytecode::instructions::Instruction;
use PunkVM::bytecode::opcodes::Opcode;
use PunkVM::pvm::vm::{VMConfig, VMState, PunkVM as VM};
use PunkVM::pvm::vm_errors::VMResult;

fn main() -> VMResult<()> {

    println!("=== PunkVM - Test d'un programme complexe ===");

    // Configuration de la VM
    let config = VMConfig {
        memory_size: 64 * 1024,       // 64 KB de mémoire
        num_registers: 16,            // 16 registres généraux
        l1_cache_size: 1024,          // 1 KB de cache L1
        store_buffer_size: 8,         // 8 entrées dans le store buffer
        stack_size: 4 * 1024,         // 4 KB de pile
        fetch_buffer_size: 8,         // 8 instructions dans le buffer de fetch
        btb_size: 16,                 // 16 entrées dans la BTB
        ras_size: 4,                  // 4 entrées dans le RAS
        enable_forwarding: true,      // Activer le forwarding
        enable_hazard_detection: true, // Activer la détection de hazards
    };

    // Créer une VM avec la configuration spécifiée
    println!("Initialisation de la VM...");
    let mut vm = PunkVM::pvm::vm::PunkVM::with_config(config);
    println!(" PunkVM initialisée avec {} registre succès", vm.registers.len());

    // Créer le programme complexe
    let program = create_complex_program();
    // let program = create_simple_complex_program();


    // Charger le programme dans la VM
    println!("Chargement du programme...");
    vm.load_program_from_bytecode(program)?;

    // Exécuter le programme et mesurer le temps
    println!("Exécution du programme...");
    let start_time = Instant::now();
    let result = vm.run();
    let duration = start_time.elapsed();

    if let Err(ref e) = result {
        println!("Erreur lors de l'exécution: {}", e);
    } else {
        println!("Programme exécuté avec succès en {:?}", duration);
    }

    // Afficher l'état final des registres
    println!("\nÉtat final des registres:");
    print_registers(&vm);

    // Afficher les statistiques d'exécution
    print_stats(&vm);

    Ok(())

}


fn create_simple_complex_program() -> BytecodeFile {
    let mut program = BytecodeFile::new();
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    program.add_metadata("name", "Programme de test complexe");
    program.add_metadata("description", "Test des fonctionnalités avancées de PunkVM");

    // Initialisation des registres avec des valeurs de test
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 0));   // R0 = 0
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 10));  // R1 = 10
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 2, 1));   // R2 = 1
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 3, 0));   // R3 = 0

////////////////////////////////////////////////////////////////////////////////////////////////////
    // // Label: LOOP_START (implicite)
    // // Incrémenter compteur: R0 = R0 + R2
    // program.add_instruction(Instruction::create_reg_reg(Opcode::Add, 0, 2));
    //
    // // Ajouter à la somme: R3 = R3 + R0
    // program.add_instruction(Instruction::create_reg_reg(Opcode::Add, 3, 0));
    //
    // // Comparer compteur à limite: R0 vs R1
    // program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 1));
    //
    // // Sauter si R0 < R1, retour à LOOP_START
    // // // Note: Vous devrez adapter ce code selon votre implémentation
    // let jump_instruction = Instruction::create_reg_imm8(Opcode::JmpIf, 0, 0xFF); // -1 signifie "retourner à l'instruction précédente"
    // program.add_instruction(jump_instruction);
    //
    // // // Pour débogage, copier résultat dans d'autres registres
    // program.add_instruction(Instruction::create_reg_reg(Opcode::Add, 10, 3)); // R10 = 0 + R3
//////////////////////////////////////////////////////////////////////////////////////////////////////////
    // Opérations en format reg_reg_reg :
    // ADD R2, R0, R1   → R2 = 5 + 3 = 8
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 2, 0, 1));

    // SUB R4, R3, R0   → R4 = 10 - 5 = 5
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Sub, 4, 3, 0));

    // MUL R5, R2, R4   → R5 = 8 * 5 = 40
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Mul, 5, 2, 4));

    // DIV R6, R5, R1   → R6 = 40 / 3 = 13 (division entière, si c'est le comportement défini)
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Div, 6, 5, 1));

    // HALT → arrête l'exécution
    program.add_instruction(Instruction::create_no_args(Opcode::Halt));


    // // HALT: Arrêter l'exécution
    program.add_instruction(Instruction::create_no_args(Opcode::Halt));

    // // Calculer la taille totale du code
    let total_size: u32 = program.code.iter()
        .map(|instr| instr.total_size() as u32)
        .sum();

    // Créer le segment de code
    program.segments = vec![
        SegmentMetadata::new(SegmentType::Code, 0, total_size, 0)
    ];

    // Créer un segment de données
    let data_size = 256; // Allouer 256 bytes pour les données
    let data_segment = SegmentMetadata::new(SegmentType::Data, 0, data_size, 0x1000);
    program.segments.push(data_segment);
    program.data = vec![0; data_size as usize];

    println!("Programme simplifié créé avec {} instructions", program.code.len());

    program
}



/// Crée un programme complexe qui teste plusieurs aspects de la VM:
/// - Dépendances de données
/// - Branchements conditionnels
/// - Boucles
/// - Accès mémoire (Load/Store)
/// - Forwarding
fn create_complex_program() -> BytecodeFile {
    let mut program = BytecodeFile::new();
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    program.add_metadata("name", "Programme de test complexe");
    program.add_metadata("description", "Test des fonctionnalités avancées de PunkVM");

    // Initialisation des registres
    // R0 = 0 (compteur de boucle)
    // R1 = 10 (limite de boucle)
    // R2 = 1 (incrément)
    // R3 = 0 (somme)
    // R4 = 100 (base pour les adresses mémoire)

    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 0));   // R0 = 0
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 10));  // R1 = 10
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 2, 1));   // R2 = 1
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 3, 0));   // R3 = 0
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 4, 100)); // R4 = 100

    // Stocker la valeur initiale du compteur en mémoire
    // Store R0 @ [R4+0]
    let store_inst = create_store_with_offset(0, 4, 0);
    program.add_instruction(store_inst);

    // Label: LOOP_START
    let loop_start_idx = program.code.len();

    // Incrémenter le compteur: R0 = R0 + R2
    program.add_instruction(Instruction::create_reg_reg(Opcode::Add, 0, 2));

    // Stocker le compteur en mémoire: Store R0 @ [R4+R0]
    // Cela crée une dépendance de données et un potentiel hazard
    let store_counter = create_store_with_offset(0, 4, 0);
    program.add_instruction(store_counter);

    // Charger la valeur précédente depuis la mémoire: R5 = Load @ [R4+(R0-1)]
    // R5 = R0 - 1
    program.add_instruction(Instruction::create_reg_reg(Opcode::Sub, 5, 0));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Add, 5, 1));

    // R6 = R4 + R5 (adresse mémoire)
    program.add_instruction(Instruction::create_reg_reg(Opcode::Add, 6, 4));
    program.add_instruction(Instruction::create_reg_reg(Opcode::Add, 6, 5));

    // R7 = Load @ [R6]
    let load_prev = create_load_with_register(7, 6);
    program.add_instruction(load_prev);

    // Ajouter à la somme: R3 = R3 + R0
    program.add_instruction(Instruction::create_reg_reg(Opcode::Add, 3, 0));

    // Comparer le compteur à la limite: Cmp R0, R1
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 1));

    // Sauter si R0 < R1
    // Calculer le décalage pour le saut vers LOOP_START
    let current_idx = program.code.len();
    let offset_to_start = -(calculate_instruction_range_size(&program.code, loop_start_idx, current_idx) as i32);

    // JmpIf R0 < R1, LOOP_START
    let jump_instruction = create_conditional_jump(offset_to_start);
    program.add_instruction(jump_instruction);

    // Stocker le résultat final: Store R3 @ [R4+20]
    let store_result = create_store_with_offset(3, 4, 20);
    program.add_instruction(store_result);

    // Charger le résultat dans R10 pour vérification: R10 = Load @ [R4+20]
    let load_result = create_load_with_offset(10, 4, 20);
    program.add_instruction(load_result);

    // Programme terminé: HALT
    program.add_instruction(Instruction::create_no_args(Opcode::Halt));

    // Calculer la taille totale du code
    let total_size: u32 = program.code.iter()
        .map(|instr| instr.total_size() as u32)
        .sum();

    // Créer le segment de code
    program.segments = vec![
        SegmentMetadata::new(SegmentType::Code, 0, total_size, 0)
    ];

    // Créer un segment de données
    let data_size = 256; // Allouer 256 bytes pour les données
    let data_segment = SegmentMetadata::new(SegmentType::Data, 0, data_size, 0x1000);
    program.segments.push(data_segment);
    program.data = vec![0; data_size as usize];

    println!("Programme complexe créé avec {} instructions", program.code.len());

    program
}

// Fonctions utilitaires pour créer des instructions spécifiques

fn create_store_with_offset(value_reg: u8, base_reg: u8, offset: u8) -> Instruction {
    // Cette fonction simule: Store R{value_reg} @ [R{base_reg} + offset]
    // L'implémentation exacte dépend de votre format d'instruction
    Instruction::create_reg_imm8(Opcode::Store, value_reg, offset)
}

fn create_load_with_offset(dest_reg: u8, base_reg: u8, offset: u8) -> Instruction {
    // Cette fonction simule: R{dest_reg} = Load @ [R{base_reg} + offset]
    // Instruction::create_reg_imm8(Opcode::Load, dest_reg, offset)
    Instruction::create_reg_imm8(Opcode::Mov, dest_reg, offset)
}

fn create_load_with_register(dest_reg: u8, addr_reg: u8) -> Instruction {
    // Cette fonction simule: R{dest_reg} = Load @ [R{addr_reg}]
    Instruction::create_reg_reg(Opcode::Load, dest_reg, addr_reg)
}

fn create_conditional_jump(offset: i32) -> Instruction {
    // Crée un saut conditionnel avec un offset relatif
    // L'implémentation exacte dépend de votre format d'instruction
    let mut instruction = Instruction::create_no_args(Opcode::JmpIf);

    // Encoder l'offset dans les arguments de l'instruction
    // Cela peut nécessiter une adaptation selon votre format
    let bytes = offset.to_le_bytes();
    instruction.args = bytes.to_vec();

    instruction
}

fn calculate_instruction_range_size(instructions: &[Instruction], start_idx: usize, end_idx: usize) -> usize {
    let mut total_size = 0;
    for i in start_idx..end_idx {
        total_size += instructions[i].total_size();
    }
    total_size
}

fn print_registers(vm: &VM) {
    for i in 0..16 {
        if i % 4 == 0 {
            println!();
        }
        print!("R{:<2} = {:<10}", i, vm.registers[i]);
    }
    println!("\n");
}

fn print_stats(vm: &VM) {
    let stats = vm.stats();
    println!("\n===== STATISTIQUES D'EXÉCUTION =====\n");
    println!("  Cycles: {}", stats.cycles);
    println!("  Instructions exécutées: {}", stats.instructions_executed);
    println!("  IPC (Instructions Par Cycle): {:.2}", stats.ipc);
    println!("  Stalls: {}", stats.stalls);
    println!("  Hazards: {}", stats.hazards);
    println!("  Forwards: {}", stats.forwards);
    println!("  Cache hits: {}", stats.memory_hits);
    println!("  Cache misses: {}", stats.memory_misses);

    // Calcul de quelques métriques supplémentaires
    if stats.cycles > 0 {
        let stall_rate = (stats.stalls as f64 / stats.cycles as f64) * 100.0;
        println!("  Taux de stalls: {:.2}%", stall_rate);
    }

    if stats.memory_hits + stats.memory_misses > 0 {
        let hit_rate = (stats.memory_hits as f64 / (stats.memory_hits + stats.memory_misses) as f64) * 100.0;
        println!("  Taux de hits cache: {:.2}%", hit_rate);
    }

    if stats.hazards > 0 && stats.forwards > 0 {
        let forwarding_efficiency = (stats.forwards as f64 / stats.hazards as f64) * 100.0;
        println!("  Efficacité du forwarding: {:.2}%", forwarding_efficiency);
    }


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

}
/////////////////////////////////////////////////////////////////////////////////////////////////////




/*

fn create_minimal_test_program() -> Result<BytecodeFile, Box<dyn std::error::Error>> {
    let mut bytecode = BytecodeFile::new();

    // Définition de la version
    bytecode.version = BytecodeVersion::new(0, 1, 0, 0);

    // Juste quelques instructions simples
    bytecode.add_instruction(Instruction::create_reg_imm8(Opcode::Load, 0, 1));
    bytecode.add_instruction(Instruction::create_no_args(Opcode::Halt));

    Ok(bytecode)
}

// Crée un programme de test qui calcule la somme des nombres de 1 à 10
fn create_test_program() -> Result<BytecodeFile, Box<dyn std::error::Error>> {
    let mut bytecode = BytecodeFile::new();

    // Définition de la version
    bytecode.version = BytecodeVersion::new(0, 1, 0, 0);

    // Ajout de métadonnées
    bytecode.add_metadata("name", "Somme 1 à 10");
    bytecode.add_metadata("author", "PunkVM Team");

    // Initialisation des registres
    // R0 = compteur (1 à 10)
    // R1 = somme totale
    // R2 = valeur constante 10 (limite)
    // R3 = valeur constante 1 (incrément)

    // LOAD R0, 1     ; Initialiser compteur à 1
    bytecode.add_instruction(Instruction::create_reg_imm8(Opcode::Load, 0, 1));

    // LOAD R1, 0     ; Initialiser somme à 0
    bytecode.add_instruction(Instruction::create_reg_imm8(Opcode::Load, 1, 0));

    // LOAD R2, 10    ; Charger limite dans R2
    bytecode.add_instruction(Instruction::create_reg_imm8(Opcode::Load, 2, 10));

    // LOAD R3, 1     ; Charger incrément dans R3
    bytecode.add_instruction(Instruction::create_reg_imm8(Opcode::Load, 3, 1));

    // Ajouter un label pour la boucle
    bytecode.add_symbol("loop_start", 4);

    // ADD R1, R0     ; Ajouter compteur à la somme
    bytecode.add_instruction(Instruction::create_reg_reg(Opcode::Add, 1, 0));

    // ADD R0, R3     ; Incrémenter compteur
    bytecode.add_instruction(Instruction::create_reg_reg(Opcode::Add, 0, 3));

    // CMP R0, R2     ; Comparer compteur avec limite
    bytecode.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 2));

    // JMP_IF_NOT -12  ; Si pas atteint limite, retourner au début de boucle
    let loop_jmp = Instruction::new(
        Opcode::JmpIfNot,
        InstructionFormat::new(ArgType::None, ArgType::RelativeAddr),
        vec![0xF4, 0xFF, 0xFF, 0xFF], // -12 en complément à 2 (environ, à ajuster selon taille réelle)
    );
    bytecode.add_instruction(loop_jmp);

    // HALT           ; Fin du programme
    bytecode.add_instruction(Instruction::create_no_args(Opcode::Halt));

    Ok(bytecode)

    /*
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

    */









}
*/
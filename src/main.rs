//src/main.rs


use std::collections::HashMap;
use std::time::Instant;
// use PunkVM::alu::v_alu::VectorResult::Vector128;
use PunkVM::bytecode::files::{BytecodeFile, BytecodeVersion, SegmentMetadata, SegmentType};
use PunkVM::bytecode::instructions::{ArgValue, Instruction};
use PunkVM::bytecode::opcodes::Opcode;
use PunkVM::bytecode::simds::Vector128;
use PunkVM::debug::PipelineTracer;
use PunkVM::pvm::vm::{PunkVM as VM, VMConfig, VMState};
use PunkVM::pvm::vm_errors::VMResult;




fn main() -> VMResult<()> {
    println!("=== PunkVM - Test debug PunkVM ===");

    // Configuration de la VM
    let config = VMConfig {
        memory_size: 64 * 1024,        // 64 KB de mémoire
        num_registers: 19,             // 16 registres généraux + 3 spéciaux (SP, BP, RA)
        l1_cache_size: 4 * 1024,       // 4 KB de cache L1
        l2_cache_size: 16 * 1024,      // 16 KB de cache L2
        store_buffer_size: 8,          // 8 entrées dans le store buffer
        stack_size: 4 * 1024,          // 4 KB de pile
        stack_base: 0xC000,            // Base de la pile (48KB) dans la mémoire 64KB
        fetch_buffer_size: 8,          // 8 instructions dans le buffer de fetch
        btb_size: 16,                  // 16 entrées dans la BTB
        ras_size: 4,                   // 4 entrées dans le RAS
        enable_forwarding: true,       // Activer le forwarding
        enable_hazard_detection: true, // Activer la détection de hazards
        enable_tracing: true,          // Activer le traçage
    };

    let mut tracer = PipelineTracer::new(Default::default());


    // Créer une VM avec la configuration spécifiée
    println!("Initialisation de la VM...");
    let mut vm = PunkVM::pvm::vm::PunkVM::with_config(config);
    println!(
        " PunkVM initialisée avec {} registre succès",
        vm.registers.len()
    );

    // Choisir le programme de test
    // let program = forwarding_stress_test(); // Tests de forwarding intensif
    // let program = store_load_forwarding_test_8(); // Tests Store-Load forwarding  immédiat 8
    // let program = store_load_forwarding_test_16(); // Tests Store-Load forwarding immédiat 16
    // let program = store_load_forwarding_test_32(); // Tests Store-Load forwarding immédiat 32
    // let program = store_load_forwarding_test_64(); // Tests Store-Load forwarding immédiat 64

    // let program = forwarding_efficiency_test(); // Tests d'efficacité forwarding
    // let program = punk_program_3(); // Tests de branchement
    // let program= punk_program_5(); // Tests multiples branchements conditionnels et inconditionnels
    let program = simd_instruction_test();


    // let program = create_stack_test_program(); // Tests de stack machine complet avec CALL/RET
    // let program = test_basic_stack_operations(); // Test basique PUSH/POP
    // let program = test_arithmetic_with_stack(); // Test arithmétique avec pile
    // let program = test_advanced_stack_register(); // Test avancé combinaison registres/pile

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



    // // let mut vm_summary = vm.tracer.unwrap().generate_summary();
    let trace_sum = tracer.generate_summary();
    println!("Trace exportée: {:?}", trace_sum);
    //
    // tracer.export_to_csv("trace.csv")?;

    Ok(())
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
    
    // Statistiques de la hiérarchie de cache
    println!("\n-- Cache Hierarchy Performance --");
    println!("  L1 Data Hits: {}", stats.l1_data_hits);
    println!("  L1 Data Misses: {}", stats.l1_data_misses);
    println!("  L1 Instruction Hits: {}", stats.l1_inst_hits);
    println!("  L1 Instruction Misses: {}", stats.l1_inst_misses);
    println!("  L2 Hits: {}", stats.l2_hits);
    println!("  L2 Misses: {}", stats.l2_misses);
    println!("  L2 Writebacks: {}", stats.l2_writebacks);
    println!("  L2 Prefetch Hits: {}", stats.l2_prefetch_hits);
    println!("  Memory Accesses: {}", stats.memory_accesses);
    
    // Calculs des taux de hits
    let l1_total = stats.l1_data_hits + stats.l1_data_misses;
    let l1_hit_rate = if l1_total > 0 {
        stats.l1_data_hits as f64 / l1_total as f64 * 100.0
    } else {
        0.0
    };
    
    let l2_total = stats.l2_hits + stats.l2_misses;
    let l2_hit_rate = if l2_total > 0 {
        stats.l2_hits as f64 / l2_total as f64 * 100.0
    } else {
        0.0
    };
    
    println!("  L1 Data Hit Rate: {:.2}%", l1_hit_rate);
    println!("  L2 Hit Rate: {:.2}%", l2_hit_rate);
    
    if stats.average_memory_latency > 0.0 {
        println!("  Average Memory Latency: {:.2} cycles", stats.average_memory_latency);
    }
    println!("  Branches flush: {}", stats.branch_flush);
    println!("  Branche predictions: {}", stats.branch_predictor);
    println!(
        "  Branch prediction rate : {:.2}%",
        stats.branch_prediction_rate
    );
    
    // Statistiques BTB
    println!("\n-- Branch Target Buffer (BTB) --");
    println!("  BTB Hits: {}", stats.btb_hits);
    println!("  BTB Misses: {}", stats.btb_misses);
    println!("  BTB Hit Rate: {:.2}%", stats.btb_hit_rate * 100.0);
    println!("  BTB Correct Targets: {}", stats.btb_correct_targets);
    println!("  BTB Incorrect Targets: {}", stats.btb_incorrect_targets);
    println!("  BTB Accuracy: {:.2}%", stats.btb_accuracy * 100.0);

    // Calcul de quelques métriques supplémentaires
    if stats.cycles > 0 {
        let stall_rate = (stats.stalls as f64 / stats.cycles as f64) * 100.0;
        println!("  Taux de stalls: {:.2}%", stall_rate);
    }

    // Déjà affiché dans la section Cache Hierarchy ci-dessus


    // Statistique du Stack
    println!("\n===== STATISTIQUES DU STACK =====\n");
    // println!("  Taille de la pile: {} octets", stats.);
    println!(" Total de Stack Push: {}", stats.stack_pushes);
    println!(" Total de Stack Pop: {}", stats.stack_pops);
    println!(" Total Hits de Stack: {}", stats.stack_hits);
    println!(" Total Miss de Stack: {}", stats.stack_misses);
    println!(" Profondeur maximale de la pile: {}", stats.stack_max_depth);
    println!(" Profondeur actuelle de la pile: {}", stats.stack_current_depth);



    // Évaluation des performances
    println!("\n===== ÉVALUATION DES PERFORMANCES =====\n");

    // Taux de hits global de la hiérarchie de cache
    let total_cache_accesses = stats.l1_data_hits + stats.l1_data_misses + stats.l2_hits + stats.l2_misses;
    let total_cache_hits = stats.l1_data_hits + stats.l2_hits;
    let overall_hit_rate = if total_cache_accesses > 0 {
        total_cache_hits as f64 / total_cache_accesses as f64 * 100.0
    } else {
        0.0
    };
    println!("Taux de hits global (L1+L2): {:.2}%", overall_hit_rate);

    // Taux de stalls
    let stall_rate = if stats.cycles > 0 {
        stats.stalls as f64 / stats.cycles as f64 * 100.0
    } else {
        0.0
    };
    println!("Taux de stalls: {:.2}%", stall_rate);

    // Efficacité du forwarding
    println!("\n-- Analyse du Forwarding --");
    println!("Dépendances de données détectées: {}", stats.data_dependencies);
    println!("Forwards potentiels identifiés: {}", stats.potential_forwards);
    println!("Forwards effectués: {}", stats.forwards);
    println!("Vrais hazards (causant stalls): {}", stats.hazards);
    
    let forwarding_efficiency = if stats.potential_forwards > 0 {
        stats.forwards as f64 / stats.potential_forwards as f64 * 100.0
    } else {
        0.0
    };
    println!("Efficacité du forwarding: {:.2}% ({}/{})", 
            forwarding_efficiency, stats.forwards, stats.potential_forwards);
    
    // Store-Load forwarding
    println!("\n-- Store-Load Forwarding --");
    println!("Store-Load tentatives: {}", stats.store_load_attempts);
    println!("Store-Load forwards: {}", stats.store_load_forwards);
    
    let store_load_efficiency = if stats.store_load_attempts > 0 {
        stats.store_load_forwards as f64 / stats.store_load_attempts as f64 * 100.0
    } else {
        0.0
    };
    println!("Efficacité Store-Load forwarding: {:.2}% ({}/{})", 
            store_load_efficiency, stats.store_load_forwards, stats.store_load_attempts);

    println!("\n===== TEST TERMINÉ =====");
    println!("=====PunkVM=By=YmC======\n");
}


pub fn punk_program_3() -> BytecodeFile {
    let mut program = BytecodeFile::new();
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    program.add_metadata("name", "PunkVM Comprehensive Branch Test");
    program.add_metadata("description", "Test complet de tous les types de branchements conditionnels et inconditionnels");
    program.add_metadata("author", "PunkVM Team");
    program.add_metadata("test_categories", "JMP, JmpIfEqual, JmpIfNotEqual, JmpIfGreater, JmpIfLess, JmpIfGreaterEqual, JmpIfLessEqual, JmpIfZero, JmpIfNotZero, Call, Ret");

    // ============================================================================
    // SECTION 1: INITIALISATION DES REGISTRES
    // ============================================================================
    println!("=== SECTION 1: INITIALISATION ===");

    // Registres pour les comparaisons
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 10)); // R0 = 10
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 20)); // R1 = 20
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 2, 10)); // R2 = 10 (égal à R0)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 3, 5));  // R3 = 5 (plus petit que R0)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 4, 0));  // R4 = 0 (pour tests de zéro)

    // Registres pour stocker les résultats des tests
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 8, 0));  // R8 = compteur de tests réussis
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 0));  // R9 = compteur de tests échoués

    // ============================================================================
    // SECTION 2: TEST JMP (SAUT INCONDITIONNEL)
    // ============================================================================
    println!("=== SECTION 2: TEST JMP INCONDITIONNEL ===");

    let mut current_address = Instruction::calculate_current_address(&program.code);
    let jmp_instruction_size = 8;
    let jmp_target = current_address + jmp_instruction_size + 6; // Sauter par-dessus l'instruction MOV suivante
    program.add_instruction(Instruction::create_jump(current_address, jmp_target));

    // Cette instruction ne doit PAS être exécutée
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 0xFF)); // ÉCHEC si exécuté

    // Cette instruction doit être exécutée
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 10, 0x01)); // R10 = 1 (succès JMP)
    current_address = Instruction::calculate_current_address(&program.code);

    // ============================================================================
    // SECTION 3: TEST JmpIfEqual (ZF = 1)
    // ============================================================================
    println!("=== SECTION 3: TEST JmpIfEqual ===");

    // Test 1: R0 == R2 (10 == 10) → ZF = 1 → branchement PRIS
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 2));
    current_address = Instruction::calculate_current_address(&program.code);
    let jmp_instruction_size = 8;
    let jmpifequal_target_1 = current_address + jmp_instruction_size + 6;
    program.add_instruction(Instruction::create_jump_if_equal(current_address, jmpifequal_target_1));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 0xFF)); // ÉCHEC si exécuté
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 11, 0x02)); // R11 = 2 (succès)
    current_address = Instruction::calculate_current_address(&program.code);

    // Test 2: R0 == R1 (10 == 20) → ZF = 0 → branchement NON PRIS
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 1));
    current_address = Instruction::calculate_current_address(&program.code);
    let jmpifequal_target_2 = current_address + 8 + 6;
    program.add_instruction(Instruction::create_jump_if_equal(current_address, jmpifequal_target_2));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 12, 0x03)); // R12 = 3 (succès, doit être exécuté)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 13, 0x04)); // R13 = 4
    current_address = Instruction::calculate_current_address(&program.code);

    // ============================================================================
    // SECTION 4: TEST JmpIfNotEqual (ZF = 0)
    // ============================================================================
    println!("=== SECTION 4: TEST JmpIfNotEqual ===");

    // Test 1: R0 != R1 (10 != 20) → ZF = 0 → branchement PRIS
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 1));
    current_address = Instruction::calculate_current_address(&program.code);
    let jmpifnotequal_target_1 = current_address + 8 + 6;
    program.add_instruction(Instruction::create_jump_if_not_equal(current_address, jmpifnotequal_target_1));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 0xFF)); // ÉCHEC si exécuté
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 14, 0x05)); // R14 = 5 (succès)
    current_address = Instruction::calculate_current_address(&program.code);

    // Test 2: R0 != R2 (10 != 10) → ZF = 1 → branchement NON PRIS
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 2));
    current_address = Instruction::calculate_current_address(&program.code);
    let jmpifnotequal_target_2 = current_address + 8 + 6;
    program.add_instruction(Instruction::create_jump_if_not_equal(current_address, jmpifnotequal_target_2));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 15, 0x06)); // R15 = 6 (succès, doit être exécuté)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 5, 0x07)); // R5 = 7
    current_address = Instruction::calculate_current_address(&program.code);

    // ============================================================================
    // SECTION 5: TEST JmpIfGreater (ZF = 0 ET SF = 0)
    // ============================================================================
    println!("=== SECTION 5: TEST JmpIfGreater ===");

    // Test 1: R1 > R0 (20 > 10) → ZF = 0, SF = 0 → branchement PRIS
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 1, 0));
    current_address = Instruction::calculate_current_address(&program.code);
    let jmpifgreater_target_1 = current_address + 8 + 6;
    program.add_instruction(Instruction::create_jump_if_greater(current_address, jmpifgreater_target_1));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 0xFF)); // ÉCHEC si exécuté
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 6, 0x08)); // R6 = 8 (succès)
    current_address = Instruction::calculate_current_address(&program.code);

    // Test 2: R3 > R0 (5 > 10) → SF = 1 → branchement NON PRIS
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 3, 0));
    current_address = Instruction::calculate_current_address(&program.code);
    let jmpifgreater_target_2 = current_address + 8 + 6;
    program.add_instruction(Instruction::create_jump_if_greater(current_address, jmpifgreater_target_2));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 7, 0x09)); // R7 = 9 (succès, doit être exécuté)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 8, 0x0A)); // R8 = 10
    current_address = Instruction::calculate_current_address(&program.code);

    // ============================================================================
    // SECTION 6: TEST JmpIfLess (SF = 1)
    // ============================================================================
    println!("=== SECTION 6: TEST JmpIfLess ===");

    // Test 1: R3 < R0 (5 < 10) → SF = 1 → branchement PRIS
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 3, 0));
    current_address = Instruction::calculate_current_address(&program.code);
    let jmpifless_target_1 = current_address + 8 + 6;
    program.add_instruction(Instruction::create_jump_if_less(current_address, jmpifless_target_1));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 0xFF)); // ÉCHEC si exécuté
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 0x0B)); // R9 = 11 (succès)
    current_address = Instruction::calculate_current_address(&program.code);

    // Test 2: R1 < R0 (20 < 10) → SF = 0 → branchement NON PRIS
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 1, 0));
    current_address = Instruction::calculate_current_address(&program.code);
    let jmpifless_target_2 = current_address + 8 + 6;
    program.add_instruction(Instruction::create_jump_if_less(current_address, jmpifless_target_2));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 10, 0x0C)); // R10 = 12 (succès, doit être exécuté)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 11, 0x0D)); // R11 = 13
    current_address = Instruction::calculate_current_address(&program.code);

    // ============================================================================
    // SECTION 7: TEST JmpIfGreaterEqual (SF = 0 ou ZF = 1)
    // ============================================================================
    println!("=== SECTION 7: TEST JmpIfGreaterEqual ===");

    // Test 1: R1 >= R0 (20 >= 10) → SF = 0 → branchement PRIS
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 1, 0));
    current_address = Instruction::calculate_current_address(&program.code);
    let jmpifgreaterequal_target_1 = current_address + 8 + 6;
    program.add_instruction(Instruction::create_jump_if_greater_equal(current_address, jmpifgreaterequal_target_1));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 0xFF)); // ÉCHEC si exécuté
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 12, 0x0E)); // R12 = 14 (succès)
    current_address = Instruction::calculate_current_address(&program.code);

    // Test 2: R0 >= R2 (10 >= 10) → ZF = 1 → branchement PRIS aussi
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 2));
    current_address = Instruction::calculate_current_address(&program.code);
    let jmpifgreaterequal_target_2 = current_address + 8 + 6;
    program.add_instruction(Instruction::create_jump_if_greater_equal(current_address, jmpifgreaterequal_target_2));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 0xFF)); // ÉCHEC si exécuté
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 13, 0x0F)); // R13 = 15 (succès)
    current_address = Instruction::calculate_current_address(&program.code);

    // Test 3: R3 >= R0 (5 >= 10) → SF = 1 → branchement NON PRIS
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 3, 0));
    current_address = Instruction::calculate_current_address(&program.code);
    let jmpifgreaterequal_target_3 = current_address + 8 + 6;
    program.add_instruction(Instruction::create_jump_if_greater_equal(current_address, jmpifgreaterequal_target_3));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 14, 0x10)); // R14 = 16 (succès, doit être exécuté)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 15, 0x11)); // R15 = 17
    current_address = Instruction::calculate_current_address(&program.code);

    // ============================================================================
    // SECTION 8: TEST JmpIfLessEqual (SF = 1 OU ZF = 1)
    // ============================================================================
    println!("=== SECTION 8: TEST JmpIfLessEqual ===");

    // Test 1: R3 <= R0 (5 <= 10) → SF = 1 → branchement PRIS
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 3, 0));
    current_address = Instruction::calculate_current_address(&program.code);
    let jmpiflessequal_target_1 = current_address + 8 + 6;
    program.add_instruction(Instruction::create_jump_if_less_equal(current_address, jmpiflessequal_target_1));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 0xFF)); // ÉCHEC si exécuté
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 5, 0x12)); // R5 = 18 (succès)
    current_address = Instruction::calculate_current_address(&program.code);

    // Test 2: R0 <= R2 (10 <= 10) → ZF = 1 → branchement PRIS aussi
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 2));
    current_address = Instruction::calculate_current_address(&program.code);
    let jmpiflessequal_target_2 = current_address + 8 + 6;
    program.add_instruction(Instruction::create_jump_if_less_equal(current_address, jmpiflessequal_target_2));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 0xFF)); // ÉCHEC si exécuté
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 6, 0x13)); // R6 = 19 (succès)
    current_address = Instruction::calculate_current_address(&program.code);

    // Test 3: R1 <= R0 (20 <= 10) → SF = 0, ZF = 0 → branchement NON PRIS
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 1, 0));
    current_address = Instruction::calculate_current_address(&program.code);
    let jmpiflessequal_target_3 = current_address + 8 + 6;
    program.add_instruction(Instruction::create_jump_if_less_equal(current_address, jmpiflessequal_target_3));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 7, 0x14)); // R7 = 20 (succès, doit être exécuté)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 8, 0x15)); // R8 = 21
    current_address = Instruction::calculate_current_address(&program.code);

    // ============================================================================
    // SECTION 9: TEST JmpIfZero (ZF = 1)
    // ============================================================================
    println!("=== SECTION 9: TEST JmpIfZero ===");

    // Test 1: R0 == R2 (10 == 10) → ZF = 1 → branchement PRIS
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 2));
    current_address = Instruction::calculate_current_address(&program.code);
    let jmpifzero_target_1 = current_address + 8 + 6;
    program.add_instruction(Instruction::create_jump_if_zero(current_address, jmpifzero_target_1));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 0xFF)); // ÉCHEC si exécuté
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 0x16)); // R9 = 22 (succès)
    current_address = Instruction::calculate_current_address(&program.code);

    // Test 2: R0 != R1 (10 != 20) → ZF = 0 → branchement NON PRIS
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 1));
    current_address = Instruction::calculate_current_address(&program.code);
    let jmpifzero_target_2 = current_address + 8 + 6;
    program.add_instruction(Instruction::create_jump_if_zero(current_address, jmpifzero_target_2));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 10, 0x17)); // R10 = 23 (succès, doit être exécuté)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 11, 0x18)); // R11 = 24
    current_address = Instruction::calculate_current_address(&program.code);

    // ============================================================================
    // SECTION 10: TEST JmpIfNotZero (ZF = 0)
    // ============================================================================
    println!("=== SECTION 10: TEST JmpIfNotZero ===");

    // Test 1: R0 != R1 (10 != 20) → ZF = 0 → branchement PRIS
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 1));
    current_address = Instruction::calculate_current_address(&program.code);
    let jmpifnotzero_target_1 = current_address + 8 + 6;
    program.add_instruction(Instruction::create_jump_if_not_zero(current_address, jmpifnotzero_target_1));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 0xFF)); // ÉCHEC si exécuté
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 12, 0x19)); // R12 = 25 (succès)
    current_address = Instruction::calculate_current_address(&program.code);

    // Test 2: R0 == R2 (10 == 10) → ZF = 1 → branchement NON PRIS
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 2));
    current_address = Instruction::calculate_current_address(&program.code);
    let jmpifnotzero_target_2 = current_address + 8 + 6;
    program.add_instruction(Instruction::create_jump_if_not_zero(current_address, jmpifnotzero_target_2));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 13, 0x1A)); // R13 = 26 (succès, doit être exécuté)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 14, 0x1B)); // R14 = 27
    current_address = Instruction::calculate_current_address(&program.code);

    // ============================================================================
    // SECTION 11: TEST DE BOUCLE (Pattern pour le prédicteur)
    // ============================================================================
    println!("=== SECTION 11: TEST DE BOUCLE ===");

    // Initialisation du compteur de boucle
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 15, 3)); // R15 = 3 (compteur)

    // Début de la boucle - cette étiquette sera utilisée pour le branchement arrière
    let loop_start_instruction_index = program.code.len();
    current_address = Instruction::calculate_current_address(&program.code);

    // Corps de la boucle
    program.add_instruction(Instruction::create_reg_reg(Opcode::Sub, 15, 4)); // R15 = R15 - 1 (R4 = 0, donc R15 - 0, mais on veut R15-1)
    current_address = Instruction::calculate_current_address(&program.code);

    // Pour décrémenter correctement, on doit d'abord mettre 1 dans un registre
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 4, 1)); // R4 = 1
    program.add_instruction(Instruction::create_reg_reg(Opcode::Sub, 15, 4)); // R15 = R15 - 1
    current_address = Instruction::calculate_current_address(&program.code);

    // Comparer avec 0
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 4, 0)); // R4 = 0 pour comparaison
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 15, 4)); // Compare R15 avec 0
    current_address = Instruction::calculate_current_address(&program.code);

    // Calculer l'offset pour retourner au début de la boucle
    let current_instruction_index = program.code.len() + 1; // +1 car on ajoute l'instruction de branchement
    let loop_body_size = current_instruction_index - loop_start_instruction_index;
    let backward_offset = -(loop_body_size as i32 * 6 + 8); // chaque instruction fait ~6 bytes, +8 pour l'instruction de branchement
    let  jmpifnotzero_loop =  current_address as i32 + backward_offset as i32 ;

    // Branchement conditionnel vers le début de la boucle si R15 != 0
    program.add_instruction(Instruction::create_jump_if_not_zero(0, 0));
    current_address = Instruction::calculate_current_address(&program.code);

    // ============================================================================
    // SECTION 12: TEST CALL/RET (Si implémenté)
    // ============================================================================
    println!("=== SECTION 12: TEST CALL/RET ===");
    current_address = Instruction::calculate_current_address(&program.code);
    let call_target = current_address + 8 + 6 ;
    // Sauter par-dessus la fonction pour aller au call
    program.add_instruction(Instruction::create_jump(current_address, call_target)); // Sauter la fonction
    current_address = Instruction::calculate_current_address(&program.code);

    // FONCTION: simple_function
    // Fonction qui met 0xFF dans R5 et retourne
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 5, 0xFF)); // R5 = 255
    program.add_instruction(Instruction::create_no_args(Opcode::Ret)); // Retour
    current_address = Instruction::calculate_current_address(&program.code);

    // Appel de la fonction (si CALL est implémenté)
    current_address = Instruction::calculate_current_address(&program.code);
    let function_offset = -12i32;// Retourner à la fonction
    program.add_instruction(Instruction::create_call(function_offset as u32));

    // ============================================================================
    // SECTION 13: FINALISATION ET VÉRIFICATION
    // ============================================================================
    println!("=== SECTION 13: FINALISATION ===");
    current_address = Instruction::calculate_current_address(&program.code);

    // Marquer la fin des tests
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 0xFE)); // R0 = 254 (marqueur de fin)

    // Fin du programme
    program.add_instruction(Instruction::create_no_args(Opcode::Halt));

    // ============================================================================
    // CONFIGURATION DES SEGMENTS
    // ============================================================================
    let total_code_size: u32 = program
        .code
        .iter()
        .map(|instr| instr.total_size() as u32)
        .sum();
    program.segments = vec![SegmentMetadata::new(SegmentType::Code, 0, total_code_size, 0)];

    let data_size = 512; // Taille augmentée pour plus de données
    let data_segment = SegmentMetadata::new(SegmentType::Data, 0, data_size, 0x1000);
    program.segments.push(data_segment);
    program.data = vec![0; data_size as usize];

    // ============================================================================
    // AFFICHAGE DE LA CARTE DES INSTRUCTIONS
    // ============================================================================
    println!("\n=== CARTE COMPLÈTE DES INSTRUCTIONS ===");
    let mut addr = 0u32;
    let mut section_counters = HashMap::new();

    for (idx, instr) in program.code.iter().enumerate() {
        let size = instr.total_size();

        // Déterminer la section basée sur l'index d'instruction
        let section = match idx {
            0..=6 => "INIT",
            7..=9 => "JMP",
            10..=15 => "JmpIfEqual",
            16..=21 => "JmpIfNotEqual",
            22..=27 => "JmpIfGreater",
            28..=33 => "JmpIfLess",
            34..=42 => "JmpIfGreaterEqual",
            43..=51 => "JmpIfLessEqual",
            52..=57 => "JmpIfZero",
            58..=63 => "JmpIfNotZero",
            64..=70 => "LOOP",
            71..=75 => "CALL/RET",
            _ => "FINAL",
        };

        *section_counters.entry(section).or_insert(0) += 1;

        println!(
            "Instruction {:2}: [{}] Adresse 0x{:04X}-0x{:04X} (taille {:2}): {:?}",
            idx,
            section,
            addr,
            addr + size as u32 - 1,
            size,
            instr.opcode
        );

        // Affichage spécial pour les branchements
        if instr.opcode.is_branch() {
            if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
                let target = (addr + size as u32) as i64 + offset as i64;
                println!(
                    "      -> Branchement relatif: offset={:+}, target=0x{:04X}",
                    offset, target
                );
            }
        }

        addr += size as u32;
    }

    println!("\n=== RÉSUMÉ DES SECTIONS ===");
    for (section, count) in section_counters {
        println!("{}: {} instructions", section, count);
    }
    println!(
        "TOTAL: {} instructions, {} bytes",
        program.code.len(),
        addr
    );

    println!("\n=== TESTS ATTENDUS ===");
    println!("Après exécution, les registres suivants devraient contenir:");
    println!("R0  = 254 (0xFE) - Marqueur de fin");
    println!("R10 = 1   (0x01) - Test JMP réussi");
    println!("R11 = 2   (0x02) - Test JmpIfEqual réussi");
    println!("R12 = 3   (0x03) - Test JmpIfEqual (non pris) réussi");
    println!("R14 = 5   (0x05) - Test JmpIfNotEqual réussi");
    println!("R15 = 6   (0x06) - Test JmpIfNotEqual (non pris) réussi");
    println!("Et ainsi de suite...");
    println!("Aucun registre ne devrait contenir 0xFF (échec)");

    program
}

/// Crée un programme de test pour valider la stack machine
fn create_stack_test_program() -> BytecodeFile {
    let mut program = BytecodeFile::new();
    let mut current_address: u32 = 0;
    
    // Initialiser la version et les métadonnées
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    program.add_metadata("name", "PunkVM Stack Machine Test");
    program.add_metadata("description", "Test complet de la stack machine avec PUSH/POP/CALL/RET");
    program.add_metadata("author", "PunkVM Team");
    
    println!("\n=== CRÉATION DU PROGRAMME DE TEST STACK MACHINE ===");
    
    // ============================================================================
    // SECTION 1: INITIALISATION
    // ============================================================================
    println!("=== SECTION 1: INITIALISATION ===");
    
    // Initialiser des valeurs dans les registres
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 42));   // R0 = 42
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 99));   // R1 = 99
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 2, 0));    // R2 = 0 (pour résultat)
    
    // ============================================================================
    // SECTION 2: TEST PUSH/POP BASIQUE
    // ============================================================================
    println!("=== SECTION 2: TEST PUSH/POP BASIQUE ===");

    // Test PUSH R0 (42)
    program.add_instruction(Instruction::create_push_register(0));

    // Test PUSH R1 (99)
    program.add_instruction(Instruction::create_push_register(1));

    // Test PUSH immédiat 77
    program.add_instruction(Instruction::create_push_immediate8(7, 77));

    // Test POP R2 (devrait récupérer 77)
    program.add_instruction(Instruction::create_pop_register(2));

    // Test POP R3 (devrait récupérer 99)
    program.add_instruction(Instruction::create_pop_register(3));

    // Test POP R4 (devrait récupérer 42)
    program.add_instruction(Instruction::create_pop_register(4));
    
    //Vérification: R2=77, R3=99, R4=42
    
    // ============================================================================
    // SECTION 3: TEST CALL/RET SIMPLE
    // ============================================================================
    println!("=== SECTION 3: TEST CALL/RET SIMPLE ===");
    
    // Préparer des valeurs pour la fonction
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 5, 10));   // R5 = 10
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 6, 20));   // R6 = 20
    
    // Calculer l'adresse actuelle avant le CALL
    current_address = Instruction::calculate_current_address(&program.code);
    
    // CALL vers la fonction - calculer l'adresse de la fonction ADD
    // La fonction sera après: CALL (8 bytes) + MOV (6 bytes) = +14 bytes d'ici
    let function_address = current_address + 8 + 6; // CALL + MOV
    program.add_instruction(Instruction::create_call_relative(current_address, function_address));
    
    // Instructions après le CALL
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 8, 0xAA)); // R8 = 0xAA (marqueur retour OK)
    
    // FONCTION: add_function (exécutée directement, pas de JMP pour l'éviter)
    // Additionne R5 et R6, met le résultat dans R7
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 7, 5, 6)); // R7 = R5 + R6
    program.add_instruction(Instruction::create_return()); // RET
    
    // ============================================================================
    // SECTION 4: TEST APPELS IMBRIQUÉS
    // ============================================================================
    println!("=== SECTION 4: TEST APPELS IMBRIQUÉS ===");
    
    // Fonction principale qui appelle une sous-fonction
    current_address = Instruction::calculate_current_address(&program.code);
    let nested_function_address = current_address + 6 + 8; // MOV + JMP
    program.add_instruction(Instruction::create_call_relative(current_address, nested_function_address));
    
    // Marquer que nous sommes revenus de l'appel imbriqué
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 0xBB)); // R9 = 0xBB
    
    // Jump pour éviter les fonctions
    current_address = Instruction::calculate_current_address(&program.code);
    let skip_nested_offset = 40;
    program.add_instruction(Instruction::create_jump(current_address, current_address + skip_nested_offset));
    
    // FONCTION NIVEAU 1: Appelle une autre fonction
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 10, 1));   // R10 = 1 (niveau 1)
    current_address = Instruction::calculate_current_address(&program.code);
    let inner_function_address = current_address + 6; // RET instruction
    program.add_instruction(Instruction::create_call_relative(current_address, inner_function_address));
    program.add_instruction(Instruction::create_return());
    
    // FONCTION NIVEAU 2: Fonction finale
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 11, 2));   // R11 = 2 (niveau 2)
    program.add_instruction(Instruction::create_return());
    
    // ============================================================================
    // SECTION 5: TEST PILE AVEC BOUCLE
    // ============================================================================
    println!("=== SECTION 5: TEST PILE AVEC BOUCLE ===");
    
    // Initialiser compteur et limite
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 12, 0));   // R12 = 0 (compteur)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 13, 5));   // R13 = 5 (limite)
    
    // Début de la boucle
    current_address = Instruction::calculate_current_address(&program.code);
    let loop_start = current_address;
    
    // Push le compteur sur la pile
    program.add_instruction(Instruction::create_push_register(12));
    
    // Incrémenter le compteur
    program.add_instruction(Instruction::create_reg_reg(Opcode::Inc, 12, 12));
    
    // Comparer avec la limite
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 12, 13));
    
    // Si pas égal, continuer la boucle
    current_address = Instruction::calculate_current_address(&program.code);
    let loop_offset = loop_start as i32 - (current_address + 8) as i32;
    program.add_instruction(Instruction::create_jump_if_not_equal(current_address, loop_start));
    
    // Après la boucle, dépiler les valeurs (on devrait avoir 0,1,2,3,4 sur la pile)
    program.add_instruction(Instruction::create_pop_register(14));  // R14 = 4
    program.add_instruction(Instruction::create_pop_register(15));  // R15 = 3
    
    // ============================================================================
    // SECTION 6: FINALISATION
    // ============================================================================
    println!("=== SECTION 6: FINALISATION ===");
    
    // Marquer la fin des tests
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 0xFE)); // R0 = 254 (marqueur de fin)
    
    // Fin du programme
    program.add_instruction(Instruction::create_no_args(Opcode::Halt));
    
    // ============================================================================
    // CONFIGURATION DES SEGMENTS
    // ============================================================================
    let total_code_size: u32 = program
        .code
        .iter()
        .map(|instr| instr.total_size() as u32)
        .sum();
    program.segments = vec![SegmentMetadata::new(SegmentType::Code, 0, total_code_size, 0)];
    
    // ============================================================================
    // AFFICHAGE DE LA CARTE DES INSTRUCTIONS
    // ============================================================================
    println!("\n=== CARTE DES INSTRUCTIONS STACK TEST ===");
    let mut addr = 0u32;
    for (idx, instr) in program.code.iter().enumerate() {
        let size = instr.total_size();
        println!(
            "Instruction {:2}: Adresse 0x{:04X}-0x{:04X} (taille {:2}): {:?}",
            idx,
            addr,
            addr + size as u32 - 1,
            size,
            instr.opcode
        );


        if instr.opcode.is_branch() {
            if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
                let target = (addr + size as u32) as i64 + offset as i64;
                println!(
                    "      -> Branchement relatif: offset={:+}, target=0x{:04X}",
                    offset, target
                );
            }
        }
        addr += size as u32;
    }
    
    println!("\n=== RÉSULTATS ATTENDUS ===");
    println!("R0  = 254 (0xFE) - Marqueur de fin");
    println!("R2  = 77  - Pop immédiat");
    println!("R3  = 99  - Pop de R1");
    println!("R4  = 42  - Pop de R0");
    println!("R7  = 30  - Résultat addition (10+20)");
    println!("R8  = 170 (0xAA) - Retour de CALL OK");
    println!("R9  = 187 (0xBB) - Retour appels imbriqués OK");
    println!("R10 = 1   - Fonction niveau 1 exécutée");
    println!("R11 = 2   - Fonction niveau 2 exécutée");
    println!("R14 = 4   - Dernière valeur empilée");
    println!("R15 = 3   - Avant-dernière valeur empilée");
    
    program
}

/// Programme 1: Test basique des opérations PUSH/POP
fn test_basic_stack_operations() -> BytecodeFile {
    let mut program = BytecodeFile::new();
    
    // Initialiser la version et les métadonnées
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    program.add_metadata("name", "PunkVM Basic Stack Test");
    program.add_metadata("description", "Test des opérations PUSH/POP basiques avec registres et immédiat");
    program.add_metadata("author", "PunkVM Team");
    
    println!("\n=== PROGRAMME 1: TEST BASIQUE STACK ===");
    
    // Initialiser des valeurs dans les registres
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 100));  // R0 = 100
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 200));  // R1 = 200
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 2, 0));    // R2 = 0
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 3, 0));    // R3 = 0
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 4, 0));    // R4 = 0
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 5, 0));    // R5 = 0
    
    // Test 1: PUSH registre
    println!("Test 1: PUSH registres");
    program.add_instruction(Instruction::create_push_register(0));    // PUSH R0 (100)
    program.add_instruction(Instruction::create_push_register(1));    // PUSH R1 (200)
    
    // Test 2: PUSH immédiat (utilise maintenant le nouveau format)
    println!("Test 2: PUSH immédiat");
    program.add_instruction(Instruction::create_push_immediate8(7, 77));   // PUSH imm 77 dans R7
    program.add_instruction(Instruction::create_push_immediate8(8, 88));   // PUSH imm 88 dans R8
    
    // Test 3: POP dans l'ordre inverse
    println!("Test 3: POP dans l'ordre inverse");
    program.add_instruction(Instruction::create_pop_register(2));     // POP R2 (devrait être 88)
    program.add_instruction(Instruction::create_pop_register(3));     // POP R3 (devrait être 77)
    program.add_instruction(Instruction::create_pop_register(4));     // POP R4 (devrait être 200)
    program.add_instruction(Instruction::create_pop_register(5));     // POP R5 (devrait être 100)
    
    // Marquer la fin
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 15, 0xFF)); // R15 = 0xFF (marqueur)
    program.add_instruction(Instruction::create_no_args(Opcode::Halt));
    
    // Configuration des segments
    let total_code_size: u32 = program.code.iter()
        .map(|instr| instr.total_size() as u32)
        .sum();
    program.segments = vec![SegmentMetadata::new(SegmentType::Code, 0, total_code_size, 0)];
    
    println!("\n=== RÉSULTATS ATTENDUS PROGRAMME 1 ===");
    println!("R0  = 100 - Valeur initiale");
    println!("R1  = 200 - Valeur initiale");
    println!("R2  = 88  - Pop immédiat 88");
    println!("R3  = 77  - Pop immédiat 77");
    println!("R4  = 200 - Pop R1");
    println!("R5  = 100 - Pop R0");
    println!("R15 = 255 - Marqueur de fin");
    
    program
}

/// Programme 2: Test arithmétique avec pile
fn test_arithmetic_with_stack() -> BytecodeFile {
    let mut program = BytecodeFile::new();
    
    // Initialiser la version et les métadonnées
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    program.add_metadata("name", "PunkVM Arithmetic Stack Test");
    program.add_metadata("description", "Test des opérations arithmétiques utilisant la pile");
    program.add_metadata("author", "PunkVM Team");
    
    println!("\n=== PROGRAMME 2: TEST ARITHMÉTIQUE AVEC STACK ===");
    
    // Initialiser des valeurs
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 10));   // R0 = 10
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 20));   // R1 = 20
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 2, 30));   // R2 = 30
    
    // Sauvegarder les valeurs sur la pile
    println!("Sauvegarde des valeurs sur la pile");
    program.add_instruction(Instruction::create_push_register(0));    // PUSH R0 (10)
    program.add_instruction(Instruction::create_push_register(1));    // PUSH R1 (20)
    program.add_instruction(Instruction::create_push_register(2));    // PUSH R2 (30)
    
    // Effectuer des calculs qui écrasent les registres
    println!("Calculs arithmétiques");
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 3, 0, 1));  // R3 = R0 + R1 = 30
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Mul, 4, 1, 2));  // R4 = R1 * R2 = 600
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Sub, 5, 2, 0));  // R5 = R2 - R0 = 20
    
    // Écraser les registres originaux
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 0));   // R0 = 0
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 0));   // R1 = 0
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 2, 0));   // R2 = 0
    
    // Restaurer depuis la pile
    println!("Restauration depuis la pile");
    program.add_instruction(Instruction::create_pop_register(2));     // POP R2 (30)
    program.add_instruction(Instruction::create_pop_register(1));     // POP R1 (20)
    program.add_instruction(Instruction::create_pop_register(0));     // POP R0 (10)
    
    // Vérifier avec une nouvelle opération
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 6, 0, 1));  // R6 = R0 + R1 = 30
    
    // Marquer la fin
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 15, 0xAA)); // R15 = 0xAA
    program.add_instruction(Instruction::create_no_args(Opcode::Halt));
    
    // Configuration des segments
    let total_code_size: u32 = program.code.iter()
        .map(|instr| instr.total_size() as u32)
        .sum();
    program.segments = vec![SegmentMetadata::new(SegmentType::Code, 0, total_code_size, 0)];
    
    println!("\n=== RÉSULTATS ATTENDUS PROGRAMME 2 ===");
    println!("R0  = 10  - Restauré depuis pile");
    println!("R1  = 20  - Restauré depuis pile");
    println!("R2  = 30  - Restauré depuis pile");
    println!("R3  = 30  - R0 + R1");
    println!("R4  = 600 - R1 * R2");
    println!("R5  = 20  - R2 - R0");
    println!("R6  = 30  - R0 + R1 (après restauration)");
    println!("R15 = 170 (0xAA) - Marqueur");
    
    program
}

/// Programme 3: Test combinaison avancée registres et pile
fn test_advanced_stack_register() -> BytecodeFile {
    let mut program = BytecodeFile::new();
    
    // Initialiser la version et les métadonnées
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    program.add_metadata("name", "PunkVM Advanced Stack-Register Test");
    program.add_metadata("description", "Test avancé de combinaison registres et pile avec boucle");
    program.add_metadata("author", "PunkVM Team");
    
    println!("\n=== PROGRAMME 3: TEST COMBINAISON AVANCÉE ===");
    
    // Test : Calcul de factorielle 5 avec pile
    // Utilise la pile pour sauvegarder les résultats intermédiaires
    
    // Initialiser
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 5));    // R0 = 5 (n)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 1));    // R1 = 1 (résultat)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 2, 1));    // R2 = 1 (compteur)
    
    // Boucle : calculer factorielle en empilant les valeurs intermédiaires
    let mut current_address = Instruction::calculate_current_address(&program.code);
    let loop_start = current_address;
    
    // Corps de la boucle
    program.add_instruction(Instruction::create_push_register(2));              // PUSH compteur
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Mul, 1, 1, 2)); // R1 = R1 * R2
    program.add_instruction(Instruction::create_reg_reg(Opcode::Inc, 2, 2));    // R2++
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 2, 0));    // Compare R2 avec R0
    
    // Saut conditionnel
    current_address = Instruction::calculate_current_address(&program.code);
    let loop_offset = loop_start as i32 - (current_address + 8) as i32;
    program.add_instruction(Instruction::create_jump_if_less_equal(current_address, loop_start));
    
    // Sauvegarder le résultat final
    program.add_instruction(Instruction::create_push_register(1));              // PUSH résultat (120)
    
    // Dépiler les valeurs empilées (5, 4, 3, 2, 1)
    program.add_instruction(Instruction::create_pop_register(10));  // R10 = dernier (5)
    program.add_instruction(Instruction::create_pop_register(11));  // R11 = 4
    program.add_instruction(Instruction::create_pop_register(12));  // R12 = 3
    program.add_instruction(Instruction::create_pop_register(13));  // R13 = 2
    program.add_instruction(Instruction::create_pop_register(14));  // R14 = 1
    program.add_instruction(Instruction::create_pop_register(15));  // R15 = résultat (120)
    
    // Fin
    program.add_instruction(Instruction::create_no_args(Opcode::Halt));
    
    // Configuration des segments
    let total_code_size: u32 = program.code.iter()
        .map(|instr| instr.total_size() as u32)
        .sum();
    program.segments = vec![SegmentMetadata::new(SegmentType::Code, 0, total_code_size, 0)];
    
    println!("\n=== RÉSULTATS ATTENDUS PROGRAMME 3 ===");
    println!("R0  = 5   - n (inchangé)");
    println!("R1  = 120 - 5! = 120");
    println!("R2  = 6   - Compteur final");
    println!("R10 = 120 - Résultat dépilé");
    println!("R11 = 5   - Valeur dépilée");
    println!("R12 = 4   - Valeur dépilée");
    println!("R13 = 3   - Valeur dépilée");
    println!("R14 = 2   - Valeur dépilée");
    println!("R15 = 1   - Valeur dépilée");
    
    program
}

/// Test de stress pour le forwarding avec chaînes de dépendances RAW
fn forwarding_stress_test() -> BytecodeFile {
    let mut program = BytecodeFile::new();
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    program.add_metadata("name", "PunkVM Forwarding Stress Test");
    program.add_metadata("description", "Test intensif de toutes les formes de forwarding");
    program.add_metadata("author", "PunkVM Team");
    
    println!("=== CRÉATION DU TEST DE STRESS FORWARDING ===");
    
    // === Test 1: Chaîne simple de dépendances RAW (Execute→Execute) ===
    println!("Test 1: Chaîne Execute→Execute forwarding");
    // Instruction 1: MOV R0, 100
    program.add_instruction(Instruction::create_reg_imm16(Opcode::Mov, 0, 100));
    
    // Instruction 2: ADD R0, R0, R0 (dépendance sur R0 depuis instr 1) 
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 0, 0, 0));
    
    // Instruction 3: MUL R0, R0, R0 (dépendance sur R0 depuis instr 2)
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Mul, 0, 0, 0));
    
    // Instruction 4: SUB R0, R0, #1 (dépendance sur R0 depuis instr 3)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Sub, 0, 1));
    
    // === Test 2: Dépendances multiples (Memory→Execute) ===
    println!("Test 2: Memory→Execute forwarding");
    // Instruction 5: MOV R1, 50
    program.add_instruction(Instruction::create_reg_imm16(Opcode::Mov, 1, 50));
    
    // Instruction 6: MOV R2, 25
    program.add_instruction(Instruction::create_reg_imm16(Opcode::Mov, 2, 25));
    
    // Instruction 7: ADD R3, R1, R2 (dépendance sur R1 et R2)
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 3, 1, 2));
    
    // === Test 3: Store-Load pattern pour tester le forwarding mémoire ===
    println!("Test 3: Store-Load forwarding");
    // Instruction 8: MOV R4, 0x1000 (adresse de base)
    program.add_instruction(Instruction::create_reg_imm16(Opcode::Mov, 4, 0x1000));
    
    // Instruction 9: STORE [R4], R3 (stocker le résultat)
    program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 3, 4, 0));
    
    // Instruction 10: LOAD R5, [R4] (charger depuis la même adresse)
    program.add_instruction(Instruction::create_load_reg_offset(5, 4, 0));
    
    // Instruction 11: ADD R6, R5, R0 (utiliser la valeur chargée)
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 6, 5, 0));
    
    // === Test 4: Chaîne longue de dépendances ===
    println!("Test 4: Chaîne longue de dépendances");
    // Série d'instructions qui créent une chaîne de 8 dépendances
    // D'abord, mettre 1 dans un registre temporaire
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 6, 1));  // R6 = 1
    for i in 7..15 {
        // ADD Ri, Ri-1, R6 (chaque registre dépend du précédent)
        program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, i, i-1, 6));
    }
    
    // === Test 5: Dépendances croisées ===
    println!("Test 5: Dépendances croisées");
    // MOV R15, 42
    program.add_instruction(Instruction::create_reg_imm16(Opcode::Mov, 15, 42));
    
    // ADD R16, R15, R14 (dépendance sur R15 immédiate et R14 à distance)
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 16, 15, 14));
    
    // SUB R17, R16, R15 (dépendances multiples récentes)
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Sub, 17, 16, 15));
    
    // === Fin du test ===
    program.add_instruction(Instruction::create_no_args(Opcode::Halt));
    
    // Configuration des segments
    let total_code_size: u32 = program.code.iter()
        .map(|instr| instr.total_size() as u32)
        .sum();
    program.segments = vec![SegmentMetadata::new(SegmentType::Code, 0, total_code_size, 0)];
    
    println!("=== RÉSULTATS ATTENDUS FORWARDING STRESS ===");
    println!("R0  = 39999 (100 + 100 = 200, 200 * 200 = 40000, 40000 - 1 = 39999)");
    println!("R1  = 50");
    println!("R2  = 25");
    println!("R3  = 75 (50 + 25)");
    println!("R4  = 0x1000");
    println!("R5  = 75 (valeur chargée depuis [R4])");
    println!("R6  = 40074 (75 + 39999)");
    println!("R7  = 76 (75 + 1)");
    println!("R8  = 77 (76 + 1)");
    println!("R9  = 78 (77 + 1)");
    println!("...etc");
    println!("R15 = 42");
    println!("R16 = 124 (42 + 82)");
    println!("R17 = 82 (124 - 42)");
    
    program
}/// Test spécifique pour le Store-Load forwarding avec imm8
fn store_load_forwarding_test_8() -> BytecodeFile {
    let mut program = BytecodeFile::new();
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    program.add_metadata("name", "Store-Load Forwarding Test");
    program.add_metadata("description", "Test spécifique pour le forwarding Store-Load");
    program.add_metadata("author", "PunkVM Team");

    println!("=== CRÉATION DU TEST STORE-LOAD FORWARDING ===");

    // Initialisation
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 0x20)); // Adresse de base
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 123));    // Valeur à stocker

    // Test 1: Store immédiatement suivi d'un Load à la même adresse
    println!("Test 1: Store→Load immédiat");
    program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 1, 0, 0));         // STORE [R0], R1
    program.add_instruction(Instruction::create_load_reg_offset(2, 0, 0));          // LOAD R2, [R0]

    // Test 2: Store avec offset, puis Load avec même offset
    println!("Test 2: Store→Load avec offset");
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 3, 255));    // Nouvelle valeur
    program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 3, 0, 4));         // STORE [R0+4], R3
    program.add_instruction(Instruction::create_load_reg_offset(4, 0, 4));          // LOAD R4, [R0+4]

    // Test 3: Store puis utilisation immédiate de la valeur chargée
    println!("Test 3: Store→Load→Use");
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 5, 215));    // Nouvelle valeur
    program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 5, 0, 8));         // STORE [R0+8], R5
    program.add_instruction(Instruction::create_load_reg_offset(6, 0, 8));          // LOAD R6, [R0+8]
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 7, 6, 1)); // ADD R7, R6, R1
    // program.add_instruction(PunkVM::bytecode::instructions::Instruction(Opcode::Add, 7, 6, 1)); // ADD R7, R6, R1

    // Test 4: Stores multiples à des adresses différentes
    println!("Test 4: Stores multiples");
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 8, 100));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 200));
    program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 8, 0, 12));        // STORE [R0+12], R8
    program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 9, 0, 16));        // STORE [R0+16], R9
    program.add_instruction(Instruction::create_load_reg_offset(10, 0, 12));        // LOAD R10, [R0+12]
    program.add_instruction(Instruction::create_load_reg_offset(11, 0, 16));        // LOAD R11, [R0+16]

    program.add_instruction(Instruction::create_no_args(Opcode::Halt));

    // Configuration des segments
    let total_code_size: u32 = program.code.iter()
        .map(|instr| instr.total_size() as u32)
        .sum();
    program.segments = vec![SegmentMetadata::new(SegmentType::Code, 0, total_code_size, 0)];

    println!("=== RÉSULTATS ATTENDUS STORE-LOAD ===");
    println!("R0  = 0x2000 (adresse de base)");
    println!("R1  = 123");
    println!("R2  = 123 (forwardé depuis store)");
    println!("R3  = 456");
    println!("R4  = 456 (forwardé depuis store)");
    println!("R5  = 789");
    println!("R6  = 789 (forwardé depuis store)");
    println!("R7  = 912 (789 + 123)");
    println!("R8  = 100");
    println!("R9  = 200");
    println!("R10 = 100 (forwardé depuis store)");
    println!("R11 = 200 (forwardé depuis store)");

    program
}

/// Test spécifique pour le Store-Load forwarding avec immediate 16bit
fn store_load_forwarding_test_16() -> BytecodeFile {
    let mut program = BytecodeFile::new();
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    program.add_metadata("name", "Store-Load Forwarding Test");
    program.add_metadata("description", "Test spécifique pour le forwarding Store-Load");
    program.add_metadata("author", "PunkVM Team");
    
    println!("=== CRÉATION DU TEST STORE-LOAD FORWARDING ===");
    
    // Initialisation
    program.add_instruction(Instruction::create_reg_imm16(Opcode::Mov, 0, 0x2000)); // Adresse de base
    program.add_instruction(Instruction::create_reg_imm16(Opcode::Mov, 1, 18599));    // Valeur à stocker
    
    // Test 1: Store immédiatement suivi d'un Load à la même adresse
    println!("Test 1: Store→Load immédiat");
    program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 1, 0, 0));         // STORE [R0], R1
    program.add_instruction(Instruction::create_load_reg_offset(2, 0, 0));          // LOAD R2, [R0]
    
    // Test 2: Store avec offset, puis Load avec même offset
    println!("Test 2: Store→Load avec offset");
    program.add_instruction(Instruction::create_reg_imm16(Opcode::Mov, 3, 45645));    // Nouvelle valeur
    program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 3, 0, 4));         // STORE [R0+4], R3
    program.add_instruction(Instruction::create_load_reg_offset(4, 0, 4));          // LOAD R4, [R0+4]
    
    // Test 3: Store puis utilisation immédiate de la valeur chargée
    println!("Test 3: Store→Load→Use");
    program.add_instruction(Instruction::create_reg_imm16(Opcode::Mov, 5, 7809));    // Nouvelle valeur
    program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 5, 0, 8));         // STORE [R0+8], R5
    program.add_instruction(Instruction::create_load_reg_offset(6, 0, 8));          // LOAD R6, [R0+8]
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 7, 6, 1)); // ADD R7, R6, R1
    
    // Test 4: Stores multiples à des adresses différentes
    println!("Test 4: Stores multiples");
    program.add_instruction(Instruction::create_reg_imm16(Opcode::Mov, 8, 10140));
    program.add_instruction(Instruction::create_reg_imm16(Opcode::Mov, 9, 29800));
    program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 8, 0, 12));        // STORE [R0+12], R8
    program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 9, 0, 16));        // STORE [R0+16], R9
    program.add_instruction(Instruction::create_load_reg_offset(10, 0, 12));        // LOAD R10, [R0+12]
    program.add_instruction(Instruction::create_load_reg_offset(11, 0, 16));        // LOAD R11, [R0+16]
    
    program.add_instruction(Instruction::create_no_args(Opcode::Halt));
    
    // Configuration des segments
    let total_code_size: u32 = program.code.iter()
        .map(|instr| instr.total_size() as u32)
        .sum();
    program.segments = vec![SegmentMetadata::new(SegmentType::Code, 0, total_code_size, 0)];
    
    println!("=== RÉSULTATS ATTENDUS STORE-LOAD ===");
    println!("R0  = 0x2000 (adresse de base)");
    println!("R1  = 18599");
    println!("R2  = 18599 (forwardé depuis store)");
    println!("R3  = 45645");
    println!("R4  = 45645 (forwardé depuis store)");
    println!("R5  = 7809");
    println!("R6  = 7809 (forwardé depuis store)");
    println!("R7  = 7998 (7809 + 18599)");
    println!("R8  = 10140");
    println!("R9  = 29800");
    println!("R10 = 10140 (forwardé depuis store)");
    println!("R11 = 29800 (forwardé depuis store)");

    program
}


/// Test spécifique pour le Store-Load forwarding avec immediate 32bit
fn store_load_forwarding_test_32() -> BytecodeFile {
    let mut program = BytecodeFile::new();
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    program.add_metadata("name", "Store-Load Forwarding Test");
    program.add_metadata("description", "Test spécifique pour le forwarding Store-Load");
    program.add_metadata("author", "PunkVM Team");

    println!("=== CRÉATION DU TEST STORE-LOAD FORWARDING ===");

    // Initialisation
    program.add_instruction(Instruction::create_reg_imm32(Opcode::Mov, 0, 0x2000)); // Adresse de base
    program.add_instruction(Instruction::create_reg_imm32(Opcode::Mov, 1, 1245453));    // Valeur à stocker

    // Test 1: Store immédiatement suivi d'un Load à la même adresse
    println!("Test 1: Store→Load immédiat");
    program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 1, 0, 0));         // STORE [R0], R1
    program.add_instruction(Instruction::create_load_reg_offset(2, 0, 0));          // LOAD R2, [R0]

    // Test 2: Store avec offset, puis Load avec même offset
    println!("Test 2: Store→Load avec offset");
    program.add_instruction(Instruction::create_reg_imm32(Opcode::Mov, 3, 4542756));    // Nouvelle valeur
    program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 3, 0, 4));         // STORE [R0+4], R3
    program.add_instruction(Instruction::create_load_reg_offset(4, 0, 4));          // LOAD R4, [R0+4]

    // Test 3: Store puis utilisation immédiate de la valeur chargée
    println!("Test 3: Store→Load→Use");
    program.add_instruction(Instruction::create_reg_imm32(Opcode::Mov, 5, 65214789));    // Nouvelle valeur
    program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 5, 0, 8));         // STORE [R0+8], R5
    program.add_instruction(Instruction::create_load_reg_offset(6, 0, 8));          // LOAD R6, [R0+8]
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 7, 6, 1)); // ADD R7, R6, R1
    // Test 4: Stores multiples à des adresses différentes
    println!("Test 4: Stores multiples");
    program.add_instruction(Instruction::create_reg_imm32(Opcode::Mov, 8, 100000560));
    program.add_instruction(Instruction::create_reg_imm32(Opcode::Mov, 9, 20004500));
    program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 8, 0, 12));        // STORE [R0+12], R8
    program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 9, 0, 16));        // STORE [R0+16], R9
    program.add_instruction(Instruction::create_load_reg_offset(10, 0, 12));        // LOAD R10, [R0+12]
    program.add_instruction(Instruction::create_load_reg_offset(11, 0, 16));        // LOAD R11, [R0+16]

    program.add_instruction(Instruction::create_no_args(Opcode::Halt));

    // Configuration des segments
    let total_code_size: u32 = program.code.iter()
        .map(|instr| instr.total_size() as u32)
        .sum();
    program.segments = vec![SegmentMetadata::new(SegmentType::Code, 0, total_code_size, 0)];

    println!("=== RÉSULTATS ATTENDUS STORE-LOAD ===");
    println!("R0  = 0x2000 (adresse de base)");
    println!("R1  = 1245453");
    println!("R2  = 1245453 (forwardé depuis store)");
    println!("R3  = 4542756");
    println!("R4  = 4542756 (forwardé depuis store)");
    println!("R5  = 65214789");
    println!("R6  = 65214789 (forwardé depuis store)");
    println!("R7  = 65214789 + 1245453 (forwardé depuis store)"); // ADD R7, R6, R1
    println!("R8  = 100000560");
    println!("R9  = 20004500");
    println!("R10 = 100000560 (forwardé depuis store)");
    println!("R11 = 20004500 (forwardé depuis store)");

    program
}


/// Test spécifique pour le Store-Load forwarding avec immediate 15bit
fn store_load_forwarding_test_64() -> BytecodeFile {
    let mut program = BytecodeFile::new();
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    program.add_metadata("name", "Store-Load Forwarding Test");
    program.add_metadata("description", "Test spécifique pour le forwarding Store-Load");
    program.add_metadata("author", "PunkVM Team");

    println!("=== CRÉATION DU TEST STORE-LOAD FORWARDING ===");

    // Initialisation
    program.add_instruction(Instruction::create_reg_imm64(Opcode::Mov, 0, 0x2000)); // Adresse de base
    program.add_instruction(Instruction::create_reg_imm64(Opcode::Mov, 1, 123));    // Valeur à stocker

    // Test 1: Store immédiatement suivi d'un Load à la même adresse
    println!("Test 1: Store→Load immédiat");
    program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 1, 0, 0));         // STORE [R0], R1
    program.add_instruction(Instruction::create_load_reg_offset(2, 0, 0));          // LOAD R2, [R0]

    // Test 2: Store avec offset, puis Load avec même offset
    println!("Test 2: Store→Load avec offset");
    program.add_instruction(Instruction::create_reg_imm64(Opcode::Mov, 3, 4561454541));    // Nouvelle valeur
    program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 3, 0, 4));         // STORE [R0+4], R3
    program.add_instruction(Instruction::create_load_reg_offset(4, 0, 4));          // LOAD R4, [R0+4]

    // Test 3: Store puis utilisation immédiate de la valeur chargée
    println!("Test 3: Store→Load→Use");
    program.add_instruction(Instruction::create_reg_imm64(Opcode::Mov, 5, 7894547412));    // Nouvelle valeur
    program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 5, 0, 8));         // STORE [R0+8], R5
    program.add_instruction(Instruction::create_load_reg_offset(6, 0, 8));          // LOAD R6, [R0+8]
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 7, 6, 1)); // ADD R7, R6, R1

    // Test 4: Stores multiples à des adresses différentes
    println!("Test 4: Stores multiples");
    program.add_instruction(Instruction::create_reg_imm64(Opcode::Mov, 8, 100000000000));
    program.add_instruction(Instruction::create_reg_imm64(Opcode::Mov, 9, 200000000000));
    program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 8, 0, 12));        // STORE [R0+12], R8
    program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 9, 0, 16));        // STORE [R0+16], R9
    program.add_instruction(Instruction::create_load_reg_offset(10, 0, 12));        // LOAD R10, [R0+12]
    program.add_instruction(Instruction::create_load_reg_offset(11, 0, 16));        // LOAD R11, [R0+16]

    program.add_instruction(Instruction::create_no_args(Opcode::Halt));

    // Configuration des segments
    let total_code_size: u32 = program.code.iter()
        .map(|instr| instr.total_size() as u32)
        .sum();
    program.segments = vec![SegmentMetadata::new(SegmentType::Code, 0, total_code_size, 0)];

    println!("=== RÉSULTATS ATTENDUS STORE-LOAD ===");
    println!("R0  = 0x2000 (adresse de base)");
    println!("R1  = 123");
    println!("R2  = 123 (forwardé depuis store)");
    println!("R3  = 4561454541");
    println!("R4  = 4561454541 (forwardé depuis store)");
    println!("R5  = 7894547412");
    println!("R6  = 7894547412 (forwardé depuis store)");
    println!("R7  = 7894547412 + 123 (forwardé depuis store)"); // ADD R7, R6, R1
    println!("R8  = 100000000000");
    println!("R9  = 200000000000");
    println!("R10 = 100000000000 (forwardé depuis store)");
    println!("R11 = 200000000000 (forwardé depuis store)");





    program
}

/// Test pour mesurer l'efficacité du forwarding avec différents patterns
/// Test spécifique pour valider la correction du bug cache
fn cache_stress_test() -> BytecodeFile {
    let mut program = BytecodeFile::new();
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    program.add_metadata("name", "PunkVM Cache Stress Test");
    program.add_metadata("description", "Test de stress pour la hiérarchie de cache L1/L2");
    program.add_metadata("author", "PunkVM Team");
    
    println!("\n=== CACHE STRESS TEST ===");
    
    // Test 1: Initialisation de beaucoup de registres (écritures)
    for i in 0..16 {
        program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, i, i * 10));
    }
    
    // Test 2: Opérations simples avec des registres pour stress test du cache lors du chargement
    program.add_instruction(Instruction::create_reg_imm32(Opcode::Mov, 0, 0x2000));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 100));
    program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 0, 0, 0)); // Store R1 dans [R0+0]
    
    program.add_instruction(Instruction::create_reg_imm32(Opcode::Mov, 0, 0x2100));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 200));
    program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 1, 0, 0)); // Store R1 dans [R0+0]
    
    // Test 3: Lectures des données écrites (test cache hit/miss)
    program.add_instruction(Instruction::create_reg_imm32(Opcode::Mov, 0, 0x2000));
    program.add_instruction(Instruction::create_load_reg_offset(2, 0, 0)); // Load R2 depuis [R0+0]
    
    program.add_instruction(Instruction::create_reg_imm32(Opcode::Mov, 0, 0x2100));
    program.add_instruction(Instruction::create_load_reg_offset(3, 0, 0)); // Load R3 depuis [R0+0]
    
    // Marquer la fin
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 15, 0xCC)); // R15 = 0xCC (marqueur)
    program.add_instruction(Instruction::create_no_args(Opcode::Halt));
    
    // Configuration des segments
    let total_code_size: u32 = program.code.iter()
        .map(|instr| instr.total_size() as u32)
        .sum();
    program.segments = vec![SegmentMetadata::new(SegmentType::Code, 0, total_code_size, 0)];
    
    // Ajouter segment de données
    
    
    println!("\n=== RÉSULTATS ATTENDUS CACHE STRESS ===");
    println!("R0-R15 = Valeurs calculées selon les MOV");
    println!("R2     = 100 (rechargé depuis mémoire)");
    println!("R3     = 200 (rechargé depuis mémoire)");
    println!("R15    = 204 (0xCC) - Marqueur de fin");
    
    program
}

fn forwarding_efficiency_test() -> BytecodeFile {
    let mut program = BytecodeFile::new();
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    program.add_metadata("name", "Forwarding Efficiency Test");
    program.add_metadata("description", "Mesure de l'efficacité du forwarding avec patterns variés");
    program.add_metadata("author", "PunkVM Team");
    
    println!("=== CRÉATION DU TEST D'EFFICACITÉ FORWARDING ===");
    
    // Pattern 1: Execute→Execute forwarding (le plus fréquent)
    println!("Pattern 1: Execute→Execute");
    program.add_instruction(Instruction::create_reg_imm16(Opcode::Mov, 0, 10));
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 1, 0, 0));   // R1 = R0 + R0
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Mul, 2, 1, 1));   // R2 = R1 * R1
    
    // Pattern 2: Memory→Execute forwarding
    println!("Pattern 2: Memory→Execute");
    program.add_instruction(Instruction::create_reg_imm16(Opcode::Mov, 3, 20));
    program.add_instruction(Instruction::create_reg_imm16(Opcode::Mov, 4, 30));
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 5, 3, 4));   // Bubble
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Sub, 6, 5, 2));   // R6 = R5 - R2
    
    // Pattern 3: Mélange de forwarding et d'instructions indépendantes
    println!("Pattern 3: Mélange");
    program.add_instruction(Instruction::create_reg_imm16(Opcode::Mov, 7, 40));       // Indépendant
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 8, 6, 7));   // R8 = R6 + R7
    program.add_instruction(Instruction::create_reg_imm16(Opcode::Mov, 9, 50));       // Indépendant
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Mul, 10, 8, 9));  // R10 = R8 * R9
    
    // Pattern 4: Load-Use qui DEVRAIT causer un stall
    println!("Pattern 4: Load-Use (stall requis)");
    program.add_instruction(Instruction::create_reg_imm16(Opcode::Mov, 11, 0x3000)); // Adresse
    program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 10, 11, 0));         // STORE [R11], R10
    program.add_instruction(Instruction::create_load_reg_offset(12, 11, 0));          // LOAD R12, [R11]
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 13, 12, 1)); // ADD R13, R12, R1 (Load-Use)
    
    program.add_instruction(Instruction::create_no_args(Opcode::Halt));

    let total_code_size: u32 = program.code.iter()
        .map(|instr| instr.total_size() as u32)
        .sum();
    program.segments = vec![SegmentMetadata::new(SegmentType::Code, 0, total_code_size, 0)];
    
    println!("=== RÉSULTATS ATTENDUS EFFICACITÉ ===");
    println!("R0  = 10");
    println!("R1  = 20 (10 + 10)");
    println!("R2  = 400 (20 * 20)");
    println!("R3  = 20");
    println!("R4  = 30");
    println!("R5  = 50 (20 + 30)");
    println!("R6  = -350 (50 - 400)");
    println!("R7  = 40");
    println!("R8  = -310 (-350 + 40)");
    println!("R9  = 50");
    println!("R10 = -15500 (-310 * 50)");
    println!("R11 = 0x3000");
    println!("R12 = -15500 (chargé depuis mémoire)");
    println!("R13 = -15480 (-15500 + 20)");
    
    program
}



fn simd_instruction_test() -> BytecodeFile {
    let mut program = BytecodeFile::new();
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    program.add_metadata("name", "PunkVM SIMD Instruction Test");
    program.add_metadata("description", "Test des instructions SIMD de PunkVM");
    program.add_metadata("author", "PunkVM Team");

    println!("=== CRÉATION DU TEST D'INSTRUCTIONS SIMD ===");

    // Initialisation des registres
    for i in 0..16 {
        program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, i, i as u8 + 1)); // R0-R7 = 1-8
    }
    //
    // Test d'addition vectorielle
    println!("Test d'addition vectorielle");

    // program.add_simd_instruction(Vector128::from_i32x4([1, 2, 3, 4])); // R0 = R1 + R2
    // program.add_simd_instruction(Vector128::from_i32x4([5, 6, 7, 8])); // R3 = R4 + R5
    program.add_instruction(Instruction::create_simd_vector_128(Opcode::Simd128Add, 0, 1, 2)); // R0 = R1 + R2
    //
    // // Test de multiplication vectorielle
    // println!("Test de multiplication vectorielle");
    // program.add_instruction(Instruction::create_simd_mul(6, 7, 0)); // R6 = R7 * R0
    //
    // // Test de soustraction vectorielle
    // println!("Test de soustraction vectorielle");
    // program.add_instruction(Instruction::create_simd_sub(1, 2, 3)); // R1 = R2 - R3
    //
    // Test de chargement et stockage SIMD
    // println!("Test de chargement et stockage SIMD");
    // program.add_instruction(Instruction::create_load_simd(8, 0x1000)); // Charger depuis l'adresse mémoire
    // program.add_instruction(Instruction::create_store_simd(9, 0x2000)); // Stocker à l'adresse mémoire

    // Fin du programme
    program.add_instruction(Instruction::create_no_args(Opcode::Halt));

    // Configuration des segments
    let total_code_size: u32 = program.code.iter()
        .map(|instr| instr.total_size() as u32)
        .sum();
    program.segments = vec![SegmentMetadata::new(SegmentType::Code, 0, total_code_size, 0)];

    println!("=== RÉSULTATS ATTENDUS SIMD ===");

    program
}



//src/main.rs


use std::collections::HashMap;
use std::time::Instant;
use PunkVM::bytecode::files::{BytecodeFile, BytecodeVersion, SegmentMetadata, SegmentType};
use PunkVM::bytecode::instructions::{ArgValue, Instruction};
use PunkVM::bytecode::opcodes::Opcode;
use PunkVM::bytecode::format::ArgType; // Added import
use PunkVM::debug::PipelineTracer;
use PunkVM::pvm::vm::{PunkVM as VM, VMConfig, VMState};
use PunkVM::pvm::vm_errors::VMResult;


fn main() -> VMResult<()> {
    println!("=== PunkVM - Test debug PunkVM ===");

    // Configuration de la VM
    let config = VMConfig {
        memory_size: 64 * 1024,        // 64 KB de mémoire
        num_registers: 16,             // 16 registres généraux
        l1_cache_size: 1024,           // 1 KB de cache L1
        store_buffer_size: 8,          // 8 entrées dans le store buffer
        stack_size: 4 * 1024,          // 4 KB de pile
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

    // Créer le programme complexe
    // let program = create_complex_program();
    // let program = create_pipeline_test_program();
    // let program = create_reg_reg_reg_test_program();
    // let program = create_hazard_detection_test_program();
    // let program = create_all_branch_test_program();
    //
    // let program = create_branch_test_program_debug();

    // let program= create_simple_test_program();
    // let program = create_conditional_branch_test_program();
    // let program = punk_program();
    // let program = punk_program_2();
    // let program = momo_program();
    // let program = punk_program_fixed();
    //
    // let program = punk_program_3();
    // let program = test_branch_not_taken_fix();
    // let program = punk_program_4();
    // let program =  punk_program_debug();
    // let program = punk_program_5();
    // let program = punk_program_test();

    // --- Running test_branch_not_taken_fix ---
    println!("\n\n--- Running test_branch_not_taken_fix ---");
    let program1 = test_branch_not_taken_fix();
    println!("Chargement du programme 'test_branch_not_taken_fix'...");
    vm.load_program_from_bytecode(program1)?;

    println!("Exécution du programme 'test_branch_not_taken_fix'...");
    let start_time1 = Instant::now();
    let result1 = vm.run();
    let duration1 = start_time1.elapsed();

    if let Err(ref e) = result1 {
        println!("Erreur lors de l'exécution de 'test_branch_not_taken_fix': {}", e);
    } else {
        println!("Programme 'test_branch_not_taken_fix' exécuté avec succès en {:?}", duration1);
    }

    println!("\nÉtat final des registres après 'test_branch_not_taken_fix':");
    print_registers(&vm);
    print_stats(&vm);

    // --- Re-initialize VM and run punk_program_3 ---
    println!("\n\n--- Running punk_program_3 ---");
    // Re-initialize VM to ensure a clean state
    println!("Réinitialisation de la VM pour punk_program_3...");
    let mut vm = PunkVM::pvm::vm::PunkVM::with_config(config.clone()); // Re-initialize with the same config
     println!(
        " PunkVM réinitialisée avec {} registre succès",
        vm.registers.len()
    );

    let program3 = punk_program_3();
    println!("Chargement du programme 'punk_program_3'...");
    vm.load_program_from_bytecode(program3)?;

    println!("Exécution du programme 'punk_program_3'...");
    let start_time3 = Instant::now();
    let result3 = vm.run();
    let duration3 = start_time3.elapsed();

    if let Err(ref e) = result3 {
        println!("Erreur lors de l'exécution de 'punk_program_3': {}", e);
    } else {
        println!("Programme 'punk_program_3' exécuté avec succès en {:?}", duration3);
    }

    println!("\nÉtat final des registres après 'punk_program_3':");
    print_registers(&vm);
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
    println!("  Cache hits: {}", stats.memory_hits);
    println!("  Cache misses: {}", stats.memory_misses);
    println!("  Branches flush: {}", stats.branch_flush);
    println!("  Branche predictions: {}", stats.branch_predictor);
    println!(
        "  Branch prediction rate : {:.2}%",
        stats.branch_prediction_rate
    );

    // Calcul de quelques métriques supplémentaires
    if stats.cycles > 0 {
        let stall_rate = (stats.stalls as f64 / stats.cycles as f64) * 100.0;
        println!("  Taux de stalls: {:.2}%", stall_rate);
    }

    if stats.memory_hits + stats.memory_misses > 0 {
        let hit_rate =
            (stats.memory_hits as f64 / (stats.memory_hits + stats.memory_misses) as f64) * 100.0;
        println!("  Taux de hits cache: {:.2}%", hit_rate);
    }

    // if stats.hazards > 0 && stats.forwards > 0 {
    //     let forwarding_efficiency = (stats.forwards as f64  / stats.hazards as f64) * 100.0;
    //     println!("  Efficacité du forwarding: {:.2}%", forwarding_efficiency);
    // }

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
    let total_data_dependencies = stats.forwards + stats.hazards;
    let forwarding_efficiency = if total_data_dependencies > 0 {
        stats.forwards as f64 / total_data_dependencies as f64 * 100.0
    } else {
        0.0
    };
    println!("Efficacité du forwarding: {:.2}%", forwarding_efficiency);

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
    let function_offset = -12; // Retourner à la fonction
    // program.add_instruction(Instruction::create_call(function_offset));

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

pub fn test_branch_not_taken_fix() -> BytecodeFile {
    let mut program = BytecodeFile::new();
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    program.add_metadata("name", "PunkVM Test Branch Not Taken Fix");
    program.add_metadata("description", "Minimal test for a non-taken conditional branch (JmpIfEqual).");
    program.add_metadata("author", "PunkVM Team");
    program.add_metadata("test_categories", "JmpIfEqual, NonTakenBranch");

    let mut address_counter: u32 = 0; // Corrected syntax

    // MOV R0, 10
    let mov_r0_instr = Instruction::create_reg_imm8(Opcode::Mov, 0, 10);
    program.add_instruction(mov_r0_instr.clone());
    address_counter += mov_r0_instr.total_size() as u32;

    // MOV R1, 20
    let mov_r1_instr = Instruction::create_reg_imm8(Opcode::Mov, 1, 20);
    program.add_instruction(mov_r1_instr.clone());
    address_counter += mov_r1_instr.total_size() as u32;

    // CMP R0, R1 (10 == 20 is false, so ZF will be 0)
    let cmp_instr = Instruction::create_reg_reg(Opcode::Cmp, 0, 1);
    program.add_instruction(cmp_instr.clone());
    address_counter += cmp_instr.total_size() as u32;

    // Address of the JmpIfEqual instruction itself
    let jie_addr = address_counter;

    // Placeholder for JmpIfEqual to get its size
    let jie_placeholder = Instruction::create_jump_if_equal(0, 0);
    let jie_size = jie_placeholder.total_size() as u32;

    // MOV R2, 42 (This should be executed)
    let mov_r2_instr = Instruction::create_reg_imm8(Opcode::Mov, 2, 42);
    let mov_r2_size = mov_r2_instr.total_size() as u32;

    // HALT (Normal successful halt)
    let halt_instr = Instruction::create_no_args(Opcode::Halt);
    let halt_size = halt_instr.total_size() as u32;

    // Target address if JmpIfEqual is (wrongly) taken:
    // It should skip MOV R2, 42 and the normal HALT.
    // Target = JIE_addr + JIE_size + MOV_R2_size + HALT_size
    let fail_target_addr = jie_addr + jie_size + mov_r2_size + halt_size;

    // JmpIfEqual FAIL_TARGET_ADDR (Should NOT be taken)
    // The first argument to create_jump_if_equal is the address of the jump instruction itself.
    let jie_instr = Instruction::create_jump_if_equal(jie_addr, fail_target_addr);
    program.add_instruction(jie_instr.clone());
    address_counter += jie_instr.total_size() as u32; // Should be jie_size

    // MOV R2, 42 (This should be executed if branch is NOT taken)
    program.add_instruction(mov_r2_instr.clone());
    address_counter += mov_r2_size;

    // HALT (End of normal execution)
    program.add_instruction(halt_instr.clone());
    address_counter += halt_size;

    // Instructions for the fail case (if JmpIfEqual is wrongly taken)
    // This is where fail_target_addr should point.
    // MOV R2, 0xFF (Marker for failure: branch taken when it shouldn't have)
    let mov_r2_fail_instr = Instruction::create_reg_imm8(Opcode::Mov, 2, 0xFF);
    program.add_instruction(mov_r2_fail_instr.clone());
    address_counter += mov_r2_fail_instr.total_size() as u32;

    // HALT (End of program if branch wrongly taken)
    program.add_instruction(halt_instr.clone());
    // address_counter for map printing will be implicitly handled by the loop sum

    // Calculate total code size and create code segment
    let total_code_size: u32 = program.code.iter().map(|instr| instr.total_size() as u32).sum();
    program.segments = vec![SegmentMetadata::new(SegmentType::Code, 0, total_code_size, 0)];

    // Add an empty data segment
    let data_size = 256;
    let data_segment = SegmentMetadata::new(SegmentType::Data, 0, data_size, 0x1000);
    program.segments.push(data_segment);
    program.data = vec![0; data_size as usize];

    println!("
--- Instruction Map for test_branch_not_taken_fix ---");
    let mut map_addr = 0u32;
    for (idx, instr) in program.code.iter().enumerate() {
        let size = instr.total_size() as u32;
        println!(
            "Instruction {:2}: [{:?}] Adresse 0x{:04X}-0x{:04X} (taille {:2})",
            idx,
            instr.opcode,
            map_addr,
            map_addr + size.saturating_sub(1), // Avoid underflow if size is 0
            size
        );
        if instr.opcode.is_branch() {
            // Attempt to get the target address argument.
            // This depends on how create_jump_if_equal stores its arguments.
            // Assuming arg1 (index 1) might be the target address if it's an absolute jump.
            // For branch instructions, try to print the target address.
            // Jumps created with create_jump_if_equal(from, to) store a relative offset.
            // This offset is typically the second argument.
            if instr.format.arg2_type == ArgType::RelativeAddr {
                if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
                    // The offset is relative to the PC of the *next* instruction.
                    let target_pc = (map_addr as i64 + size as i64 + offset as i64) as u32;
                    println!("      -> Branch relative: offset = {:+}, target_calc = 0x{:04X}", offset, target_pc);
                } else {
                    println!("      -> Branch target: (Could not interpret arg2 as RelativeAddr)");
                }
            } else if instr.format.arg2_type == ArgType::AbsoluteAddr {
                 if let Ok(ArgValue::AbsoluteAddr(target_abs)) = instr.get_arg2_value() {
                    println!("      -> Branch absolute (arg2): target=0x{:04X}", target_abs);
                } else {
                    println!("      -> Branch target: (Could not interpret arg2 as AbsoluteAddr)");
                }
            } else if instr.format.arg1_type == ArgType::RelativeAddr { // Fallback to arg1 if arg2 is not relative/absolute
                if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg1_value() {
                    let target_pc = (map_addr as i64 + size as i64 + offset as i64) as u32;
                    println!("      -> Branch relative (arg1): offset = {:+}, target_calc = 0x{:04X}", offset, target_pc);
                } else {
                     println!("      -> Branch target: (Could not interpret arg1 as RelativeAddr)");
                }
            } else if instr.format.arg1_type == ArgType::AbsoluteAddr {
                if let Ok(ArgValue::AbsoluteAddr(target_abs)) = instr.get_arg1_value() {
                    println!("      -> Branch absolute (arg1): target=0x{:04X}", target_abs);
                } else {
                    println!("      -> Branch target: (Could not interpret arg1 as AbsoluteAddr)");
                }
            }
            else {
                println!("      -> Branch target: (Arg1/Arg2 not RelativeAddr or AbsoluteAddr, or format unknown for printing)");
            }
        }
        map_addr += size;
    }
    println!("Total code size: {} bytes, {} instructions", map_addr, program.code.len());
    println!("--- End of Instruction Map ---");

    println!("
Expected outcome for test_branch_not_taken_fix:");
    println!("R0 = 10");
    println!("R1 = 20");
    println!("R2 = 42 (if JmpIfEqual is NOT taken and program halts successfully after that)");
    println!("Program halts at the first HALT instruction.");

    program
}


pub fn punk_program_5() -> BytecodeFile {
    let mut program = BytecodeFile::new();
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    program.add_metadata("name", "PunkVM Branch Test 5");
    program.add_metadata("description", "Comprehensive branch test with from_addr/to_addr");

    // Initialiser les registres
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 10)); // R0 = 10
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 20)); // R1 = 20
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 2, 10)); // R2 = 10

    let mut current_address = Instruction::calculate_current_address(&program.code);

    // 1. JMP inconditionnel (saut en avant)
    let jmp_target_1 = current_address + 24; // Sauter par-dessus 3 instructions
    program.add_instruction(Instruction::create_jump(current_address, jmp_target_1));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 3, 0xFF)); // Ne doit pas être exécuté
    current_address = Instruction::calculate_current_address(&program.code);
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 4, 0x01)); // R4 = 1 après le saut
    current_address = Instruction::calculate_current_address(&program.code);

    // 2. JMP en arriere
    let jmp_target_2 = 0; // Retour au début
    program.add_instruction(Instruction::create_jump(current_address, jmp_target_2));
    current_address = Instruction::calculate_current_address(&program.code);

    // ici  il faudra tout  refaire les meme  test  qu avant mais en utilisant l'adresse du code
    // todo!()

    // 3. JmpIfEqual (ZF=1, pris)
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 2)); // R0 == R2 (ZF=1)
    current_address = Instruction::calculate_current_address(&program.code);
    let jmpifequal_target_1 = current_address + 16;
    program.add_instruction(Instruction::create_jump_if_equal(current_address, jmpifequal_target_1));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 5, 0xFF)); // Ne doit pas être exécuté
    current_address = Instruction::calculate_current_address(&program.code);
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 5, 0x02)); // R5 = 2 (succès)
    current_address = Instruction::calculate_current_address(&program.code);

    // 4. JmpIfEqual (ZF=0, non pris)
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 1)); // R0 != R1 (ZF=0)
    current_address = Instruction::calculate_current_address(&program.code);
    let jmpifequal_target_2 = current_address + 16;
    program.add_instruction(Instruction::create_jump_if_equal(current_address, jmpifequal_target_2));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 6, 0xFF)); // Ne doit pas être exécuté
    current_address = Instruction::calculate_current_address(&program.code);
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 6, 0x03)); // R6 = 3 (succès, doit être exécuté)
    current_address = Instruction::calculate_current_address(&program.code);


    // TODO: Ajouter des tests similaires pour les autres instructions de branchement
    // (JmpIfNotEqual, JmpIfGreater, JmpIfLess, etc.)

    program.add_instruction(Instruction::create_no_args(Opcode::Halt));

    // Calcul de la taille totale du code
    let total_size: u32 = program
        .code
        .iter()
        .map(|instr| instr.total_size() as u32)
        .sum();
    program.segments = vec![SegmentMetadata::new(SegmentType::Code, 0, total_size, 0)];

    // Ajout d'un segment de données vide
    let data_size = 256;
    let data_segment = SegmentMetadata::new(SegmentType::Data, 0, data_size, 0x1000);
    program.segments.push(data_segment);
    program.data = vec![0; data_size as usize];

    println!("\n--- Carte des instructions du programme de test des branchements ---");
    let mut addr = 0;
    for (idx, instr) in program.code.iter().enumerate() {
        let size = instr.total_size();
        println!(
            "Instruction {}: Adresse 0x{:04X}-0x{:04X} (taille {}): {:?}",
            idx,
            addr,
            addr + size - 1,
            size,
            instr.opcode
        );


        if instr.opcode.is_branch() {
            if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
                let target = (addr as u32 + size as u32) as i64 + offset as i64;
                println!("      -> Branchement relatif: offset={:+}, target=0x{:04X}", offset, target);
            }
        }
        addr += size;
    }
    println!("--- Fin de la carte des instructions ---");


    program
}




pub fn create_reg_reg_reg_test_program() -> BytecodeFile {
    let mut program = BytecodeFile::new();
    // Version du programme
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    // Métadonnées (optionnel)
    program.add_metadata("name", "Test reg_reg_reg");
    program.add_metadata(
        "description",
        "Programme testant les instructions à trois registres.",
    );

    // Initialiser R0 et R1 avec des valeurs immédiates via MOV (instructions immédiates)
    // Ici, on utilise create_reg_imm8 (qui utilise un format MOV avec immediate) pour initialiser les registres
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 5)); // R0 = 5
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 10)); // R1 = 10

    // Opérations à trois registres
    // R2 = R0 + R1  --> 5 + 10 = 15
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 2, 0, 1));
    // R3 = R2 - R0  --> 15 - 5 = 10
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Sub, 3, 2, 0));
    // R4 = R3 * R1  --> 10 * 10 = 100
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Mul, 4, 3, 1));
    // R5 = R4 / R0  --> 100 / 5 = 20
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Div, 5, 4, 0));
    // R6 = R2 + R4  --> 15 + 100 = 115
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 6, 2, 4));
    // R7 = R6 - R5  --> 115 - 20 = 95
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Sub, 7, 6, 5));
    // R8 = R7 + R2  --> 95 + 15 = 110
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 8, 7, 2));

    // Fin du programme : HALT
    program.add_instruction(Instruction::create_no_args(Opcode::Halt));

    // Calculer la taille totale du code et créer le segment de code
    let total_size: u32 = program
        .code
        .iter()
        .map(|instr| instr.total_size() as u32)
        .sum();
    program.segments = vec![SegmentMetadata::new(SegmentType::Code, 0, total_size, 0)];

    // Ajout d'un segment de données vide
    let data_size = 256;
    let data_segment = SegmentMetadata::new(SegmentType::Data, 0, data_size, 0x1000);
    program.segments.push(data_segment);
    program.data = vec![0; data_size as usize];

    println!("\n--- Carte des instructions du programme de test des branchements ---");
    let mut addr = 0;
    for (idx, instr) in program.code.iter().enumerate() {
        let size = instr.total_size();
        println!(
            "Instruction {}: Adresse 0x{:04X}-0x{:04X} (taille {}): {:?}",
            idx,
            addr,
            addr + size - 1,
            size,
            instr.opcode
        );


        if instr.opcode.is_branch() {
            if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
                let target = (addr as u32 + size as u32) as i64 + offset as i64;
                println!("      -> Branchement relatif: offset={:+}, target=0x{:04X}", offset, target);
            }
        }
        addr += size;
    }
    println!("--- Fin de la carte des instructions ---");


    program


}

//
//
// pub fn punk_program_test() -> BytecodeFile {
//     let mut program = BytecodeFile::new();
//     // Version du programme
//     program.version = BytecodeVersion::new(0, 1, 0, 0);
//     // Métadonnées (optionnel)
//     program.add_metadata("name", "bug test ");
//     program.add_metadata(
//         "description",
//         "Bug test .",
//     );
//
//
//     let mut current_addr = 0u32;
//
//     println!("=== CRÉATION DU PROGRAMME DE TEST PUNKVM ===");
//
//     // ==========================================
//     // SECTION 1: INITIALISATION DES REGISTRES
//     // ==========================================
//     println!("=== SECTION 1: INITIALISATION ===");
//
//     // MOV R0, 10    - Valeur pour les tests
//     program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 10));
//     current_addr += program.last().unwrap().total_size() as u32;
//
//     // MOV R1, 5     - Compteur de boucle
//     program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 5));
//     current_addr += program.last().unwrap().total_size() as u32;
//
//     // MOV R2, 0     - Accumulateur
//     program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 2, 0));
//     current_addr += program.last().unwrap().total_size() as u32;
//
//     // MOV R3, 1     - Incrément
//     program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 3, 1));
//     current_addr += program.last().unwrap().total_size() as u32;
//
//     // ==========================================
//     // SECTION 2: TEST OPÉRATIONS ALU
//     // ==========================================
//     println!("=== SECTION 2: TESTS ALU ===");
//
//     // ADD R2, R0    - R2 = 0 + 10 = 10
//     program.add_instruction(Instruction::create_reg_reg(Opcode::Add, 2, 0));
//     current_addr += program.last().unwrap().total_size() as u32;
//
//     // SUB R2, R3    - R2 = 10 - 1 = 9
//     program.add_instruction(Instruction::create_reg_reg(Opcode::Sub, 2, 3));
//     current_addr += program.last().unwrap().total_size() as u32;
//
//     // ==========================================
//     // SECTION 3: TEST BRANCHEMENT CONDITIONNEL
//     // ==========================================
//     println!("=== SECTION 3: TEST BRANCHEMENT ===");
//
//     // CMP R1, R3    - Compare compteur avec 1
//     program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 1, 3));
//     current_addr += program.last().unwrap().total_size() as u32;
//
//     // Calculer l'adresse de saut (après la prochaine instruction)
//     let jump_instruction_addr = current_addr;
//     let skip_instruction_size = 6; // Taille de l'instruction MOV suivante
//     let target_addr = current_addr + 8 + skip_instruction_size; // Après JmpIfEqual + MOV
//
//     // JmpIfEqual target - Si R1 == 1, sauter l'instruction suivante
//     program.add_instruction(Instruction::create_relative_jump(
//         Opcode::JmpIfEqual,
//         jump_instruction_addr,
//         target_addr
//     ));
//     current_addr += program.last().unwrap().total_size() as u32;
//
//     // MOV R4, 255   - Cette instruction sera sautée si R1 == 1
//     program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 4, 255));
//     current_addr += program.last().unwrap().total_size() as u32;
//
//     // ==========================================
//     // SECTION 4: BOUCLE CONTRÔLÉE (5 ITÉRATIONS)
//     // ==========================================
//     println!("=== SECTION 4: BOUCLE CONTRÔLÉE ===");
//
//     // Marquer le début de la boucle
//     let loop_start = current_addr;
//
//     // ADD R2, R3    - Accumulateur += 1
//     program.add_instruction(Instruction::create_reg_reg(Opcode::Add, 2, 3));
//     current_addr += program.last().unwrap().total_size() as u32;
//
//     // SUB R1, R3    - Décrémenter le compteur
//     program.add_instruction(Instruction::create_reg_reg(Opcode::Sub, 1, 3));
//     current_addr += program.last().unwrap().total_size() as u32;
//
//     // CMP R1, R3    - Comparer avec 1 (condition d'arrêt)
//     program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 1, 3));
//     current_addr += program.last().unwrap().total_size() as u32;
//
//     // JmpIfGreater loop_start - Si R1 > 1, retourner au début
//     let jump_back_addr = current_addr;
//     program.add_instruction(Instruction::create_relative_jump(
//         Opcode::JmpIfGreater,
//         jump_back_addr,
//         loop_start
//     ));
//     current_addr += program.last().unwrap().total_size() as u32;
//
//     // ==========================================
//     // SECTION 5: TEST BRANCHEMENT INCONDITIONNEL
//     // ==========================================
//     println!("=== SECTION 5: SAUT INCONDITIONNEL ===");
//
//     // JMP vers la finalisation (sauter l'instruction piège)
//     let jmp_addr = current_addr;
//     let final_section_addr = current_addr + 8 + 6; // Après JMP + instruction piège
//     program.add_instruction(Instruction::create_relative_jump(
//         Opcode::Jmp,
//         jmp_addr,
//         final_section_addr
//     ));
//     current_addr += program.last().unwrap().total_size() as u32;
//
//     // MOV R5, 255   - Instruction piège (ne devrait jamais s'exécuter)
//     program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 5, 255));
//     current_addr += program.last().unwrap().total_size() as u32;
//
//     // ==========================================
//     // SECTION 6: FINALISATION ET VALIDATION
//     // ==========================================
//     println!("=== SECTION 6: FINALISATION ===");
//
//     // MOV R10, 42   - Marqueur de succès
//     program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 10, 42));
//     current_addr += program.last().unwrap().total_size() as u32;
//
//     // ADD R2, R10   - R2 devrait contenir la somme finale
//     program.add_instruction(Instruction::create_reg_reg(Opcode::Add, 2, 10));
//     current_addr += program.last().unwrap().total_size() as u32;
//
//     // HALT - Fin du programme
//     program.add_instruction(Instruction::create_no_args(Opcode::Halt));
//
//     // ==========================================
//     // RÉSUMÉ DU PROGRAMME
//     // ==========================================
//     println!("\n=== RÉSUMÉ DU PROGRAMME ===");
//     println!("Total d'instructions: {}", program.code.len());
//
//     let mut addr = 0;
//     for (i, instr) in program.code.iter().enumerate() {
//         println!("Instruction {}: Adresse 0x{:04X} - {:?}",
//                  i, addr, instr.opcode);
//         addr += instr.total_size() as u32;
//     }
//
//     println!("\n=== RÉSULTATS ATTENDUS ===");
//     println!("R0  = 10   (valeur initiale)");
//     println!("R1  = 1    (compteur final)");
//     println!("R2  = 55   (9 + 4*1 + 42 = accumulateur + boucle + succès)");
//     println!("R3  = 1    (incrément)");
//     println!("R4  = 0    (saut réussi, pas d'exécution de MOV R4, 255)");
//     println!("R5  = 0    (saut inconditionnel réussi)");
//     println!("R10 = 42   (marqueur de succès)");
//     println!("Autres registres = 0");
//
//     program
// }
//
// // Version simplifiée pour debug initial
// pub fn create_simple_test_program() -> Vec<Instruction> {
//     let mut program = Vec::new();
//     let mut current_addr = 0u32;
//
//     println!("=== PROGRAMME DE TEST SIMPLE ===");
//
//     // MOV R0, 10
//     program.push(Instruction::create_reg_imm8(Opcode::Mov, 0, 10));
//     current_addr += program.last().unwrap().total_size() as u32;
//
//     // MOV R1, 10
//     program.push(Instruction::create_reg_imm8(Opcode::Mov, 1, 10));
//     current_addr += program.last().unwrap().total_size() as u32;
//
//     // CMP R0, R1
//     program.push(Instruction::create_reg_reg(Opcode::Cmp, 0, 1));
//     current_addr += program.last().unwrap().total_size() as u32;
//
//     // JmpIfEqual vers HALT (saut +6 bytes)
//     let jump_addr = current_addr;
//     let halt_addr = current_addr + 8 + 6; // Après jump + MOV
//     program.push(Instruction::create_relative_jump(
//         Opcode::JmpIfEqual,
//         jump_addr,
//         halt_addr
//     ));
//     current_addr += program.last().unwrap().total_size() as u32;
//
//     // MOV R2, 255  (ne devrait pas s'exécuter)
//     program.push(Instruction::create_reg_imm8(Opcode::Mov, 2, 255));
//     current_addr += program.last().unwrap().total_size() as u32;
//
//     // HALT
//     program.push(Instruction::create_no_args(Opcode::Halt));
//
//     println!("Résultats attendus: R0=10, R1=10, R2=0 (saut réussi), autres=0");
//
//     program
// }

// Test de boucle très simple
// pub fn create_minimal_loop_test() -> Vec<Instruction> {
//     let mut program = Vec::new();
//     let mut current_addr = 0u32;
//
//     println!("=== TEST BOUCLE MINIMALE ===");
//
//     // MOV R0, 3    - Compteur
//     program.push(Instruction::create_reg_imm8(Opcode::Mov, 0, 3));
//     current_addr += program.last().unwrap().total_size() as u32;
//
//     // MOV R1, 1    - Décrément
//     program.push(Instruction::create_reg_imm8(Opcode::Mov, 1, 1));
//     current_addr += program.last().unwrap().total_size() as u32;
//
//     // Début de boucle
//     let loop_start = current_addr;
//
//     // SUB R0, R1   - Décrémenter
//     program.push(Instruction::create_reg_reg(Opcode::Sub, 0, 1));
//     current_addr += program.last().unwrap().total_size() as u32;
//
//     // CMP R0, R1   - Comparer avec 1
//     program.push(Instruction::create_reg_reg(Opcode::Cmp, 0, 1));
//     current_addr += program.last().unwrap().total_size() as u32;
//
//     // JmpIfGreater loop_start
//     let jump_addr = current_addr;
//     program.push(Instruction::create_relative_jump(
//         Opcode::JmpIfGreater,
//         jump_addr,
//         loop_start
//     ));
//     current_addr += program.last().unwrap().total_size() as u32;
//
//     // MOV R2, 42   - Succès
//     program.push(Instruction::create_reg_imm8(Opcode::Mov, 2, 42));
//     current_addr += program.last().unwrap().total_size() as u32;
//
//     // HALT
//     program.push(Instruction::create_no_args(Opcode::Halt));
//
//     println!("Résultats attendus: R0=1, R1=1, R2=42, boucle 2 itérations");
//
//     // ici les instructions sont créées avec des valeurs immédiates
//
//
//
//     // Calculer la taille totale du code et créer le segment de code
//     let total_size: u32 = program
//         .code
//         .iter()
//         .map(|instr| instr.total_size() as u32)
//         .sum();
//     program.segments = vec![SegmentMetadata::new(SegmentType::Code, 0, total_size, 0)];
//
//     // Ajout d'un segment de données vide
//     let data_size = 256;
//     let data_segment = SegmentMetadata::new(SegmentType::Data, 0, data_size, 0x1000);
//     program.segments.push(data_segment);
//     program.data = vec![0; data_size as usize];
//
//     println!("\n--- Carte des instructions du programme de test des branchements ---");
//     let mut addr = 0;
//     for (idx, instr) in program.code.iter().enumerate() {
//         let size = instr.total_size();
//         println!(
//             "Instruction {}: Adresse 0x{:04X}-0x{:04X} (taille {}): {:?}",
//             idx,
//             addr,
//             addr + size - 1,
//             size,
//             instr.opcode
//         );
//
//
//         if instr.opcode.is_branch() {
//             if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
//                 let target = (addr as u32 + size as u32) as i64 + offset as i64;
//                 println!("      -> Branchement relatif: offset={:+}, target=0x{:04X}", offset, target);
//             }
//         }
//         addr += size;
//     }
//     println!("--- Fin de la carte des instructions ---");
//
//
//     program
//
//
// }
//
//


























/*
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

    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 0)); // R0 = 0
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 10)); // R1 = 10
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 2, 1)); // R2 = 1
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 3, 0)); // R3 = 0
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
    let offset_to_start =
        -(calculate_instruction_range_size(&program.code, loop_start_idx, current_idx) as i32);
    let offset_to_start = -(calculate_range_size(&program.code, loop_start_idx, current_idx) as i8);

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
    let total_size: u32 = program
        .code
        .iter()
        .map(|instr| instr.total_size() as u32)
        .sum();

    // Créer le segment de code
    program.segments = vec![SegmentMetadata::new(SegmentType::Code, 0, total_size, 0)];

    // Créer un segment de données
    let data_size = 256; // Allouer 256 bytes pour les données
    let data_segment = SegmentMetadata::new(SegmentType::Data, 0, data_size, 0x1000);
    program.segments.push(data_segment);
    program.data = vec![0; data_size as usize];

    println!(
        "Programme complexe créé avec {} instructions",
        program.code.len()
    );

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

/////////////////////////////////////////////////

// fn create_conditional_jump(offset: i32) -> Instruction {
//     // Crée un saut conditionnel avec un offset relatif
//     // L'implémentation exacte dépend de votre format d'instruction
//     let mut instruction = Instruction::create_no_args(Opcode::JmpIf);
//
//     // Encoder l'offset dans les arguments de l'instruction
//     // Cela peut nécessiter une adaptation selon votre format
//     let bytes = offset.to_le_bytes();
//     instruction.args = bytes.to_vec();
//
//     instruction
// }
//
// fn calculate_instruction_range_size(instructions: &[Instruction], start_idx: usize, end_idx: usize) -> usize {
//     let mut total_size = 0;
//     for i in start_idx..end_idx {
//         total_size += instructions[i].total_size();
//     }
//     total_size
// }
///////////////////////////////////////////////////////////////

/// Calcule la somme des tailles des instructions dans l'intervalle [start, end).
fn calculate_range_size(instructions: &[Instruction], start: usize, end: usize) -> usize {
    instructions[start..end]
        .iter()
        .map(|instr| instr.total_size())
        .sum()
}

/// Crée une instruction de saut conditionnel (JmpIfNot) avec un offset relatif (en i8).
/// L'offset est encodé en Immediate8 (en deux's complement).
fn create_conditional_jump(offset: i8) -> Instruction {
    // Ici, on considère que l'instruction JmpIfNot utilise un registre fictif 0 (inutilisé) et l'immédiat est l'offset.
    println!("Création d'un saut conditionnel avec offset = {}", offset);
    Instruction::create_reg_imm8(Opcode::JmpIfNot, 0, offset as u8)
}
fn calculate_cumulative_pc(instructions: &[Instruction], idx: usize) -> u32 {
    instructions[..idx]
        .iter()
        .map(|instr| instr.total_size() as u32)
        .sum()
}

/// Crée un programme de test complet pour évaluer les performances du pipeline
pub fn create_pipeline_test_program() -> BytecodeFile {
    let mut program = BytecodeFile::new();
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    program.add_metadata("name", "Pipeline Performance Test");
    program.add_metadata(
        "description",
        "Test du pipeline, forwarding, hazards et stalls",
    );

    // ---------- Test 1: Data Dependencies (RAW Hazards) ----------
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 5)); // R0 = 5
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 10)); // R1 = 10
                                                                               //
                                                                               // RAW Hazard: R2 depends on R0, should trigger forwarding
                                                                               // program.add_instruction(Instruction::create_reg_reg(Opcode::Add, 2, 0));    // R2 = R0 (= 5) // tombe dans une loop infini
                                                                               // program.add_instruction(Instruction::create_reg_reg(Opcode::Add, 3, 2));    // R3 = R2 (= 5) - RAW Hazard, needs forwarding   // tombe dans une loop infini
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 2, 0, 1)); // R2 = R0 + R1 (= 15) // avec reg_reg_reg  tout est OK
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 3, 2, 1)); // R3 = R2 + R1 (= 25)  // avec reg_reg_reg  tout est

    // // Chain of dependencies to test multiple forwards
    // program.add_instruction(Instruction::create_reg_reg(Opcode::Add, 4, 3));    // R4 = R3 - RAW Hazard      //tombe dans une loop infini
    // program.add_instruction(Instruction::create_reg_reg(Opcode::Add, 5, 4));    // R5 = R4 - RAW Hazard      // tombe dans une loop infini
    //
    // // ---------- Test 2: Load-Use Hazard ----------
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 6, 100)); // R6 = 100 (base address)
                                                                                //
                                                                                // // Store R0 to memory location [R6]
    program.add_instruction(Instruction::create_reg_reg_offset(Opcode::Store, 0, 6, 0)); // Store R0 at [R6+0]
                                                                                         //
                                                                                         // // Load from memory then immediately use - should cause a Load-Use hazard
    program.add_instruction(Instruction::create_reg_reg_offset(Opcode::Load, 7, 6, 0)); // R7 = Mem[R6+0]
                                                                                        // program.add_instruction(Instruction::create_reg_reg(Opcode::Add, 8, 7));             // R8 = R7 - Load-Use hazard
                                                                                        //
                                                                                        // // ---------- Test 3: Structural Hazard ----------
                                                                                        // // Two memory operations in sequence - potential structural hazard
    program.add_instruction(Instruction::create_reg_reg_offset(Opcode::Store, 1, 6, 4)); // Store R1 at [R6+4]
    program.add_instruction(Instruction::create_reg_reg_offset(Opcode::Load, 9, 6, 4)); // R9 = Mem[R6+4]
                                                                                        //
                                                                                        // // ---------- Test 4: Store-Load forwarding ----------
                                                                                        // // Store followed by Load from same address - should be forwarded from store buffer
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 10, 42)); // R10 = 42
    program.add_instruction(Instruction::create_reg_reg_offset(Opcode::Store, 10, 6, 8)); // Store R10 at [R6+8]
    program.add_instruction(Instruction::create_reg_reg_offset(Opcode::Load, 11, 6, 8)); // R11 = Mem[R6+8] - Should be forwarded

    // ---------- Test 5: Branch prediction ----------
    // // Simple loop to test branch prediction (if implemented)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 12, 0)); // R12 = 0 (counter)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 13, 3)); // R13 = 3 (max iterations)
                                                                               //
                                                                               // // Loop start marker
    let loop_start_idx = program.code.len();

    // Increment counter: R12 = R12 + 1
    // program.add_instruction(Instruction::create_reg_imm8(Opcode::Add, 12, 1));       //tombe dans une loop infini

    // // Compare counter to max
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 12, 13));
    //
    // Calculate offset for jump back
    let current_idx = program.code.len();
    let code_size_to_loop_start =
        calculate_instruction_range_size(&program.code, loop_start_idx, current_idx);
    let loop_offset = -(code_size_to_loop_start as i8);

    // Jump if not equal (R12 != R13)
    // let jump_instruction = create_conditional_jump(loop_offset); // ici  on a  Erreur lors de l'exécution: ExecutionError:
    // program.add_instruction(jump_instruction);   Erreur pipeline: Format d'adresse de saut conditionnel invalide

    // ---------- Final Verification ----------
    // Store results to verify correct execution
    program.add_instruction(Instruction::create_reg_reg_offset(Opcode::Store, 5, 6, 12)); // Store R5 at [R6+12]
    program.add_instruction(Instruction::create_reg_reg_offset(Opcode::Store, 11, 6, 16)); // Store R11 at [R6+16]
    program.add_instruction(Instruction::create_reg_reg_offset(Opcode::Store, 12, 6, 20)); // Store R12 at [R6+20]

    // End program
    program.add_instruction(Instruction::create_no_args(Opcode::Halt));

    // Configure memory segments
    let total_code_size: u32 = program
        .code
        .iter()
        .map(|instr| instr.total_size() as u32)
        .sum();

    let data_size = 512; // 512 bytes for data

    program.segments = vec![
        SegmentMetadata::new(SegmentType::Code, 0, total_code_size, 0),
        SegmentMetadata::new(SegmentType::Data, 0, data_size, 0x1000),
    ];

    program.data = vec![0; data_size as usize];

    println!(
        "Programme de test du pipeline créé avec {} instructions",
        program.code.len()
    );

    program
}

/// Fonction utilitaire: calcule la taille totale des instructions dans une plage
fn calculate_instruction_range_size(
    instructions: &[Instruction],
    start: usize,
    end: usize,
) -> usize {
    instructions[start..end]
        .iter()
        .map(|instr| instr.total_size())
        .sum()
}

/// Fonction utilitaire: crée une instruction Store avec offset
fn create_reg_reg_offset(opcode: Opcode, rs: u8, rb: u8, offset: i8) -> Instruction {
    // Cette implémentation dépend de votre format d'instruction
    // Supposons que le format soit (reg_dest, reg_base + offset)
    Instruction::new(
        opcode,
        InstructionFormat::new(ArgType::Register, ArgType::RegisterOffset, ArgType::None),
        vec![rs, rb, offset as u8],
    )
}


pub fn create_hazard_detection_test_program() -> BytecodeFile {
    let mut program = BytecodeFile::new();
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    program.add_metadata("name", "Hazard Detection Test");
    program.add_metadata(
        "description",
        "Programme testant la détection des hazards et stalls.",
    );

    // -------------------------------
    // Test 1: Load-Use Hazard
    // Ce type de hazard se produit quand on essaie d'utiliser le résultat
    // d'un LOAD avant qu'il ne soit disponible
    // -------------------------------

    // Initialiser une adresse mémoire
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 0x10)); // R0 = adresse 0x10

    // Stocker une valeur à cette adresse
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 42)); // R1 = 42

    // Store - utiliser la méthode disponible create_reg_reg_offset
    program.add_instruction(Instruction::create_reg_reg_offset(Opcode::Store, 1, 0, 0)); // MEM[R0] = R1

    // Load - utiliser create_load_reg_offset
    program.add_instruction(Instruction::create_load_reg_offset(2, 0, 0)); // R2 = MEM[R0]

    // Load-Use Hazard: utiliser la valeur immédiatement (devrait générer un hazard)
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 3, 2, 1)); // R3 = R2 + R1 (hazard!)

    // -------------------------------
    // Test 2: RAW Hazards multiples en chaîne
    // Crée une séquence de dépendances entre instructions qui se suivent
    // -------------------------------

    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 4, 5)); // R4 = 5

    // Série d'instructions dépendantes (RAW hazards)
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 5, 4, 4)); // R5 = R4 + R4
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 6, 5, 5)); // R6 = R5 + R5 (dépend du résultat précédent)
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 7, 6, 6)); // R7 = R6 + R6 (dépend du résultat précédent)

    // -------------------------------
    // Test 3: Hazard de contrôle (branchement)
    // Test si un branchement cause un hazard et un flush
    // -------------------------------

    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 8, 1)); // R8 = 1
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 1)); // R9 = 1

    // Compare R8 et R9
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 8, 9));

    // program.add_instruction(Instruction::new(Opcode::JmpIf, jmpif_format, jmpif_args));
    // program.add_instruction(Instruction::create_jump_if(14)); // JmpIf (devrait être pris)
    program.add_instruction(Instruction::create_jump_if_not(14)); // JmpIfNot (devrait être pris)
    // program.add_instruction(Instruction::create_jump_if_less_equal(14)); // JmpIfEqual (devrait être pris)
    // program.add_instruction(Instruction::create_jump_if_not_equal(14)); // JmpIfNotEqual (devrait être pris)
    // program.add_instruction(Instruction::create_jump(14)); // Jmp (devrait être pris)
    // program.add_instruction(Instruction::create_jump_if_equal(8)); // JmpIfEqual (devrait être pris)

    // Instructions qui seront sautées si le branchement est pris
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 10, 0xFF)); // R10 = 0xFF (ne devrait pas être exécuté)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 11, 0xFF)); // R11 = 0xFF (ne devrait pas être exécuté)

    // Destination du saut
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 12, 0xAA)); // R12 = 0xAA

    // -------------------------------
    // Test 4: Store-Load Hazard
    // Une écriture suivie d'une lecture à la même adresse
    // -------------------------------

    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 13, 0x20)); // R13 = adresse 0x20
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 14, 77)); // R14 = 77

    // Store suivi d'un Load à la même adresse
    program.add_instruction(Instruction::create_reg_reg_offset(Opcode::Store, 14, 13, 0)); // MEM[R13] = R14
    program.add_instruction(Instruction::create_load_reg_offset(15, 13, 0)); // R15 = MEM[R13] (hazard potentiel)

    // -------------------------------
    // Test 5: Hazard structurel (accès mémoire simultanés)
    // Plusieurs accès mémoire qui peuvent causer des conflits de ressources
    // -------------------------------

    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 0x30)); // R0 = adresse 0x30
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 0x40)); // R1 = adresse 0x40

    // Accès mémoire multiples consécutifs
    program.add_instruction(Instruction::create_reg_reg_offset(Opcode::Store, 4, 0, 0)); // MEM[R0] = R4
    program.add_instruction(Instruction::create_reg_reg_offset(Opcode::Store, 5, 1, 0)); // MEM[R1] = R5
    program.add_instruction(Instruction::create_load_reg_offset(6, 0, 0)); // R6 = MEM[R0]
    program.add_instruction(Instruction::create_load_reg_offset(7, 1, 0)); // R7 = MEM[R1]

    // Fin du programme
    program.add_instruction(Instruction::create_no_args(Opcode::Halt));

    // Calculer la taille totale du code et créer le segment
    let total_size: u32 = program
        .code
        .iter()
        .map(|instr| instr.total_size() as u32)
        .sum();
    program.segments = vec![SegmentMetadata::new(SegmentType::Code, 0, total_size, 0)];

    // Créer un segment de données
    let data_size = 256;
    let data_segment = SegmentMetadata::new(Data, 0, data_size, 0x1000);
    program.segments.push(data_segment);
    program.data = vec![0; data_size as usize];

    println!("Carte des instructions");
    let mut addr = 0;
    for (idx, instr) in program.code.iter().enumerate() {
        let size = instr.total_size();
        println!(
            "Instruction {}: Adresse 0x{:04X}-0x{:04X} (taille {}): {:?}",
            idx,
            addr,
            addr + size - 1,
            size,
            instr.opcode
        );
        addr += size;
    }

    program
}

pub fn create_all_branch_test_program() -> BytecodeFile {
    let mut program = BytecodeFile::new();
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    program.add_metadata("name", "Branch Instructions Test");
    program.add_metadata(
        "description",
        "Programme testant les différentes instructions de branchement.",
    );

    // Initialiser les registres pour les tests
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 0)); // R0 = 0
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 1)); // R1 = 1
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 2, 10)); // R2 = 10
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 15, 0)); // R15 = 0 (compteur de tests réussis)

    // ================================
    // Test 1: JmpIfEqual (ZF = 1)
    // ================================
    // Compare R1 et R1 (égaux => ZF = 1)
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 1, 1));
    // Branchement (devrait être pris car ZF = 1)
    program.add_instruction(Instruction::create_jump_if_equal(2));
    // Si le branchement n'est pas pris (échec)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 3, 0xFF));
    // Si le branchement est pris (succès)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 3, 0x01));
    // Incrémenter le compteur de tests réussis
    program.add_instruction(Instruction::create_reg_reg(Opcode::Add, 15, 3));

    // ================================
    // Test 2: JmpIfNotEqual (ZF = 0)
    // ================================
    // Compare R1 et R2 (différents => ZF = 0)
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 1, 2));
    // Branchement (devrait être pris car ZF = 0)
    program.add_instruction(Instruction::create_jump_if_not_equal(2));
    // Si le branchement n'est pas pris (échec)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 4, 0xFF));
    // Si le branchement est pris (succès)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 4, 0x01));
    // Incrémenter le compteur de tests réussis
    program.add_instruction(Instruction::create_reg_reg(Opcode::Add, 15, 4));

    // ================================
    // Test 3: JmpIfLess (SF = 1)
    // ================================
    // Compare R0 et R1 (R0 < R1 => SF = 1)
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 1));
    // Branchement (devrait être pris car SF = 1)
    program.add_instruction(Instruction::create_jump_if_less(2));
    // Si le branchement n'est pas pris (échec)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 5, 0xFF));
    // Si le branchement est pris (succès)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 5, 0x01));
    // Incrémenter le compteur de tests réussis
    program.add_instruction(Instruction::create_reg_reg(Opcode::Add, 15, 5));

    // ================================
    // Test 4: JmpIfGreater (SF = 0, ZF = 0)
    // ================================
    // Compare R2 et R1 (R2 > R1 => SF = 0, ZF = 0)
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 2, 1));
    // Branchement (devrait être pris car SF = 0, ZF = 0)
    program.add_instruction(Instruction::create_jump_if_greater(2));
    // Si le branchement n'est pas pris (échec)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 6, 0xFF));
    // Si le branchement est pris (succès)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 6, 0x01));
    // Incrémenter le compteur de tests réussis
    program.add_instruction(Instruction::create_reg_reg(Opcode::Add, 15, 6));

    // ================================
    // Test 5: JmpIfLessEqual (SF = 1 ou ZF = 1)
    // ================================
    // Compare R0 et R0 (égaux => ZF = 1)
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 0));
    // Branchement (devrait être pris car ZF = 1)
    program.add_instruction(Instruction::create_jump_if_less_equal(2));
    // Si le branchement n'est pas pris (échec)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 7, 0xFF));
    // Si le branchement est pris (succès)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 7, 0x01));
    // Incrémenter le compteur de tests réussis
    program.add_instruction(Instruction::create_reg_reg(Opcode::Add, 15, 7));

    // ================================
    // Test 6: JmpIfGreaterEqual (SF = 0 ou ZF = 1)
    // ================================
    // Compare R1 et R1 (égaux => ZF = 1)
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 1, 1));
    // Branchement (devrait être pris car ZF = 1)
    program.add_instruction(Instruction::create_jump_if_greater_equal(2));
    // Si le branchement n'est pas pris (échec)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 8, 0xFF));
    // Si le branchement est pris (succès)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 8, 0x01));
    // Incrémenter le compteur de tests réussis
    program.add_instruction(Instruction::create_reg_reg(Opcode::Add, 15, 8));

    // ================================
    // Test 7: Saut inconditionnel (Jmp)
    // ================================
    // Saut inconditionnel (devrait toujours être pris)
    program.add_instruction(Instruction::create_jump(2));
    // Si le branchement n'est pas pris (échec)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 0xFF));
    // Si le branchement est pris (succès)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 0x01));
    // Incrémenter le compteur de tests réussis
    program.add_instruction(Instruction::create_reg_reg(Opcode::Add, 15, 9));

    // ================================
    // Test 8: JmpIfNotZero (ZF = 0)
    // ================================
    // Compare R1 et R2 (différents => ZF = 0)
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 1, 2));
    // Branchement (devrait être pris car ZF = 0)
    program.add_instruction(Instruction::create_jump_if_not_zero(2));
    // Si le branchement n'est pas pris (échec)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 10, 0xFF));
    // Si le branchement est pris (succès)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 10, 0x01));
    // Incrémenter le compteur de tests réussis
    program.add_instruction(Instruction::create_reg_reg(Opcode::Add, 15, 10));

    // ================================
    // Test 9: JmpIfZero (ZF = 1)
    // ================================
    // Compare R1 et R1 (égaux => ZF = 1)
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 1, 1));
    // Branchement (devrait être pris car ZF = 1)
    program.add_instruction(Instruction::create_jump_if_zero(2));
    // Si le branchement n'est pas pris (échec)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 11, 0xFF));
    // Si le branchement est pris (succès)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 11, 0x01));
    // Incrémenter le compteur de tests réussis
    program.add_instruction(Instruction::create_reg_reg(Opcode::Add, 15, 11));

    // ================================
    // Test 10: Test avec Saut Négatif
    // ================================
    // Préparation d'un registre pour le test
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 12, 0));
    // Étiquette pour le début de la boucle
    let loop_start_idx = program.code.len();

    // Incrémenter R12
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Add, 12, 1));

    // Comparer R12 avec 3
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Cmp, 12, 3));

    // Si R12 < 3, retourner au début de la boucle
    let loop_end_idx = program.code.len();

    // Calcul de l'offset pour revenir au début de la boucle
    let offset = -(((loop_end_idx - loop_start_idx) * 6) as i32); // 6 est la taille moyenne d'une instruction

    // Créer un saut relatif en arrière
    let jmpif_format = InstructionFormat::new(ArgType::None, ArgType::RelativeAddr, ArgType::None);
    let offset_bytes = offset.to_le_bytes();
    let mut jmpif_args = Vec::new();
    jmpif_args.extend_from_slice(&offset_bytes);
    program.add_instruction(Instruction::new(
        Opcode::JmpIfLess,
        jmpif_format,
        jmpif_args,
    ));

    // Si on est ici, R12 devrait valoir 3, ce qui est un succès
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Cmp, 12, 3));
    program.add_instruction(Instruction::create_jump_if_equal(2));
    // Si le test a échoué
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 13, 0));
    // Si le test a réussi
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 13, 0x01));
    // Incrémenter le compteur de tests réussis
    program.add_instruction(Instruction::create_reg_reg(Opcode::Add, 15, 13));

    // Fin du programme
    program.add_instruction(Instruction::create_no_args(Opcode::Halt));

    // Calculer la taille totale du code et créer le segment
    let total_size: u32 = program
        .code
        .iter()
        .map(|instr| instr.total_size() as u32)
        .sum();
    program.segments = vec![SegmentMetadata::new(SegmentType::Code, 0, total_size, 0)];

    let mut current_addr = 0;
    for (i, instr) in program.code.iter().enumerate() {
        let sz = instr.total_size();
        println!(
            "Instruction {} à l'adresse 0x{:X}, taille = {}, opcode={:?}",
            i, current_addr, sz, instr.opcode
        );
        current_addr += sz;
    }

    println!("total size {} \n", total_size);

    // Créer un segment de données vide
    let data_size = 256;
    let data_segment = SegmentMetadata::new(SegmentType::Data, 0, data_size, 0x1000);
    program.segments.push(data_segment);
    program.data = vec![0; data_size as usize];

    program
}

pub fn create_branch_test_program_debug() -> BytecodeFile {
    let mut program = BytecodeFile::new();
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    program.add_metadata("name", "Branch Simple Test");
    program.add_metadata("description", "Programme de test simple de branchement");

    // Initialiser les registres
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 0)); // R0 = 0
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 1)); // R1 = 1

    // Instruction de saut inconditionnel (sauter la prochaine instruction)
    // Important: utiliser un grand offset pour s'assurer de sauter l'instruction de mov R2,0xFF
    program.add_instruction(Instruction::create_jump(16));

    // Cette instruction ne devrait pas être exécutée
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 2, 0xFF)); // R2 = 0xFF

    // Cette instruction devrait être exécutée après le saut
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 2, 0xAA)); // R2 = 0xAA

    // Fin du programme
    program.add_instruction(Instruction::create_no_args(Opcode::Halt));

    // Calculer la taille totale du code et créer le segment
    let total_size: u32 = program
        .code
        .iter()
        .map(|instr| instr.total_size() as u32)
        .sum();
    program.segments = vec![SegmentMetadata::new(SegmentType::Code, 0, total_size, 0)];

    let mut current_addr = 0;
    for (i, instr) in program.code.iter().enumerate() {
        let sz = instr.total_size();
        println!(
            "Instruction {} à l'adresse 0x{:X}, taille = {}, opcode={:?}",
            i, current_addr, sz, instr.opcode
        );
        current_addr += sz;
    }

    println!("total size {} \n", total_size);

    // Créer un segment de données vide
    let data_size = 256;
    let data_segment = SegmentMetadata::new(SegmentType::Data, 0, data_size, 0x1000);
    program.segments.push(data_segment);
    program.data = vec![0; data_size as usize];

    program
}

pub fn create_simple_test_program() -> BytecodeFile {
    let mut program = BytecodeFile::new();
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    program.add_metadata("name", "Branch Testing Program");
    program.add_metadata(
        "description",
        "Programme testant différents types de branchements",
    );


    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 5)); // R0 = 5
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 0)); // R1 = 10
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 2, 0, 1)); // R2 = R0 + R1
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 3, 100));  //R3 = 100
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 4,10));  //R4 = 50
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Mul, 5, 3, 4)); //  R5 = R3 * R4

    // R0 = 1
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 10));

    // Jmp +2 (sauter par-dessus l'instruction suivante)
    // Calculez l'offset pour sauter à l'instruction R1 = 42
    program.add_instruction(Instruction::create_jump(6)); // offset pour sauter une instruction

    // R0 = 0xFF (ne devrait jamais être exécuté)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 0xFF));

    // R1 = 42 (devrait être exécuté après le saut)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 42));

    // Access memoire multiple consecutif
    program.add_instruction(Instruction::create_reg_reg_offset(Opcode::Store,1,0,0)); // MEM[R0] = R1
    program.add_instruction(Instruction::create_reg_reg_offset(Opcode::Store,0,1,0)); // MEM[R1] = R0
    program.add_instruction(Instruction::create_load_reg_offset(6,0,0)); // R6 = MEM[R0]
    program.add_instruction(Instruction::create_load_reg_offset(7,1,0)); // R7 = MEM[R1]
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add,8,6,7)); // R8 = R6 + R7


    // HALT
    program.add_instruction(Instruction::create_no_args(Opcode::Halt));

    // Mise à jour des segments
    let total_size: u32 = program
        .code
        .iter()
        .map(|instr| instr.total_size() as u32)
        .sum();
    program.segments = vec![SegmentMetadata::new(SegmentType::Code, 0, total_size, 0)];

    // Créer un segment de données vide
    let data_size = 256;
    let data_segment = SegmentMetadata::new(SegmentType::Data, 0, data_size, 0x1000);
    program.segments.push(data_segment);
    program.data = vec![0; data_size as usize];

    // Afficher la carte des instructions pour déboggage
    println!("Carte des instructions du programme de test des branchements");
    let mut addr = 0;
    for (idx, instr) in program.code.iter().enumerate() {
        let size = instr.total_size();
        println!(
            "Instruction {}: Adresse 0x{:04X}-0x{:04X} (taille {}): {:?}",
            idx,
            addr,
            addr + size - 1,
            size,
            instr.opcode
        );
        addr += size;
    }

    program
}

pub fn create_conditional_branch_test_program() -> BytecodeFile {
    let mut program = BytecodeFile::new();
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    program.add_metadata("name", "Conditional Branch Testing Program");
    program.add_metadata(
        "description",
        "Programme testant les branchements conditionnels",
    );

    // Initialisation des registres
    // R0 = 5 (compteur de boucle)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 5));
    // R1 = 0 (accumulateur)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 0));
    // R2 = 1 (constante pour décrémentation)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 2, 1));
    // R3 = 10 (valeur de test)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 3, 10));
    // R4 = 0 (code de sortie)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 4, 0));

    // Marquons le début de la boucle
    let loop_start_index = program.code.len();

    // Test 1: Vérifier si l'accumulateur est égal à la valeur de test (R1 == R3)
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 1, 3));

    // Nous allons placer ici un JmpIfEqual, l'adresse sera mise à jour plus tard
    let jump_if_equal_index = program.code.len();
    program.add_instruction(Instruction::create_jump_if_equal(8)); // Placeholder

    // Incrémenter l'accumulateur: R1 += R0
    program.add_instruction(Instruction::create_reg_reg(Opcode::Add, 1, 0));

    // Décrémenter le compteur: R0 -= R2 (R2 = 1)
    program.add_instruction(Instruction::create_reg_reg(Opcode::Sub, 0, 2));

    // Test 2: Vérifier si le compteur est non-zéro (R0 != 0)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Cmp, 0, 0));

    // Si R0 != 0, revenir au début de la boucle
    let jump_back_index = program.code.len();
    program.add_instruction(Instruction::create_jump_if_not_equal(0)); // Placeholder

    // Si nous arrivons ici, la boucle est terminée naturellement
    // R4 = 0 (déjà défini, code de sortie normale)

    // Saut inconditionnel vers la fin
    let jump_to_exit_index = program.code.len();
    program.add_instruction(Instruction::create_jump(0)); // Placeholder

    // SORTIE ANTICIPÉE:
    // Si R1 == R3, on saute ici depuis le début de la boucle
    let early_exit_index = program.code.len();

    // Définir R4 = 1 (code de sortie anticipée)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 4, 1));

    // SORTIE FINALE:
    let exit_index = program.code.len();

    // Halt
    program.add_instruction(Instruction::create_no_args(Opcode::Halt));

    // Maintenant, mettons à jour les adresses de saut

    // Calculer toutes les adresses et tailles
    let mut address_map = HashMap::new();
    let mut size_map = HashMap::new();
    let mut current_addr = 0;

    for (idx, instr) in program.code.iter().enumerate() {
        address_map.insert(idx, current_addr);
        let size = instr.total_size() as u32;
        size_map.insert(idx, size);
        current_addr += size;
    }

    // Mettre à jour le saut conditionnel avant (JmpIfEqual vers early_exit)
    let jump_if_equal_size = size_map[&jump_if_equal_index];
    let early_exit_addr = address_map[&early_exit_index];
    let jump_from_addr = address_map[&jump_if_equal_index] + jump_if_equal_size;
    let forward_offset = early_exit_addr.checked_sub(jump_from_addr).unwrap_or(0);

    // Remplacer l'instruction placeholder par l'instruction réelle avec le bon offset
    program.code[jump_if_equal_index] = Instruction::create_jump_if_equal(forward_offset as i32);

    // Mettre à jour le saut conditionnel arrière (JmpIfNotEqual vers loop_start)
    let jump_back_size = size_map[&jump_back_index];
    let loop_start_addr = address_map[&loop_start_index];
    let jump_back_from_addr = address_map[&jump_back_index] + jump_back_size;
    let backward_offset = loop_start_addr as i32 - jump_back_from_addr as i32;

    program.code[jump_back_index] = Instruction::create_jump_if_not_equal(backward_offset);

    // Mettre à jour le saut inconditionnel (Jmp vers exit)
    let jump_to_exit_size = size_map[&jump_to_exit_index];
    let exit_addr = address_map[&exit_index];
    let jump_to_exit_from_addr = address_map[&jump_to_exit_index] + jump_to_exit_size;
    let exit_offset = exit_addr.checked_sub(jump_to_exit_from_addr).unwrap_or(0);

    program.code[jump_to_exit_index] = Instruction::create_jump(exit_offset as i32);

    // Finaliser les segments
    let total_size: u32 = program
        .code
        .iter()
        .map(|instr| instr.total_size() as u32)
        .sum();
    program.segments = vec![SegmentMetadata::new(SegmentType::Code, 0, total_size, 0)];

    // Ajouter un segment de données vide
    let data_size = 256;
    let data_segment = SegmentMetadata::new(SegmentType::Data, 0, data_size, 0x1000);
    program.segments.push(data_segment);
    program.data = vec![0; data_size as usize];

    // Afficher la carte des instructions pour déboggage
    println!("Carte des instructions du programme de test des branchements conditionnels");
    let mut addr = 0;
    for (idx, instr) in program.code.iter().enumerate() {
        let size = instr.total_size();
        println!(
            "Instruction {}: Adresse 0x{:04X}-0x{:04X} (taille {}): {:?}",
            idx,
            addr,
            addr + size - 1,
            size,
            instr.opcode
        );
        addr += size;
    }

    program
}





fn momo_program() -> BytecodeFile{

    println!("=====Debut du programme MOMO PROGRAM =====");

    let mut program = BytecodeFile::new();
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    program.add_metadata("name", "Momo Test");
    program.add_metadata(
        "description",
        "Momo Test",
    );



    // ici   je  vais  metre les instruction pour la VM

    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 5)); // R0 = 5
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 10)); // R1 = 10
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 2, 0, 1)); // R2 = R0 + R1
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 3, 100));  //R3 = 100
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 4,100));  //R4 = 50
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Div, 5, 3, 4)); //  R5 = R3 / R4

    //program.add_instruction(Instruction::create_jump(9));

    // ici  on va  mettre Jump

    // Jmp +2 (sauter par-dessus l'instruction suivante)
    // Calculez l'offset pour sauter à l'instruction R1 = 42
    program.add_instruction(Instruction::create_jump(6)); // offset pour sauter une instruction

    // R0 = 0xFF (ne devrait jamais être exécuté)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 0xFF));

    // R1 = 42 (devrait être exécuté après le saut)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 42));

    // Test Cmp
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 16, 25)); // R16 = 25
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 17, 25)); // R17 = 25
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 16, 17)); // Cmp R16, R17 (ZF=1)


    // Note: I removed the memory access tests as they seem incomplete or commented out in your original code.
    // Accès mémoire multiples consécutifs
    //program.add_instruction(Instruction::create_reg_reg_offset(Opcode::Store, 0, 0, 0)); // MEM[R0] = R4
    //program.add_instruction(Instruction::create_reg_reg_offset(Opcode::Store, 5, 1, 0)); // MEM[R1] = R5
    //program.add_instruction(Instruction::create_load_reg_offset(6, 0, 0)); // R6 = MEM[R0]
    //program.add_instruction(Instruction::create_load_reg_offset(7, 1, 0)); // R7 = MEM[R1]
    //program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 8, 6, 7));
    //program.add_instruction(Instruction::create_reg_reg(Opcode::Mov, 8, 6));
    //program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 8,10));
    //program.add_instruction(Instruction::create_reg_reg_offset(Opcode::Store,9,2,0));
    //program.add_instruction(Instruction::create_load_reg_offset(9, 2, 0));


    // Test  de  Load Hazard

    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 0x10));
    // Note: R3 is used as base register here, make sure its value (100) is a valid memory address
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 10, 42)); // R1 = 42
    program.add_instruction(Instruction::create_reg_reg_offset(Opcode::Store, 10, 3, 0));

    // Load - utiliser create_load_reg_offset
    program.add_instruction(Instruction::create_load_reg_offset(11, 3, 0)); // R2 = MEM[R3]

    // Load-Use Hazard: utiliser la valeur immédiatement (devrait générer un hazard)
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 12, 10, 11)); // R12 = R10 + R11 (hazard!)




    // Note: R4 is used as a source register, ensure its value (100) makes sense for ADD operations.
    // Série d'instructions dépendantes (RAW hazards)
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 5, 4, 4)); // R5 = R4 + R4
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 6, 5, 5)); // R6 = R5 + R5 (dépend du résultat précédent)
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 7, 6, 6)); // R7 = R6 + R6 (dépend du résultat précédent)





    // -------------------------------
    // Test 3: Hazard de contrôle (branchement)
    // Test si un branchement cause un hazard et un flush
    // -------------------------------

    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 8, 1)); // R8 = 1
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 1)); // R9 = 1

    // Compare R8 et R9
    // Note: Cmp is used here, which sets the flags. JmpIf instructions then check these flags.
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 8, 9));

    // Example of Jump If Not Equal:
    // This will not jump because R8 == R9, so ZF is 1, JmpIfNotEqual (ZF=0) is false.
    // The offset (14) is an example, you'll need to calculate the correct relative offset.
    let target_instruction_after_jmp = program.code.len() + 2; // Assuming the next two instructions are skipped if jump is taken
    let jump_from_instruction = program.code.len();
    // You need to calculate the actual offset based on instruction sizes.
    // For now, using a placeholder offset.
    let placeholder_offset_not_equal = 10; // Placeholder offset
    program.add_instruction(Instruction::create_jump_if_not_equal(placeholder_offset_not_equal));

    // Example of Jump If Equal:
    // This *will* jump because R8 == R9, so ZF is 1, JmpIfEqual (ZF=1) is true.
    program.add_instruction(Instruction::create_jump_if(14));

    //program.add_instruction(Instruction::create_jump_if_not(14)); // JmpIfNot (devrait être pris)




    // Fin du programme
    program.add_instruction(Instruction::create_no_args(Opcode::Halt));


    let total_size: u32 = program
        .code
        .iter()
        .map(|instr| instr.total_size() as u32)
        .sum();

    program.segments = vec![SegmentMetadata::new(SegmentType::Code,0,total_size,0)];

    // Créer un segment de données
    let data_size = 256;
    let data_segment = SegmentMetadata::new(SegmentType::Data, 0, data_size, 0x1000);
    program.segments.push(data_segment);
    program.data = vec![0; data_size as usize];

    println!("Carte des instructions");
    let mut addr = 0;
    for (idx, instr) in program.code.iter().enumerate() {
        let size = instr.total_size();
        println!(
            "Instruction {}: Adresse 0x{:04X}-0x{:04X} (taille {}): {:?}",
            idx,
            addr,
            addr + size - 1,
            size,
            instr.opcode
        );

        // Vérification spéciale pour les branchements
        if instr.opcode.is_branch() {
            if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
                let target = ( size  as u32 + addr as u32 ) as i64 + offset as i64;
                println!("  -> Branchement relatif: offset={}, target=0x{:04X}", offset, target);
            }
        }
        addr += size;
    }


    program
}



pub fn punk_program() -> BytecodeFile {
    println!("===== Debut du programme PUNK PROGRAM Test =====");
    let mut program = BytecodeFile::new();
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    program.add_metadata("name", "Punk Branch Test Program");
    program.add_metadata(
        "description",
        "Programme testant toutes les instructions de branchement.",
    );
    // Initialiser des registres pour les tests
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 10)); // R0 = 10
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 20)); // R1 = 20
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 2, 10)); // R2 = 10
    // --- Test Jmp (inconditionnel) ---
    // On va sauter par-dessus les deux prochaines instructions
    program.add_instruction(Instruction::create_jump(12)); // Offset calculé (taille de 2x Mov instructions)
    // Cette instruction ne doit pas être exécutée
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 3, 0xFF));
    // Cette instruction doit être exécutée après le saut
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 3, 0x11)); // R3 = 0x11
    // --- Test Cmp et JmpIfEqual (ZF = 1) ---
    // Compare R0 et R2 (égaux)
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 2)); // R0 == R2 => ZF = 1
    // Sauter si égal (ZF = 1), doit être pris
    program.add_instruction(Instruction::create_jump_if_equal(12)); // Offset pour sauter les deux prochaines instructions
    // Cette instruction ne doit pas être exécutée
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 4, 0xFF));
    // Cette instruction doit être exécutée après le saut
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 4, 0x22)); // R4 = 0x22
    // --- Test Cmp et JmpIfNotEqual (ZF = 0) ---
    // Compare R0 et R1 (différents)
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 1)); // R0 != R1 => ZF = 0
    // Sauter si non égal (ZF = 0), doit être pris
    program.add_instruction(Instruction::create_jump_if_not_equal(12)); // Offset pour sauter les deux prochaines instructions
    // Cette instruction ne doit pas être exécutée
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 5, 0xFF));
    // Cette instruction doit être exécutée après le saut
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 5, 0x33)); // R5 = 0x33
    // --- Test Cmp et JmpIfGreater (SF = 0, ZF = 0) ---
    // Compare R1 et R0 (R1 > R0)
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 1, 0)); // R1 > R0 => SF = 0, ZF = 0
    // Sauter si plus grand, doit être pris
    program.add_instruction(Instruction::create_jump_if_greater(12)); // Offset pour sauter les deux prochaines instructions
    // Cette instruction ne doit pas être exécutée
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 6, 0xFF));
    // Cette instruction doit être exécutée après le saut
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 6, 0x44)); // R6 = 0x44
    // --- Test Cmp et JmpIfLess (SF = 1) ---
    // Compare R0 et R1 (R0 < R1)
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 1)); // R0 < R1 => SF = 1
    // Sauter si plus petit, doit être pris
    program.add_instruction(Instruction::create_jump_if_less(12)); // Offset pour sauter les deux prochaines instructions
    // Cette instruction ne doit pas être exécutée
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 7, 0xFF));
    // Cette instruction doit être exécutée après le saut
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 7, 0x55)); // R7 = 0x55
    // --- Test Cmp et JmpIfGreaterEqual (SF = 0 ou ZF = 1) ---
    // Compare R0 et R2 (égaux)
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 2)); // R0 == R2 => ZF = 1
    // Sauter si plus grand ou égal, doit être pris
    program.add_instruction(Instruction::create_jump_if_greater_equal(12)); // Offset pour sauter les deux prochaines instructions
    // Cette instruction ne doit pas être exécutée
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 8, 0xFF));
    // Cette instruction doit être exécutée après le saut
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 8, 0x66)); // R8 = 0x66
    // --- Test Cmp et JmpIfLessEqual (SF = 1 ou ZF = 1) ---
    // Compare R0 et R2 (égaux)
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 2)); // R0 == R2 => ZF = 1
    // Sauter si plus petit ou égal, doit être pris
    program.add_instruction(Instruction::create_jump_if_less_equal(12)); // Offset pour sauter les deux prochaines instructions
    // Cette instruction ne doit pas être exécutée
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 0xFF));
    // Cette instruction doit être exécutée après le saut
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 0x77)); // R9 = 0x77
    // --- Test Cmp et JmpIfZero (ZF = 1) ---
    // Compare R0 et R2 (égaux)
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 2)); // R0 == R2 => ZF = 1
    // Sauter si zéro, doit être pris
    program.add_instruction(Instruction::create_jump_if_zero(12)); // Offset pour sauter les deux prochaines instructions
    // Cette instruction ne doit pas être exécutée
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 10, 0xFF));
    // Cette instruction doit être exécutée après le saut
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 10, 0x88)); // R10 = 0x88
    // --- Test Cmp et JmpIfNotZero (ZF = 0) ---
    // Compare R0 et R1 (différents)
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 1)); // R0 != R1 => ZF = 0
    // Sauter si non zéro, doit être pris
    program.add_instruction(Instruction::create_jump_if_not_zero(12)); // Offset pour sauter les deux prochaines instructions
    // Cette instruction ne doit pas être exécutée
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 11, 0xFF));
    // Cette instruction doit être exécutée après le saut
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 11, 0x99)); // R11 = 0x99
    // Fin du programme
    program.add_instruction(Instruction::create_no_args(Opcode::Halt));
    // Configuration des segments (Code et Data)
    let total_code_size: u32 = program.code.iter().map(|instr| instr.total_size() as u32).sum();
    program.segments.push(SegmentMetadata::new(SegmentType::Code, 0, total_code_size, 0));
    let data_size = 256; // Taille du segment de données
    program.segments.push(SegmentMetadata::new(SegmentType::Data, 0, data_size, total_code_size)); // Segment data après le code
    program.data = vec![0; data_size as usize];
    // Afficher la carte des instructions
    println!("\n--- Carte des instructions du programme de test des branchements ---");
    let mut addr = 0;
    for (idx, instr) in program.code.iter().enumerate() {
        let size = instr.total_size();
        println!(
            "Instruction {}: Adresse 0x{:04X}-0x{:04X} (taille {}): {:?}",
            idx, addr, addr + size - 1, size, instr.opcode
        );

        // Vérification spéciale pour les branchements
        if instr.opcode.is_branch() {
            if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
                let target = ( size  as u32 + addr as u32 ) as i64 + offset as i64;
                println!("  -> Branchement relatif: offset={}, target=0x{:04X}", offset, target);
            }
        }
        addr += size;
    }
    println!("--- Fin de la carte des instructions ---\n");
    program
}


pub fn punk_program_2() -> BytecodeFile {
    println!("===Debu du test de Punk Program 2===\n");
    let mut program = BytecodeFile::new();
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    program.add_metadata("name", "Punk Branch Test 2");
    program.add_metadata(
        "description",
        "Programme testant les branchements avec cas non pris, offsets variés, et instructions intermédiaires.",
    );

    // Initialisation des registres
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 10)); // R0 = 10
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 20)); // R1 = 20
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 2, 10)); // R2 = 10
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 15, 0)); // R15 = 0 (pour marquer les succès)

    // --- Test 1: JmpIfEqual (condition VRAIE, branchement PRIS) ---
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 2)); // R0 == R2 => ZF = 1
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 5, 2)); // Instruction intermédiaire 1
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 4, 2)); // Instruction intermédiaire 2
    // Sauter si égal (ZF=1). Offset pour sauter les 2 instructions intermédiaires + l'instruction de "échec".
    // 2 instructions * 6 bytes/instr (estimation) + 1 instruction * 6 bytes/instr = 18 bytes
    program.add_instruction(Instruction::create_jump_if_equal(18)); // Offset calculé
    // Cette instruction ne doit pas être exécutée (échec)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 15, 0xFF)); // 255
    // Cette instruction doit être exécutée après le saut (succès)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 15, 1)); // Marquer succès test 1


    // --- Test 2: JmpIfNotEqual (condition FAUSSE, branchement NON PRIS) ---
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 2)); // R0 == R2 => ZF = 1
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 3, 3)); // Instruction intermédiaire 1
    // Sauter si non égal (ZF=0). Cette condition est FAUSSE, le saut NE DOIT PAS être pris.
    // L'offset pointe par-dessus l'instruction de "succès".
    // Taille de Mov R15, 0xAA = 6 bytes
    program.add_instruction(Instruction::create_jump_if_not_equal(6)); // Offset calculé
    // Cette instruction doit être exécutée car le branchement n'est pas pris (succès)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 15, 2)); // Marquer succès test 2
    // Cette instruction ne doit pas être exécutée car elle est la cible du saut (échec potentiel si le saut était pris)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 15, 0xFF)); //255

    // --- Test 3: JmpIfLess (condition FAUSSE, branchement NON PRIS) ---
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 1, 0)); // R1 > R0 => SF = 0
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 3, 4)); // Instruction intermédiaire 1
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 4, 5)); // Instruction intermédiaire 2
    // Sauter si plus petit (SF=1). Cette condition est FAUSSE, le saut NE DOIT PAS être pris.
    // L'offset pointe par-dessus l'instruction de "succès".
    // Taille de Mov R15, 0xBB = 6 bytes
    program.add_instruction(Instruction::create_jump_if_less(6)); // Offset calculé
    // Cette instruction doit être exécutée (succès)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 15, 3)); // Marquer succès test 3
    // Cette instruction ne doit pas être exécutée
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 15, 0xFF));   //255

    // --- Test 4: Saut inconditionnel (Jmp) avec un offset différent ---
    // Sauter par-dessus une seule instruction
    program.add_instruction(Instruction::create_jump(6)); // Offset 6 bytes
    // Cette instruction ne doit pas être exécutée
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 15, 0xFF));   //255
    // Cette instruction doit être exécutée après le saut (succès)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 15, 4)); // Marquer succès test 4


    // Fin du programme
    program.add_instruction(Instruction::create_no_args(Opcode::Halt));

    // Configuration des segments (Code et Data)
    let total_code_size: u32 = program.code.iter().map(|instr| instr.total_size() as u32).sum();
    program.segments.push(SegmentMetadata::new(SegmentType::Code, 0, total_code_size, 0));
    let data_size = 256; // Taille du segment de données
    program.segments.push(SegmentMetadata::new(SegmentType::Data, 0, data_size, total_code_size)); // Segment data après le code
    program.data = vec![0; data_size as usize];



    println!("\n--- Carte des instructions du programme de test des branchements ---");
    let mut addr = 0;
    for (idx, instr) in program.code.iter().enumerate() {
        let size = instr.total_size();
        println!(
            "Instruction {}: Adresse 0x{:04X}-0x{:04X} (taille {}): {:?}",
            idx, addr, addr + size - 1, size, instr.opcode
        );

        // Vérification spéciale pour les branchements
        if instr.opcode.is_branch() {
            if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
                let target = ( size  as u32 + addr as u32 ) as i64 + offset as i64;
                println!("  -> Branchement relatif: offset={}, target=0x{:04X}", offset, target);
            }
        }


        addr += size;
    }
    println!("--- Fin de la carte des instructions ---\n");

    program
}



pub fn punk_program_fixed() -> BytecodeFile {
    println!("\n===== DÉBUT DU TEST PUNK BRANCH FIXED =====\n");
    let mut program = BytecodeFile::new();
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    program.add_metadata("name", "Punk Branch Test Program - Fixed");

    // Initialiser des registres pour les tests
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 10)); // R0 = 10
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 20)); // R1 = 20
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 2, 10)); // R2 = 10

    // --- Test JmpIfEqual (ZF = 1) ---
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 2)); // R0 == R2 => ZF = 1

    // Calculer l'offset pour sauter une instruction MOV (6 bytes)
    let skip_one_mov = 6;
    program.add_instruction(Instruction::create_jump_if_equal(skip_one_mov));

    // Cette instruction doit être sautée
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 3, 0xFF)); // Ne doit pas être exécutée

    // Cette instruction doit être exécutée
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 3, 0x11)); // R3 = 0x11

    // --- Test JmpIfNotEqual (ZF = 0) ---
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 1)); // R0 != R1 => ZF = 0
    program.add_instruction(Instruction::create_jump_if_not_equal(skip_one_mov));

    // Cette instruction doit être sautée
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 4, 0xFF)); // Ne doit pas être exécutée

    // Cette instruction doit être exécutée
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 4, 0x22)); // R4 = 0x22

    // --- Test JmpIfGreater ---
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 1, 0)); // R1 > R0 => SF = 0, ZF = 0
    program.add_instruction(Instruction::create_jump_if_greater(skip_one_mov));

    // Cette instruction doit être sautée
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 5, 0xFF)); // Ne doit pas être exécutée

    // Cette instruction doit être exécutée
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 5, 0x33)); // R5 = 0x33

    // Fin du programme
    program.add_instruction(Instruction::create_no_args(Opcode::Halt));

    // Configuration des segments
    let total_code_size: u32 = program.code.iter().map(|instr| instr.total_size() as u32).sum();
    program.segments.push(SegmentMetadata::new(SegmentType::Code, 0, total_code_size, 0));
    let data_size = 256;
    program.segments.push(SegmentMetadata::new(SegmentType::Data, 0, data_size, total_code_size));
    program.data = vec![0; data_size as usize];

    // Affichage de la carte avec vérification des offsets
    println!("\n--- Carte des instructions du programme de test des branchements ---");
    let mut addr = 0;
    for (idx, instr) in program.code.iter().enumerate() {
        let size = instr.total_size();
        println!(
            "Instruction {}: Adresse 0x{:04X}-0x{:04X} (taille {}): {:?}",
            idx, addr, addr + size - 1, size, instr.opcode
        );

        // Vérification spéciale pour les branchements
        if instr.opcode.is_branch() {
            if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
                let target = ( size  as u32 + addr as u32 ) as i64 + offset as i64;
                println!("  -> Branchement relatif: offset={}, target=0x{:04X}", offset, target);
            }
        }
        addr += size;
    }
    println!("--- Fin de la carte des instructions ---\n");


    program
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
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 10));  // R0 = 10
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 20));  // R1 = 20
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 2, 10));  // R2 = 10 (égal à R0)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 3, 5));   // R3 = 5 (plus petit que R0)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 4, 0));   // R4 = 0 (pour tests de zéro)

    // Registres pour stocker les résultats des tests
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 8, 0));   // R8 = compteur de tests réussis
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 0));   // R9 = compteur de tests échoués

        //============================================================================
        // SECTION 2: TEST JMP (SAUT INCONDITIONNEL)
        // ============================================================================
        println!("=== SECTION 2: TEST JMP INCONDITIONNEL ===");

        // Test du saut inconditionnel - doit sauter par-dessus l'instruction suivante
        program.add_instruction(Instruction::create_jump(6)); // Sauter 1 instruction MOV (6 bytes)

        // Cette instruction ne doit PAS être exécutée
        program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 0xFF)); // ÉCHEC si exécuté

        // Cette instruction doit être exécutée
        program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 10, 0x01)); // R10 = 1 (succès JMP)

        // ============================================================================
        // SECTION 3: TEST JmpIfEqual (ZF = 1)
        // ============================================================================
        println!("=== SECTION 3: TEST JmpIfEqual ===");

        // Test 1: R0 == R2 (10 == 10) → ZF = 1 → branchement PRIS
        program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 2));
        program.add_instruction(Instruction::create_jump_if_equal(6));
        program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 0xFF)); // ÉCHEC si exécuté
        program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 11, 0x02)); // R11 = 2 (succès)
        //
        // // Test 2: R0 == R1 (10 == 20) → ZF = 0 → branchement NON PRIS
        program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 1));
        // program.add_instruction(Instruction::create_jump_if_equal(6));
        // program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 12, 0x03)); // R12 = 3 (succès, doit être exécuté)
        // program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 13, 0x04)); // R13 = 4

        // ============================================================================
        // SECTION 4: TEST JmpIfNotEqual (ZF = 0)
        // ============================================================================
        // println!("=== SECTION 4: TEST JmpIfNotEqual ===");
        //
        // // Test 1: R0 != R1 (10 != 20) → ZF = 0 → branchement PRIS
        // program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 1));
        // program.add_instruction(Instruction::create_jump_if_not_equal(6));
        // program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 0xFF)); // ÉCHEC si exécuté
        // program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 14, 0x05)); // R14 = 5 (succès)
        //
        // // Test 2: R0 != R2 (10 != 10) → ZF = 1 → branchement NON PRIS
        // program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 2));
        // program.add_instruction(Instruction::create_jump_if_not_equal(6));
        // program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 15, 0x06)); // R15 = 6 (succès, doit être exécuté)
        // program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 5, 0x07)); // R5 = 7

    //     // ============================================================================
    //     // SECTION 5: TEST JmpIfGreater (ZF = 0 ET SF = 0)
    //     // ============================================================================
    //     println!("=== SECTION 5: TEST JmpIfGreater ===");
    //
    //     // Test 1: R1 > R0 (20 > 10) → ZF = 0, SF = 0 → branchement PRIS
    //     program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 1, 0));
    //     program.add_instruction(Instruction::create_jump_if_greater(6));
    //     program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 0xFF)); // ÉCHEC si exécuté
    //     program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 6, 0x08)); // R6 = 8 (succès)
    //
    //     // Test 2: R3 > R0 (5 > 10) → SF = 1 → branchement NON PRIS
    //     program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 3, 0));
    //     program.add_instruction(Instruction::create_jump_if_greater(6));
    //     program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 7, 0x09)); // R7 = 9 (succès, doit être exécuté)
    //     program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 8, 0x0A)); // R8 = 10

    //     // ============================================================================
    //     // SECTION 6: TEST JmpIfLess (SF = 1)
    //     // ============================================================================
    //     println!("=== SECTION 6: TEST JmpIfLess ===");
    //
    //     // Test 1: R3 < R0 (5 < 10) → SF = 1 → branchement PRIS
    //     program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 3, 0));
    //     program.add_instruction(Instruction::create_jump_if_less(6));
    //     program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 0xFF)); // ÉCHEC si exécuté
    //     program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 0x0B)); // R9 = 11 (succès)
    //
    //     // Test 2: R1 < R0 (20 < 10) → SF = 0 → branchement NON PRIS
    //     program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 1, 0));
    //     program.add_instruction(Instruction::create_jump_if_less(6));
    //     program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 10, 0x0C)); // R10 = 12 (succès, doit être exécuté)
    //     program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 11, 0x0D)); // R11 = 13
    //
    //     // ============================================================================
    //     // SECTION 7: TEST JmpIfGreaterEqual (SF = 0)
    //     // ============================================================================
    //     println!("=== SECTION 7: TEST JmpIfGreaterEqual ===");
    //
    //     // Test 1: R1 >= R0 (20 >= 10) → SF = 0 → branchement PRIS
    //     program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 1, 0));
    //     program.add_instruction(Instruction::create_jump_if_greater_equal(6));
    //     program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 0xFF)); // ÉCHEC si exécuté
    //     program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 12, 0x0E)); // R12 = 14 (succès)
    //
    //     // Test 2: R0 >= R2 (10 >= 10) → ZF = 1 → branchement PRIS aussi
    //     program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 2));
    //     program.add_instruction(Instruction::create_jump_if_greater_equal(6));
    //     program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 0xFF)); // ÉCHEC si exécuté
    //     program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 13, 0x0F)); // R13 = 15 (succès)
    //
    //     // Test 3: R3 >= R0 (5 >= 10) → SF = 1 → branchement NON PRIS
    //     program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 3, 0));
    //     program.add_instruction(Instruction::create_jump_if_greater_equal(6));
    //     program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 14, 0x10)); // R14 = 16 (succès, doit être exécuté)
    //     program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 15, 0x11)); // R15 = 17
    //
    //     // ============================================================================
    //     // SECTION 8: TEST JmpIfLessEqual (SF = 1 OU ZF = 1)
    //     // ============================================================================
    //     println!("=== SECTION 8: TEST JmpIfLessEqual ===");
    //
    //     // Test 1: R3 <= R0 (5 <= 10) → SF = 1 → branchement PRIS
    //     program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 3, 0));
    //     program.add_instruction(Instruction::create_jump_if_less_equal(6));
    //     program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 0xFF)); // ÉCHEC si exécuté
    //     program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 5, 0x12)); // R5 = 18 (succès)
    //
    //     // Test 2: R0 <= R2 (10 <= 10) → ZF = 1 → branchement PRIS aussi
    //     program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 2));
    //     program.add_instruction(Instruction::create_jump_if_less_equal(6));
    //     program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 0xFF)); // ÉCHEC si exécuté
    //     program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 6, 0x13)); // R6 = 19 (succès)
    //
    //     // Test 3: R1 <= R0 (20 <= 10) → SF = 0, ZF = 0 → branchement NON PRIS
    //     program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 1, 0));
    //     program.add_instruction(Instruction::create_jump_if_less_equal(6));
    //     program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 7, 0x14)); // R7 = 20 (succès, doit être exécuté)
    //     program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 8, 0x15)); // R8 = 21
    //
    //     // ============================================================================
    //     // SECTION 9: TEST JmpIfZero (ZF = 1)
    //     // ============================================================================
    //     println!("=== SECTION 9: TEST JmpIfZero ===");
    //
    //     // Test 1: R0 == R2 (10 == 10) → ZF = 1 → branchement PRIS
    //     program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 2));
    //     program.add_instruction(Instruction::create_jump_if_zero(6));
    //     program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 0xFF)); // ÉCHEC si exécuté
    //     program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 0x16)); // R9 = 22 (succès)
    //
    //     // Test 2: R0 != R1 (10 != 20) → ZF = 0 → branchement NON PRIS
    //     program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 1));
    //     program.add_instruction(Instruction::create_jump_if_zero(6));
    //     program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 10, 0x17)); // R10 = 23 (succès, doit être exécuté)
    //     program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 11, 0x18)); // R11 = 24
    //
    //     // ============================================================================
    //     // SECTION 10: TEST JmpIfNotZero (ZF = 0)
    //     // ============================================================================
    //     println!("=== SECTION 10: TEST JmpIfNotZero ===");
    //
    //     // Test 1: R0 != R1 (10 != 20) → ZF = 0 → branchement PRIS
    //     program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 1));
    //     program.add_instruction(Instruction::create_jump_if_not_zero(6));
    //     program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 0xFF)); // ÉCHEC si exécuté
    //     program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 12, 0x19)); // R12 = 25 (succès)
    //
    //     // Test 2: R0 == R2 (10 == 10) → ZF = 1 → branchement NON PRIS
    //     program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 2));
    //     program.add_instruction(Instruction::create_jump_if_not_zero(6));
    //     program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 13, 0x1A)); // R13 = 26 (succès, doit être exécuté)
    //     program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 14, 0x1B)); // R14 = 27
    //
    //     // ============================================================================
    //     // SECTION 11: TEST DE BOUCLE (Pattern pour le prédicteur)
    //     // ============================================================================
    //     println!("=== SECTION 11: TEST DE BOUCLE ===");
    //
    //     // Initialisation du compteur de boucle
    //     program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 15, 3)); // R15 = 3 (compteur)
    //
    //     // Début de la boucle - cette étiquette sera utilisée pour le branchement arrière
    //     let loop_start_instruction_index = program.code.len();
    //
    //     // Corps de la boucle
    //     program.add_instruction(Instruction::create_reg_reg(Opcode::Sub, 15, 4)); // R15 = R15 - 1 (R4 = 0, donc R15 - 0, mais on veut R15-1)
    //
    //     // Pour décrémenter correctement, on doit d'abord mettre 1 dans un registre
    //     program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 4, 1)); // R4 = 1
    //     program.add_instruction(Instruction::create_reg_reg(Opcode::Sub, 15, 4)); // R15 = R15 - 1
    //
    //     // Comparer avec 0
    //     program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 4, 0)); // R4 = 0 pour comparaison
    //     program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 15, 4)); // Compare R15 avec 0
    //
    //     // Calculer l'offset pour retourner au début de la boucle
    //     let current_instruction_index = program.code.len() + 1; // +1 car on ajoute l'instruction de branchement
    //     let loop_body_size = current_instruction_index - loop_start_instruction_index;
    //     let backward_offset = -(loop_body_size as i32 * 6 + 8); // chaque instruction fait ~6 bytes, +8 pour l'instruction de branchement
    //
    //     // Branchement conditionnel vers le début de la boucle si R15 != 0
    //     program.add_instruction(Instruction::create_jump_if_not_zero(backward_offset));
    //
        // ============================================================================
    //     // SECTION 12: TEST CALL/RET (Si implémenté)
    //     // ============================================================================
        println!("=== SECTION 12: TEST CALL/RET ===");

        // Sauter par-dessus la fonction pour aller au call
        program.add_instruction(Instruction::create_jump(12)); // Sauter la fonction

        // FONCTION: simple_function
        // Fonction qui met 0xFF dans R5 et retourne
        program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 5, 0xFF)); // R5 = 255
        program.add_instruction(Instruction::create_no_args(Opcode::Ret)); // Retour

        // Appel de la fonction (si CALL est implémenté)
        let function_offset = -12; // Retourner à la fonction
        // program.add_instruction(Instruction::create_call(function_offset));

        // Vérification que la fonction a été appelée
        // program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 6, 0xAA)); // R6 = 170 (succès si R5 = 255)

        // ============================================================================
    //     // SECTION 13: FINALISATION ET VÉRIFICATION
    //     // ============================================================================
    // println!("=== SECTION 13: FINALISATION ===");

    // Marquer la fin des tests
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 0xFE)); // R0 = 254 (marqueur de fin)

    // Fin du programme
    program.add_instruction(Instruction::create_no_args(Opcode::Halt));

    // ============================================================================
    // CONFIGURATION DES SEGMENTS
    // ============================================================================
    let total_code_size: u32 = program.code.iter().map(|instr| instr.total_size() as u32).sum();
    program.segments.push(SegmentMetadata::new(SegmentType::Code, 0, total_code_size, 0));

    let data_size = 512; // Taille augmentée pour plus de données
    program.segments.push(SegmentMetadata::new(SegmentType::Data, 0, data_size, total_code_size));
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
            _ => "FINAL"
        };

        *section_counters.entry(section).or_insert(0) += 1;

        println!("Instruction {:2}: [{}] Adresse 0x{:04X}-0x{:04X} (taille {:2}): {:?}",
                 idx, section, addr, addr   + size as u32 - 1, size, instr.opcode);





        // Affichage spécial pour les branchements
        if instr.opcode.is_branch() {
            if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
                let target = (addr + size as u32) as i64 + offset as i64;
                println!("      -> Branchement relatif: offset={:+}, target=0x{:04X}", offset, target);
            }
        }

        addr += size as u32;
    }

    println!("\n=== RÉSUMÉ DES SECTIONS ===");
    for (section, count) in section_counters {
        println!("{}: {} instructions", section, count);
    }
    println!("TOTAL: {} instructions, {} bytes", program.code.len(), addr);

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


pub fn punk_program_4() -> BytecodeFile {
    let mut program = BytecodeFile::new();
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    program.add_metadata("name", "PunkVM Fixed Comprehensive Branch Test");
    program.add_metadata("description", "Test corrigé de tous les types de branchements");

    // ============================================================================
    // SECTION 1: INITIALISATION DES REGISTRES
    // ============================================================================
    println!("=== SECTION 1: INITIALISATION ===");

    // Registres pour les comparaisons
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 10));  // R0 = 10
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 20));  // R1 = 20
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 2, 10));  // R2 = 10 (égal à R0)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 3, 5));   // R3 = 5 (plus petit que R0)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 4, 0));   // R4 = 0 (pour tests de zéro)

    // ============================================================================
    // SECTION 2: TEST JMP (SAUT INCONDITIONNEL)
    // ============================================================================
    println!("=== SECTION 2: TEST JMP INCONDITIONNEL ===");

    // Test du saut inconditionnel - doit sauter par-dessus l'instruction suivante
    program.add_instruction(Instruction::create_jump(6)); // Sauter 1 instruction MOV (6 bytes)

    // Cette instruction ne doit PAS être exécutée
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 0xFF)); // ÉCHEC si exécuté

    // Cette instruction doit être exécutée
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 10, 0x01)); // R10 = 1 (succès JMP)

    // ============================================================================
    // SECTION 3: TEST JmpIfEqual (ZF = 1)
    // ============================================================================
    println!("=== SECTION 3: TEST JmpIfEqual ===");

    // Test 1: R0 == R2 (10 == 10) → ZF = 1 → branchement PRIS
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 2));
    program.add_instruction(Instruction::create_jump_if_equal(6));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 0xFF)); // ÉCHEC si exécuté
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 11, 0x02)); // R11 = 2 (succès)

    // Test 2: R0 != R1 (10 != 20) → ZF = 0 → branchement NON PRIS
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 1));
    program.add_instruction(Instruction::create_jump_if_equal(6));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 12, 0x03)); // R12 = 3 (succès, doit être exécuté)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 13, 0x04)); // R13 = 4

    // // ============================================================================
    // // SECTION 4: TEST JmpIfNotEqual (ZF = 0)
    // // ============================================================================
    // println!("=== SECTION 4: TEST JmpIfNotEqual ===");
    //
    // // Test 1: R0 != R1 (10 != 20) → ZF = 0 → branchement PRIS
    // program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 1));
    // program.add_instruction(Instruction::create_jump_if_not_equal(6));
    // program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 0xFF)); // ÉCHEC si exécuté
    // program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 14, 0x05)); // R14 = 5 (succès)
    //
    // // Test 2: R0 == R2 (10 == 10) → ZF = 1 → branchement NON PRIS
    // program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 2));
    // program.add_instruction(Instruction::create_jump_if_not_equal(6));
    // program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 15, 0x06)); // R15 = 6 (succès, doit être exécuté)
    // program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 5, 0x07)); // R5 = 7
    //
    // // ============================================================================
    // // SECTION 5: TEST JmpIfGreater (ZF = 0 ET SF = 0)
    // // ============================================================================
    // println!("=== SECTION 5: TEST JmpIfGreater ===");
    //
    // // Test 1: R1 > R0 (20 > 10) → ZF = 0, SF = 0 → branchement PRIS
    // program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 1, 0));
    // program.add_instruction(Instruction::create_jump_if_greater(6));
    // program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 0xFF)); // ÉCHEC si exécuté
    // program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 6, 0x08)); // R6 = 8 (succès)
    //
    // // Test 2: R3 > R0 (5 > 10) → SF = 1 → branchement NON PRIS
    // program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 3, 0));
    // program.add_instruction(Instruction::create_jump_if_greater(6));
    // program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 7, 0x09)); // R7 = 9 (succès, doit être exécuté)
    // program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 8, 0x0A)); // R8 = 10
    //
    // // ============================================================================
    // // SECTION 6: TEST JmpIfLess (SF = 1)
    // // ============================================================================
    // println!("=== SECTION 6: TEST JmpIfLess ===");
    //
    // // Test 1: R3 < R0 (5 < 10) → SF = 1 → branchement PRIS
    // program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 3, 0));
    // program.add_instruction(Instruction::create_jump_if_less(6));
    // program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 0xFF)); // ÉCHEC si exécuté
    // program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 0x0B)); // R9 = 11 (succès)
    //
    // // Test 2: R1 < R0 (20 < 10) → SF = 0 → branchement NON PRIS
    // program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 1, 0));
    // program.add_instruction(Instruction::create_jump_if_less(6));
    // program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 10, 0x0C)); // R10 = 12 (succès, doit être exécuté)
    // program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 11, 0x0D)); // R11 = 13
    //
    // // ============================================================================
    // // SECTION 7: FINALISATION
    // // ============================================================================
    // println!("=== SECTION 7: FINALISATION ===");
    //
    // // Marquer la fin des tests
    // program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 0xFE)); // R0 = 254 (marqueur de fin)

    // Fin du programme
    program.add_instruction(Instruction::create_no_args(Opcode::Halt));

    // ============================================================================
    // CONFIGURATION DES SEGMENTS
    // ============================================================================
    let total_code_size: u32 = program.code.iter().map(|instr| instr.total_size() as u32).sum();
    program.segments.push(SegmentMetadata::new(SegmentType::Code, 0, total_code_size, 0));

    let data_size = 256;
    program.segments.push(SegmentMetadata::new(SegmentType::Data, 0, data_size, total_code_size));
    program.data = vec![0; data_size as usize];

    // ============================================================================
    // AFFICHAGE DE LA CARTE DES INSTRUCTIONS AVEC VALIDATION
    // ============================================================================
    println!("\n=== CARTE DES INSTRUCTIONS AVEC VALIDATION ===");
    let mut addr = 0u32;

    for (idx, instr) in program.code.iter().enumerate() {
        let size = instr.total_size();

        // Déterminer la section
        let section = match idx {
            0..=4 => "INIT",
            5..=7 => "JMP",
            8..=13 => "JmpIfEqual",
            14..=19 => "JmpIfNotEqual",
            20..=25 => "JmpIfGreater",
            26..=31 => "JmpIfLess",
            32..=33 => "FINAL",
            _ => "OTHER"
        };

        println!("Instruction {:2}: [{}] Adresse 0x{:04X}-0x{:04X} (taille {:2}): {:?}",
                 idx, section, addr, addr + size as u32 - 1, size, instr.opcode);

        // Validation spéciale pour les branchements
        if instr.opcode.is_branch() {
            if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
                let target = (addr + size as u32) as i64 + offset as i64;
                println!("      -> Branchement relatif: offset={:+}, target=0x{:04X}", offset, target);

                // VALIDATION: Vérifier que la cible existe
                if target < 0 || target as u32 >= total_code_size {
                    println!("      ⚠️  ERREUR: Cible hors limites! (programme: 0x0000-0x{:04X})", total_code_size - 1);
                } else {
                    println!("      ✅ Cible valide");
                }
            }
        }

        addr += size as u32;
    }

    println!("\n=== RÉSUMÉ ===");
    println!("TOTAL: {} instructions", program.code.len());
    println!("Taille du code: {} bytes (0x0000-0x{:04X})", total_code_size, total_code_size - 1);

    println!("\n=== TESTS ATTENDUS (VERSION SIMPLIFIÉE) ===");
    println!("Après exécution, les registres suivants devraient contenir:");
    println!("R0  = 254 (0xFE) - Marqueur de fin");
    println!("R10 = 1   (0x01) - Test JMP réussi");
    println!("R11 = 2   (0x02) - Test JmpIfEqual réussi");
    println!("R12 = 3   (0x03) - Test JmpIfEqual (non pris) réussi");
    println!("R14 = 5   (0x05) - Test JmpIfNotEqual réussi");
    println!("R15 = 6   (0x06) - Test JmpIfNotEqual (non pris) réussi");
    println!("R6  = 8   (0x08) - Test JmpIfGreater réussi");
    println!("R7  = 9   (0x09) - Test JmpIfGreater (non pris) réussi");
    println!("R9  = 11  (0x0B) - Test JmpIfLess réussi");
    println!("R10 = 12  (0x0C) - Test JmpIfLess (non pris) réussi (écrase la valeur précédente)");
    println!("Aucun registre ne devrait contenir 0xFF (échec)");

    program
}

// Version encore plus simple pour déboguer
pub fn punk_program_debug() -> BytecodeFile {
    let mut program = BytecodeFile::new();
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    program.add_metadata("name", "PunkVM Debug Simple Test");

    println!("=== PROGRAMME DE DEBUG SIMPLE ===");

    // Test minimal: Juste quelques MOV et un JMP
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 10));  // R0 = 10
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 20));  // R1 = 20

    // Test JMP simple
    program.add_instruction(Instruction::create_jump(6)); // Sauter l'instruction suivante
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 0xFF)); // Ne doit pas être exécuté
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 2, 0x42)); // R2 = 66 (succès)

    // Test branchement conditionnel simple
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 1)); // 10 vs 20
    program.add_instruction(Instruction::create_jump_if_equal(6)); // Ne doit pas être pris
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 3, 0x33)); // R3 = 51 (doit être exécuté)

    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 0xFE)); // Marqueur de fin
    program.add_instruction(Instruction::create_no_args(Opcode::Halt));

    // Configuration
    let total_code_size: u32 = program.code.iter().map(|instr| instr.total_size() as u32).sum();
    program.segments.push(SegmentMetadata::new(SegmentType::Code, 0, total_code_size, 0));
    program.segments.push(SegmentMetadata::new(SegmentType::Data, 0, 256, total_code_size));
    program.data = vec![0; 256];

    println!("Programme debug créé: {} instructions, {} bytes", program.code.len(), total_code_size);

    program
}
*/

/////////////////////////////////////////////////////////////////////////////////////////////////////

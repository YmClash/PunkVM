//src/main.rs


use std::collections::HashMap;
use std::time::Instant;
use PunkVM::bytecode::files::{BytecodeFile, BytecodeVersion, SegmentMetadata, SegmentType};
use PunkVM::bytecode::instructions::{ArgValue, Instruction};
use PunkVM::bytecode::opcodes::Opcode;
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
        // stack_base:0xFF000000,         // Base de la pile
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

    // Créer le programme de test
    // Décommenter la ligne souhaitée pour tester différents programmes:
    
    // Test complet et corrigé de tous les branchements

    
    // Test minimal pour le bug de branchement non pris
    // let program = test_branch_not_taken_fix();
    
    // Programme original (contient des bugs d'adressage)
    let program = punk_program_3();
    
    // Autres programmes de test
    // let program = punk_program_5();
    // let program = create_reg_reg_reg_test_program();

    // let program = comprehensive_branch_test();





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

/// Test minimal pour valider la correction du bug de branchement non pris
/// Ce test reproduit exactement le bug décrit : PC=0x5E, JmpIfEqual non pris devrait aller à 0x66 mais allait à 0x6E
pub fn test_branch_not_taken_fix() -> BytecodeFile {
    let mut program = BytecodeFile::new();
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    program.add_metadata("name", "Test Bug Fix Branch Non Pris");
    program.add_metadata("description", "Test minimal pour vérifier que le PC est correctement calculé pour les branchements non pris");
    program.add_metadata("author", "PunkVM Team");

    // Instructions minimales pour reproduire le bug:
    // MOV R0, 10
    // MOV R1, 20  
    // CMP R0, R1     // 10 != 20, ZF=false
    // JmpIfEqual +6  // NON PRIS - devrait continuer à l'instruction suivante
    // MOV R2, 42     // DOIT être exécuté (marqueur de succès)
    // HALT
    
    println!("=== CRÉATION DU TEST MINIMAL BRANCH NOT TAKEN ===");
    
    // 1. MOV R0, 10
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 10));
    
    // 2. MOV R1, 20
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 20));
    
    // 3. CMP R0, R1 (compare 10 et 20, met ZF=0 car ils ne sont pas égaux)
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 1));
    
    // 4. JmpIfEqual (saut conditionnel - ne sera PAS pris car ZF=0)
    let current_address = Instruction::calculate_current_address(&program.code);
    let jmp_instruction_size = 8; // Taille de l'instruction JmpIfEqual
    let jmpifequal_target = current_address + jmp_instruction_size + 6; // Target après MOV R2, 42
    program.add_instruction(Instruction::create_jump_if_equal(current_address, jmpifequal_target));
    
    // 5. MOV R2, 42 - Cette instruction DOIT être exécutée (marqueur de succès)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 2, 42));
    
    // 6. HALT
    program.add_instruction(Instruction::create_no_args(Opcode::Halt));
    
    // Configuration des segments
    let total_size: u32 = program
        .code
        .iter()
        .map(|instr| instr.total_size() as u32)
        .sum();
    program.segments = vec![SegmentMetadata::new(SegmentType::Code, 0, total_size, 0)];
    
    // Segment de données vide
    let data_size = 256;
    let data_segment = SegmentMetadata::new(SegmentType::Data, 0, data_size, 0x1000);
    program.segments.push(data_segment);
    program.data = vec![0; data_size as usize];
    
    // Affichage de la carte des instructions
    println!("\n--- Carte des instructions du test minimal ---");
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
    println!("--- Fin de la carte ---\n");
    
    println!("=== RÉSULTAT ATTENDU ===");
    println!("Si le bug est corrigé:");
    println!("- Le programme doit s'exécuter complètement jusqu'à HALT");
    println!("- R2 doit contenir 42 (prouvant que l'instruction après JmpIfEqual a été exécutée)");
    println!("- Pas d'erreur 'Instruction non trouvée'");
    println!();
    
    program
}

/// Programme de test complet et corrigé pour tous les types de branchements
/// Version améliorée de punk_program_3() avec corrections des bugs d'adressage
pub fn comprehensive_branch_test() -> BytecodeFile {
    let mut program = BytecodeFile::new();
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    program.add_metadata("name", "PunkVM Complete Branch Validation Test");
    program.add_metadata("description", "Test complet et corrigé de tous les types de branchements avec adressage précis");
    program.add_metadata("author", "PunkVM Team - Bug Fix Version");
    program.add_metadata("test_categories", "JMP, JmpIfEqual, JmpIfNotEqual, JmpIfGreater, JmpIfLess, JmpIfGreaterEqual, JmpIfLessEqual, JmpIfZero, JmpIfNotZero");

    println!("=== CRÉATION DU TEST COMPLET DES BRANCHEMENTS ===");

    // ============================================================================
    // SECTION 1: INITIALISATION DES REGISTRES
    // ============================================================================
    println!("Section 1: Initialisation des registres");
    
    // Registres pour les comparaisons
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 10)); // R0 = 10
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 20)); // R1 = 20  
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 2, 10)); // R2 = 10 (égal à R0)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 3, 5));  // R3 = 5 (plus petit que R0)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 4, 0));  // R4 = 0 (pour tests de zéro)
    
    // Registres pour stocker les résultats des tests (marqueurs de succès)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 10, 0)); // R10 = compteur de succès
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 15, 0)); // R15 = marqueur d'échec

    // ============================================================================
    // SECTION 2: TEST JMP (SAUT INCONDITIONNEL)
    // ============================================================================
    println!("Section 2: Test JMP inconditionnel");
    
    let current_addr = Instruction::calculate_current_address(&program.code);
    let skip_instruction_size = 6; // Taille de l'instruction MOV à ignorer
    let jmp_target = current_addr + 8 + skip_instruction_size; // 8 = taille JMP, +6 = taille MOV à sauter
    program.add_instruction(Instruction::create_jump(current_addr, jmp_target));
    
    // Cette instruction ne doit PAS être exécutée
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 15, 0xFF)); // ÉCHEC si exécuté
    
    // Cette instruction doit être exécutée (marqueur de succès JMP)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 10, 1)); // R10 = 1 (succès JMP)

    // ============================================================================
    // SECTION 3: TEST JmpIfEqual (ZF = 1)
    // ============================================================================
    println!("Section 3: Test JmpIfEqual");
    
    // Test 1: R0 == R2 (10 == 10) → ZF = 1 → branchement PRIS
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 2)); // Compare R0 et R2
    let current_addr = Instruction::calculate_current_address(&program.code);
    let jmpifequal_target_1 = current_addr + 8 + 6; // Sauter par-dessus l'instruction d'échec
    program.add_instruction(Instruction::create_jump_if_equal(current_addr, jmpifequal_target_1));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 15, 0xFF)); // ÉCHEC si exécuté
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 10, 2)); // R10 = 2 (succès JmpIfEqual pris)
    
    // Test 2: R0 == R1 (10 == 20) → ZF = 0 → branchement NON PRIS
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 1)); // Compare R0 et R1
    let current_addr = Instruction::calculate_current_address(&program.code);
    let jmpifequal_target_2 = current_addr + 8 + 6; // Target après l'instruction suivante
    program.add_instruction(Instruction::create_jump_if_equal(current_addr, jmpifequal_target_2));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 10, 3)); // R10 = 3 (succès JmpIfEqual non pris)

    // ============================================================================
    // SECTION 4: TEST JmpIfNotEqual (ZF = 0)
    // ============================================================================
    println!("Section 4: Test JmpIfNotEqual");
    
    // Test 1: R0 != R1 (10 != 20) → ZF = 0 → branchement PRIS
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 1));
    let current_addr = Instruction::calculate_current_address(&program.code);
    let jmpifnotequal_target_1 = current_addr + 8 + 6;
    program.add_instruction(Instruction::create_jump_if_not_equal(current_addr, jmpifnotequal_target_1));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 15, 0xFF)); // ÉCHEC si exécuté
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 10, 4)); // R10 = 4 (succès JmpIfNotEqual pris)
    
    // Test 2: R0 != R2 (10 != 10) → ZF = 1 → branchement NON PRIS
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 2));
    let current_addr = Instruction::calculate_current_address(&program.code);
    let jmpifnotequal_target_2 = current_addr + 8 + 6;
    program.add_instruction(Instruction::create_jump_if_not_equal(current_addr, jmpifnotequal_target_2));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 10, 5)); // R10 = 5 (succès JmpIfNotEqual non pris)

    // ============================================================================
    // SECTION 5: TEST JmpIfGreater (ZF = 0 ET SF = 0)
    // ============================================================================
    println!("Section 5: Test JmpIfGreater");
    
    // Test 1: R1 > R0 (20 > 10) → branchement PRIS
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 1, 0));
    let current_addr = Instruction::calculate_current_address(&program.code);
    let jmpifgreater_target_1 = current_addr + 8 + 6;
    program.add_instruction(Instruction::create_jump_if_greater(current_addr, jmpifgreater_target_1));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 15, 0xFF)); // ÉCHEC si exécuté
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 10, 6)); // R10 = 6 (succès JmpIfGreater pris)
    
    // Test 2: R3 > R0 (5 > 10) → branchement NON PRIS
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 3, 0));
    let current_addr = Instruction::calculate_current_address(&program.code);
    let jmpifgreater_target_2 = current_addr + 8 + 6;
    program.add_instruction(Instruction::create_jump_if_greater(current_addr, jmpifgreater_target_2));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 10, 7)); // R10 = 7 (succès JmpIfGreater non pris)

    // ============================================================================
    // SECTION 6: TEST JmpIfLess (SF = 1)
    // ============================================================================
    println!("Section 6: Test JmpIfLess");
    
    // Test 1: R3 < R0 (5 < 10) → branchement PRIS
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 3, 0));
    let current_addr = Instruction::calculate_current_address(&program.code);
    let jmpifless_target_1 = current_addr + 8 + 6;
    program.add_instruction(Instruction::create_jump_if_less(current_addr, jmpifless_target_1));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 15, 0xFF)); // ÉCHEC si exécuté
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 10, 8)); // R10 = 8 (succès JmpIfLess pris)
    
    // Test 2: R1 < R0 (20 < 10) → branchement NON PRIS
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 1, 0));
    let current_addr = Instruction::calculate_current_address(&program.code);
    let jmpifless_target_2 = current_addr + 8 + 6;
    program.add_instruction(Instruction::create_jump_if_less(current_addr, jmpifless_target_2));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 10, 9)); // R10 = 9 (succès JmpIfLess non pris)

    // ============================================================================
    // SECTION 7: TEST JmpIfGreaterEqual (SF = 0 OU ZF = 1)
    // ============================================================================
    println!("Section 7: Test JmpIfGreaterEqual");
    
    // Test 1: R1 >= R0 (20 >= 10) → branchement PRIS
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 1, 0));
    let current_addr = Instruction::calculate_current_address(&program.code);
    let jmpifgreaterequal_target_1 = current_addr + 8 + 6;
    program.add_instruction(Instruction::create_jump_if_greater_equal(current_addr, jmpifgreaterequal_target_1));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 15, 0xFF)); // ÉCHEC si exécuté
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 10, 10)); // R10 = 10 (succès)
    
    // Test 2: R0 >= R2 (10 >= 10) → ZF = 1 → branchement PRIS
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 2));
    let current_addr = Instruction::calculate_current_address(&program.code);
    let jmpifgreaterequal_target_2 = current_addr + 8 + 6;
    program.add_instruction(Instruction::create_jump_if_greater_equal(current_addr, jmpifgreaterequal_target_2));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 15, 0xFF)); // ÉCHEC si exécuté
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 10, 11)); // R10 = 11 (succès)
    
    // Test 3: R3 >= R0 (5 >= 10) → branchement NON PRIS
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 3, 0));
    let current_addr = Instruction::calculate_current_address(&program.code);
    let jmpifgreaterequal_target_3 = current_addr + 8 + 6;
    program.add_instruction(Instruction::create_jump_if_greater_equal(current_addr, jmpifgreaterequal_target_3));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 10, 12)); // R10 = 12 (succès non pris)

    // ============================================================================
    // SECTION 8: TEST JmpIfLessEqual (SF = 1 OU ZF = 1)
    // ============================================================================
    println!("Section 8: Test JmpIfLessEqual");
    
    // Test 1: R3 <= R0 (5 <= 10) → branchement PRIS
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 3, 0));
    let current_addr = Instruction::calculate_current_address(&program.code);
    let jmpiflessequal_target_1 = current_addr + 8 + 6;
    program.add_instruction(Instruction::create_jump_if_less_equal(current_addr, jmpiflessequal_target_1));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 15, 0xFF)); // ÉCHEC si exécuté
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 10, 13)); // R10 = 13 (succès)
    
    // Test 2: R0 <= R2 (10 <= 10) → ZF = 1 → branchement PRIS
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 2));
    let current_addr = Instruction::calculate_current_address(&program.code);
    let jmpiflessequal_target_2 = current_addr + 8 + 6;
    program.add_instruction(Instruction::create_jump_if_less_equal(current_addr, jmpiflessequal_target_2));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 15, 0xFF)); // ÉCHEC si exécuté
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 10, 14)); // R10 = 14 (succès)
    
    // Test 3: R1 <= R0 (20 <= 10) → branchement NON PRIS
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 1, 0));
    let current_addr = Instruction::calculate_current_address(&program.code);
    let jmpiflessequal_target_3 = current_addr + 8 + 6;
    program.add_instruction(Instruction::create_jump_if_less_equal(current_addr, jmpiflessequal_target_3));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 10, 15)); // R10 = 15 (succès non pris)

    // ============================================================================
    // SECTION 9: TEST JmpIfZero (ZF = 1)
    // ============================================================================
    println!("Section 9: Test JmpIfZero");
    
    // Test 1: R0 == R2 (10 == 10) → ZF = 1 → branchement PRIS
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 2));
    let current_addr = Instruction::calculate_current_address(&program.code);
    let jmpifzero_target_1 = current_addr + 8 + 6;
    program.add_instruction(Instruction::create_jump_if_zero(current_addr, jmpifzero_target_1));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 15, 0xFF)); // ÉCHEC si exécuté
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 10, 16)); // R10 = 16 (succès)
    
    // Test 2: R0 != R1 (10 != 20) → ZF = 0 → branchement NON PRIS
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 1));
    let current_addr = Instruction::calculate_current_address(&program.code);
    let jmpifzero_target_2 = current_addr + 8 + 6;
    program.add_instruction(Instruction::create_jump_if_zero(current_addr, jmpifzero_target_2));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 10, 17)); // R10 = 17 (succès non pris)

    // ============================================================================
    // SECTION 10: TEST JmpIfNotZero (ZF = 0)
    // ============================================================================
    println!("Section 10: Test JmpIfNotZero");
    
    // Test 1: R0 != R1 (10 != 20) → ZF = 0 → branchement PRIS
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 1));
    let current_addr = Instruction::calculate_current_address(&program.code);
    let jmpifnotzero_target_1 = current_addr + 8 + 6;
    program.add_instruction(Instruction::create_jump_if_not_zero(current_addr, jmpifnotzero_target_1));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 15, 0xFF)); // ÉCHEC si exécuté
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 10, 18)); // R10 = 18 (succès)
    
    // Test 2: R0 == R2 (10 == 10) → ZF = 1 → branchement NON PRIS
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 2));
    let current_addr = Instruction::calculate_current_address(&program.code);
    let jmpifnotzero_target_2 = current_addr + 8 + 6;
    program.add_instruction(Instruction::create_jump_if_not_zero(current_addr, jmpifnotzero_target_2));
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 10, 19)); // R10 = 19 (succès non pris)

    // ============================================================================
    // SECTION 11: FINALISATION ET VALIDATION
    // ============================================================================
    println!("Section 11: Finalisation");
    
    // Marquer la fin des tests avec un identificateur unique
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

    let data_size = 512;
    let data_segment = SegmentMetadata::new(SegmentType::Data, 0, data_size, 0x1000);
    program.segments.push(data_segment);
    program.data = vec![0; data_size as usize];

    // ============================================================================
    // AFFICHAGE DE LA CARTE DES INSTRUCTIONS
    // ============================================================================
    println!("\n=== CARTE COMPLÈTE DES INSTRUCTIONS ===");
    let mut addr = 0u32;
    let mut section_map = HashMap::new();
    
    // Définir les sections par index d'instruction
    let sections = [
        (0..7, "INIT"),
        (7..10, "JMP"),
        (10..14, "JmpIfEqual"),
        (14..18, "JmpIfNotEqual"),
        (18..22, "JmpIfGreater"),
        (22..26, "JmpIfLess"),
        (26..32, "JmpIfGreaterEqual"),
        (32..38, "JmpIfLessEqual"),
        (38..42, "JmpIfZero"),
        (42..46, "JmpIfNotZero"),
        (46..48, "FINAL"),
    ];
    
    for (range, name) in &sections {
        for i in range.clone() {
            section_map.insert(i, *name);
        }
    }

    for (idx, instr) in program.code.iter().enumerate() {
        let size = instr.total_size();
        let section = section_map.get(&idx).unwrap_or(&"UNKNOWN");

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

    println!("\n=== RÉSUMÉ DES TESTS ===");
    println!("Total: {} instructions, {} bytes", program.code.len(), addr);
    println!("Tests de branchement: {} sections", sections.len() - 2); // -2 pour INIT et FINAL

    println!("\n=== VALIDATION ATTENDUE ===");
    println!("Si tous les tests réussissent:");
    println!("- R0  = 254 (0xFE) - Marqueur de fin");
    println!("- R10 = 19 - Compteur de succès (tous les tests passés)");
    println!("- R15 = 0  - Aucun échec détecté");
    println!("- Programme s'exécute complètement jusqu'à HALT");
    println!("- Aucune erreur 'Instruction non trouvée'");
    println!();

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

    // // ============================================================================
    // // SECTION 12: TEST CALL/RET (Si implémenté)
    // // ============================================================================
    // println!("=== SECTION 12: TEST CALL/RET ===");
    // current_address = Instruction::calculate_current_address(&program.code);
    // let call_target = current_address + 8 + 6 ;
    // // Sauter par-dessus la fonction pour aller au call
    // program.add_instruction(Instruction::create_jump(current_address, call_target)); // Sauter la fonction
    // current_address = Instruction::calculate_current_address(&program.code);
    //
    // // FONCTION: simple_function
    // // Fonction qui met 0xFF dans R5 et retourne
    // program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 5, 0xFF)); // R5 = 255
    // program.add_instruction(Instruction::create_no_args(Opcode::Ret)); // Retour
    // current_address = Instruction::calculate_current_address(&program.code);
    //
    // // Appel de la fonction (si CALL est implémenté)
    // current_address = Instruction::calculate_current_address(&program.code);
    // let function_offset = -12; // Retourner à la fonction
    // // program.add_instruction(Instruction::create_call(function_offset));

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

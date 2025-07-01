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
    // let program = memory_intensive_stress_test(); // Test mémoire intensif pour AGU avec optimisations
    let program = create_agu_parallel_optimized_test(); // Test optimisé pour exécution parallèle AGU/ALU
    // let program = punk_program_3(); // Test de branchement pour vérifier le ParallelExecutionEngine
    // let program = store_load_forwarding_test_16(); // Tests Store-Load forwarding  immédiat 16
    // let program = store_load_forwarding_test_32(); // Tests Store-Load forwarding immédiat 32
    // let program = store_load_forwarding_test_64(); // Tests Store-Load forwarding immédiat 64
    //
    // let program = cache_hierarchy_validation_test(); // Test hiérarchie cache L1/L2
    // let program = punk_program_3(); // Tests de branchement
    // let program= punk_program_5(); // Tests multiples branchements conditionnels et inconditionnels
    // let program = punk_program_3(); // Retour au test original pour confirmer BTB
    // let program = store_load_forwarding_test_64(); // Tests Store-Load forwarding immédiat 64
    // let program = simd_advanced_test(); // Test SIMD avancé (Min, Max, Sqrt, Cmp, Shuffle)
    // let program = simd_cache_validation_test(); // Test validation cache SIMD
    // let program = cache_hierarchy_validation_test(); // Test hiérarchie cache L1/L2


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
    // Affichage des registres généraux
    println!("\n===== REGISTRES GÉNÉRAUX =====");
    for i in 0..16 {
        if i % 4 == 0 && i > 0 {
            println!();
        }
        print!("R{:<2} = {:<10}", i, vm.registers[i]);
        if i % 4 == 3 {
            println!();
        }
    }
    
    // Affichage des registres spéciaux (SP, BP, RA)
    if vm.registers.len() > 16 {
        println!("\n===== REGISTRES SPÉCIAUX =====");
        for i in 16..vm.registers.len() {
            let reg_name = match i {
                16 => "SP ",
                17 => "BP ",
                18 => "RA ",
                _ => "R  ",
            };
            print!("{} = {:<10}  ", reg_name, vm.registers[i]);
        }
        println!();
    }
    
    // Affichage des registres vectoriels 128-bit
    println!("\n===== REGISTRES VECTORIELS 128-BIT =====");
    let vector_alu = vm.get_vector_alu();
    let vector_alu_borrowed = vector_alu;
    for i in 0..16 {
        let v128 = vector_alu_borrowed.v128_registers[i];
        unsafe {
            // Affichage en format i32x4 (le plus courant)
            print!("\nV{:<2} = [{:>6}, {:>6}, {:>6}, {:>6}]",
                i, v128.i32x4[0], v128.i32x4[1], v128.i32x4[2], v128.i32x4[3]);
            
            if i % 2 == 1 {
                println!();
            } else {
                print!("  ");
            }
        }
    }
    
    // Affichage des registres vectoriels 256-bit
    println!("\n\n===== REGISTRES VECTORIELS 256-BIT =====");
    for i in 0..16 {
        let v256 = vector_alu_borrowed.v256_registers[i];
        unsafe {
            // Affichage en format i32x8 (le plus courant)
            print!("Y{:<2} = [{:>6}, {:>6}, {:>6}, {:>6}, {:>6}, {:>6}, {:>6}, {:>6}]", 
                i, 
                v256.i32x8[0], v256.i32x8[1], v256.i32x8[2], v256.i32x8[3],
                v256.i32x8[4], v256.i32x8[5], v256.i32x8[6], v256.i32x8[7]);
            println!();
        }
    }
    
    // Affichage des flags vectoriels
    println!("\n===== FLAGS VECTORIELS =====");
    let flags = &vector_alu_borrowed.flags;
    println!("Zero: {} | Sign: {} | Overflow: {} | Underflow: {} | Denormal: {} | Invalid: {}",
        flags.zero, flags.sign, flags.overflow, flags.underflow, flags.denormal, flags.invalid);
    
    println!();
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

    // Statistiques SIMD
    println!("\n===== STATISTIQUES SIMD =====");
    println!("Opérations SIMD 128-bit: {}", stats.simd128_ops);
    println!("Opérations SIMD 256-bit: {}", stats.simd256_ops);
    let total_simd_ops = stats.simd128_ops + stats.simd256_ops;
    println!("Total opérations SIMD: {}", total_simd_ops);
    
    if total_simd_ops > 0 {
        println!("Cycles SIMD totaux: {}", stats.simd_total_cycles);
        println!("Opérations SIMD/cycle: {:.2}", stats.simd_ops_per_cycle);
        println!("Opérations parallélisées: {}", stats.simd_parallel_ops);
        
        println!("\n--- Cache d'Opérations SIMD ---");
        println!("Cache hits: {}", stats.simd_cache_hits);
        println!("Cache misses: {}", stats.simd_cache_misses);
        println!("Taux de réussite du cache: {:.2}%", stats.simd_cache_hit_rate);
        
        let total_cache_accesses = stats.simd_cache_hits + stats.simd_cache_misses;
        if total_cache_accesses > 0 {
            let cache_efficiency = (stats.simd_cache_hits as f64 / total_cache_accesses as f64) * 100.0;
            println!("Efficacité globale du cache SIMD: {:.1}%", cache_efficiency);
        }
        
        // Analyse de performance
        if stats.simd_parallel_ops > 0 {
            let parallelization_rate = (stats.simd_parallel_ops as f64 / total_simd_ops as f64) * 100.0;
            println!("Taux de parallélisation SIMD: {:.1}%", parallelization_rate);
        }
    } else {
        println!("Aucune opération SIMD détectée");
    }

    // Statistiques AGU (Address Generation Unit)
    println!("\n===== STATISTIQUES AGU =====");
    println!("Calculs d'adresse totaux: {}", stats.agu_total_calculations);
    println!("Résolutions anticipées: {}", stats.agu_early_resolutions);
    
    if stats.agu_total_calculations > 0 {
        println!("\n--- Stride Predictor ---");
        println!("Prédictions de stride totales: {}", stats.agu_stride_predictions_total);
        println!("Prédictions correctes: {}", stats.agu_stride_predictions_correct);
        println!("Précision du stride predictor: {:.2}%", stats.agu_stride_accuracy * 100.0);
        
        println!("\n--- Base Address Cache ---");
        println!("Cache hits: {}", stats.agu_base_cache_hits);
        println!("Cache misses: {}", stats.agu_base_cache_misses);
        println!("Taux de réussite: {:.2}%", stats.agu_base_cache_hit_rate * 100.0);
        
        println!("\n--- Performance AGU ---");
        println!("Exécutions parallèles AGU/ALU: {}", stats.agu_parallel_executions);
        if stats.agu_average_latency > 0.0 {
            println!("Latence moyenne: {:.2} cycles", stats.agu_average_latency);
        }
        
        // Analyse de l'efficacité
        if stats.agu_total_calculations > 0 {
            let early_resolution_rate = (stats.agu_early_resolutions as f64 / stats.agu_total_calculations as f64) * 100.0;
            println!("Taux de résolution anticipée: {:.1}%", early_resolution_rate);
        }
        
        if stats.cycles > 0 {
            let agu_utilization = (stats.agu_total_calculations as f64 / stats.cycles as f64) * 100.0;
            println!("Utilisation AGU: {:.1}% des cycles", agu_utilization);
        }
    } else {
        println!("Aucun calcul d'adresse AGU détecté");
    }

    // Statistiques Dual-Issue Controller
    println!("\n===== STATISTIQUES DUAL-ISSUE =====");
    println!("Instructions traitées: {}", stats.dual_issue_total_instructions);
    println!("Exécutions parallèles: {}", stats.dual_issue_parallel_executions);
    println!("Taux de parallélisme: {:.2}%", stats.dual_issue_parallel_rate);
    
    if stats.dual_issue_total_instructions > 0 {
        println!("\n--- Répartition par Unité ---");
        println!("Instructions ALU uniquement: {}", stats.dual_issue_alu_only);
        println!("Instructions AGU uniquement: {}", stats.dual_issue_agu_only);
        println!("Conflits de ressources: {}", stats.dual_issue_resource_conflicts);
        
        // Analyse de l'efficacité dual-issue
        let alu_ratio = (stats.dual_issue_alu_only as f64 / stats.dual_issue_total_instructions as f64) * 100.0;
        let agu_ratio = (stats.dual_issue_agu_only as f64 / stats.dual_issue_total_instructions as f64) * 100.0;
        
        println!("\n--- Efficacité Dual-Issue ---");
        println!("Ratio ALU: {:.1}%", alu_ratio);
        println!("Ratio AGU: {:.1}%", agu_ratio);
        
        if stats.dual_issue_parallel_executions > 0 {
            let theoretical_max = stats.dual_issue_total_instructions / 2;
            let efficiency = (stats.dual_issue_parallel_executions as f64 / theoretical_max as f64) * 100.0;
            println!("Efficacité théorique: {:.1}% ({}/{} max)", 
                    efficiency, stats.dual_issue_parallel_executions, theoretical_max);
        }
        
        if stats.cycles > 0 {
            let dual_issue_impact = (stats.dual_issue_parallel_executions as f64 / stats.cycles as f64) * 100.0;
            println!("Impact dual-issue: {:.1}% des cycles", dual_issue_impact);
        }
    } else {
        println!("Aucune instruction traitée par dual-issue");
    }

    // Statistiques Parallel Execution Engine
    println!("\n===== STATISTIQUES PARALLEL ENGINE =====");
    println!("Instructions totales: {}", stats.parallel_engine_total_instructions);
    println!("Exécutions parallèles: {}", stats.parallel_engine_parallel_executions);
    println!("Instructions ALU: {}", stats.parallel_engine_alu_instructions);
    println!("Instructions AGU: {}", stats.parallel_engine_agu_instructions);
    println!("Instructions SIMD: {}", stats.parallel_engine_simd_instructions);
    
    if stats.parallel_engine_total_instructions > 0 {
        println!("\n--- Analyse des Dépendances ---");
        println!("Dépendances RAW: {}", stats.parallel_engine_raw_dependencies);
        println!("Dépendances WAR: {}", stats.parallel_engine_war_dependencies);
        println!("Dépendances WAW: {}", stats.parallel_engine_waw_dependencies);
        println!("Stalls dépendances: {}", stats.parallel_engine_dependency_stalls);
        println!("Conflits ressources: {}", stats.parallel_engine_resource_conflicts);
        
        println!("\n--- Utilisation des Unités ---");
        println!("Utilisation ALU: {:.2}%", stats.parallel_engine_alu_utilization);
        println!("Utilisation AGU: {:.2}%", stats.parallel_engine_agu_utilization);
        println!("Profondeur queue moyenne: {:.2}", stats.parallel_engine_average_queue_depth);
        
        // Analyse du parallélisme
        println!("\n--- Efficacité Parallélisme ---");
        println!("Taux d'exécution parallèle: {:.2}%", stats.parallel_engine_parallel_rate);
        
        if stats.parallel_engine_parallel_executions > 0 {
            println!("Gain théorique IPC: +{:.1}%", stats.parallel_engine_parallel_rate);
            
            // Comparer avec dual-issue
            if stats.dual_issue_parallel_executions > 0 {
                println!("Amélioration vs dual-issue: {}x plus d'exécutions parallèles", 
                         stats.parallel_engine_parallel_executions as f64 / stats.dual_issue_parallel_executions as f64);
            }
        }
    } else {
        println!("Aucune instruction traitée par le parallel engine");
    }

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
    program.add_metadata("author", "PunkVM #YmC");

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
    program.add_metadata("author", "PunkVM @YmC");
    
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
    program.add_metadata("description", "Test complet des instructions SIMD avec nouveaux helpers");
    program.add_metadata("author", "PunkVM Team");

    println!("=== CRÉATION DU TEST D'INSTRUCTIONS SIMD ===");

    // ============================================================================
    // SECTION 1: INITIALISATION DES VECTEURS CONSTANTS
    // ============================================================================
    println!("\n1. Initialisation des vecteurs constants");
    
    // Maintenant on utilise directement les instructions de constantes SIMD
    
    // V0 = [1, 2, 3, 4]
    program.add_instruction(Instruction::create_simd128_const_i32x4(0, [1, 2, 3, 4]));
    
    // V1 = [5, 6, 7, 8]
    program.add_instruction(Instruction::create_simd128_const_i32x4(1, [5, 6, 7, 8]));
    
    // V2 = [10, 20, 30, 40]
    program.add_instruction(Instruction::create_simd128_const_i32x4(2, [10, 20, 30, 40]));
    
    // V3 = [1.0, 2.0, 3.0, 4.0] (float)
    program.add_instruction(Instruction::create_simd128_const_f32x4(3, [1.0, 2.0, 3.0, 4.0]));
    
    // V4 = [5.5, 6.5, 7.5, 8.5] (float)
    program.add_instruction(Instruction::create_simd128_const_f32x4(4, [5.5, 6.5, 7.5, 8.5]));
    
    // Vecteurs 256-bit pour démonstration
    // Y0 = [1, 2, 3, 4, 5, 6, 7, 8]
    program.add_instruction(Instruction::create_simd256_const_i32x8(0, [1, 2, 3, 4, 5, 6, 7, 8]));
    
    // Y1 = [10, 20, 30, 40, 50, 60, 70, 80]
    program.add_instruction(Instruction::create_simd256_const_i32x8(1, [10, 20, 30, 40, 50, 60, 70, 80]));

    // ============================================================================
    // SECTION 2: OPÉRATIONS ARITHMÉTIQUES VECTORIELLES
    // ============================================================================
    println!("\n2. Opérations arithmétiques vectorielles");
    
    // Addition: V5 = V0 + V1 = [6, 8, 10, 12]
    program.add_instruction(Instruction::create_simd128_add(5, 0, 1));
    println!("   V5 = V0 + V1 = [6, 8, 10, 12]");
    
    // Soustraction: V6 = V2 - V1 = [5, 14, 23, 32]
    program.add_instruction(Instruction::create_simd128_sub(6, 2, 1));
    println!("   V6 = V2 - V1 = [5, 14, 23, 32]");
    
    // Multiplication: V7 = V0 * V1 = [5, 12, 21, 32]
    program.add_instruction(Instruction::create_simd128_mul(7, 0, 1));
    println!("   V7 = V0 * V1 = [5, 12, 21, 32]");
    
    // Division: V8 = V2 / V0 = [10, 10, 10, 10]
    program.add_instruction(Instruction::create_simd128_div(8, 2, 0));
    println!("   V8 = V2 / V0 = [10, 10, 10, 10]");

    // ============================================================================
    // SECTION 3: OPÉRATIONS LOGIQUES VECTORIELLES
    // ============================================================================
    println!("\n3. Opérations logiques vectorielles");
    
    // Utiliser les vecteurs déjà chargés pour les opérations logiques
    // AND: V9 = V0 & V1
    program.add_instruction(Instruction::create_simd128_and(9, 0, 1));
    println!("   V9 = V0 & V1");
    
    // OR: V10 = V0 | V1
    program.add_instruction(Instruction::create_simd128_or(10, 0, 1));
    println!("   V10 = V0 | V1");
    
    // XOR: V11 = V0 ^ V1
    program.add_instruction(Instruction::create_simd128_xor(11, 0, 1));
    println!("   V11 = V0 ^ V1");
    
    // NOT: V12 = ~V0
    program.add_instruction(Instruction::create_simd128_not(12, 0));
    println!("   V12 = ~V0");

    // ============================================================================
    // SECTION 4: MOUVEMENT ET COPIE DE VECTEURS
    // ============================================================================
    println!("\n4. Mouvement de vecteurs");
    
    // Copier V5 dans V13
    program.add_instruction(Instruction::create_simd128_mov(13, 5));
    println!("   V13 = V5 (copie)");

    // ============================================================================
    // SECTION 5: MÉMOIRE VECTORIELLE (Load/Store)
    // ============================================================================
    println!("\n5. Opérations mémoire vectorielles");
    
    // Initialiser un registre de base pour les adresses de stockage
    program.add_instruction(Instruction::create_reg_imm16(Opcode::Mov, 13, 0x3000)); // R13 = 0x3000
    
    // Stocker V5 à l'adresse R13 + 0
    program.add_instruction(Instruction::create_simd128_store(5, 13, 0));
    println!("   Store V5 -> [R13 + 0]");
    
    // Stocker V7 à l'adresse R13 + 16
    program.add_instruction(Instruction::create_simd128_store(7, 13, 16));
    println!("   Store V7 -> [R13 + 16]");
    
    // Charger depuis la mémoire dans V14
    program.add_instruction(Instruction::create_simd128_load(14, 13, 0));
    println!("   Load [R13 + 0] -> V14");

    // ============================================================================
    // SECTION 6: TESTS AVEC VECTEURS 256-bit
    // ============================================================================
    println!("\n6. Opérations vectorielles 256-bit");
    
    // Addition 256-bit: V15 = V0 + V1 (interprétés comme 256-bit)
    program.add_instruction(Instruction::create_simd256_add(15, 0, 1));
    println!("   V15 = V0 + V1 (256-bit)");
    
    // Multiplication 256-bit: V16 = V2 * V0
    program.add_instruction(Instruction::create_simd256_mul(16, 2, 0));
    println!("   V16 = V2 * V0 (256-bit)");

    // ============================================================================
    // SECTION 7: CHAÎNAGE D'OPÉRATIONS
    // ============================================================================
    println!("\n7. Chaînage d'opérations vectorielles");
    
    // V17 = (V0 + V1) * V2
    program.add_instruction(Instruction::create_simd128_add(17, 0, 1));  // V17 = V0 + V1
    program.add_instruction(Instruction::create_simd128_mul(17, 17, 2)); // V17 = V17 * V2
    println!("   V17 = (V0 + V1) * V2");

    // Fin du programme
    program.add_instruction(Instruction::create_no_args(Opcode::Halt));

    // Configuration des segments
    let total_code_size: u32 = program.code.iter()
        .map(|instr| instr.total_size() as u32)
        .sum();
    program.segments = vec![SegmentMetadata::new(SegmentType::Code, 0, total_code_size, 0)];


    println!("\n=== CARTE DES INSTRUCTIONS TEST ===");
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

    println!("\n=== RÉSULTATS ATTENDUS SIMD ===");

    program
}

/// Test spécialisé pour les opérations SIMD avancées (Min, Max, Sqrt, Cmp, Shuffle)
fn simd_advanced_test() -> BytecodeFile {
    let mut program = BytecodeFile::new();
    
    println!("=== CRÉATION DU TEST SIMD AVANCÉ ===");
    
    // Configuration des métadonnées
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    program.add_metadata("name", "PunkVM SIMD Advanced");
    program.add_metadata("description", "Test complet des instructions SIMD Advanced ");
    program.add_metadata("author", "PunkVM @YmC");
    
    // 1. Initialisation de vecteurs pour les tests
    println!("1. Initialisation des vecteurs pour tests avancés");
    
    // V0 = [1, 2, 3, 4] - Vecteur de base
    program.add_instruction(Instruction::create_simd128_const_i32x4(0, [1, 2, 3, 4]));
    
    // V1 = [4, 1, 5, 2] - Vecteur pour comparaisons Min/Max
    program.add_instruction(Instruction::create_simd128_const_i32x4(1, [4, 1, 5, 2]));
    
    // V2 = [1.0, 4.0, 9.0, 16.0] - Vecteur pour Sqrt
    program.add_instruction(Instruction::create_simd128_const_f32x4(2, [1.0, 4.0, 9.0, 16.0]));
    
    // V3 = [1, 2, 3, 4] - Vecteur pour comparaison (identique à V0)
    program.add_instruction(Instruction::create_simd128_const_i32x4(3, [1, 2, 3, 4]));
    
    // V4 = masque pour shuffle (réorganisation byte-level)
    program.add_instruction(Instruction::create_simd128_const_i32x4(4, [0x03020100, 0x07060504, 0x0B0A0908, 0x0F0E0D0C]));
    
    // 2. Opérations Min/Max
    println!("2. Test des opérations Min/Max");
    
    // V5 = min(V0, V1) => [1, 1, 3, 2]
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Simd128Min, 5, 0, 1));
    
    // V6 = max(V0, V1) => [4, 2, 5, 4]
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Simd128Max, 6, 0, 1));
    
    // 3. Opération Sqrt (racine carrée)
    println!("3. Test de l'opération Sqrt");
    
    // V7 = sqrt(V2) => [1.0, 2.0, 3.0, 4.0]
    program.add_instruction(Instruction::create_reg_reg(Opcode::Simd128Sqrt, 7, 2));
    
    // 4. Opération de comparaison
    println!("4. Test de l'opération Cmp");
    
    // V8 = cmp(V0, V3) => [-1, -1, -1, -1] (tous égaux)
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Simd128Cmp, 8, 0, 3));
    
    // 5. Opération Shuffle
    println!("5. Test de l'opération Shuffle");
    
    // V9 = shuffle(V0, V4) => réorganise selon le masque
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Simd128Shuffle, 9, 0, 4));
    
    // 6. Tests 256-bit
    println!("6. Tests d'opérations 256-bit avancées");
    
    // Y0 = [1, 2, 3, 4, 5, 6, 7, 8]
    program.add_instruction(Instruction::create_simd256_const_i32x8(0, [1, 2, 3, 4, 5, 6, 7, 8]));
    
    // Y1 = [8, 1, 6, 3, 4, 7, 2, 9]
    program.add_instruction(Instruction::create_simd256_const_i32x8(1, [8, 1, 6, 3, 4, 7, 2, 9]));
    
    // Y2 = min(Y0, Y1) => [1, 1, 3, 3, 4, 6, 2, 8]
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Simd256Min, 2, 0, 1));
    
    // Y3 = max(Y0, Y1) => [8, 2, 6, 4, 5, 7, 7, 9]
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Simd256Max, 3, 0, 1));
    
    // Y4 = cmp(Y0, Y1) => masque de comparaison
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Simd256Cmp, 4, 0, 1));
    
    // 7. Tests avec vecteurs flottants 256-bit
    println!("7. Tests sqrt 256-bit");
    
    // Y5 = [1.0, 4.0, 9.0, 16.0, 25.0, 36.0, 49.0, 64.0]
    program.add_instruction(Instruction::create_simd256_const_f32x8(5, [1.0, 4.0, 9.0, 16.0, 25.0, 36.0, 49.0, 64.0]));
    
    // Y6 = sqrt(Y5) => [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]
    program.add_instruction(Instruction::create_reg_reg(Opcode::Simd256Sqrt, 6, 5));
    
    // Fin du programme
    program.add_instruction(Instruction::create_no_args(Opcode::Halt));
    
    // Configuration des segments
    let total_code_size: u32 = program.code.iter()
        .map(|instr| instr.total_size() as u32)
        .sum();
    program.segments = vec![SegmentMetadata::new(SegmentType::Code, 0, total_code_size, 0)];
    
    println!("\n=== RÉSULTATS ATTENDUS SIMD AVANCÉ ===");
    println!("V5 (min 128): [1, 1, 3, 2]");
    println!("V6 (max 128): [4, 2, 5, 4]");
    println!("V7 (sqrt 128): [1.0, 2.0, 3.0, 4.0]");
    println!("V8 (cmp 128): [-1, -1, -1, -1] (tous égaux)");
    println!("V9 (shuffle 128): réorganisé selon masque");
    println!("Y2 (min 256): [1, 1, 3, 3, 4, 6, 2, 8]");
    println!("Y3 (max 256): [8, 2, 6, 4, 5, 7, 7, 9]");
    println!("Y4 (cmp 256): masque de comparaison");
    println!("Y6 (sqrt 256): [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]");
    
    program
}

/// Test de validation du cache SIMD avec opérations répétées
fn simd_cache_validation_test() -> BytecodeFile {
    let mut program = BytecodeFile::new();
    
    println!("=== CRÉATION DU TEST DE VALIDATION CACHE SIMD ===");
    
    // Métadonnées du programme
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    program.add_metadata("name", "PunkVM SIMD Cache Validation");
    program.add_metadata("description", "Test validation cache SIMD avec opérations répétées");
    program.add_metadata("author", "PunkVM @YmC");
    
    // Initialiser les vecteurs de test
    program.add_instruction(Instruction::create_simd128_const_i32x4(0, [100, 200, 300, 400]));
    program.add_instruction(Instruction::create_simd128_const_i32x4(1, [10, 20, 30, 40]));
    program.add_instruction(Instruction::create_simd128_const_i32x4(2, [1, 2, 3, 4]));
    
    println!("1. Initialisation: V0=[100,200,300,400], V1=[10,20,30,40], V2=[1,2,3,4]");
    
    // ============================================================================
    // PHASE 1: Opérations répétées identiques (pour tester le cache)
    // ============================================================================
    println!("\n2. Test cache - 10 opérations Add identiques (V0 + V1)");
    for i in 3..13 {
        program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Simd128Add, i, 0, 1));
    }
    
    println!("3. Test cache - 10 opérations Mul identiques (V0 * V2)");
    for i in 13..23 {
        program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Simd128Mul, i, 0, 2));
    }
    
    // ============================================================================
    // PHASE 2: Séquence d'opérations variées pour tester la gestion du cache
    // ============================================================================
    println!("\n4. Test cache - Séquence d'opérations variées");
    
    // Répéter la même séquence 3 fois pour valider le cache
    for cycle in 0..3 {
        let base_reg = 23 + cycle * 6;
        
        // V(base) = V0 + V1 (devrait être en cache après le premier cycle)
        program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Simd128Add, base_reg, 0, 1));
        
        // V(base+1) = V0 * V2 (devrait être en cache après le premier cycle)
        program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Simd128Mul, base_reg + 1, 0, 2));
        
        // V(base+2) = V1 - V2 (nouvelle opération)
        program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Simd128Sub, base_reg + 2, 1, 2));
        
        // V(base+3) = V0 & V1 (opération logique)
        program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Simd128And, base_reg + 3, 0, 1));
        
        // V(base+4) = V1 | V2 (opération logique)
        program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Simd128Or, base_reg + 4, 1, 2));
        
        // V(base+5) = V0 ^ V2 (opération logique)
        program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Simd128Xor, base_reg + 5, 0, 2));
    }
    
    // ============================================================================
    // PHASE 3: Test des opérations avancées avec cache
    // ============================================================================
    println!("\n5. Test cache - Opérations avancées répétées");
    
    // Vecteurs pour opérations flottantes
    program.add_instruction(Instruction::create_simd128_const_f32x4(41, [4.0, 9.0, 16.0, 25.0]));
    program.add_instruction(Instruction::create_simd128_const_f32x4(42, [2.0, 3.0, 4.0, 5.0]));
    
    // Répéter Sqrt 5 fois (opération coûteuse)
    for i in 43..48 {
        program.add_instruction(Instruction::create_reg_reg(Opcode::Simd128Sqrt, i, 41));
    }
    
    // Répéter Min 5 fois
    for i in 48..53 {
        program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Simd128Min, i, 41, 42));
    }
    
    // Répéter Max 5 fois
    for i in 53..58 {
        program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Simd128Max, i, 41, 42));
    }
    
    println!("\n6. Instructions générées: ~70 instructions SIMD avec répétitions");
    
    // Fin du programme
    program.add_instruction(Instruction::create_no_args(Opcode::Halt));
    
    // Configuration des segments
    let total_code_size: u32 = program.code.iter()
        .map(|instr| instr.total_size() as u32)
        .sum();
    program.segments = vec![SegmentMetadata::new(SegmentType::Code, 0, total_code_size, 0)];
    
    println!("\n=== VALIDATION ATTENDUE ===");
    println!("- Premières opérations: cache misses");
    println!("- Opérations répétées: cache hits élevés");
    println!("- Taux de hit attendu: >70% après initialisation");
    println!("- Performance améliorée sur opérations répétitives");
    
    program
}

/// Test spécialement conçu pour tester la hiérarchie cache L1/L2
fn cache_hierarchy_validation_test() -> BytecodeFile {
    let mut program = BytecodeFile::new();
    
    println!("=== CRÉATION DU TEST HIÉRARCHIE CACHE ===");
    
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    program.add_metadata("name", "Cache Hierarchy Test");
    program.add_metadata("description", "Test de validation de la hiérarchie cache L1/L2");
    program.add_metadata("author", "PunkVM Cache Validation");
    
    // Phase 1: Remplir L1 cache avec des accès séquentiels
    println!("Phase 1: Accès séquentiels pour remplir L1");
    
    // Initialiser une base d'adresse
    program.add_instruction(Instruction::create_reg_imm32(Opcode::Mov, 0, 0x1000)); // R0 = base addr
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 1));       // R1 = compteur
    
    // Écrire 64 valeurs espacées (4KB total pour dépasser L1 de 2KB)
    for i in 0..64 {
        // Calculer adresse = base + i*64 (espacer d'une ligne de cache complète)
        program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 2, i as u8));
        program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 7, 64)); // 64 bytes par ligne
        program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Mul, 8, 2, 7)); // R8 = i * 64
        program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 3, 0, 8)); // R3 = base + (i*64)
        
        // Écrire valeur à cette adresse
        program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 4, (i + 10) as u8)); // valeur
        program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 4, 3, 0)); // Stocker R4 à [R3+0]
    }
    
    // Phase 2: Relire les mêmes données (devrait être des L1 hits maintenant)
    println!("Phase 2: Relecture des mêmes données (L1 hits attendus)");
    
    for i in 0..64 {
        // Calculer la même adresse (i * 64)
        program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 2, i as u8));
        program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 7, 64)); 
        program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Mul, 8, 2, 7)); // R8 = i * 64
        program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 3, 0, 8)); // R3 = base + (i*64)
        
        // Lire la valeur
        program.add_instruction(Instruction::create_load_reg_offset(5, 3, 0)); // R5 = [R3+0]
    }
    
    // Phase 3: Accès à un range d'adresses plus large pour dépasser L1 et aller en L2
    println!("Phase 3: Accès large pour tester L2");
    
    // Changer de base d'adresse pour éviter les conflits
    program.add_instruction(Instruction::create_reg_imm32(Opcode::Mov, 0, 0x2000)); // Nouvelle base
    
    // Accès espacés pour forcer l'éviction de L1 (beaucoup plus d'adresses)
    for i in 0..200 {  // Plus d'accès pour remplir le cache
        // Adresses très espacées (tous les 512 bytes pour maximiser les conflits)
        program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 2, (i % 256) as u8));
        program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 6, 0));
        
        // R6 = i * 512 (shift left de 9 positions)
        for _ in 0..9 {
            program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 6, 6, 6)); // R6 = R6 * 2
        }
        program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 3, 0, 6)); // R3 = base + offset
        
        // Écrire une valeur unique
        program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 4, ((i + 100) % 256) as u8));
        program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 4, 3, 0)); // Store
        
        // Pour certaines adresses, relire immédiatement (hit probable)
        if i % 3 == 0 {
            program.add_instruction(Instruction::create_load_reg_offset(5, 3, 0));  // Load immédiat
        }
    }
    
    // Phase 4: Revisiter les adresses du début pour tester L2 hits
    println!("Phase 4: Revisiter adresses initiales (L2 hits attendus)");
    
    program.add_instruction(Instruction::create_reg_imm32(Opcode::Mov, 0, 0x1000)); // Retour à la base originale
    
    // Relire TOUTES les adresses du début (elles devraient être en L2 maintenant car évincées de L1)
    for i in 0..32 {
        // Calculer adresse = base + i*64 (même espacement que Phase 1)
        program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 2, i as u8));
        program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 7, 64)); // 64 bytes par ligne
        program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Mul, 8, 2, 7)); // R8 = i * 64
        program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 3, 0, 8)); // R3 = base + (i*64)
        program.add_instruction(Instruction::create_load_reg_offset(5, 3, 0)); // Lecture qui devrait être L2 hit
    }
    
    // Phase 5: Encore plus d'accès pour forcer l'éviction et confirmer le comportement L2
    println!("Phase 5: Accès répétés pour tester consistance L2");
    
    // Répéter l'accès aux mêmes adresses pour confirmer qu'elles sont bien en L2
    for i in 0..16 {
        // Calculer adresse = base + i*64 (même espacement que Phase 1)
        program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 2, i as u8));
        program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 7, 64)); // 64 bytes par ligne
        program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Mul, 8, 2, 7)); // R8 = i * 64
        program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 3, 0, 8)); // R3 = base + (i*64)
        program.add_instruction(Instruction::create_load_reg_offset(5, 3, 0)); // Re-lecture
    }
    
    // Fin du programme
    program.add_instruction(Instruction::create_no_args(Opcode::Halt));
    
    // Configuration des segments
    let total_code_size: u32 = program.code.iter()
        .map(|instr| instr.total_size() as u32)
        .sum();
    program.segments = vec![SegmentMetadata::new(SegmentType::Code, 0, total_code_size, 0)];
    
    println!("\n=== RÉSULTATS ATTENDUS ===");
    println!("Phase 1: L1 misses initiaux");
    println!("Phase 2: L1 hits élevés (réutilisation)");
    println!("Phase 3: Mix L1/L2 misses (éviction L1)");
    println!("Phase 4: L2 hits (données évincées de L1)");
    println!("=> L2 Hits > 0 si hiérarchie fonctionne correctement");
    
    program
}

/// Programme 5: Test BTB simple avec boucle répétitive
pub fn punk_program_5() -> BytecodeFile {
    let mut program = BytecodeFile::new();
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    program.add_metadata("name", "PunkVM BTB Simple Loop Test");
    program.add_metadata("description", "Test simple avec une boucle for BTB hits");
    program.add_metadata("author", "PunkVM Team");

    println!("=== CRÉATION D'UNE BOUCLE SIMPLE POUR BTB HITS ===");

    // Initialisation
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 3));   // R0 = 3 (compteur plus petit)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 1));   // R1 = 1 (décrément)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 2, 0));   // R2 = 0 (accumulator)

    // DÉBUT DE LA BOUCLE - même PC sera exécuté 3 fois
    let loop_start = Instruction::calculate_current_address(&program.code);
    println!("Loop start address: 0x{:X}", loop_start);

    // Corps de la boucle
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 2, 2, 1)); // R2 = R2 + 1
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Sub, 0, 0, 1)); // R0 = R0 - 1
    
    // Comparer R0 avec 0 pour vérifier si on continue
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Cmp, 0, 0));      // Compare R0 avec 0

    // BRANCHEMENT CONDITIONNEL - Si R0 > 0, retourner au début
    let current_addr = Instruction::calculate_current_address(&program.code);
    program.add_instruction(Instruction::create_jump_if_not_zero(current_addr, loop_start));
    
    println!("Branch PC: 0x{:X} -> Target: 0x{:X}", current_addr + 8, loop_start);
    println!("Expected loop iterations: 3");
    println!("Expected BTB pattern: Miss(1st), Hit(2nd), Hit(3rd)");

    // Instructions après la boucle
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 15, 0xDD)); // R15 = 0xDD (marqueur de fin)
    program.add_instruction(Instruction::create_no_args(Opcode::Halt));

    // Configuration des segments
    let total_code_size: u32 = program.code.iter()
        .map(|instr| instr.total_size() as u32)
        .sum();
    program.segments = vec![SegmentMetadata::new(SegmentType::Code, 0, total_code_size, 0)];

    println!("\n=== RÉSULTATS ATTENDUS ===");
    println!("Après 3 itérations:");
    println!("- BTB Misses: 1 (première fois)");
    println!("- BTB Hits: 2 (deuxième et troisième fois)"); 
    println!("- BTB Hit Rate: 66.7%");
    println!("- R0 = 0 (compteur final)");
    println!("- R2 = 3 (accumulator final)");
    println!("- R15 = 221 (0xDD, marqueur de fin)");

    program
}

/// Programme 4: Test spécifique pour BTB (Branch Target Buffer) avec patterns répétitifs
pub fn punk_program_4() -> BytecodeFile {
    let mut program = BytecodeFile::new();
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    program.add_metadata("name", "PunkVM BTB Test - Repetitive Branch Patterns");
    program.add_metadata("description", "Test pour générer des hits dans le BTB via des patterns de branchement répétitifs");
    program.add_metadata("author", "PunkVM Team");
    program.add_metadata("focus", "BTB Hits, Target Prediction, Cache Behavior");

    println!("=== CRÉATION DU TEST BTB AVEC PATTERNS RÉPÉTITIFS ===");

    // ============================================================================
    // SECTION 1: INITIALISATION
    // ============================================================================
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 5));   // R0 = 5 (compteur de boucle externe)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 3));   // R1 = 3 (compteur de boucle interne)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 2, 0));   // R2 = 0 (registre de travail)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 3, 1));   // R3 = 1 (constante pour décrément)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 10, 0));  // R10 = compteur de hits BTB attendus

    // ============================================================================
    // SECTION 2: PATTERN 1 - Même branchement répété plusieurs fois
    // ============================================================================
    println!("=== SECTION 2: PATTERN RÉPÉTITIF SIMPLE ===");
    
    // Marquer le début de la boucle Pattern 1
    let pattern1_loop_start = Instruction::calculate_current_address(&program.code);
    
    // Corps de la boucle Pattern 1 - Répéter 5 fois le même branchement
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 2, 2, 3)); // R2 = R2 + 1
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Sub, 0, 0, 3)); // R0 = R0 - 1 (décrément compteur)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Cmp, 0, 0));      // Compare R0 avec 0
    
    // Calculer l'offset pour retourner au début
    let current_addr = Instruction::calculate_current_address(&program.code);
    let pattern1_target = pattern1_loop_start;
    
    // Branchement conditionnel - sera répété 5 fois avec la même cible
    program.add_instruction(Instruction::create_jump_if_not_zero(current_addr, pattern1_target));
    
    // Incrémenter le compteur de succès BTB
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 10, 10, 3)); // R10++

    // ============================================================================
    // SECTION 3: PATTERN 2 - Branchement avant/arrière alternant
    // ============================================================================
    println!("=== SECTION 3: PATTERN ALTERNANT ===");
    
    // Réinitialiser les compteurs
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 4));   // R0 = 4 (nouveau compteur)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 4, 0));   // R4 = compteur alternance
    
    // Marquer le début de la boucle Pattern 2
    let pattern2_loop_start = Instruction::calculate_current_address(&program.code);
    
    // Test d'alternance - si pair, sauter en avant, si impair, continuer
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 5, 2));      // R5 = 2 (pour modulo)
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Mod, 6, 4, 5)); // R6 = R4 % 2
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Cmp, 6, 0));      // Compare R6 avec 0
    
    // Si pair (R6 == 0), sauter vers skip_section
    let current_addr = Instruction::calculate_current_address(&program.code);
    let skip_target = current_addr + 32; // Sauter quelques instructions
    program.add_instruction(Instruction::create_jump_if_equal(current_addr, skip_target));
    
    // Section pour les itérations impaires
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 11, 11, 3)); // R11++ (compteur impair)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 7, 0xAA));      // R7 = marqueur impair
    
    // Saut inconditionnel pour éviter la section paire
    let current_addr = Instruction::calculate_current_address(&program.code);
    let after_pair_section = current_addr + 20;
    program.add_instruction(Instruction::create_jump(current_addr, after_pair_section));
    
    // Section pour les itérations paires (skip_section)
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 12, 12, 3)); // R12++ (compteur pair)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 8, 0xBB));      // R8 = marqueur pair
    
    // Point de convergence (after_pair_section)
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 4, 4, 3)); // R4++ (compteur alternance)
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Sub, 0, 0, 3)); // R0-- (compteur principal)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Cmp, 0, 0));      // Compare R0 avec 0
    
    // Branchement de retour - sera répété 4 fois avec la même cible
    let current_addr = Instruction::calculate_current_address(&program.code);
    program.add_instruction(Instruction::create_jump_if_not_zero(current_addr, pattern2_loop_start));

    // ============================================================================
    // SECTION 4: PATTERN 3 - Boucles imbriquées pour tester la capacité du BTB
    // ============================================================================
    println!("=== SECTION 4: BOUCLES IMBRIQUÉES ===");
    
    // Réinitialiser les compteurs
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 3));   // R0 = 3 (boucle externe)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 2));   // R1 = 2 (boucle interne)
    
    // Début de la boucle externe
    let outer_loop_start = Instruction::calculate_current_address(&program.code);
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 2));   // Réinitialiser compteur interne
    
    // Début de la boucle interne
    let inner_loop_start = Instruction::calculate_current_address(&program.code);
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 13, 13, 3)); // R13++ (compteur total)
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Sub, 1, 1, 3));   // R1-- (compteur interne)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Cmp, 1, 0));        // Compare R1 avec 0
    
    // Branchement de la boucle interne
    let current_addr = Instruction::calculate_current_address(&program.code);
    program.add_instruction(Instruction::create_jump_if_not_zero(current_addr, inner_loop_start));
    
    // Sortie de la boucle interne
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Sub, 0, 0, 3)); // R0-- (compteur externe)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Cmp, 0, 0));      // Compare R0 avec 0
    
    // Branchement de la boucle externe
    let current_addr = Instruction::calculate_current_address(&program.code);
    program.add_instruction(Instruction::create_jump_if_not_zero(current_addr, outer_loop_start));

    // ============================================================================
    // SECTION 5: PATTERN 4 - Branchements vers adresses fixes (targets répétitifs)
    // ============================================================================
    println!("=== SECTION 5: TARGETS RÉPÉTITIFS ===");
    
    // Créer plusieurs sauts vers la même fonction
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 14, 0));     // R14 = compteur d'appels
    
    // Calculer l'adresse de la fonction utilitaire
    let current_addr = Instruction::calculate_current_address(&program.code);
    let utility_function_addr = current_addr + 50; // Fonction après les appels
    
    // Appel 1
    program.add_instruction(Instruction::create_jump(current_addr, utility_function_addr));
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 14, 14, 3)); // R14++ (ne sera pas exécuté)
    
    // Appel 2 (même cible)
    let current_addr = Instruction::calculate_current_address(&program.code);
    program.add_instruction(Instruction::create_jump(current_addr, utility_function_addr));
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 14, 14, 3)); // R14++ (ne sera pas exécuté)
    
    // Appel 3 (même cible)
    let current_addr = Instruction::calculate_current_address(&program.code);
    program.add_instruction(Instruction::create_jump(current_addr, utility_function_addr));
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 14, 14, 3)); // R14++ (ne sera pas exécuté)
    
    // Sauter par-dessus la fonction
    let current_addr = Instruction::calculate_current_address(&program.code);
    let after_function = current_addr + 20;
    program.add_instruction(Instruction::create_jump(current_addr, after_function));
    
    // FONCTION UTILITAIRE (utility_function_addr)
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 15, 15, 3)); // R15++ (compteur d'exécutions)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 0xCC));      // R9 = marqueur fonction
    
    // Continuer vers after_function
    let current_addr = Instruction::calculate_current_address(&program.code);
    let after_function = current_addr + 8;
    program.add_instruction(Instruction::create_jump(current_addr, after_function));
    
    // POINT APRÈS LA FONCTION (after_function)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 0xFE)); // R0 = 254 (marqueur de fin)

    // ============================================================================
    // FINALISATION
    // ============================================================================
    program.add_instruction(Instruction::create_no_args(Opcode::Halt));

    // Configuration des segments
    let total_code_size: u32 = program.code.iter()
        .map(|instr| instr.total_size() as u32)
        .sum();
    program.segments = vec![SegmentMetadata::new(SegmentType::Code, 0, total_code_size, 0)];

    println!("\n=== RÉSULTATS ATTENDUS APRÈS CORRECTIONS BTB ===");
    println!("Avec les corrections du BTB, nous devrions voir:");
    println!("- BTB Hits > 0 (au lieu de 0)");
    println!("- BTB Hit Rate > 0% (pour les branchements répétitifs)");
    println!("- BTB Correct Targets > 0");
    println!("- R2 = 5 (Pattern 1 exécuté 5 fois)");
    println!("- R11 ≥ 2 (itérations impaires)");
    println!("- R12 ≥ 2 (itérations paires)");
    println!("- R13 = 6 (total boucles imbriquées: 3×2)");
    println!("- R15 = 3 (fonction appelée 3 fois)");
    println!("- R0 = 254 (marqueur de fin)");

    program
}

/// Test de validation AGU - Address Generation Unit
fn agu_validation_test() -> BytecodeFile {
    let mut program = BytecodeFile::new();
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    program.add_metadata("name", "AGU Validation Test");
    program.add_metadata("description", "Test spécifique pour valider l'Address Generation Unit");
    program.add_metadata("author", "PunkVM Team");

    println!("=== CRÉATION DU TEST VALIDATION AGU ===");

    // Test 1: Base + Offset simple
    println!("Test 1: AGU BaseOffset - STORE/LOAD avec adresse de base");
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 0x10)); // R0 = adresse de base
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 42));   // R1 = valeur à stocker
    program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 1, 0, 0));     // STORE [R0], R1 (AGU: BaseOffset)
    program.add_instruction(Instruction::create_load_reg_offset(2, 0, 0));        // LOAD R2, [R0] (AGU: BaseOffset)

    // Test 2: Base + Offset avec déplacement
    println!("Test 2: AGU BaseOffset - STORE/LOAD avec offset");
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 3, 84));   // R3 = nouvelle valeur
    program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 3, 0, 8));     // STORE [R0+8], R3 (AGU: BaseOffset)
    program.add_instruction(Instruction::create_load_reg_offset(4, 0, 8));        // LOAD R4, [R0+8] (AGU: BaseOffset)

    // Test 3: Accès séquentiels pour tester le stride predictor de l'AGU
    println!("Test 3: AGU Stride Prediction - Accès séquentiels");
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 5, 1));    // R5 = compteur
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 6, 100));  // R6 = valeur de base

    // Boucle pour tester le stride predictor (4 itérations)
    for i in 0..4 {
        let offset = i * 4; // Stride de 4 bytes
        program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 7, 6, 5)); // R7 = valeur unique
        program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 7, 0, offset)); // STORE [R0+offset], R7
        program.add_instruction(Instruction::create_load_reg_offset(8, 0, offset));     // LOAD R8, [R0+offset]
        program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 5, 5, 1));  // R5++ (incrément compteur)
    }

    // Test 4: Accès avec motifs irréguliers pour tester la robustesse de l'AGU
    println!("Test 4: AGU Robustess - Accès non-séquentiels");
    let offsets = [16, 4, 20, 12]; // Motif irrégulier
    for (i, &offset) in offsets.iter().enumerate() {
        program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, (200 + i) as u8)); // Valeurs différentes
        program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 9, 0, offset)); // STORE avec offset irrégulier
        program.add_instruction(Instruction::create_load_reg_offset(10, 0, offset));    // LOAD correspondant
    }

    // Test 5: Test de base cache de l'AGU avec réutilisation de registre de base
    println!("Test 5: AGU Base Cache - Réutilisation registre de base");
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 11, 0x20)); // R11 = nouvelle adresse de base
    for i in 0..3 {
        let offset = i * 8;
        program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 12, (50 + i) as u8));
        program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 12, 11, offset)); // Réutilise R11 comme base
        program.add_instruction(Instruction::create_load_reg_offset(13, 11, offset));
    }

    // Test 6: Validation finale - vérification que toutes les valeurs sont cohérentes
    println!("Test 6: AGU Validation - Vérification cohérence");
    program.add_instruction(Instruction::create_load_reg_offset(14, 0, 0));  // Re-charger première valeur
    program.add_instruction(Instruction::create_load_reg_offset(15, 0, 8));  // Re-charger deuxième valeur

    // Fin du programme
    program.add_instruction(Instruction::create_no_args(Opcode::Halt));

    // Configuration des segments
    let total_code_size: u32 = program.code.iter()
        .map(|instr| instr.total_size() as u32)
        .sum();
    program.segments = vec![SegmentMetadata::new(SegmentType::Code, 0, total_code_size, 0)];

    println!("\n=== RÉSULTATS ATTENDUS AGU ===");
    println!("- Messages 'AGU: Calculated address' dans les logs");
    println!("- Toutes les valeurs Store/Load doivent correspondre");
    println!("- R2 = 42 (première valeur)");
    println!("- R4 = 84 (deuxième valeur)");
    println!("- R14 = 42 (validation première valeur)");
    println!("- R15 = 84 (validation deuxième valeur)");
    println!("- Amélioration potentielle des statistiques de performance");

    program
}

/// Test parallèle AGU/ALU optimisé pour exécution superscalaire
fn create_agu_parallel_optimized_test() -> BytecodeFile {
    let mut program = BytecodeFile::new();
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    program.add_metadata("name", "AGU Parallel Execution Optimized Test");
    program.add_metadata("description", "Test optimisé pour maximiser l'exécution parallèle AGU/ALU");
    program.add_metadata("author", "PunkVM Team");

    println!("=== CRÉATION DU TEST PARALLÈLE AGU/ALU OPTIMISÉ ===");

    // Phase 1: Initialisation pour maximiser le parallélisme
    println!("Phase 1: Initialisation pour dual-issue optimal");
    
    // Initialiser plusieurs registres de base pour l'AGU
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 10, 0x10)); // R10 = Base array
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 11, 0x40)); // R11 = Base matrix
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 12, 0x80)); // R12 = Base buffer
    
    // Initialiser registres pour calculs ALU
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 10));   // R0 = 10
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 20));   // R1 = 20
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 2, 30));   // R2 = 30
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 3, 1));    // R3 = 1 (increment)

    // Phase 2: Pattern optimal pour dual-issue (AGU/ALU entrelacés)
    println!("Phase 2: Pattern dual-issue AGU/ALU entrelacé");
    
    // Pattern répété 20 fois pour observer le parallélisme
    for i in 0..20 {
        // Groupe 1: Load (AGU) + Add (ALU) - peuvent s'exécuter en parallèle
        program.add_instruction(Instruction::create_load_reg_offset(4, 10, (i * 8) as i8));     // AGU: LOAD R4, [R10+i*8]
        program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 0, 0, 1));        // ALU: R0 = R0 + R1
        
        // Groupe 2: Store (AGU) + Mul (ALU) - peuvent s'exécuter en parallèle
        program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 0, 11, (i * 4) as i8)); // AGU: STORE [R11+i*4], R0
        program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Mul, 1, 1, 2));        // ALU: R1 = R1 * R2
        
        // Groupe 3: Load (AGU) + Sub (ALU) - peuvent s'exécuter en parallèle
        program.add_instruction(Instruction::create_load_reg_offset(5, 12, (i * 4) as i8));     // AGU: LOAD R5, [R12+i*4]
        program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Sub, 2, 2, 3));        // ALU: R2 = R2 - R3
        
        // Groupe 4: Store (AGU) + And (ALU) - peuvent s'exécuter en parallèle
        program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 5, 10, ((i+1) * 8) as i8)); // AGU: STORE
        program.add_instruction(Instruction::create_reg_reg_reg(Opcode::And, 6, 4, 5));        // ALU: R6 = R4 & R5
    }

    // Phase 3: Pattern pour tester le stride predictor avec parallélisme
    println!("Phase 3: Stride pattern avec calculs parallèles");
    
    // Accès avec stride constant + calculs indépendants
    for i in 0..10 {
        let stride = 16; // Stride constant
        let offset = (i * stride) as i8;
        
        // Ces deux instructions peuvent s'exécuter en parallèle
        program.add_instruction(Instruction::create_load_reg_offset(7, 10, offset));           // AGU: Load avec stride
        program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Or, 8, 0, 1));         // ALU: Opération logique
        
        // Ces deux aussi
        program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 8, 11, offset)); // AGU: Store avec stride
        program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Xor, 9, 7, 8));        // ALU: XOR
    }

    // Phase 4: Test du base cache avec opérations parallèles
    println!("Phase 4: Base cache test avec parallélisme maximal");
    
    // Réutiliser les mêmes bases pour tester le cache
    for cycle in 0..3 {
        for i in 0..5 {
            // Toujours même base (R10) pour maximiser cache hits
            program.add_instruction(Instruction::create_load_reg_offset(13, 10, (i * 4) as i8));    // AGU: Base cache hit
            program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Inc, 13, 0, 0));       // ALU: Inc en parallèle
            
            program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 13, 10, ((i+1) * 4) as i8)); // AGU
            program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Dec, 14, 0, 0));       // ALU: Dec en parallèle
        }
    }

    // Phase 5: Validation finale
    println!("Phase 5: Validation des résultats");
    
    // Charger quelques valeurs pour vérification
    program.add_instruction(Instruction::create_load_reg_offset(15, 10, 0));    // Première valeur
    program.add_instruction(Instruction::create_load_reg_offset(14, 11, 0));    // Deuxième valeur
    
    // Marqueur de fin
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 0xFF));
    program.add_instruction(Instruction::create_no_args(Opcode::Halt));
    
    // Configuration des segments
    let total_code_size: u32 = program.code.iter()
        .map(|instr| instr.total_size() as u32)
        .sum();
    program.segments = vec![SegmentMetadata::new(SegmentType::Code, 0, total_code_size, 0)];
    
    println!("\n=== RÉSULTATS ATTENDUS ===");
    println!("- Exécution parallèle AGU/ALU détectée");
    println!("- Messages 'PARALLEL EXECUTION: 2 instructions exécutées en parallèle!'");
    println!("- Dual-issue ratio > 40%");
    println!("- Stride prediction hits > 70%");
    println!("- Base cache hits > 50%");
    println!("- IPC > 1.2 (objectif 1.5)");
    println!("- Parallel executions > 50% des instructions totales");
    
    program
}

/// Test mémoire intensif pour maximiser l'utilisation de l'AGU
fn memory_intensive_stress_test() -> BytecodeFile {
    let mut program = BytecodeFile::new();
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    program.add_metadata("name", "Memory Intensive AGU Stress Test");
    program.add_metadata("description", "Test intensif pour maximiser l'utilisation de l'AGU");
    program.add_metadata("author", "PunkVM Team");

    println!("=== CRÉATION DU TEST MÉMOIRE INTENSIF AGU ===");

    // Phase 1: Initialisation des bases d'adresses multiples
    println!("Phase 1: Setup adresses de base multiples");
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 0x40)); // Base 1: Array data
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 0x80)); // Base 2: Matrix data
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 2, 0xC0)); // Base 3: Buffer data
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 3, 0x10)); // Base 4: Temp data

    // Phase 2: Séquences d'accès mémoire séquentiels (pour stride predictor)
    println!("Phase 2: Accès séquentiels - Test Stride Predictor");
    for i in 0..8 {
        let offset = (i * 8) as i8; // Stride constant de 8 bytes
        let value = 100 + i;
        program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 4, value as u8));
        program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 4, 0, offset)); // STORE [R0+offset], R4
        program.add_instruction(Instruction::create_load_reg_offset(5, 0, offset));               // LOAD R5, [R0+offset]
        program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 5, 5, 4));           // Use loaded value
    }

    // Phase 3: Accès avec motifs réguliers différents (stride predictor training)
    println!("Phase 3: Motifs stride multiples");
    // Motif 1: stride 4
    for i in 0..6 {
        let offset = (i * 4) as i8;
        let value = 50 + i; // Changé de 200 à 50 pour rester dans les limites
        program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 6, value as u8));
        program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 6, 1, offset)); // Base R1
        program.add_instruction(Instruction::create_load_reg_offset(7, 1, offset));
    }
    
    // Motif 2: stride 16 (pour exercer différents patterns)
    for i in 0..4 {
        let offset = (i * 16) as i8;
        let value = 60 + i; // Changé de 250 à 60 pour rester dans les limites
        program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 8, value as u8));
        program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 8, 2, offset)); // Base R2
        program.add_instruction(Instruction::create_load_reg_offset(9, 2, offset));
    }

    // Phase 4: Réutilisation intensive des adresses de base (base cache test)
    println!("Phase 4: Test Base Address Cache - Réutilisation intensive");
    for round in 0..3 {
        // Utilisation de chaque base plusieurs fois
        for base_reg in 0..4_u8 {
            for access in 0..4 {
                let offset = (access * 4) as i8;
                let value = round * 20 + base_reg as usize * 5 + access;
                program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 10, value as u8));
                program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 10, base_reg, offset));
                program.add_instruction(Instruction::create_load_reg_offset(11, base_reg, offset));
            }
        }
    }

    // Phase 5: Accès mémoire avec dépendances pour tester forwarding + AGU
    println!("Phase 5: AGU + Forwarding interaction");
    for i in 0..5 {
        let offset1 = i * 8;
        let offset2 = offset1 + 4;
        
        // Store suivi immédiatement d'un Load (test store-load forwarding avec AGU)
        program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 12, (50 + i) as u8));
        program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 12, 3, offset1)); // STORE
        program.add_instruction(Instruction::create_load_reg_offset(13, 3, offset1));               // LOAD immédiat
        
        // Calcul sur la valeur chargée puis store adjacent
        program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 14, 13, 12));
        program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 14, 3, offset2)); // STORE adjacent
    }

    // Phase 6: Accès aléatoires pour tester la robustesse AGU
    println!("Phase 6: Accès non-prédictibles");
    let random_offsets = [24, 8, 56, 16, 40, 0, 32, 48]; // Pattern imprévisible
    for (i, &offset) in random_offsets.iter().enumerate() {
        let value = 75 + i;
        program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 15, value as u8));
        program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 15, 0, offset));
        program.add_instruction(Instruction::create_load_reg_offset(15, 0, offset));
    }

    // Phase 7: Test de saturation - maximiser l'activité AGU
    println!("Phase 7: Saturation AGU - Maximum throughput");
    for burst in 0..4 {
        for i in 0..6 {
            let offset = burst * 24 + i * 4;
            let value = burst * 10 + i;
            program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 4, value as u8));
            program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 4, 1, offset));
            program.add_instruction(Instruction::create_load_reg_offset(5, 1, offset));
            program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 5, 2, offset));
            program.add_instruction(Instruction::create_load_reg_offset(6, 2, offset));
        }
    }

    // Fin du programme
    program.add_instruction(Instruction::create_no_args(Opcode::Halt));

    // Configuration des segments
    let total_code_size: u32 = program.code.iter()
        .map(|instr| instr.total_size() as u32)
        .sum();
    program.segments = vec![SegmentMetadata::new(SegmentType::Code, 0, total_code_size, 0)];

    println!("\n=== CARACTÉRISTIQUES DU TEST ===");
    println!("- ~200+ opérations mémoire Load/Store");
    println!("- 4 bases d'adresses différentes");
    println!("- Motifs stride: 8, 4, 16 bytes");
    println!("- Réutilisation intensive base cache");
    println!("- Interaction AGU + Store-Load forwarding");
    println!("- Accès aléatoires pour robustesse");
    println!("- Phases de saturation pour throughput max");

    println!("\n=== OBJECTIFS DE PERFORMANCE ===");
    println!("- Utilisation AGU: 80%+ des cycles");
    println!("- Stride accuracy: 70%+ pour motifs réguliers");
    println!("- Base cache hit rate: 60%+");
    println!("- IPC target: 0.9+ (vs ~0.8 sans AGU)");

    program
}

// Test parallel_execution_test() temporairement supprimé à cause d'erreurs d'API
// TODO: Réimplémenter quand l'API des instructions sera clarifiée



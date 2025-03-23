// examples/branch_debug.rs

use std::time::Instant;
use std::io::{self, Write};
use crate::bytecode::files::{BytecodeVersion, SegmentMetadata, SegmentType};
use crate::bytecode::format::{ArgType, InstructionFormat};
use crate::bytecode::instructions::Instruction;
use crate::bytecode::opcodes::Opcode;
use crate::BytecodeFile;
use crate::debug::TracerConfig;
use crate::pvm::vm::PunkVM;
use crate::pvm::vm_errors::VMResult;

use crate::bytecode::instructions::ArgValue;

/// Programme de débogage spécifique pour les branchements dans PunkVM
/// Il combine les tests de branchement avec un traçage complet du pipeline
fn main() -> VMResult<()> {
    println!("=== PunkVM Branch Debugging Tool ===\n");

    // Configurer le type de test à exécuter
    println!("Sélectionner le type de test:");
    println!("1. Saut inconditionnel simple (vers l'avant)");
    println!("2. Saut conditionnel simple (pris)");
    println!("3. Saut conditionnel simple (non pris)");
    println!("4. Boucle simple (saut arrière)");
    println!("5. Test avec mesure des sauts précise");

    let mut input = String::new();
    print!("Choix (1-5): ");
    io::stdout().flush()?;
    std::io::stdin().read_line(&mut input)?;

    let test_type = input.trim().parse::<usize>().unwrap_or(1);

    // Créer et configurer la VM avec traçage détaillé
    let mut vm = PunkVM::new();

    let tracer_config = TracerConfig {
        enabled: true,
        log_to_console: true,
        log_to_file: true,
        log_file_path: Some(format!("branch_test_{}.log", test_type)),
        trace_fetch: true,
        trace_decode: true,
        trace_execute: true,
        trace_memory: true,
        trace_writeback: true,
        trace_hazards: true,
        trace_branches: true,
        trace_registers: true,
    };
    vm.enable_tracing(tracer_config);

    // Créer le programme de test approprié
    let bytecode = match test_type {
        1 => create_forward_jump_test(),
        2 => create_conditional_jump_taken_test(),
        3 => create_conditional_jump_not_taken_test(),
        4 => create_simple_loop_test(),
        5 => create_precise_branch_measurement_test(),
        _ => create_forward_jump_test(),
    };

    // Sauvegarder une copie du bytecode pour référence
    let filename = format!("branch_test_{}.punk", test_type);
    bytecode.write_to_file(&filename)?;
    println!("Bytecode écrit dans {}", filename);

    // Générer un fichier de désassemblage du programme
    let disassembly = disassemble_bytecode(&bytecode);
    let disasm_filename = format!("branch_test_{}.asm", test_type);
    std::fs::write(&disasm_filename, disassembly)?;
    println!("Désassemblage écrit dans {}", disasm_filename);

    // Charger et exécuter le programme avec instrumentation détaillée
    vm.load_program_from_bytecode(bytecode)?;

    // Exécuter avec mesure de temps et affichage de l'état détaillé
    println!("\nExécution du test {}...", test_type);
    let start = Instant::now();

    let result = vm.run();

    let duration = start.elapsed();

    match result {
        Ok(_) => {
            println!("\n✅ Exécution réussie en {:?}", duration);

            // Afficher l'état des registres
            println!("\nRegistres après exécution:");
            for (i, value) in vm.registers.iter().enumerate().take(8) {
                println!("  R{}: {}", i, value);
            }

            // Afficher les statistiques
            let stats = vm.stats();
            println!("\nStatistiques d'exécution:");
            println!("  Cycles: {}", stats.cycles);
            println!("  Instructions: {}", stats.instructions_executed);
            println!("  IPC: {:.2}", stats.ipc);
            println!("  Stalls: {}", stats.stalls);
            println!("  Hazards: {}", stats.hazards);
            println!("  Forwards: {}", stats.forwards);

            // // Exporter les traces dans un fichier CSV pour analyse
            // let csv_filename = format!("branch_test_{}_trace.csv", test_type);
            // vm.export_trace_to_csv(&csv_filename)?;
            // println!("\nTraces exportées dans {}", csv_filename);
        },
        Err(e) => {
            println!("\n❌ Erreur d'exécution: {}", e);
        }
    }

    Ok(())
}

/// Désassemble un fichier bytecode en texte lisible
fn disassemble_bytecode(bytecode: &BytecodeFile) -> String {
    let mut output = String::new();
    output.push_str("=== PunkVM Bytecode Disassembly ===\n\n");

    // En-tête avec métadonnées
    output.push_str(&format!("Version: {}.{}.{}.{}\n",
                             bytecode.version.major, bytecode.version.minor,
                             bytecode.version.patch, bytecode.version.build));

    output.push_str("\nMetadata:\n");
    for (key, value) in &bytecode.metadata {
        output.push_str(&format!("  {}: {}\n", key, value));
    }

    output.push_str("\n=== Code Section ===\n");

    // Désassembler les instructions
    let mut addr = 0;
    for (i, instr) in bytecode.code.iter().enumerate() {
        let instr_size = instr.total_size();

        // Format de base de l'instruction
        let mut instr_str = format!("{:04X}: [{:02X}] {:?}", addr, instr.opcode as u8, instr.opcode);

        // Traitement spécial pour les branchements
        if instr.opcode.is_branch() {
            match instr.get_arg2_value() {
                Ok(ArgValue::RelativeAddr(offset)) => {
                    let target = (addr as i32 + offset) as u32;
                    instr_str.push_str(&format!(" {:+} (-> 0x{:04X})", offset, target));
                },
                Ok(ArgValue::AbsoluteAddr(target)) => {
                    instr_str.push_str(&format!(" -> 0x{:04X}", target));
                },
                _ => {
                    instr_str.push_str(" [format d'adresse invalide]");
                }
            }
        } else {
            // Formater les autres types d'instructions
            match (instr.get_arg1_value(), instr.get_arg2_value()) {
                (Ok(arg1), Ok(arg2)) => {
                    instr_str.push_str(&format!(" {:?}, {:?}", arg1, arg2));
                },
                (Ok(arg1), _) => {
                    instr_str.push_str(&format!(" {:?}", arg1));
                },
                _ => {}
            }
        }

        // Ajouter la taille de l'instruction et le code hexadécimal
        let mut hex_bytes = String::new();
        let encoded = instr.encode();
        for byte in &encoded {
            hex_bytes.push_str(&format!("{:02X} ", byte));
        }

        output.push_str(&format!("{:<50} ; size={:2}, bytes=[{}]\n",
                                 instr_str, instr_size, hex_bytes.trim_end()));

        addr += instr_size as u32;
    }

    output
}

/// Test 1: Saut inconditionnel simple (vers l'avant)
fn create_forward_jump_test() -> BytecodeFile {
    let mut bytecode = BytecodeFile::new();
    bytecode.version = BytecodeVersion::new(0, 1, 0, 0);
    bytecode.add_metadata("name", "Forward Jump Test");

    // LOAD R0, 1     ; R0 = 1
    bytecode.add_instruction(Instruction::create_reg_imm8(Opcode::Load, 0, 1));

    // LOAD R1, 2     ; R1 = 2
    bytecode.add_instruction(Instruction::create_reg_imm8(Opcode::Load, 1, 2));

    // JMP +3         ; Sauter à l'instruction 5 (attention: taille variable!)
    // Utilisons une adresse calculée précisément plutôt qu'une approximation
    // On doit sauter par-dessus deux instructions LOAD (chacune fait 4 bytes)
    let jump_offset: i32 = 8; // 2 instructions * 4 bytes par instruction
    let jmp = Instruction::new(
        Opcode::Jmp,
        InstructionFormat::new(ArgType::None, ArgType::RelativeAddr,ArgType::None),
        jump_offset.to_le_bytes()[0..4].to_vec()
    );
    bytecode.add_instruction(jmp);

    // LOAD R2, 3     ; R2 = 3 (devrait être sauté)
    bytecode.add_instruction(Instruction::create_reg_imm8(Opcode::Load, 2, 3));

    // LOAD R3, 4     ; R3 = 4 (devrait être sauté)
    bytecode.add_instruction(Instruction::create_reg_imm8(Opcode::Load, 3, 4));

    // LOAD R4, 5     ; R4 = 5 (destination du saut)
    bytecode.add_instruction(Instruction::create_reg_imm8(Opcode::Load, 4, 5));

    // HALT           ; Fin du programme
    bytecode.add_instruction(Instruction::create_no_args(Opcode::Halt));

    bytecode
}

/// Test 2: Saut conditionnel simple (pris)
fn create_conditional_jump_taken_test() -> BytecodeFile {
    let mut bytecode = BytecodeFile::new();
    bytecode.version = BytecodeVersion::new(0, 1, 0, 0);
    bytecode.add_metadata("name", "Conditional Jump Taken Test");

    // LOAD R0, 10    ; R0 = 10
    bytecode.add_instruction(Instruction::create_reg_imm8(Opcode::Load, 0, 10));

    // LOAD R1, 10    ; R1 = 10
    bytecode.add_instruction(Instruction::create_reg_imm8(Opcode::Load, 1, 10));

    // CMP R0, R1     ; Comparer R0 et R1 (égaux)
    bytecode.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 1));

    // JmpIf eq, +3   ; Sauter si égal (devrait être pris)
    // Utiliser une adresse absolue au lieu d'un offset pour éviter les imprécisions
    let jump_offset: i32 = 8; // 2 instructions * 4 bytes par instruction
    let jmpif = Instruction::new(
        Opcode::JmpIf,
        InstructionFormat::new(ArgType::None, ArgType::RelativeAddr,ArgType::None),
        jump_offset.to_le_bytes()[0..4].to_vec()
    );
    bytecode.add_instruction(jmpif);

    // LOAD R2, 20    ; R2 = 20 (devrait être sauté)
    bytecode.add_instruction(Instruction::create_reg_imm8(Opcode::Load, 2, 20));

    // LOAD R3, 30    ; R3 = 30 (devrait être sauté)
    bytecode.add_instruction(Instruction::create_reg_imm8(Opcode::Load, 3, 30));

    // LOAD R4, 40    ; R4 = 40 (destination du saut)
    bytecode.add_instruction(Instruction::create_reg_imm8(Opcode::Load, 4, 40));

    // HALT           ; Fin du programme
    bytecode.add_instruction(Instruction::create_no_args(Opcode::Halt));

    bytecode
}

/// Test 3: Saut conditionnel simple (non pris)
fn create_conditional_jump_not_taken_test() -> BytecodeFile {
    let mut bytecode = BytecodeFile::new();
    bytecode.version = BytecodeVersion::new(0, 1, 0, 0);
    bytecode.add_metadata("name", "Conditional Jump Not Taken Test");

    // LOAD R0, 10    ; R0 = 10
    bytecode.add_instruction(Instruction::create_reg_imm8(Opcode::Load, 0, 10));

    // LOAD R1, 20    ; R1 = 20
    bytecode.add_instruction(Instruction::create_reg_imm8(Opcode::Load, 1, 20));

    // CMP R0, R1     ; Comparer R0 et R1 (pas égaux)
    bytecode.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 1));

    // JmpIf eq, +3   ; Sauter si égal (ne devrait pas être pris)
    let jump_offset: i32 = 8; // 2 instructions * 4 bytes par instruction
    let jmpif = Instruction::new(
        Opcode::JmpIf,
        InstructionFormat::new(ArgType::None, ArgType::RelativeAddr,ArgType::None),
        jump_offset.to_le_bytes()[0..4].to_vec()
    );
    bytecode.add_instruction(jmpif);

    // LOAD R2, 30    ; R2 = 30 (devrait être exécuté)
    bytecode.add_instruction(Instruction::create_reg_imm8(Opcode::Load, 2, 30));

    // LOAD R3, 40    ; R3 = 40 (devrait être exécuté)
    bytecode.add_instruction(Instruction::create_reg_imm8(Opcode::Load, 3, 40));

    // LOAD R4, 50    ; R4 = 50
    bytecode.add_instruction(Instruction::create_reg_imm8(Opcode::Load, 4, 50));

    // HALT           ; Fin du programme
    bytecode.add_instruction(Instruction::create_no_args(Opcode::Halt));

    bytecode
}

/// Test 4: Boucle simple (saut arrière)
fn create_simple_loop_test() -> BytecodeFile {
    let mut bytecode = BytecodeFile::new();
    bytecode.version = BytecodeVersion::new(0, 1, 0, 0);
    bytecode.add_metadata("name", "Simple Loop Test");

    // LOAD R0, 0     ; R0 = 0 (compteur)
    let instr1 = Instruction::create_reg_imm8(Opcode::Load, 0, 0);
    bytecode.add_instruction(instr1.clone());
    let instr1_size = instr1.total_size() as u32;

    // LOAD R1, 5     ; R1 = 5 (limite)
    let instr2 = Instruction::create_reg_imm8(Opcode::Load, 1, 5);
    bytecode.add_instruction(instr2.clone());
    let instr2_size = instr2.total_size() as u32;

    // LOAD R2, 1     ; R2 = 1 (incrément)
    let instr3 = Instruction::create_reg_imm8(Opcode::Load, 2, 1);
    bytecode.add_instruction(instr3.clone());
    let instr3_size = instr3.total_size() as u32;

    // LOAD R3, 0     ; R3 = 0 (somme)
    let instr4 = Instruction::create_reg_imm8(Opcode::Load, 3, 0);
    bytecode.add_instruction(instr4.clone());
    let instr4_size = instr4.total_size() as u32;

    // On est maintenant à l'adresse instr1_size + instr2_size + instr3_size + instr4_size
    let loop_start_addr = instr1_size + instr2_size + instr3_size + instr4_size;

    // Début de la boucle
    // ADD R3, R3, R0 ; Ajouter compteur à la somme: R3 += R0
    let instr5 = Instruction::create_reg_reg(Opcode::Add, 3, 0);
    bytecode.add_instruction(instr5.clone());
    let instr5_size = instr5.total_size() as u32;

    // ADD R0, R0, R2 ; Incrémenter compteur: R0 += R2 (R0 += 1)
    let instr6 = Instruction::create_reg_reg(Opcode::Add, 0, 2);
    bytecode.add_instruction(instr6.clone());
    let instr6_size = instr6.total_size() as u32;

    // CMP R0, R1     ; Comparer compteur avec limite
    let instr7 = Instruction::create_reg_reg(Opcode::Cmp, 0, 1);
    bytecode.add_instruction(instr7.clone());
    let instr7_size = instr7.total_size() as u32;

    // La taille totale de la boucle est la somme des tailles des instructions dans la boucle
    let loop_size = instr5_size + instr6_size + instr7_size + 5; // +5 pour l'instruction JmpIfNot

    // JmpIfNot eq, -loop_size ; Sauter si pas égal (retour au début de la boucle)
    let jump_offset = -(loop_size as i32);
    let jmpifnot = Instruction::new(
        Opcode::JmpIfNot,
        InstructionFormat::new(ArgType::None, ArgType::RelativeAddr,ArgType::None),
        jump_offset.to_le_bytes()[0..4].to_vec()
    );
    bytecode.add_instruction(jmpifnot);

    // HALT           ; Fin du programme
    bytecode.add_instruction(Instruction::create_no_args(Opcode::Halt));

    bytecode
}

/// Test 5: Test avec mesure des sauts précise
fn create_precise_branch_measurement_test() -> BytecodeFile {
    let mut bytecode = BytecodeFile::new();
    bytecode.version = BytecodeVersion::new(0, 1, 0, 0);
    bytecode.add_metadata("name", "Precise Branch Measurement Test");

    // Créer un programme qui mesure précisément les tailles d'instructions
    // et utilise ces mesures pour les sauts

    // Instruction avec opcode et taille connue (sans symboles)
    let mut instructions = Vec::new();

    // LOAD R0, 0     ; Compteur d'instructions
    instructions.push(Instruction::create_reg_imm8(Opcode::Load, 0, 0));

    // LOAD R1, 10    ; Valeur maximale
    instructions.push(Instruction::create_reg_imm8(Opcode::Load, 1, 10));

    // Générer les adresses des instructions
    let mut addresses = Vec::new();
    let mut current_addr = 0;

    for instr in &instructions {
        addresses.push(current_addr);
        current_addr += instr.total_size() as u32;
    }

    // Maintenant, calculer les offsets de branchement précisément
    // Nous allons créer une boucle qui incrémente R0 jusqu'à R1

    // Ajouter les instructions au bytecode
    for instr in instructions {
        bytecode.add_instruction(instr);
    }

    // Début de la boucle
    let loop_start_addr = current_addr;

    // ADD R0, R0, 1  ; Incrémenter compteur
    let instr_add = Instruction::create_reg_imm8(Opcode::Add, 0, 1);
    bytecode.add_instruction(instr_add.clone());
    current_addr += instr_add.total_size() as u32;

    // CMP R0, R1     ; Comparer compteur avec limite
    let instr_cmp = Instruction::create_reg_reg(Opcode::Cmp, 0, 1);
    bytecode.add_instruction(instr_cmp.clone());
    current_addr += instr_cmp.total_size() as u32;

    // JmpIf eq, +X   ; Sauter à HALT si égal
    // Calculer l'offset en fonction de la taille de l'instruction JmpIf et JmpIfNot
    let jmpif_size = 5; // Approximation - à ajuster selon le format réel
    let jmpifnot_size = 5; // Approximation - à ajuster selon le format réel

    let forward_offset: i32 = jmpifnot_size; // Sauter par-dessus l'instruction JmpIfNot suivante
    let jmpif = Instruction::new(
        Opcode::JmpIf,
        InstructionFormat::new(ArgType::None, ArgType::RelativeAddr,ArgType::None),
        forward_offset.to_le_bytes()[0..4].to_vec()
    );
    bytecode.add_instruction(jmpif);
    current_addr += jmpif_size;

    // JmpIfNot eq, -X ; Boucler si pas égal (retour à loop_start)
    let backward_offset = -((current_addr - loop_start_addr) as i32);
    let jmpifnot = Instruction::new(
        Opcode::JmpIfNot,
        InstructionFormat::new(ArgType::None, ArgType::RelativeAddr,ArgType::None),
        backward_offset.to_le_bytes()[0..4].to_vec()
    );
    bytecode.add_instruction(jmpifnot);

    // HALT           ; Fin du programme
    bytecode.add_instruction(Instruction::create_no_args(Opcode::Halt));

    let total_size: u32 = bytecode.code.iter()
        .map(|instr| instr.total_size() as u32)
        .sum();

    // Créer le segment de code
    bytecode.segments = vec![
        SegmentMetadata::new(SegmentType::Code, 0, total_size, 0)
    ];

    // Créer un segment de données
    let data_size = 256; // Allouer 256 bytes pour les données
    let data_segment = SegmentMetadata::new(SegmentType::Data, 0, data_size, 0x1000);
    bytecode.segments.push(data_segment);
    bytecode.data = vec![0; data_size as usize];

    println!("Programme simplifié créé avec {} instructions", bytecode.code.len());

    bytecode
}
//src/main.rs
// Utilisez "crate::" au lieu de "PunkVM::" pour les imports internes
// extern crate PunkVM;

use std::time::Instant;
use PunkVM::alu::alu::BranchCondition;
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
    // let program = create_complex_program();
    // let program = create_simple_complex_program();
    // let program = create_cmp_loop_program();
    // let program = create_pipeline_test_program();
    // let program = create_reg_reg_reg_test_program();
    let program = create_hazard_detection_test_program();




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
    // let offset_to_start = -(calculate_instruction_range_size(&program.code, loop_start_idx, current_idx) as i32);
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
    instructions[start..end].iter().map(|instr| instr.total_size()).sum()
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

fn create_branch_jump(offset: i32) -> Instruction {
    let fmt = InstructionFormat::new(ArgType::None, ArgType::RelativeAddr, ArgType::None);
    // Encodage de l'offset sur 4 octets en little-endian.
    // let offset_bytes = offset.to_le_bytes().to_vec();
    // Instruction::new(Opcode::JmpIfNot, fmt, offset_bytes)

    println!("Création d'un saut conditionnel avec offset = {}", offset);
    Instruction::create_reg_imm8(Opcode::JmpIfNot, 0, offset as u8)
}

/// Crée un programme qui teste CMP et un saut conditionnel dans une boucle.
/// La boucle effectue :
///   MOV R0, #0          ; initialisation du compteur
///   MOV R1, #5          ; limite
///   MOV R2, #1          ; incrément
/// LOOP_START:
///   ADD R0, R0, R2      ; R0 = R0 + 1
///   CMP R0, R1          ; compare R0 et R1
///   JmpIfNot <offset>   ; si R0 != R1, retour à LOOP_START
///   HALT                ; sinon, fin du programme
pub fn create_cmp_loop_program() -> BytecodeFile {
    let mut program = BytecodeFile::new();
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    program.add_metadata("name", "CMP Loop Program");
    program.add_metadata("description", "Boucle contrôlée par CMP et branchement conditionnel");

    // Initialisation avec MOV
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 0)); // R0 = 0
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 5)); // R1 = 5
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 2, 1)); // R2 = 1

    // Marquer le début de la boucle : on calcule le PC de début de boucle
    let loop_start_idx = program.code.len();
    let loop_start_pc = calculate_cumulative_pc(&program.code, loop_start_idx);

    // Instruction 4 : ADD R0, R0, R2 (format reg_reg_reg)
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 0, 0, 2));

    // Instruction 5 : CMP R0, R1 (format reg_reg)
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 0, 1));

    // Calculer le PC de l'instruction de saut (avant insertion du saut)
    let jump_idx = program.code.len();
    let jump_pc = calculate_cumulative_pc(&program.code, jump_idx);

    // L'offset relatif (en octets) nécessaire pour revenir au début de la boucle :
    // offset = loop_start_pc - jump_pc.
    let offset_i32 = (loop_start_pc as i32) - (jump_pc as i32);
    // Vérifier que l'offset tient dans i8 (si besoin, adapter le format ou les tests)
    let offset: i8 = offset_i32 as i8;
    println!(
        "Loop start idx: {}, loop_start_pc: {}, jump_idx: {}, jump_pc: {}, offset: {}",
        loop_start_idx, loop_start_pc, jump_idx, jump_pc, offset
    );

    // Instruction 6 : JmpIfNot avec l'offset calculé
    let jump_inst = create_branch_jump(offset as i32);
    program.add_instruction(jump_inst);

    // Instruction 7 : HALT
    program.add_instruction(Instruction::create_no_args(Opcode::Halt));

    // Calculer la taille totale du code et créer le segment de code
    let total_size: u32 = program.code.iter().map(|instr| instr.total_size() as u32).sum();
    program.segments = vec![SegmentMetadata::new(SegmentType::Code, 0, total_size, 0)];

    program
}




/// Crée un programme de test complet pour évaluer les performances du pipeline
pub fn create_pipeline_test_program() -> BytecodeFile {
    let mut program = BytecodeFile::new();
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    program.add_metadata("name", "Pipeline Performance Test");
    program.add_metadata("description", "Test du pipeline, forwarding, hazards et stalls");

    // ---------- Test 1: Data Dependencies (RAW Hazards) ----------
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 5));   // R0 = 5
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 10));  // R1 = 10
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
    program.add_instruction(Instruction::create_reg_reg_offset(Opcode::Load, 7, 6, 0));  // R7 = Mem[R6+0]
    // program.add_instruction(Instruction::create_reg_reg(Opcode::Add, 8, 7));             // R8 = R7 - Load-Use hazard
    //
    // // ---------- Test 3: Structural Hazard ----------
    // // Two memory operations in sequence - potential structural hazard
    program.add_instruction(Instruction::create_reg_reg_offset(Opcode::Store, 1, 6, 4)); // Store R1 at [R6+4]
    program.add_instruction(Instruction::create_reg_reg_offset(Opcode::Load, 9, 6, 4));  // R9 = Mem[R6+4]
    //
    // // ---------- Test 4: Store-Load forwarding ----------
    // // Store followed by Load from same address - should be forwarded from store buffer
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 10, 42));          // R10 = 42
    program.add_instruction(Instruction::create_reg_reg_offset(Opcode::Store, 10, 6, 8)); // Store R10 at [R6+8]
    program.add_instruction(Instruction::create_reg_reg_offset(Opcode::Load, 11, 6, 8));  // R11 = Mem[R6+8] - Should be forwarded

    // ---------- Test 5: Branch prediction ----------
    // // Simple loop to test branch prediction (if implemented)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 12, 0));   // R12 = 0 (counter)
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 13, 3));   // R13 = 3 (max iterations)
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
    let code_size_to_loop_start = calculate_instruction_range_size(&program.code, loop_start_idx, current_idx);
    let loop_offset = -(code_size_to_loop_start as i8);

    // Jump if not equal (R12 != R13)
    // let jump_instruction = create_conditional_jump(loop_offset); // ici  on a  Erreur lors de l'exécution: ExecutionError:
    // program.add_instruction(jump_instruction);   Erreur pipeline: Format d'adresse de saut conditionnel invalide

    // ---------- Final Verification ----------
    // Store results to verify correct execution
    program.add_instruction(Instruction::create_reg_reg_offset(Opcode::Store, 5, 6, 12));  // Store R5 at [R6+12]
    program.add_instruction(Instruction::create_reg_reg_offset(Opcode::Store, 11, 6, 16)); // Store R11 at [R6+16]
    program.add_instruction(Instruction::create_reg_reg_offset(Opcode::Store, 12, 6, 20)); // Store R12 at [R6+20]

    // End program
    program.add_instruction(Instruction::create_no_args(Opcode::Halt));

    // Configure memory segments
    let total_code_size: u32 = program.code.iter()
        .map(|instr| instr.total_size() as u32)
        .sum();

    let data_size = 512; // 512 bytes for data

    program.segments = vec![
        SegmentMetadata::new(SegmentType::Code, 0, total_code_size, 0),
        SegmentMetadata::new(SegmentType::Data, 0, data_size, 0x1000)
    ];

    program.data = vec![0; data_size as usize];

    println!("Programme de test du pipeline créé avec {} instructions", program.code.len());

    program
}

/// Fonction utilitaire: calcule la taille totale des instructions dans une plage
fn calculate_instruction_range_size(instructions: &[Instruction], start: usize, end: usize) -> usize {
    instructions[start..end].iter().map(|instr| instr.total_size()).sum()
}

/// Fonction utilitaire: crée une instruction Store avec offset
fn create_reg_reg_offset(opcode: Opcode, rs: u8, rb: u8, offset: i8) -> Instruction {
    // Cette implémentation dépend de votre format d'instruction
    // Supposons que le format soit (reg_dest, reg_base + offset)
    Instruction::new(
        opcode,
        InstructionFormat::new(ArgType::Register, ArgType::RegisterOffset, ArgType::None),
        vec![rs, rb, offset as u8]
    )
}



pub fn create_reg_reg_reg_test_program() -> BytecodeFile {
    let mut program = BytecodeFile::new();
    // Version du programme
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    // Métadonnées (optionnel)
    program.add_metadata("name", "Test reg_reg_reg");
    program.add_metadata("description", "Programme testant les instructions à trois registres.");

    // Initialiser R0 et R1 avec des valeurs immédiates via MOV (instructions immédiates)
    // Ici, on utilise create_reg_imm8 (qui utilise un format MOV avec immediate) pour initialiser les registres
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 5));   // R0 = 5
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 10));  // R1 = 10

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
    let total_size: u32 = program.code.iter().map(|instr| instr.total_size() as u32).sum();
    program.segments = vec![SegmentMetadata::new(SegmentType::Code, 0, total_size, 0)];

    // (Optionnel) Créer un segment de données si nécessaire
    let data_size = 256;
    let data_segment = SegmentMetadata::new(SegmentType::Data, 0, data_size, 0x1000);
    program.segments.push(data_segment);
    program.data = vec![0; data_size as usize];

    program
}

pub fn create_hazard_detection_test_program() -> BytecodeFile {
    let mut program = BytecodeFile::new();
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    program.add_metadata("name", "Hazard Detection Test");
    program.add_metadata("description", "Programme testant la détection des hazards et stalls.");

    // -------------------------------
    // Test 1: Load-Use Hazard
    // Ce type de hazard se produit quand on essaie d'utiliser le résultat
    // d'un LOAD avant qu'il ne soit disponible
    // -------------------------------

    // Initialiser une adresse mémoire
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 0, 0x10)); // R0 = adresse 0x10

    // Stocker une valeur à cette adresse
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, 42));  // R1 = 42

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

    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 4, 5));  // R4 = 5

    // Série d'instructions dépendantes (RAW hazards)
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 5, 4, 4)); // R5 = R4 + R4
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 6, 5, 5)); // R6 = R5 + R5 (dépend du résultat précédent)
    program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 7, 6, 6)); // R7 = R6 + R6 (dépend du résultat précédent)

    // -------------------------------
    // Test 3: Hazard de contrôle (branchement)
    // Test si un branchement cause un hazard et un flush
    // -------------------------------

    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 8, 1));  // R8 = 1
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 9, 1));  // R9 = 1

    // Compare R8 et R9
    program.add_instruction(Instruction::create_reg_reg(Opcode::Cmp, 8, 9));

    // Branchement conditionnel (JmpIf) - puisque create_jumpif_rel n'existe pas
    // Utilisons une autre approche - créer manuellement la structure
    let jmpif_format = InstructionFormat::new(ArgType::None, ArgType::RelativeAddr, ArgType::None);
    let offset_bytes = (14i32).to_le_bytes();
    let mut jmpif_args = Vec::new();
    jmpif_args.extend_from_slice(&offset_bytes);
    program.add_instruction(Instruction::new(Opcode::JmpIf, jmpif_format, jmpif_args));

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
    program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 14, 77));   // R14 = 77

    // Store suivi d'un Load à la même adresse
    program.add_instruction(Instruction::create_reg_reg_offset(Opcode::Store, 14, 13, 0)); // MEM[R13] = R14
    program.add_instruction(Instruction::create_load_reg_offset(15, 13, 0));  // R15 = MEM[R13] (hazard potentiel)

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
    let total_size: u32 = program.code.iter().map(|instr| instr.total_size() as u32).sum();
    program.segments = vec![SegmentMetadata::new(SegmentType::Code, 0, total_size, 0)];

    // Créer un segment de données
    let data_size = 256;
    let data_segment = SegmentMetadata::new(SegmentType::Data, 0, data_size, 0x1000);
    program.segments.push(data_segment);
    program.data = vec![0; data_size as usize];

    program
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
/////////////////////////////////////////////////////////////////////////////////////////////////////






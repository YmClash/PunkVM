// examples/simple_vm.rs


use PunkVM::bytecode::files::BytecodeVersion;
use PunkVM::bytecode::format::{ArgType, InstructionFormat};
use PunkVM::bytecode::instructions::Instruction;
use PunkVM::bytecode::opcodes::Opcode;
use PunkVM::BytecodeFile;
use PunkVM::pvm::vm::VMConfig;
// use crate::pvm::vm::PunkVM;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Création d'un programme simple qui calcule la somme des nombres de 1 à 10
    println!("Création du programme de test...");
    let bytecode = create_test_program()?;

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

    // Création de la VM
    println!("Initialisation de la VM...");
    // let mut vm = PunkVM::with_config(config);
    let mut vm = PunkVM::pvm::vm::PunkVM::with_config(config);

    // Chargement du programme
    println!("Chargement du programme...");
    vm.load_program_from_bytecode(bytecode)?;

    // Exécution du programme
    println!("Exécution du programme...");
    vm.run()?;

    // Affichage des résultats
    println!("Programme terminé !");
    println!("Résultat (R1): {}", vm.registers[1]);

    // Affichage des statistiques
    let stats = vm.stats();
    println!("\nStatistiques d'exécution:");
    println!("  Cycles: {}", stats.cycles);
    println!("  Instructions exécutées: {}", stats.instructions_executed);
    println!("  IPC (Instructions Par Cycle): {:.2}", stats.ipc);
    println!("  Stalls: {}", stats.stalls);
    println!("  Hazards: {}", stats.hazards);
    println!("  Forwards: {}", stats.forwards);
    println!("  Memory hits: {}", stats.memory_hits);
    println!("  Memory misses: {}", stats.memory_misses);

    Ok(())
}

/// Crée un programme de test qui calcule la somme des nombres de 1 à 10
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
}
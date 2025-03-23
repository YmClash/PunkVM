//
// use PunkVM::bytecode::{
//     BytecodeFile, BytecodeVersion, Instruction, InstructionFormat, Opcode, ArgType
// };
use crate::bytecode::files::BytecodeVersion;
use crate::bytecode::format::{ArgType, InstructionFormat};
use crate::bytecode::instructions::Instruction;
use crate::bytecode::opcodes::Opcode;
use crate::BytecodeFile;

fn main() -> std::io::Result<()> {
    // Création d'un nouveau fichier bytecode
    let mut bytecode = BytecodeFile::new();

    // Définition de la version
    bytecode.version = BytecodeVersion::new(0, 1, 0, 0);

    // Ajout de métadonnées
    bytecode.add_metadata("name", "Exemple PunkVM");
    bytecode.add_metadata("author", "PunkVM Team");
    bytecode.add_metadata("created", &chrono::Utc::now().to_rfc3339());

    // Création d'un petit programme qui calcule la somme des nombres de 1 à 10

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

    // JMP_IF_NOT 4   ; Si pas atteint limite, retourner au début de boucle
    let loop_jmp = Instruction::new(
        Opcode::JmpIfNot,
        InstructionFormat::new(ArgType::None, ArgType::RelativeAddr,ArgType::None),
        vec![0xFC, 0xFF, 0xFF, 0xFF] // -4 en complément à 2 (retour au début de la boucle)
    );
    bytecode.add_instruction(loop_jmp);

    // HALT           ; Fin du programme
    bytecode.add_instruction(Instruction::create_no_args(Opcode::Halt));

    // Ajout de données constantes pour démonstration
    let message = b"Resultat: ";
    let message_offset = bytecode.add_readonly_data(message);

    // Ajouter un symbole vers le message
    bytecode.add_symbol("message", message_offset);

    // Écriture du fichier bytecode sur disque
    bytecode.write_to_file("sum_1_to_10.punk")?;

    println!("Fichier bytecode créé avec succès: sum_1_to_10.punk");

    // Exemple de lecture
    let loaded_bytecode = BytecodeFile::read_from_file("sum_1_to_10.punk")?;

    println!("Version du bytecode: {}", loaded_bytecode.version.to_string());
    println!("Nombre d'instructions: {}", loaded_bytecode.code.len());
    println!("Données en lecture seule: {} bytes", loaded_bytecode.readonly_data.len());
    println!("Symboles définis:");

    for (name, address) in &loaded_bytecode.symbols {
        println!("  {} -> 0x{:08X}", name, address);
    }

    Ok(())
}

// Tests Intégrés pour PunkVM
// Ce fichier contient l'ensemble des tests unitaires et d'intégration
// pour valider les composants principaux de PunkVM

// Module 1: Tests du Bytecode
#[cfg(test)]
mod bytecode_tests {
    use super::*;

    // 1.1 Test d'encodage/décodage d'opcodes
    #[test]
    fn test_opcode_encoding_decoding() {
        // Créer différents opcodes
        let opcode1 = Opcode::ADD;
        let opcode2 = Opcode::SUB;
        let opcode3 = Opcode::LOAD;

        // Encoder en bytes
        let byte1 = opcode1.encode();
        let byte2 = opcode2.encode();
        let byte3 = opcode3.encode();

        // Décoder et vérifier
        assert_eq!(Opcode::decode(byte1), opcode1);
        assert_eq!(Opcode::decode(byte2), opcode2);
        assert_eq!(Opcode::decode(byte3), opcode3);
    }

    // 1.2 Test de création et manipulation d'instructions
    #[test]
    fn test_instruction_creation() {
        // Créer une instruction simple
        let instr = Instruction::new(
            Opcode::ADD,
            InstructionFormat::RegReg,
            vec![0x01, 0x02] // registres 1 et 2
        );

        // Vérifier les propriétés
        assert_eq!(instr.opcode, Opcode::ADD);
        assert_eq!(instr.format, InstructionFormat::RegReg);
        assert_eq!(instr.operands, vec![0x01, 0x02]);

        // Vérifier la taille totale
        assert_eq!(instr.size(), 4); // 1B opcode + 1B format + 2B operands
    }

    // 1.3 Test d'encodage/décodage d'instructions complètes
    #[test]
    fn test_instruction_encoding_decoding() {
        // Créer une instruction avec différents types d'opérandes
        let original = Instruction::new(
            Opcode::STORE,
            InstructionFormat::RegMem,
            vec![0x03, 0x00, 0x10, 0x00] // reg 3, addr 0x1000
        );

        // Encoder en bytecode
        let encoded = original.encode();

        // Décoder le bytecode
        let decoded = Instruction::decode(&encoded).unwrap();

        // Vérifier l'égalité
        assert_eq!(decoded.opcode, original.opcode);
        assert_eq!(decoded.format, original.format);
        assert_eq!(decoded.operands, original.operands);
    }

    // 1.4 Test de fichier bytecode
    #[test]
    fn test_bytecode_file() {
        // Créer un ensemble d'instructions
        let instructions = vec![
            Instruction::new(Opcode::LOAD, InstructionFormat::RegImm, vec![0x01, 0x00, 0x05, 0x00]),
            Instruction::new(Opcode::LOAD, InstructionFormat::RegImm, vec![0x02, 0x00, 0x0A, 0x00]),
            Instruction::new(Opcode::ADD, InstructionFormat::RegReg, vec![0x03, 0x01, 0x02, 0x00]),
            Instruction::new(Opcode::STORE, InstructionFormat::RegMem, vec![0x03, 0x00, 0x20, 0x00])
        ];

        // Créer un fichier bytecode
        let bf = BytecodeFile::new(instructions, vec![], HashMap::new());

        // Sérialiser le fichier
        let serialized = bf.serialize();

        // Désérialiser et vérifier
        let deserialized = BytecodeFile::deserialize(&serialized).unwrap();

        // Vérifier le nombre d'instructions
        assert_eq!(deserialized.instructions.len(), 4);

        // Vérifier les opcodes des instructions
        assert_eq!(deserialized.instructions[0].opcode, Opcode::LOAD);
        assert_eq!(deserialized.instructions[1].opcode, Opcode::LOAD);
        assert_eq!(deserialized.instructions[2].opcode, Opcode::ADD);
        assert_eq!(deserialized.instructions[3].opcode, Opcode::STORE);
    }
}

// Module 2: Tests de l'ALU
#[cfg(test)]
mod alu_tests {
    use super::*;

    // Configuration de base pour les tests ALU
    fn setup_alu() -> ALU {
        ALU::new()
    }

    // 2.1 Test des opérations arithmétiques de base
    #[test]
    fn test_alu_arithmetic() {
        let mut alu = setup_alu();

        // Addition
        let (result, flags) = alu.execute(ALUOp::Add, 5, 7);
        assert_eq!(result, 12);
        assert_eq!(flags.zero, false);
        assert_eq!(flags.negative, false);

        // Soustraction
        let (result, flags) = alu.execute(ALUOp::Sub, 10, 7);
        assert_eq!(result, 3);
        assert_eq!(flags.zero, false);
        assert_eq!(flags.negative, false);

        // Multiplication
        let (result, flags) = alu.execute(ALUOp::Mul, 3, 4);
        assert_eq!(result, 12);

        // Division
        let (result, flags) = alu.execute(ALUOp::Div, 20, 4);
        assert_eq!(result, 5);
    }

    // 2.2 Test des opérations logiques
    #[test]
    fn test_alu_logical() {
        let mut alu = setup_alu();

        // AND
        let (result, _) = alu.execute(ALUOp::And, 0b1010, 0b1100);
        assert_eq!(result, 0b1000);

        // OR
        let (result, _) = alu.execute(ALUOp::Or, 0b1010, 0b1100);
        assert_eq!(result, 0b1110);

        // XOR
        let (result, _) = alu.execute(ALUOp::Xor, 0b1010, 0b1100);
        assert_eq!(result, 0b0110);

        // NOT
        let (result, _) = alu.execute(ALUOp::Not, 0b1010, 0);
        assert_eq!(result, !0b1010);
    }

    // 2.3 Test des flags
    #[test]
    fn test_alu_flags() {
        let mut alu = setup_alu();

        // Test flag Zero
        let (_, flags) = alu.execute(ALUOp::Sub, 5, 5);
        assert_eq!(flags.zero, true);

        // Test flag Negative
        let (_, flags) = alu.execute(ALUOp::Sub, 5, 10);
        assert_eq!(flags.negative, true);

        // Test flag Overflow (addition)
        let (_, flags) = alu.execute(ALUOp::Add, i32::MAX, 1);
        assert_eq!(flags.overflow, true);

        // Test flag Carry
        let (_, flags) = alu.execute(ALUOp::Add, u32::MAX, 1);
        assert_eq!(flags.carry, true);
    }
}

// Module 3: Tests des composants du Pipeline
#[cfg(test)]
mod pipeline_tests {
    use super::*;

    // 3.1 Test de l'étage Fetch
    #[test]
    fn test_fetch_stage() {
        // Créer un programme simple
        let program = vec![
            Instruction::new(Opcode::LOAD, InstructionFormat::RegImm, vec![0x01, 0x00, 0x05, 0x00]),
            Instruction::new(Opcode::LOAD, InstructionFormat::RegImm, vec![0x02, 0x00, 0x0A, 0x00])
        ];

        let bytecode_file = BytecodeFile::new(program, vec![], HashMap::new());

        // Créer l'étage Fetch
        let mut fetch = FetchStage::new(&bytecode_file);

        // Exécuter deux cycles de fetch
        let instr1 = fetch.execute().unwrap();
        let instr2 = fetch.execute().unwrap();

        // Vérifier les instructions récupérées
        assert_eq!(instr1.opcode, Opcode::LOAD);
        assert_eq!(instr1.format, InstructionFormat::RegImm);

        assert_eq!(instr2.opcode, Opcode::LOAD);
        assert_eq!(instr2.format, InstructionFormat::RegImm);

        // Vérifier la fin du programme
        let instr3 = fetch.execute();
        assert!(instr3.is_none());
    }

    // 3.2 Test de l'étage Decode
    #[test]
    fn test_decode_stage() {
        // Créer une instruction à décoder
        let instruction = Instruction::new(
            Opcode::ADD,
            InstructionFormat::RegReg,
            vec![0x01, 0x02, 0x03, 0x00] // r1 = r2 + r3
        );

        // Créer l'étage Decode
        let mut decode = DecodeStage::new();

        // Décoder l'instruction
        let decoded = decode.execute(&instruction).unwrap();

        // Vérifier le résultat du décodage
        assert_eq!(decoded.opcode, Opcode::ADD);
        assert_eq!(decoded.dest_reg, 0x01);
        assert_eq!(decoded.src_regs, vec![0x02, 0x03]);
        assert_eq!(decoded.immediate, None);
        assert_eq!(decoded.memory_addr, None);
    }

    // 3.3 Test de l'étage Execute (avec l'ALU)
    #[test]
    fn test_execute_stage() {
        // Créer une instruction décodée
        let decoded_instr = DecodedInstruction {
            opcode: Opcode::ADD,
            dest_reg: 0x01,
            src_regs: vec![0x02, 0x03],
            immediate: None,
            memory_addr: None,
        };

        // Créer un état des registres
        let mut registers = [0; 16];
        registers[0x02] = 10;
        registers[0x03] = 15;

        // Créer l'étage Execute
        let mut execute = ExecuteStage::new();

        // Exécuter l'instruction
        let result = execute.execute(&decoded_instr, &registers).unwrap();

        // Vérifier le résultat
        assert_eq!(result.result, 25); // 10 + 15
        assert_eq!(result.dest_reg, 0x01);
        assert_eq!(result.memory_op, None);
    }

    // 3.4 Test de l'étage Memory
    #[test]
    fn test_memory_stage() {
        // Créer un résultat d'exécution pour une opération store
        let exec_result = ExecutionResult {
            result: 42,
            dest_reg: 0x01,
            memory_op: Some(MemoryOperation::Store(0x1000)),
            flags: ALUFlags::default(),
        };

        // Créer la mémoire
        let mut memory = vec![0; 8192];

        // Créer l'étage Memory
        let mut mem_stage = MemoryStage::new();

        // Exécuter l'opération mémoire
        let mem_result = mem_stage.execute(&exec_result, &mut memory).unwrap();

        // Vérifier le résultat
        assert_eq!(mem_result.result, 42);
        assert_eq!(mem_result.dest_reg, 0x01);
        assert_eq!(memory[0x1000], 42);

        // Test d'une opération load
        let exec_result_load = ExecutionResult {
            result: 0,
            dest_reg: 0x04,
            memory_op: Some(MemoryOperation::Load(0x1000)),
            flags: ALUFlags::default(),
        };

        let mem_result_load = mem_stage.execute(&exec_result_load, &mut memory).unwrap();

        // Vérifier le résultat du load
        assert_eq!(mem_result_load.result, 42);
        assert_eq!(mem_result_load.dest_reg, 0x04);
    }

    // 3.5 Test de l'étage Writeback
    #[test]
    fn test_writeback_stage() {
        // Créer un résultat de l'étage mémoire
        let mem_result = MemoryResult {
            result: 42,
            dest_reg: 0x05,
            flags: ALUFlags::default(),
        };

        // Créer un état des registres
        let mut registers = [0; 16];

        // Créer l'étage Writeback
        let mut wb_stage = WritebackStage::new();

        // Exécuter l'opération writeback
        wb_stage.execute(&mem_result, &mut registers).unwrap();

        // Vérifier que le registre a été mis à jour
        assert_eq!(registers[0x05], 42);
    }

    // 3.6 Test d'intégration du pipeline complet
    #[test]
    fn test_pipeline_integration() {
        // Créer un programme simple: charge 2 valeurs, les additionne et stocke le résultat
        let program = vec![
            Instruction::new(Opcode::LOAD, InstructionFormat::RegImm, vec![0x01, 0x00, 0x05, 0x00]), // r1 = 5
            Instruction::new(Opcode::LOAD, InstructionFormat::RegImm, vec![0x02, 0x00, 0x07, 0x00]), // r2 = 7
            Instruction::new(Opcode::ADD, InstructionFormat::RegReg, vec![0x03, 0x01, 0x02, 0x00]),  // r3 = r1 + r2
            Instruction::new(Opcode::STORE, InstructionFormat::RegMem, vec![0x03, 0x00, 0x40, 0x00]) // mem[0x4000] = r3
        ];

        let bytecode_file = BytecodeFile::new(program, vec![], HashMap::new());

        // Créer un pipeline complet
        let mut pipeline = Pipeline::new(&bytecode_file);

        // Exécuter le programme
        while pipeline.step() {
            // Continuer jusqu'à ce que le programme soit terminé
        }

        // Vérifier les résultats
        assert_eq!(pipeline.registers[0x01], 5);
        assert_eq!(pipeline.registers[0x02], 7);
        assert_eq!(pipeline.registers[0x03], 12);
        assert_eq!(pipeline.memory[0x4000], 12);
    }

    // 3.7 Test de détection et résolution des hazards
    #[test]
    fn test_hazard_detection() {
        // Créer un programme avec des dépendances de données
        let program = vec![
            Instruction::new(Opcode::LOAD, InstructionFormat::RegImm, vec![0x01, 0x00, 0x05, 0x00]), // r1 = 5
            Instruction::new(Opcode::ADD, InstructionFormat::RegReg, vec![0x02, 0x01, 0x01, 0x00]),  // r2 = r1 + r1 (dépendance)
            Instruction::new(Opcode::ADD, InstructionFormat::RegReg, vec![0x03, 0x02, 0x01, 0x00]),  // r3 = r2 + r1 (dépendance)
        ];

        let bytecode_file = BytecodeFile::new(program, vec![], HashMap::new());

        // Créer un pipeline avec détection de hazards
        let mut pipeline = Pipeline::new_with_hazard_detection(&bytecode_file);

        // Exécuter le programme
        let mut cycles = 0;
        while pipeline.step() && cycles < 10 {
            cycles += 1;
        }

        // Vérifier les résultats
        assert_eq!(pipeline.registers[0x01], 5);
        assert_eq!(pipeline.registers[0x02], 10); // 5 + 5
        assert_eq!(pipeline.registers[0x03], 15); // 10 + 5

        // Vérifier le nombre de cycles (avec stalls)
        // Normalement, ce programme prendrait plus de 3 cycles à cause des hazards
        assert!(cycles > 3);
    }

    // 3.8 Test de forwarding
    #[test]
    fn test_forwarding() {
        // Même programme que précédemment
        let program = vec![
            Instruction::new(Opcode::LOAD, InstructionFormat::RegImm, vec![0x01, 0x00, 0x05, 0x00]), // r1 = 5
            Instruction::new(Opcode::ADD, InstructionFormat::RegReg, vec![0x02, 0x01, 0x01, 0x00]),  // r2 = r1 + r1 (dépendance)
            Instruction::new(Opcode::ADD, InstructionFormat::RegReg, vec![0x03, 0x02, 0x01, 0x00]),  // r3 = r2 + r1 (dépendance)
        ];

        let bytecode_file = BytecodeFile::new(program, vec![], HashMap::new());

        // Créer un pipeline avec forwarding
        let mut pipeline = Pipeline::new_with_forwarding(&bytecode_file);

        // Exécuter le programme
        let mut cycles = 0;
        while pipeline.step() && cycles < 10 {
            cycles += 1;
        }

        // Vérifier les résultats
        assert_eq!(pipeline.registers[0x01], 5);
        assert_eq!(pipeline.registers[0x02], 10); // 5 + 5
        assert_eq!(pipeline.registers[0x03], 15); // 10 + 5

        // Vérifier le nombre de cycles (avec forwarding)
        // Le forwarding devrait réduire le nombre de cycles par rapport au test précédent
        assert!(cycles <= 5); // Moins de stalls grâce au forwarding
    }
}

// Module 4: Tests de programmes complets
#[cfg(test)]
mod program_tests {
    use super::*;

    // 4.1 Test d'un programme d'addition simple
    #[test]
    fn test_simple_add_program() {
        // Programme: r1 = 5, r2 = 7, r3 = r1 + r2
        let program = vec![
            Instruction::new(Opcode::LOAD, InstructionFormat::RegImm, vec![0x01, 0x00, 0x05, 0x00]),
            Instruction::new(Opcode::LOAD, InstructionFormat::RegImm, vec![0x02, 0x00, 0x07, 0x00]),
            Instruction::new(Opcode::ADD, InstructionFormat::RegReg, vec![0x03, 0x01, 0x02, 0x00]),
        ];

        // Exécuter le programme
        let result = execute_program(program);

        // Vérifier les résultats
        assert_eq!(result.registers[0x01], 5);
        assert_eq!(result.registers[0x02], 7);
        assert_eq!(result.registers[0x03], 12);
    }

    // 4.2 Test d'un programme avec boucle
    #[test]
    fn test_loop_program() {
        // Programme qui calcule la somme des nombres de 1 à 5
        // r1 = compteur, r2 = total, r3 = limite
        let program = vec![
            // Initialisation
            Instruction::new(Opcode::LOAD, InstructionFormat::RegImm, vec![0x01, 0x00, 0x01, 0x00]), // compteur = 1
            Instruction::new(Opcode::LOAD, InstructionFormat::RegImm, vec![0x02, 0x00, 0x00, 0x00]), // total = 0
            Instruction::new(Opcode::LOAD, InstructionFormat::RegImm, vec![0x03, 0x00, 0x05, 0x00]), // limite = 5

            // Corps de boucle (addr 3)
            Instruction::new(Opcode::ADD, InstructionFormat::RegReg, vec![0x02, 0x02, 0x01, 0x00]),  // total += compteur
            Instruction::new(Opcode::ADD, InstructionFormat::RegImm, vec![0x01, 0x01, 0x01, 0x00]),  // compteur++

            // Condition de boucle
            Instruction::new(Opcode::CMP, InstructionFormat::RegReg, vec![0x00, 0x01, 0x03, 0x00]),  // compare compteur et limite
            Instruction::new(Opcode::JLE, InstructionFormat::Imm, vec![0x03, 0x00, 0x00, 0x00]),     // si <= retourne à addr 3
        ];

        // Exécuter le programme
        let result = execute_program(program);

        // Vérifier le résultat: somme de 1 à 5 = 15
        assert_eq!(result.registers[0x02], 15);
        assert_eq!(result.registers[0x01], 6); // Le compteur dépasse la limite à la fin
    }

    // 4.3 Test d'un programme avec accès mémoire
    #[test]
    fn test_memory_operations() {
        // Programme qui initialise un tableau de 5 éléments et calcule leur somme
        let program = vec![
            // Initialisation des indices
            Instruction::new(Opcode::LOAD, InstructionFormat::RegImm, vec![0x01, 0x00, 0x00, 0x10]), // r1 = adresse de base (0x1000)
            Instruction::new(Opcode::LOAD, InstructionFormat::RegImm, vec![0x02, 0x00, 0x00, 0x00]), // r2 = index (0)
            Instruction::new(Opcode::LOAD, InstructionFormat::RegImm, vec![0x03, 0x00, 0x05, 0x00]), // r3 = limite (5)
            Instruction::new(Opcode::LOAD, InstructionFormat::RegImm, vec![0x04, 0x00, 0x00, 0x00]), // r4 = somme (0)

            // Remplir le tableau (addr 4)
            Instruction::new(Opcode::ADD, InstructionFormat::RegImm, vec![0x05, 0x02, 0x01, 0x00]),  // r5 = index + 1 (valeur à stocker)

            // Calculer l'adresse mémoire: base + index
            Instruction::new(Opcode::ADD, InstructionFormat::RegReg, vec![0x06, 0x01, 0x02, 0x00]),  // r6 = base + index

            // Stocker la valeur
            Instruction::new(Opcode::STORE, InstructionFormat::RegMem, vec![0x05, 0x00, 0x06, 0x00]), // mem[r6] = r5

            // Incrémenter l'index
            Instruction::new(Opcode::ADD, InstructionFormat::RegImm, vec![0x02, 0x02, 0x01, 0x00]),  // index++

            // Vérifier si terminé
            Instruction::new(Opcode::CMP, InstructionFormat::RegReg, vec![0x00, 0x02, 0x03, 0x00]),  // compare index et limite
            Instruction::new(Opcode::JLT, InstructionFormat::Imm, vec![0x04, 0x00, 0x00, 0x00]),     // si < retourne à addr 4

            // Réinitialiser pour la sommation
            Instruction::new(Opcode::LOAD, InstructionFormat::RegImm, vec![0x02, 0x00, 0x00, 0x00]), // index = 0

            // Somme du tableau (addr 11)
            // Calculer l'adresse mémoire
            Instruction::new(Opcode::ADD, InstructionFormat::RegReg, vec![0x06, 0x01, 0x02, 0x00]),  // r6 = base + index

            // Charger la valeur
            Instruction::new(Opcode::LOAD, InstructionFormat::MemReg, vec![0x05, 0x00, 0x06, 0x00]), // r5 = mem[r6]

            // Ajouter à la somme
            Instruction::new(Opcode::ADD, InstructionFormat::RegReg, vec![0x04, 0x04, 0x05, 0x00]),  // somme += valeur

            // Incrémenter l'index
            Instruction::new(Opcode::ADD, InstructionFormat::RegImm, vec![0x02, 0x02, 0x01, 0x00]),  // index++

            // Vérifier si terminé
            Instruction::new(Opcode::CMP, InstructionFormat::RegReg, vec![0x00, 0x02, 0x03, 0x00]),  // compare index et limite
            Instruction::new(Opcode::JLT, InstructionFormat::Imm, vec![0x0B, 0x00, 0x00, 0x00]),     // si < retourne à addr 11
        ];

        // Exécuter le programme
        let result = execute_program(program);

        // Vérifier le résultat: somme de 1 à 5 = 15
        assert_eq!(result.registers[0x04], 15);

        // Vérifier les valeurs du tableau
        assert_eq!(result.memory[0x1000], 1);
        assert_eq!(result.memory[0x1001], 2);
        assert_eq!(result.memory[0x1002], 3);
        assert_eq!(result.memory[0x1003], 4);
        assert_eq!(result.memory[0x1004], 5);
    }

    // 4.4 Test de programme avec hazards intensifs
    #[test]
    fn test_hazard_intensive_program() {
        // Programme créé pour maximiser les dépendances de données
        let program = vec![
            Instruction::new(Opcode::LOAD, InstructionFormat::RegImm, vec![0x01, 0x00, 0x01, 0x00]), // r1 = 1
            Instruction::new(Opcode::ADD, InstructionFormat::RegReg, vec![0x02, 0x01, 0x01, 0x00]),  // r2 = r1 + r1 = 2
            Instruction::new(Opcode::ADD, InstructionFormat::RegReg, vec![0x03, 0x02, 0x01, 0x00]),  // r3 = r2 + r1 = 3
            Instruction::new(Opcode::ADD, InstructionFormat::RegReg, vec![0x04, 0x03, 0x02, 0x00]),  // r4 = r3 + r2 = 5
            Instruction::new(Opcode::ADD, InstructionFormat::RegReg, vec![0x05, 0x04, 0x03, 0x00]),  // r5 = r4 + r3 = 8
            Instruction::new(Opcode::ADD, InstructionFormat::RegReg, vec![0x06, 0x05, 0x04, 0x00]),  // r6 = r5 + r4 = 13
            Instruction::new(Opcode::ADD, InstructionFormat::RegReg, vec![0x07, 0x06, 0x05, 0x00]),  // r7 = r6 + r5 = 21
        ];

        // Exécuter sans forwarding
        let result_no_fwd = execute_program_with_options(program.clone(), false, true);

        // Exécuter avec forwarding
        let result_fwd = execute_program_with_options(program, true, true);

        // Vérifier les résultats (identiques pour les deux modes)
        assert_eq!(result_no_fwd.registers[0x07], 21);
        assert_eq!(result_fwd.registers[0x07], 21);

        // Vérifier que le forwarding a réduit le nombre de cycles
        assert!(result_fwd.cycles < result_no_fwd.cycles);
    }

    // 4.5 Test de programme avec Store-Load hazards
    #[test]
    fn test_store_load_hazards() {
        // Programme qui teste les hazards store-load
        let program = vec![
            Instruction::new(Opcode::LOAD, InstructionFormat::RegImm, vec![0x01, 0x00, 0x42, 0x00]), // r1 = 42
            Instruction::new(Opcode::STORE, InstructionFormat::RegMem, vec![0x01, 0x00, 0x00, 0x10]), // mem[0x1000] = r1
            Instruction::new(Opcode::LOAD, InstructionFormat::MemReg, vec![0x02, 0x00, 0x00, 0x10]),  // r2 = mem[0x1000]
            Instruction::new(Opcode::ADD, InstructionFormat::RegReg, vec![0x03, 0x02, 0x01, 0x00]),   // r3 = r2 + r1
        ];

        // Exécuter sans store-buffer (devrait détecter le hazard et stall)
        let result_no_sb = execute_program_with_options(program.clone(), true, false);

        // Exécuter avec store-buffer (devrait faire du forwarding)
        let result_sb = execute_program_with_options(program, true, true);

        // Vérifier les résultats (identiques pour les deux modes)
        assert_eq!(result_no_sb.registers[0x03], 84); // 42 + 42
        assert_eq!(result_sb.registers[0x03], 84);    // 42 + 42

        // Vérifier que le store-buffer a réduit le nombre de cycles
        assert!(result_sb.cycles < result_no_sb.cycles);
    }
}


////////////////////////////////////////////////////////////////////////


////////////////////////////////////////////////////////////////////////////////////////////////////
//
//
// #[cfg(test)]
// mod branch_instruction_tests {
//
//     use crate::bytecode::instructions::{ArgValue, calculate_branch_offset, Instruction};
//     use crate::bytecode::opcodes::Opcode;
//
//     /// Test du calcul d'offset de branchement
//     #[test]
//     fn test_calculate_branch_offset() {
//         // Test saut en avant
//         let from = 0x10;
//         let to = 0x20;
//         let instr_size = 7;
//         let offset = calculate_branch_offset(from, to, instr_size);
//         assert_eq!(offset, 9); // 0x20 - (0x10 + 7) = 9
//
//         // Test saut en arrière
//         let from = 0x30;
//         let to = 0x10;
//         let instr_size = 7;
//         let offset = calculate_branch_offset(from, to, instr_size);
//         assert_eq!(offset, -39); // 0x10 - (0x30 + 7) = -39
//
//         // Test saut sur place (boucle infinie)
//         let from = 0x40;
//         let to = 0x40;
//         let instr_size = 7;
//         let offset = calculate_branch_offset(from, to, instr_size);
//         assert_eq!(offset, -7); // 0x40 - (0x40 + 7) = -7
//     }
//
//     /// Test JMP (saut inconditionnel)
//     #[test]
//     fn test_create_jump() {
//         let from_addr = 0x1000;
//         let to_addr = 0x1020;
//
//         let instr = Instruction::create_jump(from_addr, to_addr);
//
//         assert_eq!(instr.opcode, Opcode::Jmp);
//
//         // Vérifier l'offset encodé
//         if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
//             let expected_offset = calculate_branch_offset(from_addr, to_addr, instr.total_size() as u32);
//             assert_eq!(offset, expected_offset);
//             println!("JMP offset: {} (from 0x{:X} to 0x{:X})", offset, from_addr, to_addr);
//         } else {
//             panic!("Failed to extract offset from JMP instruction");
//         }
//     }
//
//     /// Test JmpIf
//     #[test]
//     fn test_create_jump_if() {
//         let from_addr = 0x2000;
//         let to_addr = 0x2010;
//
//         let instr = Instruction::create_jump_if(from_addr, to_addr);
//
//         assert_eq!(instr.opcode, Opcode::JmpIf);
//
//         if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
//             let expected_offset = calculate_branch_offset(from_addr, to_addr, instr.total_size() as u32);
//             assert_eq!(offset, expected_offset);
//         } else {
//             panic!("Failed to extract offset from JmpIf instruction");
//         }
//     }
//
//     /// Test JmpIfNot
//     #[test]
//     fn test_create_jump_if_not() {
//         let from_addr = 0x3000;
//         let to_addr = 0x2FF0; // Saut en arrière
//
//         let instr = Instruction::create_jump_if_not(from_addr, to_addr);
//
//         assert_eq!(instr.opcode, Opcode::JmpIfNot);
//
//         if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
//             assert!(offset < 0, "Offset should be negative for backward jump");
//             let expected_offset = calculate_branch_offset(from_addr, to_addr, instr.total_size() as u32);
//             assert_eq!(offset, expected_offset);
//         } else {
//             panic!("Failed to extract offset from JmpIfNot instruction");
//         }
//     }
//
//     /// Test JmpIfEqual
//     #[test]
//     fn test_create_jump_if_equal() {
//         let from_addr = 0x4000;
//         let to_addr = 0x4100;
//
//         let instr = Instruction::create_jump_if_equal(from_addr, to_addr);
//
//         assert_eq!(instr.opcode, Opcode::JmpIfEqual);
//
//         if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
//             assert!(offset > 0, "Offset should be positive for forward jump");
//             let expected_offset = calculate_branch_offset(from_addr, to_addr, instr.total_size() as u32);
//             assert_eq!(offset, expected_offset);
//         } else {
//             panic!("Failed to extract offset from JmpIfEqual instruction");
//         }
//     }
//
//     /// Test JmpIfNotEqual
//     #[test]
//     fn test_create_jump_if_not_equal() {
//         let from_addr = 0x5000;
//         let to_addr = 0x5050;
//
//         let instr = Instruction::create_jump_if_not_equal(from_addr, to_addr);
//
//         assert_eq!(instr.opcode, Opcode::JmpIfNotEqual);
//
//         if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
//             let expected_offset = calculate_branch_offset(from_addr, to_addr, instr.total_size() as u32);
//             assert_eq!(offset, expected_offset);
//         } else {
//             panic!("Failed to extract offset from JmpIfNotEqual instruction");
//         }
//     }
//
//     /// Test JmpIfGreater
//     #[test]
//     fn test_create_jump_if_greater() {
//         let from_addr = 0x6000;
//         let to_addr = 0x6040;
//
//         let instr = Instruction::create_jump_if_greater(from_addr, to_addr);
//
//         assert_eq!(instr.opcode, Opcode::JmpIfGreater);
//
//         if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
//             let expected_offset = calculate_branch_offset(from_addr, to_addr, instr.total_size() as u32);
//             assert_eq!(offset, expected_offset);
//         } else {
//             panic!("Failed to extract offset from JmpIfGreater instruction");
//         }
//     }
//
//     /// Test JmpIfGreaterEqual
//     #[test]
//     fn test_create_jump_if_greater_equal() {
//         let from_addr = 0x7000;
//         let to_addr = 0x7030;
//
//         let instr = Instruction::create_jump_if_greater_equal(from_addr, to_addr);
//
//         assert_eq!(instr.opcode, Opcode::JmpIfGreaterEqual);
//
//         if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
//             let expected_offset = calculate_branch_offset(from_addr, to_addr, instr.total_size() as u32);
//             assert_eq!(offset, expected_offset);
//         } else {
//             panic!("Failed to extract offset from JmpIfGreaterEqual instruction");
//         }
//     }
//
//     /// Test JmpIfLess
//     #[test]
//     fn test_create_jump_if_less() {
//         let from_addr = 0x8000;
//         let to_addr = 0x7FF0; // Saut en arrière
//
//         let instr = Instruction::create_jump_if_less(from_addr, to_addr);
//
//         assert_eq!(instr.opcode, Opcode::JmpIfLess);
//
//         if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
//             assert!(offset < 0, "Offset should be negative for backward jump");
//             let expected_offset = calculate_branch_offset(from_addr, to_addr, instr.total_size() as u32);
//             assert_eq!(offset, expected_offset);
//         } else {
//             panic!("Failed to extract offset from JmpIfLess instruction");
//         }
//     }
//
//     /// Test JmpIfLessEqual
//     #[test]
//     fn test_create_jump_if_less_equal() {
//         let from_addr = 0x9000;
//         let to_addr = 0x9020;
//
//         let instr = Instruction::create_jump_if_less_equal(from_addr, to_addr);
//
//         assert_eq!(instr.opcode, Opcode::JmpIfLessEqual);
//
//         if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
//             let expected_offset = calculate_branch_offset(from_addr, to_addr, instr.total_size() as u32);
//             assert_eq!(offset, expected_offset);
//         } else {
//             panic!("Failed to extract offset from JmpIfLessEqual instruction");
//         }
//     }
//
//     /// Test JmpIfAbove (non signé)
//     #[test]
//     fn test_create_jump_if_above() {
//         let from_addr = 0xA000;
//         let to_addr = 0xA040;
//
//         let instr = Instruction::create_jump_if_above(from_addr, to_addr);
//
//         // Note: Il y a un bug dans votre code - create_jump_if_above crée JmpIfLessEqual
//         // Ce test détectera ce bug
//         assert_eq!(instr.opcode, Opcode::JmpIfAbove, "Bug: create_jump_if_above crée le mauvais opcode!");
//     }
//
//     /// Test JmpIfAboveEqual (non signé)
//     #[test]
//     fn test_create_jump_if_above_equal() {
//         let from_addr = 0xB000;
//         let to_addr = 0xB030;
//
//         let instr = Instruction::create_jump_if_above_equal(from_addr, to_addr);
//
//         assert_eq!(instr.opcode, Opcode::JmpIfAboveEqual);
//
//         if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
//             let expected_offset = calculate_branch_offset(from_addr, to_addr, instr.total_size() as u32);
//             assert_eq!(offset, expected_offset);
//         } else {
//             panic!("Failed to extract offset from JmpIfAboveEqual instruction");
//         }
//     }
//
//     /// Test JmpIfBelow (non signé)
//     #[test]
//     fn test_create_jump_below() {
//         let from_addr = 0xC000;
//         let to_addr = 0xC050;
//
//         let instr = Instruction::create_jump_below(from_addr, to_addr);
//
//         assert_eq!(instr.opcode, Opcode::JmpIfBelow);
//
//         if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
//             let expected_offset = calculate_branch_offset(from_addr, to_addr, instr.total_size() as u32);
//             assert_eq!(offset, expected_offset);
//         } else {
//             panic!("Failed to extract offset from JmpIfBelow instruction");
//         }
//     }
//
//     /// Test JmpIfBelowEqual (non signé)
//     #[test]
//     fn test_create_jump_if_below_equal() {
//         let from_addr = 0xD000;
//         let to_addr = 0xD020;
//
//         let instr = Instruction::create_jump_if_below_equal(from_addr, to_addr);
//
//         assert_eq!(instr.opcode, Opcode::JmpIfBelowEqual);
//
//         if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
//             let expected_offset = calculate_branch_offset(from_addr, to_addr, instr.total_size() as u32);
//             assert_eq!(offset, expected_offset);
//         } else {
//             panic!("Failed to extract offset from JmpIfBelowEqual instruction");
//         }
//     }
//
//     /// Test JmpIfZero
//     #[test]
//     fn test_create_jump_if_zero() {
//         let from_addr = 0xE000;
//         let to_addr = 0xE100;
//
//         let instr = Instruction::create_jump_if_zero(from_addr, to_addr);
//
//         assert_eq!(instr.opcode, Opcode::JmpIfZero);
//
//         if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
//             let expected_offset = calculate_branch_offset(from_addr, to_addr, instr.total_size() as u32);
//             assert_eq!(offset, expected_offset);
//         } else {
//             panic!("Failed to extract offset from JmpIfZero instruction");
//         }
//     }
//
//     /// Test JmpIfNotZero
//     #[test]
//     fn test_create_jump_if_not_zero() {
//         let from_addr = 0xF000;
//         let to_addr = 0xEFF0; // Saut en arrière
//
//         let instr = Instruction::create_jump_if_not_zero(from_addr, to_addr);
//
//         assert_eq!(instr.opcode, Opcode::JmpIfNotZero);
//
//         if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
//             assert!(offset < 0, "Offset should be negative for backward jump");
//             let expected_offset = calculate_branch_offset(from_addr, to_addr, instr.total_size() as u32);
//             assert_eq!(offset, expected_offset);
//         } else {
//             panic!("Failed to extract offset from JmpIfNotZero instruction");
//         }
//     }
//
//     /// Test JmpIfOverflow
//     #[test]
//     fn test_create_jump_if_overflow() {
//         let from_addr = 0x10000;
//         let to_addr = 0x10020;
//
//         let instr = Instruction::create_jump_if_overflow(from_addr, to_addr);
//
//         assert_eq!(instr.opcode, Opcode::JmpIfOverflow);
//
//         if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
//             let expected_offset = calculate_branch_offset(from_addr, to_addr, instr.total_size() as u32);
//             assert_eq!(offset, expected_offset);
//         } else {
//             panic!("Failed to extract offset from JmpIfOverflow instruction");
//         }
//     }
//
//     /// Test JmpIfNotOverflow
//     #[test]
//     fn test_create_jump_if_not_overflow() {
//         let from_addr = 0x11000;
//         let to_addr = 0x11040;
//
//         let instr = Instruction::create_jump_if_not_overflow(from_addr, to_addr); // Note: typo dans le nom de la fonction
//
//         assert_eq!(instr.opcode, Opcode::JmpIfNotOverflow);
//
//         if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
//             let expected_offset = calculate_branch_offset(from_addr, to_addr, instr.total_size() as u32);
//             assert_eq!(offset, expected_offset);
//         } else {
//             panic!("Failed to extract offset from JmpIfNotOverflow instruction");
//         }
//     }
//
//     /// Test JmpIfPositive
//     #[test]
//     fn test_create_jump_if_positive() {
//         let from_addr = 0x12000;
//         let to_addr = 0x12050;
//
//         let instr = Instruction::create_jump_if_positive(from_addr, to_addr);
//
//         assert_eq!(instr.opcode, Opcode::JmpIfPositive);
//
//         if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
//             let expected_offset = calculate_branch_offset(from_addr, to_addr, instr.total_size() as u32);
//             assert_eq!(offset, expected_offset);
//         } else {
//             panic!("Failed to extract offset from JmpIfPositive instruction");
//         }
//     }
//
//     /// Test JmpIfNegative
//     #[test]
//     fn test_create_jump_if_negative() {
//         let from_addr = 0x13000;
//         let to_addr = 0x13030;
//
//         let instr = Instruction::create_jump_if_negative(from_addr, to_addr);
//
//         assert_eq!(instr.opcode, Opcode::JmpIfNegative);
//
//         if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
//             let expected_offset = calculate_branch_offset(from_addr, to_addr, instr.total_size() as u32);
//             assert_eq!(offset, expected_offset);
//         } else {
//             panic!("Failed to extract offset from JmpIfNegative instruction");
//         }
//     }
//
//     /// Test de boucle avec plusieurs types de sauts
//     #[test]
//     fn test_complex_branching_scenario() {
//         // Simuler un programme avec différents types de branchements
//         let mut current_addr = 0x20000;
//         let mut instructions = Vec::new();
//
//         // MOV R0, 0
//         let instr = Instruction::create_reg_imm8(Opcode::Mov, 0, 0);
//         instructions.push((current_addr, instr.clone()));
//         current_addr += instr.total_size() as u32;
//
//         // MOV R1, 10
//         let instr = Instruction::create_reg_imm8(Opcode::Mov, 1, 10);
//         instructions.push((current_addr, instr.clone()));
//         current_addr += instr.total_size() as u32;
//
//         // Début de boucle
//         let loop_start = current_addr;
//
//         // INC R0
//         let instr = Instruction::create_single_reg(Opcode::Inc, 0);
//         instructions.push((current_addr, instr.clone()));
//         current_addr += instr.total_size() as u32;
//
//         // CMP R0, R1
//         let instr = Instruction::create_reg_reg(Opcode::Cmp, 0, 1);
//         instructions.push((current_addr, instr.clone()));
//         current_addr += instr.total_size() as u32;
//
//         // JmpIfLess loop_start (saut en arrière)
//         let jump_addr = current_addr;
//         let instr = Instruction::create_jump_if_less(jump_addr, loop_start);
//         instructions.push((current_addr, instr.clone()));
//
//         // Vérifier que l'offset est négatif pour un saut en arrière
//         if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
//             assert!(offset < 0, "Offset should be negative for backward jump in loop");
//             println!("Loop backward jump offset: {} (from 0x{:X} to 0x{:X})",
//                      offset, jump_addr, loop_start);
//         }
//
//         // Vérifier que toutes les instructions ont été créées correctement
//         assert_eq!(instructions.len(), 5);
//         for (addr, instr) in &instructions {
//             println!("0x{:X}: {:?} (size: {} bytes)", addr, instr.opcode, instr.total_size());
//         }
//     }
//
//     /// Test d'encodage et décodage des instructions de branchement
//     #[test]
//     fn test_branch_encode_decode() {
//         let from_addr = 0x30000;
//         let to_addr = 0x30100;
//
//         // Créer une instruction de branchement
//         let original = Instruction::create_jump_if_equal(from_addr, to_addr);
//
//         // Encoder
//         let encoded = original.encode();
//         println!("Encoded instruction: {:?}", encoded);
//
//         // Decoder
//         let (decoded, size) = Instruction::decode(&encoded).expect("Failed to decode instruction");
//
//         // Vérifier que l'instruction décodée est identique
//         assert_eq!(decoded.opcode, original.opcode);
//         assert_eq!(decoded.format, original.format);
//         assert_eq!(decoded.args, original.args);
//         assert_eq!(size, encoded.len());
//
//         // Vérifier que l'offset est préservé
//         if let (Ok(ArgValue::RelativeAddr(orig_offset)), Ok(ArgValue::RelativeAddr(dec_offset))) =
//             (original.get_arg2_value(), decoded.get_arg2_value()) {
//             assert_eq!(orig_offset, dec_offset);
//         } else {
//             panic!("Failed to extract offsets for comparison");
//         }
//     }
//
//     /// Test des cas limites
//     #[test]
//
//
//     fn test_branch_edge_cases() {
//         // Test avec offset positif proche de la limite
//         let from_addr = 0;
//         let to_addr = 0x7FFFFF00; // Un peu moins que i32::MAX
//         let instr = Instruction::create_jump(from_addr, to_addr);
//
//         if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
//             assert!(offset > 0, "Large positive offset should be positive");
//             println!("Large positive offset: {}", offset);
//         } else {
//             panic!("Failed to extract offset from large positive jump");
//         }
//
//         // Test avec offset négatif sécurisé
//         let from_addr = 0x7FFFFF00;
//         let to_addr = 0x100; // Proche du début
//         let instr = Instruction::create_jump(from_addr, to_addr);
//
//         if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
//             assert!(offset < 0, "Large negative offset should be negative");
//             println!("Large negative offset: {}", offset);
//         } else {
//             panic!("Failed to extract offset from large negative jump");
//         }
//     }
// }
//
//
// // Test unitaire pour les instructions
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_instruction_new() {
//         // Test création d'instruction simple
//         let instr = Instruction::new(
//             Opcode::Add,
//             InstructionFormat::double_reg(),
//             // vec![0x03, 0x05], // rd=3, rs1=5
//             vec![1, 2, 3]
//         );
//
//         assert_eq!(instr.opcode, Opcode::Add);
//         assert_eq!(instr.format.arg1_type, ArgType::Register);
//         assert_eq!(instr.format.arg2_type, ArgType::Register);
//         assert_eq!(instr.format.arg3_type, ArgType::None);
//         assert_eq!(instr.size_type, SizeType::Compact);
//         // assert_eq!(instr.args, vec![0x03, 0x05]);
//         assert_eq!(instr.args, vec![1, 2, 3]);
//     }
//
//     #[test]
//     fn test_instruction_total_size() {
//         // Instruction sans arguments
//         let instr1 = Instruction::create_no_args(Opcode::Nop);
//         assert_eq!(instr1.total_size(), 4); // opcode (1) + format (2) + size (1)
//
//         // Instruction avec 1 registre
//         let instr2 = Instruction::create_single_reg(Opcode::Inc, 3);
//         assert_eq!(instr2.total_size(), 5); // opcode (1) + format (2) + size (1) + reg (1)
//
//         // Instruction avec 2 registres
//         let instr3 = Instruction::create_reg_reg(Opcode::Add, 2, 3);
//         assert_eq!(instr3.total_size(), 6); // opcode (1) + format (2) + size (1) + regs (2)
//
//         // Instruction avec 3 registres
//         let instr4 = Instruction::create_reg_reg_reg(Opcode::Add, 2, 3, 4);
//         assert_eq!(instr4.total_size(), 7); // opcode (1) + format (2) + size (1) + regs (3)
//     }
//
//
//     #[test]
//     fn test_instruction_helpers() {
//         // Test création sans arguments
//         let nop = Instruction::create_no_args(Opcode::Nop);
//         assert_eq!(nop.opcode, Opcode::Nop);
//         assert_eq!(nop.args.len(), 0);
//
//         // Test création avec un registre
//         let inc = Instruction::create_single_reg(Opcode::Inc, 3);
//         assert_eq!(inc.opcode, Opcode::Inc);
//         assert_eq!(inc.args, vec![3]);
//
//         // Test création avec deux registres
//         let mov = Instruction::create_reg_reg(Opcode::Mov, 1, 2);
//         assert_eq!(mov.opcode, Opcode::Mov);
//         assert_eq!(mov.args, vec![1, 2]);
//
//         // Test création avec trois registres
//         let add = Instruction::create_reg_reg_reg(Opcode::Add, 1, 2, 3);
//         assert_eq!(add.opcode, Opcode::Add);
//         assert_eq!(add.args, vec![1, 2, 3]);
//     }
//
//     // Tests pour le calcul de la taille des instructions
//     mod size_calculation {
//         use super::*;
//
//         #[test]
//         fn test_instruction_sizes() {
//             let sizes = vec![
//                 (Instruction::create_no_args(Opcode::Nop), 4),
//                 (Instruction::create_single_reg(Opcode::Inc, 1), 5),
//                 (Instruction::create_reg_reg(Opcode::Mov, 1, 2), 6),
//                 (Instruction::create_reg_reg_reg(Opcode::Add, 1, 2, 3), 7),
//                 (Instruction::create_reg_imm8(Opcode::Load, 1, 42), 6),
//                 (Instruction::create_reg_imm16(Opcode::Load, 1, 1000), 7),
//             ];
//
//             for (instr, expected_size) in sizes {
//                 assert_eq!(instr.total_size(), expected_size);
//             }
//         }
//     }
//
//     #[test]
//     fn test_instruction_encoding_decoding() {
//         let instructions = vec![
//             Instruction::create_no_args(Opcode::Nop),
//             Instruction::create_single_reg(Opcode::Inc, 3),
//             Instruction::create_reg_reg(Opcode::Mov, 1, 2),
//             Instruction::create_reg_reg_reg(Opcode::Add, 1, 2, 3),
//             Instruction::create_reg_imm8(Opcode::Load, 1, 42),
//             Instruction::create_reg_imm16(Opcode::Load, 1, 1000),
//         ];
//
//         for original in instructions {
//             let encoded = original.encode();
//             let (decoded, size) = Instruction::decode(&encoded).unwrap();
//
//             assert_eq!(decoded.opcode, original.opcode);
//             assert_eq!(decoded.format, original.format);
//             assert_eq!(decoded.args, original.args);
//             assert_eq!(size, original.total_size());
//         }
//     }
//     //
//     #[test]
//     fn test_instruction_encode_decode() {
//         // Créer et encoder une instruction
//         let original = Instruction::create_reg_imm8(Opcode::Load, 3, 42);
//         let encoded = original.encode();
//
//         // Décoder l'instruction encodée
//         let (decoded, size) = Instruction::decode(&encoded).unwrap();
//
//         // Vérifier que le décodage correspond à l'original
//         assert_eq!(decoded.opcode, original.opcode);
//         assert_eq!(decoded.format.arg1_type, original.format.arg1_type);
//         assert_eq!(decoded.format.arg2_type, original.format.arg2_type);
//         assert_eq!(decoded.format.arg3_type, original.format.arg3_type);
//         assert_eq!(decoded.args, original.args);
//         assert_eq!(size, original.total_size());
//     }
//
//
//     #[test]
//     fn test_get_argument_values_1() {
//         // Test pour les instructions à 3 registres
//         let add = Instruction::create_reg_reg_reg(Opcode::Add, 1, 2, 3);
//         assert_eq!(add.get_arg1_value().unwrap(), ArgValue::Register(1));
//         assert_eq!(add.get_arg2_value().unwrap(), ArgValue::Register(2));
//         assert_eq!(add.get_arg3_value().unwrap(), ArgValue::Register(3));
//
//         // Test pour les instructions reg + imm8
//         let load = Instruction::create_reg_imm8(Opcode::Load, 1, 42);
//         assert_eq!(load.get_arg1_value().unwrap(), ArgValue::Register(1));
//         assert_eq!(load.get_arg2_value().unwrap(), ArgValue::Immediate(42));
//     }
//
//
//
//     #[test]
//     fn test_encode_decode_reg_reg_reg() {
//         // Test spécifique pour l'encodage/décodage des instructions à 3 registres
//         let original = Instruction::create_reg_reg_reg(Opcode::Add, 2, 3, 4);
//         let encoded = original.encode();
//
//         // Décoder l'instruction encodée
//         let (decoded, size) = Instruction::decode(&encoded).unwrap();
//
//         // Vérifier que le décodage correspond à l'original
//         assert_eq!(decoded.opcode, original.opcode);
//         assert_eq!(decoded.format.arg1_type, original.format.arg1_type);
//         assert_eq!(decoded.format.arg2_type, original.format.arg2_type);
//         assert_eq!(decoded.format.arg3_type, original.format.arg3_type);
//         assert_eq!(decoded.args, original.args);
//         assert_eq!(size, original.total_size());
//
//         // Vérifier que les arguments sont correctement extraits
//         if let Ok(ArgValue::Register(rd)) = decoded.get_arg1_value() {
//             assert_eq!(rd, 2);
//         } else {
//             panic!("Failed to get destination register value");
//         }
//
//         if let Ok(ArgValue::Register(rs1)) = decoded.get_arg2_value() {
//             assert_eq!(rs1, 3);
//         } else {
//             panic!("Failed to get first source register value");
//         }
//
//         if let Ok(ArgValue::Register(rs2)) = decoded.get_arg3_value() {
//             assert_eq!(rs2, 4);
//         } else {
//             panic!("Failed to get second source register value");
//         }
//     }
//
//     #[test]
//     fn test_get_argument_values_2() {
//         // Test pour les instructions à 3 registres
//         let add = Instruction::create_reg_reg_reg(Opcode::Add, 1, 2, 3);
//         assert_eq!(add.get_arg1_value().unwrap(), ArgValue::Register(1));
//         assert_eq!(add.get_arg2_value().unwrap(), ArgValue::Register(2));
//         assert_eq!(add.get_arg3_value().unwrap(), ArgValue::Register(3));
//
//         // Test pour les instructions reg + imm8
//         let load = Instruction::create_reg_imm8(Opcode::Load, 1, 42);
//         assert_eq!(load.get_arg1_value().unwrap(), ArgValue::Register(1));
//         assert_eq!(load.get_arg2_value().unwrap(), ArgValue::Immediate(42));
//     }
//
//     #[test]
//     fn test_get_argument_values() {
//         // Test avec un registre unique
//         let instr1 = Instruction::create_single_reg(Opcode::Inc, 3);
//
//         if let Ok(ArgValue::Register(r1)) = instr1.get_arg1_value() {
//             assert_eq!(r1, 3);
//         } else {
//             panic!("Failed to get register value");
//         }
//
//         // Test avec deux registres
//         let instr2 = Instruction::create_reg_reg(Opcode::Add, 3, 5);
//
//         // Vérifier que les arguments sont correctement stockés
//         assert_eq!(instr2.args.len(), 2);
//         assert_eq!(instr2.args[0], 3);
//         assert_eq!(instr2.args[1], 5);
//
//         // Tester get_arg1_value
//         if let Ok(ArgValue::Register(r1)) = instr2.get_arg1_value() {
//             assert_eq!(r1, 3);
//         } else {
//             panic!("Failed to get first register value");
//         }
//
//         // Tester get_arg2_value
//         if let Ok(ArgValue::Register(r2)) = instr2.get_arg2_value() {
//             assert_eq!(r2, 5);
//         } else {
//             panic!("Failed to get second register value");
//         }
//
//         // Test avec valeur immédiate
//         let instr3 = Instruction::create_reg_imm8(Opcode::Load, 2, 123);
//
//         if let Ok(ArgValue::Register(r)) = instr3.get_arg1_value() {
//             assert_eq!(r, 2);
//         } else {
//             panic!("Failed to get register value");
//         }
//
//         if let Ok(ArgValue::Immediate(imm)) = instr3.get_arg2_value() {
//             assert_eq!(imm, 123);
//         } else {
//             panic!("Failed to get immediate value");
//         }
//     }
//     //
//     #[test]
//     fn test_get_argument_values_reg_reg_reg() {
//         // Test avec trois registres pour les opérations arithmétiques
//         let instr = Instruction::create_reg_reg_reg(Opcode::Add, 2, 3, 4);
//
//         // Vérifier que les arguments sont correctement stockés
//         assert_eq!(instr.args.len(), 3);
//         assert_eq!(instr.args[0], 2);
//         assert_eq!(instr.args[1], 3);
//         assert_eq!(instr.args[2], 4);
//
//         // Tester get_arg1_value (registre destination)
//         if let Ok(ArgValue::Register(rd)) = instr.get_arg1_value() {
//             assert_eq!(rd, 2);
//         } else {
//             panic!("Failed to get destination register value");
//         }
//
//         // Tester get_arg2_value (premier registre source)
//         if let Ok(ArgValue::Register(rs1)) = instr.get_arg2_value() {
//             assert_eq!(rs1, 3);
//         } else {
//             panic!("Failed to get first source register value");
//         }
//
//         // Tester get_arg3_value (deuxième registre source)
//         if let Ok(ArgValue::Register(rs2)) = instr.get_arg3_value() {
//             assert_eq!(rs2, 4);
//         } else {
//             panic!("Failed to get second source register value");
//         }
//     }
//
//     #[test]
//     fn test_create_helper_functions() {
//         // Test les fonctions helper pour créer différents types d'instructions
//
//         // Instruction sans arguments
//         let instr1 = Instruction::create_no_args(Opcode::Nop);
//         assert_eq!(instr1.opcode, Opcode::Nop);
//         assert_eq!(instr1.args.len(), 0);
//
//         // Instruction avec un seul registre
//         let instr2 = Instruction::create_single_reg(Opcode::Inc, 7);
//         assert_eq!(instr2.opcode, Opcode::Inc);
//         assert_eq!(instr2.args.len(), 1);
//         assert_eq!(instr2.args[0], 7);
//
//         // Instruction avec deux registres
//         let instr3 = Instruction::create_reg_reg(Opcode::Add, 3, 4);
//         assert_eq!(instr3.opcode, Opcode::Add);
//         assert_eq!(instr3.args.len(), 2);
//         assert_eq!(instr3.args[0], 3);
//         assert_eq!(instr3.args[1], 4);
//
//         // Instruction avec trois registres
//         let instr4 = Instruction::create_reg_reg_reg(Opcode::Add, 2, 3, 4);
//         assert_eq!(instr4.opcode, Opcode::Add);
//         assert_eq!(instr4.args.len(), 3);
//         assert_eq!(instr4.args[0], 2);
//         assert_eq!(instr4.args[1], 3);
//         assert_eq!(instr4.args[2], 4);
//
//         // Instruction avec registre et immédiat 8-bit
//         let instr5 = Instruction::create_reg_imm8(Opcode::Load, 2, 42);
//         assert_eq!(instr5.opcode, Opcode::Load);
//         assert_eq!(instr5.args.len(), 2);
//         assert_eq!(instr5.args[0], 2);
//         assert_eq!(instr5.args[1], 42);
//     }
//
//     #[test]
//     fn test_arithmetic_instruction_creation() {
//         // Test ADD R2, R0, R1
//         let add_instr = Instruction::create_reg_reg_reg(Opcode::Add, 2, 0, 1);
//         assert_eq!(add_instr.opcode, Opcode::Add);
//         assert_eq!(add_instr.format.arg1_type, ArgType::Register);
//         assert_eq!(add_instr.format.arg2_type, ArgType::Register);
//         assert_eq!(add_instr.format.arg3_type, ArgType::Register);
//         assert_eq!(add_instr.args, vec![2, 0, 1]);
//
//         // Test SUB R3, R0, R1
//         let sub_instr = Instruction::create_reg_reg_reg(Opcode::Sub, 3, 0, 1);
//         assert_eq!(sub_instr.opcode, Opcode::Sub);
//         assert_eq!(sub_instr.args, vec![3, 0, 1]);
//
//         // Test MUL R4, R0, R1
//         let mul_instr = Instruction::create_reg_reg_reg(Opcode::Mul, 4, 0, 1);
//         assert_eq!(mul_instr.opcode, Opcode::Mul);
//         assert_eq!(mul_instr.args, vec![4, 0, 1]);
//
//         // Test DIV R5, R0, R1
//         let div_instr = Instruction::create_reg_reg_reg(Opcode::Div, 5, 0, 1);
//         assert_eq!(div_instr.opcode, Opcode::Div);
//         assert_eq!(div_instr.args, vec![5, 0, 1]);
//     }
//
//     #[test]
//     fn test_format_encoding_decoding() {
//         // Tester l'encodage et le décodage du format à 2 octets
//         let format = InstructionFormat::reg_reg_reg();
//         let encoded = format.encode();
//         let decoded = InstructionFormat::decode(encoded).unwrap();
//
//         assert_eq!(decoded.arg1_type, ArgType::Register);
//         assert_eq!(decoded.arg2_type, ArgType::Register);
//         assert_eq!(decoded.arg3_type, ArgType::Register);
//     }
//
//     #[test]
//     fn test_error_conditions() {
//         // Test de décodage avec données insuffisantes
//         let result = Instruction::decode(&[0x01]);
//         assert!(result.is_err());
//
//         if let Err(e) = result {
//             assert_eq!(e, DecodeError::InsufficientData);
//         }
//
//         // Test de décodage avec opcode invalide
//         let result = Instruction::decode(&[0xFF, 0x00, 0x00, 0x03]);
//         assert!(result.is_err());
//
//         if let Err(e) = result {
//             match e {
//                 DecodeError::InvalidOpcode(_) => (), // Expected
//                 _ => panic!("Unexpected error type"),
//             }
//         }
//
//         // Test de décodage avec format invalide
//         let result = Instruction::decode(&[0x01, 0xFF, 0xFF, 0x03]);
//         assert!(result.is_err());
//
//         if let Err(e) = result {
//             match e {
//                 DecodeError::InvalidFormat(_) => (), // Expected
//                 _ => panic!("Unexpected error type"),
//             }
//         }
//     }
//
//     #[test]
//     fn test_extended_size_encoding() {
//         // Créer une instruction avec suffisamment d'arguments pour forcer un encodage Extended
//         let large_args = vec![0; 254]; // Suffisant pour dépasser la limite de 255 octets
//
//         // Création de l'instruction avec un format simple
//         let instr = Instruction::new(Opcode::Add, InstructionFormat::reg_reg(), large_args);
//
//         // Vérifier qu'elle est bien en mode Extended
//         assert_eq!(instr.size_type, SizeType::Extended);
//
//         // Encoder l'instruction
//         let encoded = instr.encode();
//
//         // Vérifier que l'encodage est correct
//         assert_eq!(encoded[3], 0xFF); // Marqueur du format Extended
//
//         // Décoder l'instruction encodée
//         let (decoded, size) = Instruction::decode(&encoded).unwrap();
//
//         // Vérifier que le décodage a bien identifié le format Extended
//         assert_eq!(decoded.size_type, SizeType::Extended);
//         assert_eq!(size, instr.total_size());
//     }
//
//     #[test]
//     fn test_args_size_calculation() {
//         // Vérifier que le calcul de la taille des arguments est correct
//         let format = InstructionFormat::reg_reg_reg();
//         let args_size = format.args_size();
//         assert_eq!(args_size, 3); // 1 octet par registre
//
//         let format2 = InstructionFormat::reg_imm8();
//         let args_size2 = format2.args_size();
//         assert_eq!(args_size2, 2); // 1 octet pour registre + 1 octet pour immédiat
//     }
//
//     #[test]
//     fn test_jump_instructions() {
//         let from_addr = 0x1000;
//         let to_addr = 0x1020;
//         let instr_size = 8;
//         let offset = Instruction::calculate_branch_offset(from_addr, to_addr, instr_size);
//
//         // Créer un vecteur de tuples (instruction, opcode attendu)
//         let jumps = vec![
//             (Instruction::create_jump(from_addr, to_addr), Opcode::Jmp),
//             (Instruction::create_jump_if(from_addr, to_addr), Opcode::JmpIf),
//             (Instruction::create_jump_if_zero(from_addr, to_addr), Opcode::JmpIfZero),
//             (Instruction::create_jump_if_not_zero(from_addr, to_addr), Opcode::JmpIfNotZero)
//         ];
//
//         for (instr, expected_opcode) in jumps {
//             assert_eq!(instr.opcode, expected_opcode);
//
//             // Vérifier que l'argument est bien un offset relatif
//             if let Ok(ArgValue::Immediate(addr)) = instr.get_arg1_value() {
//                 assert_eq!(addr as i32, offset);
//             } else {
//                 panic!("Expected Immediate argument for jump offset");
//             }
//
//             // Vérifier la taille de l'instruction
//             assert_eq!(instr.total_size(), instr_size as usize);
//
//             // Vérifier l'encodage/décodage
//             let encoded = instr.encode();
//             let (decoded, size) = Instruction::decode(&encoded).unwrap();
//
//             assert_eq!(decoded.opcode, instr.opcode);
//             assert_eq!(size, instr.total_size());
//
//             if let Ok(ArgValue::Immediate(decoded_offset)) = decoded.get_arg1_value() {
//                 assert_eq!(decoded_offset as i32, offset);
//             } else {
//                 panic!("Expected Immediate argument after decode");
//             }
//         }
//     }
//     // //
//     // #[test]
//     // fn test_jump_instructions() {
//     //     // Test de création d'instruction de saut
//     //     let offset = 42;
//     //     let jump_instr = Instruction::create_jump(offset);
//     //     assert_eq!(jump_instr.opcode, Opcode::Jmp);
//     //     assert_eq!(jump_instr.args.len(), 4); // 4 octets pour l'offset
//     //
//     //     // Test de création d'instruction de saut conditionnel
//     //     let jump_if_instr = Instruction::create_jump_if(offset);
//     //     assert_eq!(jump_if_instr.opcode, Opcode::JmpIf);
//     //     assert_eq!(jump_if_instr.args.len(), 4); // 4 octets pour l'offset
//     // }
//     // //
//     #[test]
//     fn test_jump_if() {
//         // Test de création d'instruction de saut conditionnel
//         // let offset = 42;
//         let from_addr = 0x1000;
//         let to_addr = 0x1020;
//         // let jump_if_instr = Instruction::create_jump_if(offset);
//         let jump_if_instr = Instruction::create_jump_if(from_addr, to_addr);
//         assert_eq!(jump_if_instr.opcode, Opcode::JmpIf);
//         assert_eq!(jump_if_instr.args.len(), 4); // 4 octets pour l'offset
//     }
//
//     #[test]
//     fn test_jump_if_not() {
//         // Test de création d'instruction de saut conditionnel
//         // let offset = 42;
//         let from_addr = 0x1000;
//         let to_addr = 0x1020;
//         // let jump_if_not_instr = Instruction::create_jump_if_not(offset);
//         let jump_if_not_instr = Instruction::create_jump_if_not(from_addr, to_addr);
//         assert_eq!(jump_if_not_instr.opcode, Opcode::JmpIfNot);
//         assert_eq!(jump_if_not_instr.args.len(), 4); // 4 octets pour l'offset
//     }
//
//     #[test]
//     fn test_jump_if_equal() {
//         // Test de création d'instruction de saut conditionnel
//         // let offset = 42;
//         let from_addr = 0x1000;
//         let to_addr = 0x1020;
//         // let jump_if_equal_instr = Instruction::create_jump_if_equal(offset);
//         let jump_if_equal_instr = Instruction::create_jump_if_equal(from_addr, to_addr);
//         assert_eq!(jump_if_equal_instr.opcode, Opcode::JmpIfEqual);
//
//         assert_eq!(jump_if_equal_instr.args.len(), 4); // 4 octets pour l'offset
//     }
//
//     #[test]
//     fn test_jump_if_not_equal() {
//         // Test de création d'instruction de saut conditionnel
//         // let offset = 42;
//         let from_addr = 0x1000;
//         let to_addr = 0x1020;
//         // let jump_if_not_equal_instr = Instruction::create_jump_if_not_equal(offset);
//         let jump_if_not_equal_instr = Instruction::create_jump_if_not_equal(from_addr, to_addr);
//         assert_eq!(jump_if_not_equal_instr.opcode, Opcode::JmpIfNotEqual);
//         assert_eq!(jump_if_not_equal_instr.args.len(), 4); // 4 octets pour l'offset
//     }
//
//     #[test]
//     fn test_jump_if_greater() {
//         // Test de création d'instruction de saut conditionnel
//         // let offset = 42;
//         let from_addr = 0x1000;
//         let to_addr = 0x1020;
//         // let jump_if_greater_instr = Instruction::create_jump_if_greater(offset);
//         let jump_if_greater_instr = Instruction::create_jump_if_greater(from_addr, to_addr);
//         assert_eq!(jump_if_greater_instr.opcode, Opcode::JmpIfGreater);
//         assert_eq!(jump_if_greater_instr.args.len(), 4); // 4 octets pour l'offset
//     }
//
//     #[test]
//     fn test_jump_if_greater_equal() {
//         // Test de création d'instruction de saut conditionnel
//         // let offset = 42;
//         let from_addr = 0x1000;
//         let to_addr = 0x1020;
//         // let jump_if_greater_equal_instr = Instruction::create_jump_if_greater_equal(offset);
//         let jump_if_greater_equal_instr = Instruction::create_jump_if_greater_equal(from_addr, to_addr);
//         assert_eq!(
//             jump_if_greater_equal_instr.opcode,
//             Opcode::JmpIfGreaterEqual
//         );
//         assert_eq!(jump_if_greater_equal_instr.args.len(), 4); // 4 octets pour l'offset
//     }
//
//     #[test]
//     fn test_jump_if_less() {
//         // Test de création d'instruction de saut conditionnel
//         // let offset = 42;
//         let from_addr = 0x1000;
//         let to_addr = 0x1020;
//         // let jump_if_less_instr = Instruction::create_jump_if_less(offset);
//         let jump_if_less_instr = Instruction::create_jump_if_less(from_addr, to_addr);
//         assert_eq!(jump_if_less_instr.opcode, Opcode::JmpIfLess);
//         assert_eq!(jump_if_less_instr.args.len(), 4); // 4 octets pour l'offset
//     }
//
//     #[test]
//     fn test_jump_if_less_equal() {
//         // Test de création d'instruction de saut conditionnel
//         // let offset = 42;
//         let from_addr = 0x1000;
//         let to_addr = 0x1020;
//         // let jump_if_less_equal_instr = Instruction::create_jump_if_less_equal(offset);
//         let jump_if_less_equal_instr = Instruction::create_jump_if_less_equal(from_addr, to_addr);
//         assert_eq!(jump_if_less_equal_instr.opcode, Opcode::JmpIfLessEqual);
//         assert_eq!(jump_if_less_equal_instr.args.len(), 4); // 4 octets pour l'offset
//     }
//
//     #[test]
//     fn test_jump_above() {
//         // Test de création d'instruction de saut conditionnel
//         // let offset = 42;
//         let from_addr = 0x1000;
//         let to_addr = 0x1020;
//         // let jump_above_instr = Instruction::create_jump_if_above(offset);
//         // let jump_above_instr = Instruction::create_jump_if_above(from_addr, to_addr);
//         let jump_above_inst = Instruction::create_jump_if_above(from_addr, to_addr);
//         assert_eq!(jump_above_inst.opcode, Opcode::JmpIfAbove);
//         assert_eq!(jump_above_inst.args.len(), 4); // 4 octets pour l'offset
//     }
//     #[test]
//     fn test_jump_above_equal() {
//         // Test de création d'instruction de saut conditionnel
//         // let offset = 42;
//         let from_addr = 0x1000;
//         let to_addr = 0x1020;
//         // let jump_above_equal_instr = Instruction::create_jump_if_above_equal(offset);
//         let jump_above_equal_instr = Instruction::create_jump_if_above_equal(from_addr, to_addr);
//         assert_eq!(jump_above_equal_instr.opcode, Opcode::JmpIfAboveEqual);
//         assert_eq!(jump_above_equal_instr.args.len(), 4); // 4 octets pour l'offset
//     }
//
//     #[test]
//     fn test_jump_below() {
//         // Test de création d'instruction de saut conditionnel
//         // let offset = 42;
//         let from_addr = 0x1000;
//         let to_addr = 0x1020;
//         // let jump_below_instr = Instruction::create_jump_below(offset);
//         let jump_below_instr = Instruction::create_jump_below(from_addr, to_addr);
//         assert_eq!(jump_below_instr.opcode, Opcode::JmpIfBelow);
//         assert_eq!(jump_below_instr.args.len(), 4); // 4 octets pour l'offset
//     }
//
//     #[test]
//     fn test_jump_below_equal() {
//         // Test de création d'instruction de saut conditionnel
//         // let offset = 42;
//         let from_addr = 0x1000;
//         let to_addr = 0x1020;
//         // let jump_below_equal_instr = Instruction::create_jump_if_below_equal(offset);
//         let jump_below_equal_instr = Instruction::create_jump_if_below_equal(from_addr, to_addr);
//         assert_eq!(jump_below_equal_instr.opcode, Opcode::JmpIfBelowEqual);
//         assert_eq!(jump_below_equal_instr.args.len(), 4); // 4 octets pour l'offset
//     }
//
//     #[test]
//     fn test_jump_zero() {
//         // Test de création d'instruction de saut conditionnel
//         // let offset = 42;
//         let from_addr = 0x1000;
//         let to_addr = 0x1020;
//         // let jump_zero_instr = Instruction::create_jump_if_zero(offset);
//         let jump_zero_instr = Instruction::create_jump_if_zero(from_addr, to_addr);
//         assert_eq!(jump_zero_instr.opcode, Opcode::JmpIfZero);
//         assert_eq!(jump_zero_instr.args.len(), 4); // 4 octets pour l'offset
//     }
//
//     #[test]
//     fn test_jump_not_zero() {
//         // Test de création d'instruction de saut conditionnel
//         // let offset = 42;
//         let from_addr = 0x1000;
//         let to_addr = 0x1020;
//         // let jump_not_zero_instr = Instruction::create_jump_if_not_zero(offset);
//         let jump_not_zero_instr = Instruction::create_jump_if_not_zero(from_addr, to_addr);
//         assert_eq!(jump_not_zero_instr.opcode, Opcode::JmpIfNotZero);
//         assert_eq!(jump_not_zero_instr.args.len(), 4); // 4 octets pour l'offset
//     }
//
//     #[test]
//     fn test_jump_if_overflow() {
//         // Test de création d'instruction de saut conditionnel
//         // let offset = 42;
//         let from_addr = 0x1000;
//         let to_addr = 0x1020;
//         // let jump_if_overflow_instr = Instruction::create_jump_if_overflow(offset);
//         let jump_if_overflow_instr = Instruction::create_jump_if_overflow(from_addr, to_addr);
//         assert_eq!(jump_if_overflow_instr.opcode, Opcode::JmpIfOverflow);
//         assert_eq!(jump_if_overflow_instr.args.len(), 4); // 4 octets pour l'offset
//     }
//
//     #[test]
//     fn test_jump_if_not_overflow() {
//         // Test de création d'instruction de saut conditionnel
//         // let offset = 42;
//         let from_addr = 0x1000;
//         let to_addr = 0x1020;
//         // let jump_if_not_overflow_instr = Instruction::create_jump_if_not_overflow(offset);
//         let jump_if_not_overflow_instr = Instruction::create_jump_if_not_overflow(from_addr, to_addr);
//
//
//         assert_eq!(jump_if_not_overflow_instr.opcode, Opcode::JmpIfNotOverflow);
//         assert_eq!(jump_if_not_overflow_instr.args.len(), 4); // 4 octets pour l'offset
//     }
//
//     #[test]
//     fn test_jump_if_positive() {
//         // Test de création d'instruction de saut conditionnel
//         // let offset = 42;
//         let from_addr = 0x1000;
//         let to_addr = 0x1020;
//         // let jump_if_positive_instr = Instruction::create_jump_if_positive(offset);
//         let jump_if_positive_instr = Instruction::create_jump_if_positive(from_addr, to_addr);
//         assert_eq!(jump_if_positive_instr.opcode, Opcode::JmpIfPositive);
//         assert_eq!(jump_if_positive_instr.args.len(), 4); // 4 octets pour l'offset
//     }
//
//     #[test]
//     fn test_push_register(){
//         // Test de création d'instruction PUSH
//         let reg = 5;
//         let push_instr = Instruction::create_push_register(reg);
//         assert_eq!(push_instr.opcode, Opcode::Push);
//         assert_eq!(push_instr.format.arg1_type, ArgType::Register);
//         assert_eq!(push_instr.args.len(), 1); // 1 registre à pousser
//         assert_eq!(push_instr.args[0], reg);
//
//     }
//
//     #[test]
//     fn test_pop_register() {
//         // Test de création d'instruction POP
//         let reg = 5;
//         let pop_instr = Instruction::create_pop_register(reg);
//         assert_eq!(pop_instr.opcode, Opcode::Pop);
//         assert_eq!(pop_instr.format.arg1_type, ArgType::Register);
//         assert_eq!(pop_instr.args.len(), 1); // 1 registre à dépiler
//         assert_eq!(pop_instr.args[0], reg);
//     }
//
//     // Je suis pas trop sûr de la pertinence de ce test, mais je le laisse pour l'instant
//     #[test]
//     fn test_call_function() {
//         // Test de création d'instruction CALL
//         let func_addr = 0x1234;
//         let call_intr = Instruction::create_call(func_addr);
//         assert_eq!(call_intr.opcode, Opcode::Call);
//         assert_eq!(call_intr.format.arg2_type, ArgType::AbsoluteAddr);
//         assert_eq!(call_intr.args.len(), 4); // 1 adresse de fonction
//
//
//       ;
//     }
//
//     #[test]
//     fn test_return(){
//         let ret_instr = Instruction::create_return();
//         assert_eq!(ret_instr.opcode, Opcode::Ret);
//         assert_eq!(ret_instr.format.arg1_type, ArgType::None);
//         assert_eq!(ret_instr.args.len(), 0); // Pas d'arguments pour RET
//         assert_eq!(ret_instr.total_size(), 4); // Taille de l'instruction RET
//     }
//
//
// }

///////////////////////////////////////////////////////////////


//
//
//
// // Test unitaire pour les fichiers de bytecode
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::bytecode::files::{BytecodeVersion, SegmentMetadata, SegmentType};
//     use crate::bytecode::format::{ArgType, InstructionFormat};
//     use crate::bytecode::instructions::Instruction;
//     use crate::bytecode::opcodes::Opcode;
//     use crate::BytecodeFile;
//     use std::collections::HashMap;
//     use std::io::ErrorKind;
//     use tempfile::tempdir;
//
//     #[test]
//     fn test_bytecode_version() {
//         let version = BytecodeVersion::new(1, 2, 3, 4);
//
//         assert_eq!(version.major, 1);
//         assert_eq!(version.minor, 2);
//         assert_eq!(version.patch, 3);
//         assert_eq!(version.build, 4);
//
//         // Test encode/decode
//         let encoded = version.encode();
//         let decoded = BytecodeVersion::decode(encoded);
//
//         assert_eq!(decoded.major, 1);
//         assert_eq!(decoded.minor, 2);
//         assert_eq!(decoded.patch, 3);
//         assert_eq!(decoded.build, 4);
//
//         // Test to_string
//         assert_eq!(version.to_string(), "1.2.3.4");
//     }
//
//     #[test]
//     fn test_segment_type() {
//         // Test des conversions valides
//         assert_eq!(SegmentType::from_u8(0), Some(SegmentType::Code));
//         assert_eq!(SegmentType::from_u8(1), Some(SegmentType::Data));
//         assert_eq!(SegmentType::from_u8(2), Some(SegmentType::ReadOnlyData));
//         assert_eq!(SegmentType::from_u8(3), Some(SegmentType::Symbols));
//         assert_eq!(SegmentType::from_u8(4), Some(SegmentType::Debug));
//
//         // Test avec valeur invalide
//         assert_eq!(SegmentType::from_u8(5), None);
//     }
//
//     #[test]
//     fn test_segment_metadata() {
//         let segment = SegmentMetadata::new(SegmentType::Code, 100, 200, 300);
//
//         assert_eq!(segment.segment_type, SegmentType::Code);
//         assert_eq!(segment.offset, 100);
//         assert_eq!(segment.size, 200);
//         assert_eq!(segment.load_addr, 300);
//
//         // Test encode/decode
//         let encoded = segment.encode();
//         let decoded = SegmentMetadata::decode(&encoded).unwrap();
//
//         assert_eq!(decoded.segment_type, SegmentType::Code);
//         assert_eq!(decoded.offset, 100);
//         assert_eq!(decoded.size, 200);
//         assert_eq!(decoded.load_addr, 300);
//     }
//
//     #[test]
//     fn test_bytecode_file_simple() {
//         // Création d'un fichier bytecode simple
//         let mut bytecode = BytecodeFile::new();
//
//         // Ajout de métadonnées
//         bytecode.add_metadata("name", "Test");
//         bytecode.add_metadata("author", "PunkVM");
//
//         // Vérification
//         assert_eq!(bytecode.metadata.get("name"), Some(&"Test".to_string()));
//         assert_eq!(bytecode.metadata.get("author"), Some(&"PunkVM".to_string()));
//
//         // Ajout d'instructions
//         let instr1 = Instruction::create_no_args(Opcode::Nop);
//         let instr2 = Instruction::create_reg_imm8(Opcode::Load, 0, 42);
//
//         bytecode.add_instruction(instr1);
//         bytecode.add_instruction(instr2);
//
//         assert_eq!(bytecode.code.len(), 2);
//         assert_eq!(bytecode.code[0].opcode, Opcode::Nop);
//         assert_eq!(bytecode.code[1].opcode, Opcode::Load);
//
//         // Ajout de données
//         let offset = bytecode.add_data(&[1, 2, 3, 4]);
//         assert_eq!(offset, 0);
//         assert_eq!(bytecode.data, vec![1, 2, 3, 4]);
//
//         // Ajout de données en lecture seule
//         let offset = bytecode.add_readonly_data(&[5, 6, 7, 8]);
//         assert_eq!(offset, 0);
//         assert_eq!(bytecode.readonly_data, vec![5, 6, 7, 8]);
//
//         // Ajout de symboles
//         bytecode.add_symbol("main", 0x1000);
//         assert_eq!(bytecode.symbols.get("main"), Some(&0x1000));
//     }
//
//     #[test]
//     fn test_bytecode_file_with_arithmetic_instructions() {
//         // Création d'un fichier bytecode avec des instructions arithmétiques
//         let mut bytecode = BytecodeFile::new();
//
//         // Ajouter des instructions arithmétiques avec le nouveau format à 3 registres
//         let instr1 = Instruction::create_reg_reg_reg(Opcode::Add, 2, 0, 1); // R2 = R0 + R1
//         let instr2 = Instruction::create_reg_reg_reg(Opcode::Sub, 3, 0, 1); // R3 = R0 - R1
//         let instr3 = Instruction::create_reg_reg_reg(Opcode::Mul, 4, 0, 1); // R4 = R0 * R1
//
//         bytecode.add_instruction(instr1);
//         bytecode.add_instruction(instr2);
//         bytecode.add_instruction(instr3);
//
//         assert_eq!(bytecode.code.len(), 3);
//
//         // Vérifier le premier opcode
//         assert_eq!(bytecode.code[0].opcode, Opcode::Add);
//
//         // Vérifier les types d'arguments du format
//         assert_eq!(bytecode.code[0].format.arg1_type, ArgType::Register);
//         assert_eq!(bytecode.code[0].format.arg2_type, ArgType::Register);
//         assert_eq!(bytecode.code[0].format.arg3_type, ArgType::Register);
//
//         // Vérifier les valeurs des registres
//         assert_eq!(bytecode.code[0].args[0], 2); // Rd (destination)
//         assert_eq!(bytecode.code[0].args[1], 0); // Rs1 (source 1)
//         assert_eq!(bytecode.code[0].args[2], 1); // Rs2 (source 2)
//     }
//
//     #[test]
//     fn test_bytecode_file_io() {
//         // Création d'un répertoire temporaire pour les tests
//         let dir = tempdir().expect("Impossible de créer un répertoire temporaire");
//         let file_path = dir.path().join("test.punk");
//
//         // Création d'un fichier bytecode à écrire
//         let mut bytecode = BytecodeFile::new();
//         bytecode.version = BytecodeVersion::new(1, 0, 0, 0);
//         bytecode.add_metadata("name", "TestIO");
//         bytecode.add_instruction(Instruction::create_no_args(Opcode::Halt));
//         bytecode.add_data(&[1, 2, 3]);
//         bytecode.add_readonly_data(&[4, 5, 6]);
//         bytecode.add_symbol("main", 0);
//
//         // Écrire le fichier
//         bytecode
//             .write_to_file(&file_path)
//             .expect("Impossible d'écrire le fichier bytecode");
//
//         // Lire le fichier
//         let loaded = BytecodeFile::read_from_file(&file_path)
//             .expect("Impossible de lire le fichier bytecode");
//
//         // Vérifier que le contenu est identique
//         assert_eq!(loaded.version.major, 1);
//         assert_eq!(loaded.version.minor, 0);
//         assert_eq!(loaded.metadata.get("name"), Some(&"TestIO".to_string()));
//         assert_eq!(loaded.code.len(), 1);
//         assert_eq!(loaded.code[0].opcode, Opcode::Halt);
//         assert_eq!(loaded.data, vec![1, 2, 3]);
//         assert_eq!(loaded.readonly_data, vec![4, 5, 6]);
//         assert_eq!(loaded.symbols.get("main"), Some(&0));
//     }
//
//     #[test]
//     fn test_bytecode_file_with_three_register_instructions_io() {
//         // Test d'écriture et lecture d'un fichier contenant des instructions à 3 registres
//         let dir = tempdir().expect("Impossible de créer un répertoire temporaire");
//         let file_path = dir.path().join("test_three_reg.punk");
//
//         // Création du fichier bytecode
//         let mut bytecode = BytecodeFile::new();
//         bytecode.version = BytecodeVersion::new(1, 0, 0, 0);
//
//         // Ajouter une instruction ADD avec 3 registres
//         let add_instr = Instruction::create_reg_reg_reg(Opcode::Add, 2, 0, 1); // R2 = R0 + R1
//         bytecode.add_instruction(add_instr);
//
//         // Écrire le fichier
//         bytecode
//             .write_to_file(&file_path)
//             .expect("Impossible d'écrire le fichier bytecode");
//
//         // Lire le fichier
//         let loaded = BytecodeFile::read_from_file(&file_path)
//             .expect("Impossible de lire le fichier bytecode");
//
//         // Vérifier que l'instruction est correctement chargée
//         assert_eq!(loaded.code.len(), 1);
//         assert_eq!(loaded.code[0].opcode, Opcode::Add);
//
//         // Vérifier les valeurs des registres
//         assert_eq!(loaded.code[0].args.len(), 3);
//         assert_eq!(loaded.code[0].args[0], 2); // Rd
//         assert_eq!(loaded.code[0].args[1], 0); // Rs1
//         assert_eq!(loaded.code[0].args[2], 1); // Rs2
//     }
//
//     #[test]
//     fn test_bytecode_file_extended_size_io() {
//         // Test d'écriture et lecture d'un fichier avec une instruction de grande taille
//         let dir = tempdir().expect("Impossible de créer un répertoire temporaire");
//         let file_path = dir.path().join("test_extended.punk");
//
//         // Création du fichier bytecode
//         let mut bytecode = BytecodeFile::new();
//
//         // Créer une instruction avec beaucoup de données pour forcer un size_type Extended
//         let large_args = vec![0; 248]; // Suffisant pour dépasser la limite de 255 octets
//         let large_instr =
//             Instruction::new(Opcode::Add, InstructionFormat::double_reg(), large_args);
//
//         bytecode.add_instruction(large_instr);
//
//         // Écrire le fichier
//         bytecode
//             .write_to_file(&file_path)
//             .expect("Impossible d'écrire le fichier bytecode");
//
//         // Lire le fichier
//         let loaded = BytecodeFile::read_from_file(&file_path)
//             .expect("Impossible de lire le fichier bytecode");
//
//         // Vérifier que l'instruction est correctement chargée
//         assert_eq!(loaded.code.len(), 1);
//         assert_eq!(loaded.code[0].opcode, Opcode::Add);
//         assert_eq!(loaded.code[0].args.len(), 248);
//     }
//
//     #[test]
//     fn test_bytecode_file_complex_program() {
//         // Créer un programme complet avec des instructions variées
//         let mut bytecode = BytecodeFile::new();
//
//         // Ajouter des métadonnées
//         bytecode.add_metadata("name", "Programme de test");
//         bytecode.add_metadata("author", "PunkVM Team");
//         bytecode.add_metadata("version", "1.0.0");
//
//         // Initialisation des registres
//         bytecode.add_instruction(Instruction::create_reg_imm8(Opcode::Load, 0, 10)); // R0 = 10
//         bytecode.add_instruction(Instruction::create_reg_imm8(Opcode::Load, 1, 5)); // R1 = 5
//
//         // Opérations arithmétiques
//         bytecode.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 2, 0, 1)); // R2 = R0 + R1
//         bytecode.add_instruction(Instruction::create_reg_reg_reg(Opcode::Sub, 3, 0, 1)); // R3 = R0 - R1
//         bytecode.add_instruction(Instruction::create_reg_reg_reg(Opcode::Mul, 4, 0, 1)); // R4 = R0 * R1
//         bytecode.add_instruction(Instruction::create_reg_reg_reg(Opcode::Div, 5, 0, 1)); // R5 = R0 / R1
//
//         // Opérations logiques
//         bytecode.add_instruction(Instruction::create_reg_reg_reg(Opcode::And, 6, 0, 1)); // R6 = R0 & R1
//         bytecode.add_instruction(Instruction::create_reg_reg_reg(Opcode::Or, 7, 0, 1)); // R7 = R0 | R1
//
//         // Fin du programme
//         bytecode.add_instruction(Instruction::create_no_args(Opcode::Halt));
//
//         // Ajouter un symbole pour le début du programme
//         bytecode.add_symbol("start", 0);
//
//         // Vérifier le nombre d'instructions
//         assert_eq!(bytecode.code.len(), 9);
//
//         // Vérifier les métadonnées
//         assert_eq!(bytecode.metadata.len(), 3);
//         assert_eq!(
//             bytecode.metadata.get("name"),
//             Some(&"Programme de test".to_string())
//         );
//
//         // Vérifier les symboles
//         assert_eq!(bytecode.symbols.len(), 1);
//         assert_eq!(bytecode.symbols.get("start"), Some(&0));
//     }
//
//     #[test]
//     fn test_bytecode_file_io_errors() {
//         // Test avec un fichier inexistant
//         let result = BytecodeFile::read_from_file("nonexistent_file.punk");
//         assert!(result.is_err());
//
//         // Test avec un fichier trop petit
//         let dir = tempdir().expect("Impossible de créer un répertoire temporaire");
//         let invalid_file_path = dir.path().join("invalid.punk");
//
//         // Créer un fichier invalide avec juste quelques octets
//         std::fs::write(&invalid_file_path, &[0, 1, 2])
//             .expect("Impossible d'écrire le fichier de test");
//
//         let result = BytecodeFile::read_from_file(&invalid_file_path);
//         assert!(result.is_err());
//
//         // Vérifier le type d'erreur
//         match result {
//             Err(e) => assert_eq!(e.kind(), ErrorKind::InvalidData),
//             _ => panic!("Expected an error but got success"),
//         }
//     }
//
//     #[test]
//     fn test_encode_decode_metadata() {
//         let mut metadata = HashMap::new();
//         metadata.insert("key1".to_string(), "value1".to_string());
//         metadata.insert("key2".to_string(), "value2".to_string());
//
//         let mut bytecode = BytecodeFile::new();
//         bytecode.metadata = metadata.clone();
//
//         let encoded = bytecode.encode_metadata();
//         let decoded = BytecodeFile::decode_metadata(&encoded).expect("Failed to decode metadata");
//
//         assert_eq!(decoded.len(), 2);
//         assert_eq!(decoded.get("key1"), Some(&"value1".to_string()));
//         assert_eq!(decoded.get("key2"), Some(&"value2".to_string()));
//     }
//
//     #[test]
//     fn test_encode_decode_symbols() {
//         let mut symbols = HashMap::new();
//         symbols.insert("sym1".to_string(), 0x1000);
//         symbols.insert("sym2".to_string(), 0x2000);
//
//         let mut bytecode = BytecodeFile::new();
//         bytecode.symbols = symbols.clone();
//
//         let encoded = bytecode.encode_symbols();
//         let decoded = BytecodeFile::decode_symbols(&encoded).expect("Failed to decode symbols");
//
//         assert_eq!(decoded.len(), 2);
//         assert_eq!(decoded.get("sym1"), Some(&0x1000));
//         assert_eq!(decoded.get("sym2"), Some(&0x2000));
//     }
//
//     #[test]
//     fn test_encode_decode_code() {
//         // Créer un ensemble d'instructions de test
//         let mut code = Vec::new();
//         code.push(Instruction::create_no_args(Opcode::Nop));
//         code.push(Instruction::create_reg_imm8(Opcode::Load, 0, 42));
//         code.push(Instruction::create_reg_reg_reg(Opcode::Add, 2, 0, 1));
//
//         let mut bytecode = BytecodeFile::new();
//         bytecode.code = code.clone();
//
//         let encoded = bytecode.encode_code();
//         let decoded = BytecodeFile::decode_code(&encoded).expect("Failed to decode code");
//
//         assert_eq!(decoded.len(), 3);
//         assert_eq!(decoded[0].opcode, Opcode::Nop);
//         assert_eq!(decoded[1].opcode, Opcode::Load);
//         assert_eq!(decoded[2].opcode, Opcode::Add);
//
//         // Vérifier les arguments de l'instruction Add
//         assert_eq!(decoded[2].args.len(), 3);
//         assert_eq!(decoded[2].args[0], 2);
//         assert_eq!(decoded[2].args[1], 0);
//         assert_eq!(decoded[2].args[2], 1);
//     }
// }
//
//
// #[cfg(test)]
// mod pipeline_mod_tests {
//     use super::*; // Importe Pipeline et les structures du mod.rs
//     use crate::alu::alu::ALU;
//     use crate::bytecode::instructions::Instruction;
//     use crate::bytecode::opcodes::Opcode;
//     use crate::pvm::memorys::{Memory, MemoryConfig}; // Assurez-vous que les chemins sont corrects
//     use crate::pipeline::{FetchDecodeRegister, DecodeExecuteRegister, ExecuteMemoryRegister, MemoryWritebackRegister};
//
//     // Helper pour créer une VM minimale pour les tests de pipeline
//     fn setup_test_pipeline(instructions: Vec<Instruction>) -> (Pipeline, Vec<u64>, Memory, ALU) {
//         let pipeline = Pipeline::new(4, true, true); // Buffersize 4, forwarding/hazard ON
//         let registers = vec![0u64; 16];
//         let memory_config = MemoryConfig { size: 1024, l1_cache_size: 64, store_buffer_size: 4 };
//         let memory = Memory::new(memory_config);
//         let alu = ALU::new();
//         // Note: La mémoire n'est pas pré-chargée avec les instructions ici.
//         // La fonction `cycle` reçoit le slice `instructions` directement.
//         (pipeline, registers, memory, alu)
//     }
//
//     #[test]
//     fn test_pipeline_simple_add() {
//         // Instructions: ADD R2, R0, R1; HALT
//         let instructions = vec![
//             Instruction::create_reg_reg_reg(Opcode::Add, 2, 0, 1), // R2 = R0 + R1
//             Instruction::create_no_args(Opcode::Halt),
//         ];
//         let (mut pipeline, mut registers, mut memory, mut alu) = setup_test_pipeline(instructions.clone());
//
//         // Initialiser les registres
//         registers[0] = 10;
//         registers[1] = 5;
//
//         // Simuler les cycles
//         let mut current_pc: u32 = 0;
//         let max_cycles = 10; // Éviter boucle infinie
//         for _ in 0..max_cycles {
//             let result = pipeline.cycle(current_pc, &mut registers, &mut memory, &mut alu, &instructions);
//             assert!(result.is_ok(), "Le cycle du pipeline a échoué");
//             let state = result.unwrap();
//             current_pc = state.next_pc; // Mettre à jour le PC pour le prochain cycle
//             println!("Cycle: {}, PC: 0x{:X}, Halted: {}, Stalled: {}",
//                      pipeline.stats.cycles, current_pc, state.halted, state.stalled);
//             if state.halted {
//                 break;
//             }
//         }
//
//         assert!(pipeline.state.halted, "Le pipeline devrait être arrêté (halted)");
//         // ADD prend ~5 cycles pour compléter (F-D-E-M-W)
//         assert!(pipeline.stats.cycles >= 5, "Il faut au moins 5 cycles");
//         assert_eq!(registers[2], 15, "R2 devrait contenir 10 + 5"); // Vérifier le résultat
//         assert_eq!(pipeline.stats.instructions, 1, "Une seule instruction (ADD) devrait être complétée");
//     }
//
//     #[test]
//     fn test_pipeline_data_hazard_raw_forwarding() {
//         // Instructions: ADD R1, R0, R0; ADD R2, R1, R1; HALT
//         let instructions = vec![
//             Instruction::create_reg_reg_reg(Opcode::Add, 1, 0, 0), // R1 = R0 + R0 (R1=10)
//             Instruction::create_reg_reg_reg(Opcode::Add, 2, 1, 1), // R2 = R1 + R1 (dépend de R1)
//             Instruction::create_no_args(Opcode::Halt),
//         ];
//         let (mut pipeline, mut registers, mut memory, mut alu) = setup_test_pipeline(instructions.clone());
//         registers[0] = 5; // R0 = 5
//
//         // Simuler les cycles
//         let mut current_pc: u32 = 0;
//         let max_cycles = 15;
//         for _ in 0..max_cycles {
//             let result = pipeline.cycle(current_pc, &mut registers, &mut memory, &mut alu, &instructions);
//             assert!(result.is_ok());
//             let state = result.unwrap();
//             current_pc = state.next_pc;
//             if state.halted { break; }
//         }
//
//         assert!(pipeline.state.halted);
//         assert_eq!(registers[1], 10, "R1 devrait être 5 + 5");
//         assert_eq!(registers[2], 20, "R2 devrait être 10 + 10 (avec forwarding)");
//         assert!(pipeline.forwarding.get_forwards_count() >= 1, "Au moins un forward aurait dû se produire pour R1");
//         // Le nombre exact de cycles dépend si le forwarding évite complètement le stall
//         // S'il y a un stall d'un cycle malgré le forwarding: cycles = 5 (ADD1) + 1 (stall) + 1 (ADD2 complété) = 7?
//         // S'il n'y a pas de stall: cycles = 5 (ADD1) + 1 (ADD2 complété) = 6?
//         println!("Stats: Cycles={}, Instructions={}, Forwards={}, Stalls={}",
//                  pipeline.stats.cycles, pipeline.stats.instructions,
//                  pipeline.forwarding.get_forwards_count(), pipeline.stats.stalls);
//         assert_eq!(pipeline.stats.stalls, 0, "Aucun stall ne devrait être nécessaire avec forwarding EX->DE");
//         assert_eq!(pipeline.stats.instructions, 2);
//     }
//     //
//     #[test]
//     fn test_pipeline_load_use_hazard_stall() {
//         // Instructions: LOAD R1, [0]; ADD R2, R1, R0; HALT
//         let load_addr = 0x100; // Adresse arbitraire
//         let instructions = vec![
//             Instruction::create_reg_imm8(Opcode::Load, 1, 0), // LOAD R1, [0x100] (simplifié: addr dans immediate)
//             Instruction::create_reg_reg_reg(Opcode::Add, 2, 1, 0), // ADD R2, R1, R0 (utilise R1)
//             Instruction::create_no_args(Opcode::Halt),
//         ];
//         let (mut pipeline, mut registers, mut memory, mut alu) = setup_test_pipeline(instructions.clone());
//
//         // Mettre une valeur en mémoire
//         memory.write_qword(load_addr, 99).unwrap();
//         registers[0] = 1; // R0 = 1
//
//         // Simuler les cycles
//         let mut current_pc: u32 = 0;
//         let max_cycles = 15;
//         for i in 0..max_cycles {
//             println!("--- Test Cycle {} ---", i + 1);
//             let result = pipeline.cycle(current_pc, &mut registers, &mut memory, &mut alu, &instructions);
//             assert!(result.is_ok());
//             let state = result.unwrap();
//             current_pc = state.next_pc;
//             if state.halted { break; }
//         }
//
//         assert!(pipeline.state.halted);
//         assert_eq!(registers[1], 99, "R1 doit contenir la valeur chargée");
//         assert_eq!(registers[2], 100, "R2 doit être 99 + 1");
//         println!("Stats: Cycles={}, Instructions={}, Forwards={}, Stalls={}",
//                  pipeline.stats.cycles, pipeline.stats.instructions,
//                  pipeline.forwarding.get_forwards_count(), pipeline.stats.stalls);
//         assert!(pipeline.stats.stalls >= 1, "Au moins un stall est attendu pour Load-Use");
//         assert_eq!(pipeline.stats.instructions, 2);
//         assert!(pipeline.hazard_detection.hazards_count >= 1, "Au moins un hazard LoadUse détecté");
//     }
//
//
//
//
//     //
//     // #[test]
//     // fn test_pipeline_branch_taken_correctly_predicted() {
//     //     // JMP_IF_EQUAL target ; ADD R0, R0, 1 (ne devrait pas s'exécuter); target: HALT
//     //     let target_offset = 8; // Taille JIE + ADD = 4 + 4 = 8 ? Vérifier tailles!
//     //     let branch_instr = Instruction::create_jump_if_equal(target_offset);
//     //     let nop_instr = Instruction::create_no_args(Opcode::Nop); // Utiliser NOP pour simplicité taille
//     //     let halt_instr = Instruction::create_no_args(Opcode::Halt);
//     //
//     //     // Calcul des tailles réelles
//     //     let branch_size = branch_instr.total_size() as u32;
//     //     let nop_size = nop_instr.total_size() as u32;
//     //     // L'offset doit pointer *après* le NOP vers HALT
//     //     let correct_offset = nop_size as i32;
//     //     let branch_instr_corrected = Instruction::create_jump_if_equal(correct_offset);
//     //
//     //
//     //     let instructions = vec![
//     //         Instruction::create_reg_reg_reg(Opcode::Sub, 0, 0, 0), // SUB R0, R0, R0 => Met Z=1
//     //         branch_instr_corrected.clone(),                      // JIE target (devrait être pris)
//     //         nop_instr.clone(),                                   // NOP (devrait être sauté)
//     //         halt_instr.clone(),                                  // target: HALT
//     //     ];
//     //     let (mut pipeline, mut registers, mut memory, mut alu) = setup_test_pipeline(instructions.clone());
//     //
//     //     // Forcer la prédiction initiale à Taken (pour tester prédiction correcte)
//     //     let branch_pc = instructions[0].total_size() as u64;
//     //     pipeline.decode.branch_predictor.two_bit_states.insert(branch_pc, crate::pvm::branch_predictor::TwoBitState::StronglyTaken);
//     //
//     //
//     //     // Simuler les cycles
//     //     let mut current_pc: u32 = 0;
//     //     let max_cycles = 15;
//     //     for i in 0..max_cycles {
//     //         println!("--- Test Cycle {} ---", i + 1);
//     //         let result = pipeline.cycle(current_pc, &mut registers, &mut memory, &mut alu, &instructions);
//     //         assert!(result.is_ok());
//     //         let state = result.unwrap();
//     //         current_pc = state.next_pc;
//     //         if state.halted { break; }
//     //     }
//     //
//     //     assert!(pipeline.state.halted);
//     //     println!("Stats: Cycles={}, Instructions={}, Branches={}, Hits={}, Misses={}, Flushes={}, Stalls={}",
//     //              pipeline.stats.cycles, pipeline.stats.instructions,
//     //              pipeline.stats.branch_predictions, pipeline.stats.branch_hits,
//     //              pipeline.stats.branch_misses, pipeline.stats.branch_flush,
//     //              pipeline.stats.stalls);
//     //
//     //     assert_eq!(pipeline.stats.branch_predictions, 1, "Une seule prédiction de branchement");
//     //     assert_eq!(pipeline.stats.branch_hits, 1, "La prédiction doit être correcte");
//     //     assert_eq!(pipeline.stats.branch_misses, 0, "Aucune misprediction");
//     //     assert_eq!(pipeline.stats.branch_flush, 0, "Aucun flush nécessaire");
//     //     // Le nombre d'instructions complétées: SUB, JIE, HALT = 3
//     //     assert_eq!(pipeline.stats.instructions, 3);
//     // }
// }
//
//
// ///////////////////////////////////////////////////////
// //     pub fn cycle(
// //         &mut self,
// //         pc: u32,
// //         registers: &mut [u64],
// //         memory: &mut Memory,
// //         alu: &mut ALU,
// //         instructions: &[Instruction],
// //     ) -> Result<PipelineState, String> {
// //         self.stats.cycles += 1;
// //         println!("DEBUG: Cycle {} Start - Current PC = 0x{:X}", self.stats.cycles, pc);
// //
// //         // 1. Préparer le nouvel état (basé sur l'ancien)
// //         let mut next_pipeline_state = self.state.clone();
// //         next_pipeline_state.stalled = false; // Sera peut-être mis à true par les hazards
// //         next_pipeline_state.instructions_completed = 0;
// //         // Important : le `next_pc` de l'état actuel est le PC *attendu* pour CE cycle.
// //         let current_pc_target = self.state.next_pc; // Le PC que Fetch devrait utiliser
// //
// //         // --- Exécution des étages (ordre logique inverse pour faciliter la propagation) ---
// //         // Cela évite d'utiliser les données *juste* calculées dans le même cycle.
// //
// //         // 5. Writeback Stage
// //         let mut completed_in_wb = false;
// //         if let Some(mem_wb_reg) = &self.state.memory_writeback { // Lire l'état *précédent*
// //             match self.writeback.process_direct(mem_wb_reg, registers) {
// //                 Ok(_) => {
// //                     println!("DEBUG: WB successful for PC=0x{:X} ({:?})", mem_wb_reg.instruction., mem_wb_reg.instruction.opcode); // Ajout PC si dispo
// //                     next_pipeline_state.instructions_completed += 1;
// //                     self.stats.instructions += 1; // Compter instruction terminée
// //                     completed_in_wb = true;
// //                 },
// //                 Err(e) => return Err(format!("Writeback Error: {}", e)),
// //             }
// //         }
// //         // Vider le registre MEM/WB pour le prochain cycle
// //         next_pipeline_state.memory_writeback = None;
// //
// //
// //         // 4. Memory Stage
// //         let mut mem_wb_output: Option<MemoryWritebackRegister> = None;
// //         if let Some(ex_mem_reg) = &self.state.execute_memory { // Lire l'état *précédent*
// //             match self.memory.process_direct(ex_mem_reg, memory) {
// //                 Ok(wb_reg) => {
// //                     println!("DEBUG: MEM successful for PC=0x{:X} ({:?})", ex_mem_reg.pc, ex_mem_reg.instruction.opcode); // Ajout PC si dispo
// //                     mem_wb_output = Some(wb_reg);
// //                     // Gestion HALT spécifique
// //                     if ex_mem_reg.halted { // Vérifier le flag Halted venant d'Execute
// //                         println!("DEBUG: HALT detected in MEM stage. Setting pipeline to halted.");
// //                         next_pipeline_state.halted = true;
// //                         // Important: Ne pas flusher ici, laisser le pipeline se vider des étages précédents
// //                         // Mais ne plus rien chercher (Fetch sera bloqué par halted).
// //                         // On propage quand même le résultat du HALT (qui est vide) vers WB.
// //                     }
// //                 },
// //                 Err(e) => return Err(format!("Memory Error: {}", e)),
// //             }
// //         }
// //         // Mettre à jour le registre MEM/WB pour le prochain cycle
// //         next_pipeline_state.memory_writeback = mem_wb_output;
// //
// //
// //         // --- Détection des Hazards & Application du Forwarding ---
// //         // Important: Basé sur l'état *avant* l'exécution de DE et EX de ce cycle.
// //         let mut hazard_stall_needed = false;
// //         if self.enable_hazard_detection {
// //             // Utilise l'état *actuel* pour voir s'il y a des dépendances ou conflits
// //             // qui nécessitent un stall *pour le prochain cycle*.
// //             if self.hazard_detection.detect_stall_hazard(&self.state) { // Vérifie Load-Use, Control simple, Structural
// //                 println!("DEBUG: STALL required by Hazard Detection Unit.");
// //                 hazard_stall_needed = true;
// //                 self.stats.stalls += 1;
// //                 // hazards_count est déjà incrémenté dans detect_stall_hazard
// //             }
// //         }
// //         // Appliquer le stall maintenant si nécessaire
// //         next_pipeline_state.stalled = hazard_stall_needed;
// //
// //
// //         // --- Préparation pour Execute & Decode ---
// //         // On clone le registre DE/EX *avant* le forwarding pour la logique de branchement
// //         let de_reg_before_forward = self.state.decode_execute.clone();
// //
// //         // Appliquer le Forwarding (modifie le contenu du registre DE/EX *pour l'étage Execute*)
// //         let mut ex_mem_input = self.state.decode_execute.clone(); // Prendre l'entrée de EX
// //         if self.enable_forwarding {
// //             if let Some(ref mut de_reg_to_forward) = ex_mem_input {
// //                 println!("DEBUG: Applying forwarding for EX stage input (PC=0x{:X})", de_reg_to_forward.pc);
// //                 let _forwarding_info = self.forwarding.forward(
// //                     de_reg_to_forward,
// //                     &self.state.execute_memory, // Source EX/MEM de l'état précédent
// //                     &self.state.memory_writeback, // Source MEM/WB de l'état précédent
// //                 );
// //                 // Mettre à jour les stats de forwarding (déjà fait dans forward())
// //                 self.stats.forwards = self.forwarding.get_forwards_count();
// //             }
// //         }
// //
// //
// //         // 3. Execute Stage
// //         let mut ex_mem_output: Option<ExecuteMemoryRegister> = None;
// //         let mut branch_mispredicted = false;
// //         let mut correct_next_pc_on_mispredict = current_pc_target; // Default, sera écrasé si mispredict
// //
// //         if !next_pipeline_state.stalled { // Ne pas exécuter si stall général
// //             if let Some(de_reg) = &ex_mem_input { // Utiliser l'état potentiellement forwardé
// //                 match self.execute.process_direct(de_reg, alu) {
// //                     Ok(mut mem_reg) => { // `mut` pour pouvoir potentiellement corriger prediction_correct
// //                         println!("DEBUG: EX successful for PC=0x{:X} ({:?})", de_reg.pc, de_reg.instruction.opcode);
// //                         ex_mem_output = Some(mem_reg.clone()); // Sauvegarder le résultat
// //
// //                         // --- Gestion Spécifique des Branchements ICI ---
// //                         if de_reg.instruction.opcode.is_branch() {
// //                             println!("DEBUG: Branch instruction resolved in EX. PC=0x{:X}, Taken={}, Target={:?}, Predicted={:?}",
// //                                      de_reg.pc, mem_reg.branch_taken, mem_reg.branch_target, de_reg.branch_prediction);
// //                             self.stats.branch_predictions += 1; // Compter chaque branche résolue
// //
// //                             let prediction = de_reg.branch_prediction.unwrap_or(BranchPrediction::NotTaken); // Default prediction if None
// //                             let actual_taken = mem_reg.branch_taken;
// //
// //                             // Comparer prédiction et résultat réel
// //                             if (prediction == BranchPrediction::Taken) != actual_taken {
// //                                 // *** MISPREDICTION ***
// //                                 branch_mispredicted = true;
// //                                 self.stats.branch_misses += 1;
// //                                 self.stats.branch_flush += 1; // Compter le flush causé
// //                                 println!("DEBUG: Branch MISPREDICTED! PC=0x{:X}", de_reg.pc);
// //
// //                                 // Déterminer le PC correct
// //                                 if actual_taken {
// //                                     correct_next_pc_on_mispredict = mem_reg.branch_target.expect("Branch taken but no target!");
// //                                     println!("   >> Mispredict recovery: Branch was TAKEN. Correct PC = 0x{:X}", correct_next_pc_on_mispredict);
// //                                 } else {
// //                                     correct_next_pc_on_mispredict = de_reg.pc.wrapping_add(de_reg.instruction.total_size() as u32);
// //                                     println!("   >> Mispredict recovery: Branch was NOT TAKEN. Correct PC = 0x{:X}", correct_next_pc_on_mispredict);
// //                                 }
// //                                 // Marquer comme incorrect dans le registre pour info (même si on flush)
// //                                 if let Some(ref mut output) = ex_mem_output {
// //                                     output.branch_prediction_correct = Some(false);
// //                                 }
// //
// //                             } else {
// //                                 // *** PREDICTION CORRECTE ***
// //                                 self.stats.branch_hits += 1;
// //                                 println!("DEBUG: Branch PREDICTED correctly. PC=0x{:X}", de_reg.pc);
// //                                 // Marquer comme correct
// //                                 if let Some(ref mut output) = ex_mem_output {
// //                                     output.branch_prediction_correct = Some(true);
// //                                 }
// //                             }
// //
// //                             // Mettre à jour le prédicteur de branchement (toujours après résolution)
// //                             self.decode.branch_predictor.update(de_reg.pc as u64, actual_taken, prediction);
// //                         }
// //
// //                     },
// //                     Err(e) => return Err(format!("Execute Error: {}", e)),
// //                 }
// //             }
// //         } else {
// //             println!("DEBUG: EX stage stalled.");
// //             // Propager la bulle (None)
// //             ex_mem_output = None;
// //         }
// //         // Mettre à jour le registre EX/MEM pour le prochain cycle
// //         next_pipeline_state.execute_memory = ex_mem_output;
// //
// //
// //         // 2. Decode Stage
// //         let mut de_ex_output: Option<DecodeExecuteRegister> = None;
// //         if !next_pipeline_state.stalled { // Ne pas décoder si stall général
// //             // Si misprediction => insérer bulle en Decode
// //             if branch_mispredicted {
// //                 println!("DEBUG: Flushing DE stage due to misprediction.");
// //                 de_ex_output = None;
// //             } else if let Some(fd_reg) = &self.state.fetch_decode { // Lire l'état *précédent*
// //                 match self.decode.process_direct(fd_reg, registers) {
// //                     Ok(mut de_reg) => { // `mut` pour injecter prédiction
// //                         println!("DEBUG: DE successful for PC=0x{:X} ({:?})", fd_reg.pc, fd_reg.instruction.opcode);
// //
// //                         // Injecter la prédiction si c'est un branchement
// //                         if de_reg.instruction.opcode.is_branch() {
// //                             let prediction = self.decode.branch_predictor.predict(de_reg.pc as u64);
// //                             de_reg.branch_prediction = Some(prediction);
// //                             println!("   >> Branch predicted in DE: {:?} for PC=0x{:X}", prediction, de_reg.pc);
// //                             // Si prédit pris, on pourrait mettre à jour next_pc *spéculativement*
// //                             // if prediction == BranchPrediction::Taken {
// //                             //    if let Some(target) = de_reg.branch_addr { // Utiliser l'adresse calculée en DE
// //                             //       next_pipeline_state.next_pc = target; // Mise à jour spéculative
// //                             //       println!("   >> Speculative PC update for predicted taken branch: 0x{:X}", target);
// //                             //    }
// //                             // }
// //                         }
// //
// //                         de_ex_output = Some(de_reg);
// //                     },
// //                     Err(e) => return Err(format!("Decode Error: {}", e)),
// //                 }
// //             }
// //         } else {
// //             println!("DEBUG: DE stage stalled.");
// //             // Propager la bulle
// //             de_ex_output = None;
// //         }
// //         // Mettre à jour le registre DE/EX pour le prochain cycle
// //         next_pipeline_state.decode_execute = de_ex_output;
// //
// //
// //         // 1. Fetch Stage
// //         let mut fd_output: Option<FetchDecodeRegister> = None;
// //         let pc_to_fetch = if branch_mispredicted {
// //             // Si mispredict, utiliser le PC corrigé
// //             correct_next_pc_on_mispredict
// //         } else {
// //             // Sinon, utiliser le PC cible de ce cycle
// //             current_pc_target
// //         };
// //
// //         if !next_pipeline_state.stalled { // Ne pas fetcher si stall général
// //             // Si misprediction => insérer bulle en Fetch
// //             if branch_mispredicted {
// //                 println!("DEBUG: Flushing FD stage due to misprediction.");
// //                 fd_output = None;
// //             } else if !next_pipeline_state.halted { // Ne pas fetcher si HALT signalé
// //                 match self.fetch.process_direct(pc_to_fetch, instructions) {
// //                     Ok(fd_reg) => {
// //                         println!("DEBUG: IF successful for PC=0x{:X} ({:?})", pc_to_fetch, fd_reg.instruction.opcode);
// //                         fd_output = Some(fd_reg.clone());
// //
// //                         // --- Mise à jour du Prochain PC ---
// //                         // Si on vient de fetcher une instruction de branchement prédite prise
// //                         let mut predicted_taken_target: Option<u32> = None;
// //                         if let Some(de_output) = &next_pipeline_state.decode_execute { // Regarder ce qui vient d'être décodé
// //                             if de_output.instruction.opcode.is_branch() && de_output.branch_prediction == Some(BranchPrediction::Taken) {
// //                                 predicted_taken_target = de_output.branch_addr;
// //                                 println!("   >> Branch TAKEN predicted for instruction at 0x{:X}. Target=0x{:X}", de_output.pc, predicted_taken_target.unwrap_or(0));
// //                             }
// //                         }
// //
// //                         // Calculer le PC suivant normal (instruction+taille)
// //                         let pc_after_current = pc_to_fetch.wrapping_add(fd_reg.instruction.total_size() as u32);
// //
// //                         // Choisir le prochain PC
// //                         if let Some(target) = predicted_taken_target {
// //                             next_pipeline_state.next_pc = target; // Spéculatif basé sur prédiction 'Taken'
// //                             println!("   >> NEXT PC set to predicted target: 0x{:X}", target);
// //                         } else {
// //                             next_pipeline_state.next_pc = pc_after_current; // Normal ou branche prédite 'Not Taken'
// //                             println!("   >> NEXT PC set sequentially or predicted not taken: 0x{:X}", pc_after_current);
// //                         }
// //
// //                     },
// //                     // Gérer l'erreur de fetch (ex: PC invalide)
// //                     Err(e) => {
// //                         // Si Fetch échoue (ex: fin du programme sans HALT), on peut vouloir s'arrêter
// //                         println!("ERROR: Fetch failed at PC=0x{:X}: {}. Halting.", pc_to_fetch, e);
// //                         next_pipeline_state.halted = true; // Considérer comme un arrêt
// //                         // Pas besoin de retourner une erreur ici, le prochain cycle verra halted=true
// //                         fd_output = None; // Pas d'instruction fetchée
// //                         next_pipeline_state.next_pc = pc_to_fetch; // Garder le PC où l'erreur s'est produite
// //                     }
// //                 }
// //             } else {
// //                 println!("DEBUG: IF stage halted or stalled.");
// //                 // Si halted ou stalled, ne rien fetcher et ne pas changer next_pc
// //                 fd_output = None;
// //                 next_pipeline_state.next_pc = pc_to_fetch; // Maintient le PC
// //             }
// //         } else {
// //             println!("DEBUG: IF stage stalled.");
// //             // Si stall général, ne rien fetcher et ne pas changer next_pc
// //             fd_output = None;
// //             next_pipeline_state.next_pc = pc_to_fetch; // Maintient le PC
// //         }
// //         // Mettre à jour le registre IF/ID pour le prochain cycle
// //         next_pipeline_state.fetch_decode = fd_output;
// //
// //
// //         // --- Finalisation du Cycle ---
// //         // Mettre à jour l'état global du pipeline pour le prochain cycle
// //         self.state = next_pipeline_state.clone();
// //
// //         // Mettre à jour les stats globales à la fin
// //         self.stats.branch_predictor_rate = self.decode.branch_predictor.get_accuracy(); // Recalculer le taux
// //
// //         println!("DEBUG: Cycle {} End - Next PC=0x{:X}, Halted={}, Stalled={}", self.stats.cycles, self.state.next_pc, self.state.halted, self.state.stalled);
// //         println!("---"); // Séparateur de cycle
// //
// //         // Retourner l'état à la fin de CE cycle
// //         Ok(self.state.clone())
// //     }





//
// //////////////////////////////////src/pipeline/mod.rs////////////////////////////////////////////////////


//
//
//
// // Test unitaire pour les fichiers de bytecode
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::bytecode::files::{BytecodeVersion, SegmentMetadata, SegmentType};
//     use crate::bytecode::format::{ArgType, InstructionFormat};
//     use crate::bytecode::instructions::Instruction;
//     use crate::bytecode::opcodes::Opcode;
//     use crate::BytecodeFile;
//     use std::collections::HashMap;
//     use std::io::ErrorKind;
//     use tempfile::tempdir;
//
//     #[test]
//     fn test_bytecode_version() {
//         let version = BytecodeVersion::new(1, 2, 3, 4);
//
//         assert_eq!(version.major, 1);
//         assert_eq!(version.minor, 2);
//         assert_eq!(version.patch, 3);
//         assert_eq!(version.build, 4);
//
//         // Test encode/decode
//         let encoded = version.encode();
//         let decoded = BytecodeVersion::decode(encoded);
//
//         assert_eq!(decoded.major, 1);
//         assert_eq!(decoded.minor, 2);
//         assert_eq!(decoded.patch, 3);
//         assert_eq!(decoded.build, 4);
//
//         // Test to_string
//         assert_eq!(version.to_string(), "1.2.3.4");
//     }
//
//     #[test]
//     fn test_segment_type() {
//         // Test des conversions valides
//         assert_eq!(SegmentType::from_u8(0), Some(SegmentType::Code));
//         assert_eq!(SegmentType::from_u8(1), Some(SegmentType::Data));
//         assert_eq!(SegmentType::from_u8(2), Some(SegmentType::ReadOnlyData));
//         assert_eq!(SegmentType::from_u8(3), Some(SegmentType::Symbols));
//         assert_eq!(SegmentType::from_u8(4), Some(SegmentType::Debug));
//
//         // Test avec valeur invalide
//         assert_eq!(SegmentType::from_u8(5), None);
//     }
//
//     #[test]
//     fn test_segment_metadata() {
//         let segment = SegmentMetadata::new(SegmentType::Code, 100, 200, 300);
//
//         assert_eq!(segment.segment_type, SegmentType::Code);
//         assert_eq!(segment.offset, 100);
//         assert_eq!(segment.size, 200);
//         assert_eq!(segment.load_addr, 300);
//
//         // Test encode/decode
//         let encoded = segment.encode();
//         let decoded = SegmentMetadata::decode(&encoded).unwrap();
//
//         assert_eq!(decoded.segment_type, SegmentType::Code);
//         assert_eq!(decoded.offset, 100);
//         assert_eq!(decoded.size, 200);
//         assert_eq!(decoded.load_addr, 300);
//     }
//
//     #[test]
//     fn test_bytecode_file_simple() {
//         // Création d'un fichier bytecode simple
//         let mut bytecode = BytecodeFile::new();
//
//         // Ajout de métadonnées
//         bytecode.add_metadata("name", "Test");
//         bytecode.add_metadata("author", "PunkVM");
//
//         // Vérification
//         assert_eq!(bytecode.metadata.get("name"), Some(&"Test".to_string()));
//         assert_eq!(bytecode.metadata.get("author"), Some(&"PunkVM".to_string()));
//
//         // Ajout d'instructions
//         let instr1 = Instruction::create_no_args(Opcode::Nop);
//         let instr2 = Instruction::create_reg_imm8(Opcode::Load, 0, 42);
//
//         bytecode.add_instruction(instr1);
//         bytecode.add_instruction(instr2);
//
//         assert_eq!(bytecode.code.len(), 2);
//         assert_eq!(bytecode.code[0].opcode, Opcode::Nop);
//         assert_eq!(bytecode.code[1].opcode, Opcode::Load);
//
//         // Ajout de données
//         let offset = bytecode.add_data(&[1, 2, 3, 4]);
//         assert_eq!(offset, 0);
//         assert_eq!(bytecode.data, vec![1, 2, 3, 4]);
//
//         // Ajout de données en lecture seule
//         let offset = bytecode.add_readonly_data(&[5, 6, 7, 8]);
//         assert_eq!(offset, 0);
//         assert_eq!(bytecode.readonly_data, vec![5, 6, 7, 8]);
//
//         // Ajout de symboles
//         bytecode.add_symbol("main", 0x1000);
//         assert_eq!(bytecode.symbols.get("main"), Some(&0x1000));
//     }
//
//     #[test]
//     fn test_bytecode_file_with_arithmetic_instructions() {
//         // Création d'un fichier bytecode avec des instructions arithmétiques
//         let mut bytecode = BytecodeFile::new();
//
//         // Ajouter des instructions arithmétiques avec le nouveau format à 3 registres
//         let instr1 = Instruction::create_reg_reg_reg(Opcode::Add, 2, 0, 1); // R2 = R0 + R1
//         let instr2 = Instruction::create_reg_reg_reg(Opcode::Sub, 3, 0, 1); // R3 = R0 - R1
//         let instr3 = Instruction::create_reg_reg_reg(Opcode::Mul, 4, 0, 1); // R4 = R0 * R1
//
//         bytecode.add_instruction(instr1);
//         bytecode.add_instruction(instr2);
//         bytecode.add_instruction(instr3);
//
//         assert_eq!(bytecode.code.len(), 3);
//
//         // Vérifier le premier opcode
//         assert_eq!(bytecode.code[0].opcode, Opcode::Add);
//
//         // Vérifier les types d'arguments du format
//         assert_eq!(bytecode.code[0].format.arg1_type, ArgType::Register);
//         assert_eq!(bytecode.code[0].format.arg2_type, ArgType::Register);
//         assert_eq!(bytecode.code[0].format.arg3_type, ArgType::Register);
//
//         // Vérifier les valeurs des registres
//         assert_eq!(bytecode.code[0].args[0], 2); // Rd (destination)
//         assert_eq!(bytecode.code[0].args[1], 0); // Rs1 (source 1)
//         assert_eq!(bytecode.code[0].args[2], 1); // Rs2 (source 2)
//     }
//
//     #[test]
//     fn test_bytecode_file_io() {
//         // Création d'un répertoire temporaire pour les tests
//         let dir = tempdir().expect("Impossible de créer un répertoire temporaire");
//         let file_path = dir.path().join("test.punk");
//
//         // Création d'un fichier bytecode à écrire
//         let mut bytecode = BytecodeFile::new();
//         bytecode.version = BytecodeVersion::new(1, 0, 0, 0);
//         bytecode.add_metadata("name", "TestIO");
//         bytecode.add_instruction(Instruction::create_no_args(Opcode::Halt));
//         bytecode.add_data(&[1, 2, 3]);
//         bytecode.add_readonly_data(&[4, 5, 6]);
//         bytecode.add_symbol("main", 0);
//
//         // Écrire le fichier
//         bytecode
//             .write_to_file(&file_path)
//             .expect("Impossible d'écrire le fichier bytecode");
//
//         // Lire le fichier
//         let loaded = BytecodeFile::read_from_file(&file_path)
//             .expect("Impossible de lire le fichier bytecode");
//
//         // Vérifier que le contenu est identique
//         assert_eq!(loaded.version.major, 1);
//         assert_eq!(loaded.version.minor, 0);
//         assert_eq!(loaded.metadata.get("name"), Some(&"TestIO".to_string()));
//         assert_eq!(loaded.code.len(), 1);
//         assert_eq!(loaded.code[0].opcode, Opcode::Halt);
//         assert_eq!(loaded.data, vec![1, 2, 3]);
//         assert_eq!(loaded.readonly_data, vec![4, 5, 6]);
//         assert_eq!(loaded.symbols.get("main"), Some(&0));
//     }
//
//     #[test]
//     fn test_bytecode_file_with_three_register_instructions_io() {
//         // Test d'écriture et lecture d'un fichier contenant des instructions à 3 registres
//         let dir = tempdir().expect("Impossible de créer un répertoire temporaire");
//         let file_path = dir.path().join("test_three_reg.punk");
//
//         // Création du fichier bytecode
//         let mut bytecode = BytecodeFile::new();
//         bytecode.version = BytecodeVersion::new(1, 0, 0, 0);
//
//         // Ajouter une instruction ADD avec 3 registres
//         let add_instr = Instruction::create_reg_reg_reg(Opcode::Add, 2, 0, 1); // R2 = R0 + R1
//         bytecode.add_instruction(add_instr);
//
//         // Écrire le fichier
//         bytecode
//             .write_to_file(&file_path)
//             .expect("Impossible d'écrire le fichier bytecode");
//
//         // Lire le fichier
//         let loaded = BytecodeFile::read_from_file(&file_path)
//             .expect("Impossible de lire le fichier bytecode");
//
//         // Vérifier que l'instruction est correctement chargée
//         assert_eq!(loaded.code.len(), 1);
//         assert_eq!(loaded.code[0].opcode, Opcode::Add);
//
//         // Vérifier les valeurs des registres
//         assert_eq!(loaded.code[0].args.len(), 3);
//         assert_eq!(loaded.code[0].args[0], 2); // Rd
//         assert_eq!(loaded.code[0].args[1], 0); // Rs1
//         assert_eq!(loaded.code[0].args[2], 1); // Rs2
//     }
//
//     #[test]
//     fn test_bytecode_file_extended_size_io() {
//         // Test d'écriture et lecture d'un fichier avec une instruction de grande taille
//         let dir = tempdir().expect("Impossible de créer un répertoire temporaire");
//         let file_path = dir.path().join("test_extended.punk");
//
//         // Création du fichier bytecode
//         let mut bytecode = BytecodeFile::new();
//
//         // Créer une instruction avec beaucoup de données pour forcer un size_type Extended
//         let large_args = vec![0; 248]; // Suffisant pour dépasser la limite de 255 octets
//         let large_instr =
//             Instruction::new(Opcode::Add, InstructionFormat::double_reg(), large_args);
//
//         bytecode.add_instruction(large_instr);
//
//         // Écrire le fichier
//         bytecode
//             .write_to_file(&file_path)
//             .expect("Impossible d'écrire le fichier bytecode");
//
//         // Lire le fichier
//         let loaded = BytecodeFile::read_from_file(&file_path)
//             .expect("Impossible de lire le fichier bytecode");
//
//         // Vérifier que l'instruction est correctement chargée
//         assert_eq!(loaded.code.len(), 1);
//         assert_eq!(loaded.code[0].opcode, Opcode::Add);
//         assert_eq!(loaded.code[0].args.len(), 248);
//     }
//
//     #[test]
//     fn test_bytecode_file_complex_program() {
//         // Créer un programme complet avec des instructions variées
//         let mut bytecode = BytecodeFile::new();
//
//         // Ajouter des métadonnées
//         bytecode.add_metadata("name", "Programme de test");
//         bytecode.add_metadata("author", "PunkVM Team");
//         bytecode.add_metadata("version", "1.0.0");
//
//         // Initialisation des registres
//         bytecode.add_instruction(Instruction::create_reg_imm8(Opcode::Load, 0, 10)); // R0 = 10
//         bytecode.add_instruction(Instruction::create_reg_imm8(Opcode::Load, 1, 5)); // R1 = 5
//
//         // Opérations arithmétiques
//         bytecode.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 2, 0, 1)); // R2 = R0 + R1
//         bytecode.add_instruction(Instruction::create_reg_reg_reg(Opcode::Sub, 3, 0, 1)); // R3 = R0 - R1
//         bytecode.add_instruction(Instruction::create_reg_reg_reg(Opcode::Mul, 4, 0, 1)); // R4 = R0 * R1
//         bytecode.add_instruction(Instruction::create_reg_reg_reg(Opcode::Div, 5, 0, 1)); // R5 = R0 / R1
//
//         // Opérations logiques
//         bytecode.add_instruction(Instruction::create_reg_reg_reg(Opcode::And, 6, 0, 1)); // R6 = R0 & R1
//         bytecode.add_instruction(Instruction::create_reg_reg_reg(Opcode::Or, 7, 0, 1)); // R7 = R0 | R1
//
//         // Fin du programme
//         bytecode.add_instruction(Instruction::create_no_args(Opcode::Halt));
//
//         // Ajouter un symbole pour le début du programme
//         bytecode.add_symbol("start", 0);
//
//         // Vérifier le nombre d'instructions
//         assert_eq!(bytecode.code.len(), 9);
//
//         // Vérifier les métadonnées
//         assert_eq!(bytecode.metadata.len(), 3);
//         assert_eq!(
//             bytecode.metadata.get("name"),
//             Some(&"Programme de test".to_string())
//         );
//
//         // Vérifier les symboles
//         assert_eq!(bytecode.symbols.len(), 1);
//         assert_eq!(bytecode.symbols.get("start"), Some(&0));
//     }
//
//     #[test]
//     fn test_bytecode_file_io_errors() {
//         // Test avec un fichier inexistant
//         let result = BytecodeFile::read_from_file("nonexistent_file.punk");
//         assert!(result.is_err());
//
//         // Test avec un fichier trop petit
//         let dir = tempdir().expect("Impossible de créer un répertoire temporaire");
//         let invalid_file_path = dir.path().join("invalid.punk");
//
//         // Créer un fichier invalide avec juste quelques octets
//         std::fs::write(&invalid_file_path, &[0, 1, 2])
//             .expect("Impossible d'écrire le fichier de test");
//
//         let result = BytecodeFile::read_from_file(&invalid_file_path);
//         assert!(result.is_err());
//
//         // Vérifier le type d'erreur
//         match result {
//             Err(e) => assert_eq!(e.kind(), ErrorKind::InvalidData),
//             _ => panic!("Expected an error but got success"),
//         }
//     }
//
//     #[test]
//     fn test_encode_decode_metadata() {
//         let mut metadata = HashMap::new();
//         metadata.insert("key1".to_string(), "value1".to_string());
//         metadata.insert("key2".to_string(), "value2".to_string());
//
//         let mut bytecode = BytecodeFile::new();
//         bytecode.metadata = metadata.clone();
//
//         let encoded = bytecode.encode_metadata();
//         let decoded = BytecodeFile::decode_metadata(&encoded).expect("Failed to decode metadata");
//
//         assert_eq!(decoded.len(), 2);
//         assert_eq!(decoded.get("key1"), Some(&"value1".to_string()));
//         assert_eq!(decoded.get("key2"), Some(&"value2".to_string()));
//     }
//
//     #[test]
//     fn test_encode_decode_symbols() {
//         let mut symbols = HashMap::new();
//         symbols.insert("sym1".to_string(), 0x1000);
//         symbols.insert("sym2".to_string(), 0x2000);
//
//         let mut bytecode = BytecodeFile::new();
//         bytecode.symbols = symbols.clone();
//
//         let encoded = bytecode.encode_symbols();
//         let decoded = BytecodeFile::decode_symbols(&encoded).expect("Failed to decode symbols");
//
//         assert_eq!(decoded.len(), 2);
//         assert_eq!(decoded.get("sym1"), Some(&0x1000));
//         assert_eq!(decoded.get("sym2"), Some(&0x2000));
//     }
//
//     #[test]
//     fn test_encode_decode_code() {
//         // Créer un ensemble d'instructions de test
//         let mut code = Vec::new();
//         code.push(Instruction::create_no_args(Opcode::Nop));
//         code.push(Instruction::create_reg_imm8(Opcode::Load, 0, 42));
//         code.push(Instruction::create_reg_reg_reg(Opcode::Add, 2, 0, 1));
//
//         let mut bytecode = BytecodeFile::new();
//         bytecode.code = code.clone();
//
//         let encoded = bytecode.encode_code();
//         let decoded = BytecodeFile::decode_code(&encoded).expect("Failed to decode code");
//
//         assert_eq!(decoded.len(), 3);
//         assert_eq!(decoded[0].opcode, Opcode::Nop);
//         assert_eq!(decoded[1].opcode, Opcode::Load);
//         assert_eq!(decoded[2].opcode, Opcode::Add);
//
//         // Vérifier les arguments de l'instruction Add
//         assert_eq!(decoded[2].args.len(), 3);
//         assert_eq!(decoded[2].args[0], 2);
//         assert_eq!(decoded[2].args[1], 0);
//         assert_eq!(decoded[2].args[2], 1);
//     }
// }
//
//
// #[cfg(test)]
// mod pipeline_mod_tests {
//     use super::*; // Importe Pipeline et les structures du mod.rs
//     use crate::alu::alu::ALU;
//     use crate::bytecode::instructions::Instruction;
//     use crate::bytecode::opcodes::Opcode;
//     use crate::pvm::memorys::{Memory, MemoryConfig}; // Assurez-vous que les chemins sont corrects
//     use crate::pipeline::{FetchDecodeRegister, DecodeExecuteRegister, ExecuteMemoryRegister, MemoryWritebackRegister};
//
//     // Helper pour créer une VM minimale pour les tests de pipeline
//     fn setup_test_pipeline(instructions: Vec<Instruction>) -> (Pipeline, Vec<u64>, Memory, ALU) {
//         let pipeline = Pipeline::new(4, true, true); // Buffersize 4, forwarding/hazard ON
//         let registers = vec![0u64; 16];
//         let memory_config = MemoryConfig { size: 1024, l1_cache_size: 64, store_buffer_size: 4 };
//         let memory = Memory::new(memory_config);
//         let alu = ALU::new();
//         // Note: La mémoire n'est pas pré-chargée avec les instructions ici.
//         // La fonction `cycle` reçoit le slice `instructions` directement.
//         (pipeline, registers, memory, alu)
//     }
//
//     #[test]
//     fn test_pipeline_simple_add() {
//         // Instructions: ADD R2, R0, R1; HALT
//         let instructions = vec![
//             Instruction::create_reg_reg_reg(Opcode::Add, 2, 0, 1), // R2 = R0 + R1
//             Instruction::create_no_args(Opcode::Halt),
//         ];
//         let (mut pipeline, mut registers, mut memory, mut alu) = setup_test_pipeline(instructions.clone());
//
//         // Initialiser les registres
//         registers[0] = 10;
//         registers[1] = 5;
//
//         // Simuler les cycles
//         let mut current_pc: u32 = 0;
//         let max_cycles = 10; // Éviter boucle infinie
//         for _ in 0..max_cycles {
//             let result = pipeline.cycle(current_pc, &mut registers, &mut memory, &mut alu, &instructions);
//             assert!(result.is_ok(), "Le cycle du pipeline a échoué");
//             let state = result.unwrap();
//             current_pc = state.next_pc; // Mettre à jour le PC pour le prochain cycle
//             println!("Cycle: {}, PC: 0x{:X}, Halted: {}, Stalled: {}",
//                      pipeline.stats.cycles, current_pc, state.halted, state.stalled);
//             if state.halted {
//                 break;
//             }
//         }
//
//         assert!(pipeline.state.halted, "Le pipeline devrait être arrêté (halted)");
//         // ADD prend ~5 cycles pour compléter (F-D-E-M-W)
//         assert!(pipeline.stats.cycles >= 5, "Il faut au moins 5 cycles");
//         assert_eq!(registers[2], 15, "R2 devrait contenir 10 + 5"); // Vérifier le résultat
//         assert_eq!(pipeline.stats.instructions, 1, "Une seule instruction (ADD) devrait être complétée");
//     }
//
//     #[test]
//     fn test_pipeline_data_hazard_raw_forwarding() {
//         // Instructions: ADD R1, R0, R0; ADD R2, R1, R1; HALT
//         let instructions = vec![
//             Instruction::create_reg_reg_reg(Opcode::Add, 1, 0, 0), // R1 = R0 + R0 (R1=10)
//             Instruction::create_reg_reg_reg(Opcode::Add, 2, 1, 1), // R2 = R1 + R1 (dépend de R1)
//             Instruction::create_no_args(Opcode::Halt),
//         ];
//         let (mut pipeline, mut registers, mut memory, mut alu) = setup_test_pipeline(instructions.clone());
//         registers[0] = 5; // R0 = 5
//
//         // Simuler les cycles
//         let mut current_pc: u32 = 0;
//         let max_cycles = 15;
//         for _ in 0..max_cycles {
//             let result = pipeline.cycle(current_pc, &mut registers, &mut memory, &mut alu, &instructions);
//             assert!(result.is_ok());
//             let state = result.unwrap();
//             current_pc = state.next_pc;
//             if state.halted { break; }
//         }
//
//         assert!(pipeline.state.halted);
//         assert_eq!(registers[1], 10, "R1 devrait être 5 + 5");
//         assert_eq!(registers[2], 20, "R2 devrait être 10 + 10 (avec forwarding)");
//         assert!(pipeline.forwarding.get_forwards_count() >= 1, "Au moins un forward aurait dû se produire pour R1");
//         // Le nombre exact de cycles dépend si le forwarding évite complètement le stall
//         // S'il y a un stall d'un cycle malgré le forwarding: cycles = 5 (ADD1) + 1 (stall) + 1 (ADD2 complété) = 7?
//         // S'il n'y a pas de stall: cycles = 5 (ADD1) + 1 (ADD2 complété) = 6?
//         println!("Stats: Cycles={}, Instructions={}, Forwards={}, Stalls={}",
//                  pipeline.stats.cycles, pipeline.stats.instructions,
//                  pipeline.forwarding.get_forwards_count(), pipeline.stats.stalls);
//         assert_eq!(pipeline.stats.stalls, 0, "Aucun stall ne devrait être nécessaire avec forwarding EX->DE");
//         assert_eq!(pipeline.stats.instructions, 2);
//     }
//     //
//     #[test]
//     fn test_pipeline_load_use_hazard_stall() {
//         // Instructions: LOAD R1, [0]; ADD R2, R1, R0; HALT
//         let load_addr = 0x100; // Adresse arbitraire
//         let instructions = vec![
//             Instruction::create_reg_imm8(Opcode::Load, 1, 0), // LOAD R1, [0x100] (simplifié: addr dans immediate)
//             Instruction::create_reg_reg_reg(Opcode::Add, 2, 1, 0), // ADD R2, R1, R0 (utilise R1)
//             Instruction::create_no_args(Opcode::Halt),
//         ];
//         let (mut pipeline, mut registers, mut memory, mut alu) = setup_test_pipeline(instructions.clone());
//
//         // Mettre une valeur en mémoire
//         memory.write_qword(load_addr, 99).unwrap();
//         registers[0] = 1; // R0 = 1
//
//         // Simuler les cycles
//         let mut current_pc: u32 = 0;
//         let max_cycles = 15;
//         for i in 0..max_cycles {
//             println!("--- Test Cycle {} ---", i + 1);
//             let result = pipeline.cycle(current_pc, &mut registers, &mut memory, &mut alu, &instructions);
//             assert!(result.is_ok());
//             let state = result.unwrap();
//             current_pc = state.next_pc;
//             if state.halted { break; }
//         }
//
//         assert!(pipeline.state.halted);
//         assert_eq!(registers[1], 99, "R1 doit contenir la valeur chargée");
//         assert_eq!(registers[2], 100, "R2 doit être 99 + 1");
//         println!("Stats: Cycles={}, Instructions={}, Forwards={}, Stalls={}",
//                  pipeline.stats.cycles, pipeline.stats.instructions,
//                  pipeline.forwarding.get_forwards_count(), pipeline.stats.stalls);
//         assert!(pipeline.stats.stalls >= 1, "Au moins un stall est attendu pour Load-Use");
//         assert_eq!(pipeline.stats.instructions, 2);
//         assert!(pipeline.hazard_detection.hazards_count >= 1, "Au moins un hazard LoadUse détecté");
//     }
//
//
//
//
//     //
//     // #[test]
//     // fn test_pipeline_branch_taken_correctly_predicted() {
//     //     // JMP_IF_EQUAL target ; ADD R0, R0, 1 (ne devrait pas s'exécuter); target: HALT
//     //     let target_offset = 8; // Taille JIE + ADD = 4 + 4 = 8 ? Vérifier tailles!
//     //     let branch_instr = Instruction::create_jump_if_equal(target_offset);
//     //     let nop_instr = Instruction::create_no_args(Opcode::Nop); // Utiliser NOP pour simplicité taille
//     //     let halt_instr = Instruction::create_no_args(Opcode::Halt);
//     //
//     //     // Calcul des tailles réelles
//     //     let branch_size = branch_instr.total_size() as u32;
//     //     let nop_size = nop_instr.total_size() as u32;
//     //     // L'offset doit pointer *après* le NOP vers HALT
//     //     let correct_offset = nop_size as i32;
//     //     let branch_instr_corrected = Instruction::create_jump_if_equal(correct_offset);
//     //
//     //
//     //     let instructions = vec![
//     //         Instruction::create_reg_reg_reg(Opcode::Sub, 0, 0, 0), // SUB R0, R0, R0 => Met Z=1
//     //         branch_instr_corrected.clone(),                      // JIE target (devrait être pris)
//     //         nop_instr.clone(),                                   // NOP (devrait être sauté)
//     //         halt_instr.clone(),                                  // target: HALT
//     //     ];
//     //     let (mut pipeline, mut registers, mut memory, mut alu) = setup_test_pipeline(instructions.clone());
//     //
//     //     // Forcer la prédiction initiale à Taken (pour tester prédiction correcte)
//     //     let branch_pc = instructions[0].total_size() as u64;
//     //     pipeline.decode.branch_predictor.two_bit_states.insert(branch_pc, crate::pvm::branch_predictor::TwoBitState::StronglyTaken);
//     //
//     //
//     //     // Simuler les cycles
//     //     let mut current_pc: u32 = 0;
//     //     let max_cycles = 15;
//     //     for i in 0..max_cycles {
//     //         println!("--- Test Cycle {} ---", i + 1);
//     //         let result = pipeline.cycle(current_pc, &mut registers, &mut memory, &mut alu, &instructions);
//     //         assert!(result.is_ok());
//     //         let state = result.unwrap();
//     //         current_pc = state.next_pc;
//     //         if state.halted { break; }
//     //     }
//     //
//     //     assert!(pipeline.state.halted);
//     //     println!("Stats: Cycles={}, Instructions={}, Branches={}, Hits={}, Misses={}, Flushes={}, Stalls={}",
//     //              pipeline.stats.cycles, pipeline.stats.instructions,
//     //              pipeline.stats.branch_predictions, pipeline.stats.branch_hits,
//     //              pipeline.stats.branch_misses, pipeline.stats.branch_flush,
//     //              pipeline.stats.stalls);
//     //
//     //     assert_eq!(pipeline.stats.branch_predictions, 1, "Une seule prédiction de branchement");
//     //     assert_eq!(pipeline.stats.branch_hits, 1, "La prédiction doit être correcte");
//     //     assert_eq!(pipeline.stats.branch_misses, 0, "Aucune misprediction");
//     //     assert_eq!(pipeline.stats.branch_flush, 0, "Aucun flush nécessaire");
//     //     // Le nombre d'instructions complétées: SUB, JIE, HALT = 3
//     //     assert_eq!(pipeline.stats.instructions, 3);
//     // }
// }
//
//
// ///////////////////////////////////////////////////////
// //     pub fn cycle(
// //         &mut self,
// //         pc: u32,
// //         registers: &mut [u64],
// //         memory: &mut Memory,
// //         alu: &mut ALU,
// //         instructions: &[Instruction],
// //     ) -> Result<PipelineState, String> {
// //         self.stats.cycles += 1;
// //         println!("DEBUG: Cycle {} Start - Current PC = 0x{:X}", self.stats.cycles, pc);
// //
// //         // 1. Préparer le nouvel état (basé sur l'ancien)
// //         let mut next_pipeline_state = self.state.clone();
// //         next_pipeline_state.stalled = false; // Sera peut-être mis à true par les hazards
// //         next_pipeline_state.instructions_completed = 0;
// //         // Important : le `next_pc` de l'état actuel est le PC *attendu* pour CE cycle.
// //         let current_pc_target = self.state.next_pc; // Le PC que Fetch devrait utiliser
// //
// //         // --- Exécution des étages (ordre logique inverse pour faciliter la propagation) ---
// //         // Cela évite d'utiliser les données *juste* calculées dans le même cycle.
// //
// //         // 5. Writeback Stage
// //         let mut completed_in_wb = false;
// //         if let Some(mem_wb_reg) = &self.state.memory_writeback { // Lire l'état *précédent*
// //             match self.writeback.process_direct(mem_wb_reg, registers) {
// //                 Ok(_) => {
// //                     println!("DEBUG: WB successful for PC=0x{:X} ({:?})", mem_wb_reg.instruction., mem_wb_reg.instruction.opcode); // Ajout PC si dispo
// //                     next_pipeline_state.instructions_completed += 1;
// //                     self.stats.instructions += 1; // Compter instruction terminée
// //                     completed_in_wb = true;
// //                 },
// //                 Err(e) => return Err(format!("Writeback Error: {}", e)),
// //             }
// //         }
// //         // Vider le registre MEM/WB pour le prochain cycle
// //         next_pipeline_state.memory_writeback = None;
// //
// //
// //         // 4. Memory Stage
// //         let mut mem_wb_output: Option<MemoryWritebackRegister> = None;
// //         if let Some(ex_mem_reg) = &self.state.execute_memory { // Lire l'état *précédent*
// //             match self.memory.process_direct(ex_mem_reg, memory) {
// //                 Ok(wb_reg) => {
// //                     println!("DEBUG: MEM successful for PC=0x{:X} ({:?})", ex_mem_reg.pc, ex_mem_reg.instruction.opcode); // Ajout PC si dispo
// //                     mem_wb_output = Some(wb_reg);
// //                     // Gestion HALT spécifique
// //                     if ex_mem_reg.halted { // Vérifier le flag Halted venant d'Execute
// //                         println!("DEBUG: HALT detected in MEM stage. Setting pipeline to halted.");
// //                         next_pipeline_state.halted = true;
// //                         // Important: Ne pas flusher ici, laisser le pipeline se vider des étages précédents
// //                         // Mais ne plus rien chercher (Fetch sera bloqué par halted).
// //                         // On propage quand même le résultat du HALT (qui est vide) vers WB.
// //                     }
// //                 },
// //                 Err(e) => return Err(format!("Memory Error: {}", e)),
// //             }
// //         }
// //         // Mettre à jour le registre MEM/WB pour le prochain cycle
// //         next_pipeline_state.memory_writeback = mem_wb_output;
// //
// //
// //         // --- Détection des Hazards & Application du Forwarding ---
// //         // Important: Basé sur l'état *avant* l'exécution de DE et EX de ce cycle.
// //         let mut hazard_stall_needed = false;
// //         if self.enable_hazard_detection {
// //             // Utilise l'état *actuel* pour voir s'il y a des dépendances ou conflits
// //             // qui nécessitent un stall *pour le prochain cycle*.
// //             if self.hazard_detection.detect_stall_hazard(&self.state) { // Vérifie Load-Use, Control simple, Structural
// //                 println!("DEBUG: STALL required by Hazard Detection Unit.");
// //                 hazard_stall_needed = true;
// //                 self.stats.stalls += 1;
// //                 // hazards_count est déjà incrémenté dans detect_stall_hazard
// //             }
// //         }
// //         // Appliquer le stall maintenant si nécessaire
// //         next_pipeline_state.stalled = hazard_stall_needed;
// //
// //
// //         // --- Préparation pour Execute & Decode ---
// //         // On clone le registre DE/EX *avant* le forwarding pour la logique de branchement
// //         let de_reg_before_forward = self.state.decode_execute.clone();
// //
// //         // Appliquer le Forwarding (modifie le contenu du registre DE/EX *pour l'étage Execute*)
// //         let mut ex_mem_input = self.state.decode_execute.clone(); // Prendre l'entrée de EX
// //         if self.enable_forwarding {
// //             if let Some(ref mut de_reg_to_forward) = ex_mem_input {
// //                 println!("DEBUG: Applying forwarding for EX stage input (PC=0x{:X})", de_reg_to_forward.pc);
// //                 let _forwarding_info = self.forwarding.forward(
// //                     de_reg_to_forward,
// //                     &self.state.execute_memory, // Source EX/MEM de l'état précédent
// //                     &self.state.memory_writeback, // Source MEM/WB de l'état précédent
// //                 );
// //                 // Mettre à jour les stats de forwarding (déjà fait dans forward())
// //                 self.stats.forwards = self.forwarding.get_forwards_count();
// //             }
// //         }
// //
// //
// //         // 3. Execute Stage
// //         let mut ex_mem_output: Option<ExecuteMemoryRegister> = None;
// //         let mut branch_mispredicted = false;
// //         let mut correct_next_pc_on_mispredict = current_pc_target; // Default, sera écrasé si mispredict
// //
// //         if !next_pipeline_state.stalled { // Ne pas exécuter si stall général
// //             if let Some(de_reg) = &ex_mem_input { // Utiliser l'état potentiellement forwardé
// //                 match self.execute.process_direct(de_reg, alu) {
// //                     Ok(mut mem_reg) => { // `mut` pour pouvoir potentiellement corriger prediction_correct
// //                         println!("DEBUG: EX successful for PC=0x{:X} ({:?})", de_reg.pc, de_reg.instruction.opcode);
// //                         ex_mem_output = Some(mem_reg.clone()); // Sauvegarder le résultat
// //
// //                         // --- Gestion Spécifique des Branchements ICI ---
// //                         if de_reg.instruction.opcode.is_branch() {
// //                             println!("DEBUG: Branch instruction resolved in EX. PC=0x{:X}, Taken={}, Target={:?}, Predicted={:?}",
// //                                      de_reg.pc, mem_reg.branch_taken, mem_reg.branch_target, de_reg.branch_prediction);
// //                             self.stats.branch_predictions += 1; // Compter chaque branche résolue
// //
// //                             let prediction = de_reg.branch_prediction.unwrap_or(BranchPrediction::NotTaken); // Default prediction if None
// //                             let actual_taken = mem_reg.branch_taken;
// //
// //                             // Comparer prédiction et résultat réel
// //                             if (prediction == BranchPrediction::Taken) != actual_taken {
// //                                 // *** MISPREDICTION ***
// //                                 branch_mispredicted = true;
// //                                 self.stats.branch_misses += 1;
// //                                 self.stats.branch_flush += 1; // Compter le flush causé
// //                                 println!("DEBUG: Branch MISPREDICTED! PC=0x{:X}", de_reg.pc);
// //
// //                                 // Déterminer le PC correct
// //                                 if actual_taken {
// //                                     correct_next_pc_on_mispredict = mem_reg.branch_target.expect("Branch taken but no target!");
// //                                     println!("   >> Mispredict recovery: Branch was TAKEN. Correct PC = 0x{:X}", correct_next_pc_on_mispredict);
// //                                 } else {
// //                                     correct_next_pc_on_mispredict = de_reg.pc.wrapping_add(de_reg.instruction.total_size() as u32);
// //                                     println!("   >> Mispredict recovery: Branch was NOT TAKEN. Correct PC = 0x{:X}", correct_next_pc_on_mispredict);
// //                                 }
// //                                 // Marquer comme incorrect dans le registre pour info (même si on flush)
// //                                 if let Some(ref mut output) = ex_mem_output {
// //                                     output.branch_prediction_correct = Some(false);
// //                                 }
// //
// //                             } else {
// //                                 // *** PREDICTION CORRECTE ***
// //                                 self.stats.branch_hits += 1;
// //                                 println!("DEBUG: Branch PREDICTED correctly. PC=0x{:X}", de_reg.pc);
// //                                 // Marquer comme correct
// //                                 if let Some(ref mut output) = ex_mem_output {
// //                                     output.branch_prediction_correct = Some(true);
// //                                 }
// //                             }
// //
// //                             // Mettre à jour le prédicteur de branchement (toujours après résolution)
// //                             self.decode.branch_predictor.update(de_reg.pc as u64, actual_taken, prediction);
// //                         }
// //
// //                     },
// //                     Err(e) => return Err(format!("Execute Error: {}", e)),
// //                 }
// //             }
// //         } else {
// //             println!("DEBUG: EX stage stalled.");
// //             // Propager la bulle (None)
// //             ex_mem_output = None;
// //         }
// //         // Mettre à jour le registre EX/MEM pour le prochain cycle
// //         next_pipeline_state.execute_memory = ex_mem_output;
// //
// //
// //         // 2. Decode Stage
// //         let mut de_ex_output: Option<DecodeExecuteRegister> = None;
// //         if !next_pipeline_state.stalled { // Ne pas décoder si stall général
// //             // Si misprediction => insérer bulle en Decode
// //             if branch_mispredicted {
// //                 println!("DEBUG: Flushing DE stage due to misprediction.");
// //                 de_ex_output = None;
// //             } else if let Some(fd_reg) = &self.state.fetch_decode { // Lire l'état *précédent*
// //                 match self.decode.process_direct(fd_reg, registers) {
// //                     Ok(mut de_reg) => { // `mut` pour injecter prédiction
// //                         println!("DEBUG: DE successful for PC=0x{:X} ({:?})", fd_reg.pc, fd_reg.instruction.opcode);
// //
// //                         // Injecter la prédiction si c'est un branchement
// //                         if de_reg.instruction.opcode.is_branch() {
// //                             let prediction = self.decode.branch_predictor.predict(de_reg.pc as u64);
// //                             de_reg.branch_prediction = Some(prediction);
// //                             println!("   >> Branch predicted in DE: {:?} for PC=0x{:X}", prediction, de_reg.pc);
// //                             // Si prédit pris, on pourrait mettre à jour next_pc *spéculativement*
// //                             // if prediction == BranchPrediction::Taken {
// //                             //    if let Some(target) = de_reg.branch_addr { // Utiliser l'adresse calculée en DE
// //                             //       next_pipeline_state.next_pc = target; // Mise à jour spéculative
// //                             //       println!("   >> Speculative PC update for predicted taken branch: 0x{:X}", target);
// //                             //    }
// //                             // }
// //                         }
// //
// //                         de_ex_output = Some(de_reg);
// //                     },
// //                     Err(e) => return Err(format!("Decode Error: {}", e)),
// //                 }
// //             }
// //         } else {
// //             println!("DEBUG: DE stage stalled.");
// //             // Propager la bulle
// //             de_ex_output = None;
// //         }
// //         // Mettre à jour le registre DE/EX pour le prochain cycle
// //         next_pipeline_state.decode_execute = de_ex_output;
// //
// //
// //         // 1. Fetch Stage
// //         let mut fd_output: Option<FetchDecodeRegister> = None;
// //         let pc_to_fetch = if branch_mispredicted {
// //             // Si mispredict, utiliser le PC corrigé
// //             correct_next_pc_on_mispredict
// //         } else {
// //             // Sinon, utiliser le PC cible de ce cycle
// //             current_pc_target
// //         };
// //
// //         if !next_pipeline_state.stalled { // Ne pas fetcher si stall général
// //             // Si misprediction => insérer bulle en Fetch
// //             if branch_mispredicted {
// //                 println!("DEBUG: Flushing FD stage due to misprediction.");
// //                 fd_output = None;
// //             } else if !next_pipeline_state.halted { // Ne pas fetcher si HALT signalé
// //                 match self.fetch.process_direct(pc_to_fetch, instructions) {
// //                     Ok(fd_reg) => {
// //                         println!("DEBUG: IF successful for PC=0x{:X} ({:?})", pc_to_fetch, fd_reg.instruction.opcode);
// //                         fd_output = Some(fd_reg.clone());
// //
// //                         // --- Mise à jour du Prochain PC ---
// //                         // Si on vient de fetcher une instruction de branchement prédite prise
// //                         let mut predicted_taken_target: Option<u32> = None;
// //                         if let Some(de_output) = &next_pipeline_state.decode_execute { // Regarder ce qui vient d'être décodé
// //                             if de_output.instruction.opcode.is_branch() && de_output.branch_prediction == Some(BranchPrediction::Taken) {
// //                                 predicted_taken_target = de_output.branch_addr;
// //                                 println!("   >> Branch TAKEN predicted for instruction at 0x{:X}. Target=0x{:X}", de_output.pc, predicted_taken_target.unwrap_or(0));
// //                             }
// //                         }
// //
// //                         // Calculer le PC suivant normal (instruction+taille)
// //                         let pc_after_current = pc_to_fetch.wrapping_add(fd_reg.instruction.total_size() as u32);
// //
// //                         // Choisir le prochain PC
// //                         if let Some(target) = predicted_taken_target {
// //                             next_pipeline_state.next_pc = target; // Spéculatif basé sur prédiction 'Taken'
// //                             println!("   >> NEXT PC set to predicted target: 0x{:X}", target);
// //                         } else {
// //                             next_pipeline_state.next_pc = pc_after_current; // Normal ou branche prédite 'Not Taken'
// //                             println!("   >> NEXT PC set sequentially or predicted not taken: 0x{:X}", pc_after_current);
// //                         }
// //
// //                     },
// //                     // Gérer l'erreur de fetch (ex: PC invalide)
// //                     Err(e) => {
// //                         // Si Fetch échoue (ex: fin du programme sans HALT), on peut vouloir s'arrêter
// //                         println!("ERROR: Fetch failed at PC=0x{:X}: {}. Halting.", pc_to_fetch, e);
// //                         next_pipeline_state.halted = true; // Considérer comme un arrêt
// //                         // Pas besoin de retourner une erreur ici, le prochain cycle verra halted=true
// //                         fd_output = None; // Pas d'instruction fetchée
// //                         next_pipeline_state.next_pc = pc_to_fetch; // Garder le PC où l'erreur s'est produite
// //                     }
// //                 }
// //             } else {
// //                 println!("DEBUG: IF stage halted or stalled.");
// //                 // Si halted ou stalled, ne rien fetcher et ne pas changer next_pc
// //                 fd_output = None;
// //                 next_pipeline_state.next_pc = pc_to_fetch; // Maintient le PC
// //             }
// //         } else {
// //             println!("DEBUG: IF stage stalled.");
// //             // Si stall général, ne rien fetcher et ne pas changer next_pc
// //             fd_output = None;
// //             next_pipeline_state.next_pc = pc_to_fetch; // Maintient le PC
// //         }
// //         // Mettre à jour le registre IF/ID pour le prochain cycle
// //         next_pipeline_state.fetch_decode = fd_output;
// //
// //
// //         // --- Finalisation du Cycle ---
// //         // Mettre à jour l'état global du pipeline pour le prochain cycle
// //         self.state = next_pipeline_state.clone();
// //
// //         // Mettre à jour les stats globales à la fin
// //         self.stats.branch_predictor_rate = self.decode.branch_predictor.get_accuracy(); // Recalculer le taux
// //
// //         println!("DEBUG: Cycle {} End - Next PC=0x{:X}, Halted={}, Stalled={}", self.stats.cycles, self.state.next_pc, self.state.halted, self.state.stalled);
// //         println!("---"); // Séparateur de cycle
// //
// //         // Retourner l'état à la fin de CE cycle
// //         Ok(self.state.clone())
// //     }
//
// //////////////////////////////////////////////////////////////////////////////////////



////////////////////////////////////////////////////////////////


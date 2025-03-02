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
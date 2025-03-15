// src/tests/mod.rs
//
// mod instruction_tests {
//     use crate::bytecode::instructions::*;
//     use crate::bytecode::opcodes::Opcode;
//     use crate::bytecode::format::*;
//
//     #[test]
//     fn test_instruction_lifecycle() {
//         // Créer un programme simple qui additionne deux nombres
//         // R0 = 5
//         let load_r0 = Instruction::create_reg_imm8(Opcode::Load, 0, 5);
//
//         // R1 = 10
//         let load_r1 = Instruction::create_reg_imm8(Opcode::Load, 1, 10);
//
//         // R2 = R0 + R1
//         let add_r2 = Instruction::create_reg_reg(Opcode::Add, 2, 0);
//
//         // Encoder les instructions
//         let encoded_load_r0 = load_r0.encode();
//         let encoded_load_r1 = load_r1.encode();
//         let encoded_add_r2 = add_r2.encode();
//
//         // Combiner en un seul programme
//         let mut program = Vec::new();
//         program.extend_from_slice(&encoded_load_r0);
//         program.extend_from_slice(&encoded_load_r1);
//         program.extend_from_slice(&encoded_add_r2);
//
//         // Décoder séquentiellement les instructions
//         let mut offset = 0;
//         let mut decoded_instructions = Vec::new();
//
//         while offset < program.len() {
//             match Instruction::decode(&program[offset..]) {
//                 Ok((instr, size)) => {
//                     decoded_instructions.push(instr);
//                     offset += size;
//                 },
//                 Err(e) => {
//                     panic!("Failed to decode at offset {}: {:?}", offset, e);
//                 }
//             }
//         }
//
//         // Vérifier que nous avons décodé 3 instructions
//         assert_eq!(decoded_instructions.len(), 3);
//
//         // Vérifier chaque instruction
//         assert_eq!(decoded_instructions[0].opcode, Opcode::Load);
//         assert_eq!(decoded_instructions[1].opcode, Opcode::Load);
//         assert_eq!(decoded_instructions[2].opcode, Opcode::Add);
//     }
//
//     #[test]
//     fn test_create_complex_program() {
//         // Créer un programme plus complexe pour tester la robustesse
//         let mut instructions = Vec::new();
//
//         // R0 = 1 (compteur)
//         instructions.push(Instruction::create_reg_imm8(Opcode::Load, 0, 1));
//
//         // R1 = 0 (somme)
//         instructions.push(Instruction::create_reg_imm8(Opcode::Load, 1, 0));
//
//         // R2 = 10 (limite)
//         instructions.push(Instruction::create_reg_imm8(Opcode::Load, 2, 10));
//
//         // Label: boucle
//         // R1 += R0
//         instructions.push(Instruction::create_reg_reg(Opcode::Add, 1, 0));
//
//         // R0 += 1
//         instructions.push(Instruction::create_reg_imm8(Opcode::Add, 0, 1));
//
//         // Compare R0, R2
//         instructions.push(Instruction::create_reg_reg(Opcode::Cmp, 0, 2));
//
//         // Jump if less (R0 < R2)
//         // Normalement on utiliserait une adresse relative, mais pour les tests on utilise une valeur arbitraire
//         let jump_instr = Instruction::new(
//             Opcode::JmpIf,
//             InstructionFormat::new(ArgType::None, ArgType::RelativeAddr),
//             vec![0xF0, 0xFF, 0xFF, 0xFF] // -16 en complément à 2 (retour au début de la boucle)
//         );
//         instructions.push(jump_instr);
//
//         // Halt
//         instructions.push(Instruction::create_no_args(Opcode::Halt));
//
//         // Encoder tout le programme
//         let mut program_bytes = Vec::new();
//         for instr in &instructions {
//             program_bytes.extend_from_slice(&instr.encode());
//         }
//
//         // Décoder et vérifier
//         let mut offset = 0;
//         let mut decoded = Vec::new();
//
//         while offset < program_bytes.len() {
//             match Instruction::decode(&program_bytes[offset..]) {
//                 Ok((instr, size)) => {
//                     decoded.push(instr);
//                     offset += size;
//                 },
//                 Err(e) => {
//                     panic!("Failed to decode at offset {}: {:?}", offset, e);
//                 }
//             }
//         }
//
//         // Vérifier que nous avons le bon nombre d'instructions
//         assert_eq!(decoded.len(), instructions.len());
//
//         // Vérifier quelques instructions spécifiques
//         assert_eq!(decoded[0].opcode, Opcode::Load);
//         assert_eq!(decoded[6].opcode, Opcode::JmpIf);
//         assert_eq!(decoded[7].opcode, Opcode::Halt);
//     }
// }

// D'autres modules de test à ajouter au fur et à mesure
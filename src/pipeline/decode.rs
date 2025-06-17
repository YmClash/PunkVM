//src/pipeline/decode.rs

use crate::bytecode::instructions::{ArgValue, Instruction};
use crate::bytecode::opcodes::Opcode;
use crate::pipeline::{DecodeExecuteRegister, FetchDecodeRegister};
use crate::pvm::branch_predictor::{BranchPredictor, PredictorType};
use crate::pipeline::ras::{RASStats, ReturnAddressStack};

/// implementation de l'étage Decode du pipeline
pub struct DecodeStage {
    // Registre intermédiaire Decode -> Execute
    // pub decode_register: Option<DecodeExecuteRegister>,
    //données de l'état interne si nécessaire
    pub branch_predictor: BranchPredictor,
    pub ras : ReturnAddressStack,
}


/// Types d'opérations sur la pile
#[derive(Debug, Clone, Copy)]
pub enum StackOperation {
    Push,
    Pop,
}

impl DecodeStage {
    /// Crée un nouvel étage Decode

    pub fn new() -> Self {
        Self {
            branch_predictor: BranchPredictor::new(PredictorType::Dynamic),
            // ras: ReturnAddressStack::new(16), // Taille par défaut de 16 entrées
            ras: ReturnAddressStack::new(32),
        }
    }

    /// Effectue le décodage :
    ///
    /// - détermine rs1_index, rs2_index, rd_index
    /// - lit rs1_value, rs2_value dans la banque de registres (si applicable)
    /// - calcule un éventuel immediate
    /// - calcule branch_addr et mem_addr
    /// - retourne un DecodeExecuteRegister

    /// Traite l'étage Decode directement
    pub fn process_direct(
        &mut self,
        fd_reg: &FetchDecodeRegister,
        registers: &[u64],
    ) -> Result<DecodeExecuteRegister, String> {
        let instruction = &fd_reg.instruction;

        // Extraction des registres source et destination
        let (rs1_index, rs2_index, rd_index) = self.extract_registers(instruction)?;

        // lire rs1_value et rs2_value dans la banque de registres
        let rs1_value = rs1_index.map_or(0, |ix| {
            if ix < registers.len() {
                registers[ix]
            } else {
                // On pourrait renvoyer une erreur, ou 0. Au choix.
                0
            }
        });

        let rs2_value = rs2_index.map_or(0, |ix| {
            if ix < registers.len() {
                registers[ix]
            } else {
                // On pourrait renvoyer une erreur, ou 0. Au choix.
                0
            }
        });

        // Extraction de la valeur immédiate
        let immediate = self.extract_immediate(instruction)?;
        println!("Valeur immédiate extraite: {:?}", immediate);

        // Calcul de l'adresse de branchement (si instruction de branchement)
        let mut branch_addr = self.calculate_branch_address(instruction, fd_reg.pc)?;
        println!("Adresse de branchement calculée: {:?}", branch_addr);

        // si c'est une instruction de branchement, utiliser le prédicteur de branchement
        let mut prediction = None;
        if instruction.opcode.is_branch() && branch_addr.is_some() {
            // prédire l'adresse de branchement
            prediction = Some(self.branch_predictor.predict(fd_reg.pc as u64));
            println!("Branch prediction at PC={:X}: {:?}", fd_reg.pc, prediction);
        }

        // Gestion Special pour CALL et RET avec le RAS
        if instruction.opcode == Opcode::Call {
            // Pour CALL, mettre à jour le RAS avec l'adresse de retour
            let return_address = fd_reg.pc + instruction.total_size() as u32;
            self.ras.push(return_address);
            println!(" RAS UPDATE: CALL pushes return address: 0x{:08X}", return_address);
        } else if instruction.opcode == Opcode::Ret {
            // Pour RET, prédire l'adresse de retour avec le RAS
            if let Some(predicted_addr) = self.ras.predict() {
                branch_addr = Some(predicted_addr);
                println!(" RAS PREDICT: Ret branch address predicted: 0x{:08X}", predicted_addr);
            } else {
                println!(" RAS PREDICT: Ret branch address predicted: None (RAS is empty)");
                branch_addr = None;
            }
        }


        // Calcul de l'adresse mémoire (si instruction mémoire)
        let mem_addr = self.calculate_memory_address(instruction, registers)?;
        println!("Adresse mémoire calculée: {:?}", mem_addr);


        // Gestion speciale pour PUSH/POP/CALL/RET avec le Stack Pointer
        let (stack_operation, stack_value) = match instruction.opcode {
            Opcode::Push => {
                // Pour PUSH, prioriser la valeur immédiate si présente
                let value = if immediate.is_some() {
                    // PUSH immédiat - utiliser la valeur immédiate
                    immediate
                } else if let Some(reg) = rs1_index {
                    // PUSH registre - utiliser la valeur du registre
                    if reg < registers.len() {
                        Some(registers[reg])
                    } else {
                        None
                    }
                } else {
                    None
                };
                (Some(StackOperation::Push), value)
            },
            Opcode::Pop => {
                // Pour POP, pas de valeur spécifique
                (Some(StackOperation::Pop), None)
            },
            Opcode::Call => {
                // Pour CALL, on empilera l'adresse de retour
                let return_address = fd_reg.pc + instruction.total_size() as u32;
                (Some(StackOperation::Push), Some(return_address as u64))
            },
            Opcode::Ret => {
                // Pour RET, on dépilera l'adresse de retour
                (Some(StackOperation::Pop), None)
            },
            _ => (None, None),
        };




        Ok(DecodeExecuteRegister {
            instruction: instruction.clone(),
            pc: fd_reg.pc,
            rs1: rs1_index,
            rs2: rs2_index,
            rd: rd_index,
            rs1_value,
            rs2_value,
            immediate,
            branch_addr,
            branch_prediction: prediction,
            mem_addr,
            stack_operation,
            stack_value,
        })
    }

    /// Extrait les registres source et destination
    fn extract_registers(
        &self,
        instruction: &Instruction,
    ) -> Result<(Option<usize>, Option<usize>, Option<usize>), String> {
        let mut rs1 = None;
        let mut rs2 = None;
        let mut rd = None;

        // Vérifier d'abord si nous avons une instruction à trois registres
        // en essayant d'extraire un troisième argument
        if let Ok(ArgValue::Register(r3)) = instruction.get_arg3_value() {
            // Format à trois registres (rd, rs1, rs2)
            if let Ok(ArgValue::Register(r1)) = instruction.get_arg1_value() {
                rd = Some(r1 as usize);
                println!("Registre destination: {:?}", rd);
            }

            if let Ok(ArgValue::Register(r2)) = instruction.get_arg2_value() {
                rs1 = Some(r2 as usize);
                println!("Registre source 1: {:?}", rs1);
            }

            rs2 = Some(r3 as usize);
            println!("Registre source 2: {:?}", rs2);

            // Retourner immédiatement car c'est une instruction à trois registres
            return Ok((rs1, rs2, rd));
        }

        // Si ce n'est pas une instruction à trois registres, continuer avec la logique existante
        // Si ce n'est pas une instruction à trois registres, continuer avec la logique existante
        // Extraction en fonction du type d'instruction
        match instruction.opcode {
            // Instructions à deux registres (destination = premier argument)
            Opcode::Add
            | Opcode::Sub
            | Opcode::Mul
            | Opcode::Div
            | Opcode::And
            | Opcode::Or
            | Opcode::Xor
            | Opcode::Shl
            | Opcode::Shr
            | Opcode::Sar
            | Opcode::Rol
            | Opcode::Ror => {
                if let Ok(ArgValue::Register(r)) = instruction.get_arg1_value() {
                    rd = Some(r as usize);
                    rs1 = Some(r as usize); // Dans certaines architectures, rd est aussi rs1
                }

                if let Ok(ArgValue::Register(r)) = instruction.get_arg2_value() {
                    rs2 = Some(r as usize);
                }
            }

            // Instructions à un registre (destination = premier argument)
            Opcode::Inc | Opcode::Dec | Opcode::Neg | Opcode::Not => {
                if let Ok(ArgValue::Register(r)) = instruction.get_arg1_value() {
                    rd = Some(r as usize);
                    rs1 = Some(r as usize); // Le registre est à la fois source et destination
                }
            }

            // Instructions de comparaison (pas de registre destination)
            Opcode::Cmp | Opcode::Test => {
                if let Ok(ArgValue::Register(r)) = instruction.get_arg1_value() {
                    rs1 = Some(r as usize);
                    println!("DecodeStage: Registre source 1 pour CMP: {:?}", rs1);
                }

                if let Ok(ArgValue::Register(r)) = instruction.get_arg2_value() {
                    rs2 = Some(r as usize);
                    println!("DecodeStage: Registre source 2 pour CMP: {:?}", rs2);
                }
            }

            // Instructions de charge (load)
            Opcode::Load | Opcode::LoadB | Opcode::LoadW | Opcode::LoadD => {
                if let Ok(ArgValue::Register(r)) = instruction.get_arg1_value() {
                    rd = Some(r as usize);
                    println!("Registre destination: {:?}", rd);
                }

                // Extraction du registre base pour les adresses indexées
                if let Ok(ArgValue::RegisterOffset(r, _)) = instruction.get_arg2_value() {
                    rs1 = Some(r as usize);
                    println!("Registre base 1: {:?}", rs1);
                }
            }

            // Instructions de stockage (store)
            Opcode::Store | Opcode::StoreB | Opcode::StoreW | Opcode::StoreD => {
                if let Ok(ArgValue::Register(r)) = instruction.get_arg1_value() {
                    rs1 = Some(r as usize); // Registre contenant la valeur à stocker
                    println!("Registre source: {:?}", rs1);
                }

                // Extraction du registre base pour les adresses indexées
                if let Ok(ArgValue::RegisterOffset(r, _)) = instruction.get_arg2_value() {
                    rs2 = Some(r as usize);
                    println!("Registre base 2: {:?}", rs2);
                }
            }

            // Instructions de pile
            Opcode::Push => {
                if let Ok(ArgValue::Register(r)) = instruction.get_arg1_value() {
                    rs1 = Some(r as usize);
                    println!("Registre source pour PUSH: {:?}", rs1);
                }
            }

            Opcode::Pop => {
                if let Ok(ArgValue::Register(r)) = instruction.get_arg1_value() {
                    rd = Some(r as usize);
                    println!("Registre destination pour POP: {:?}", rd);
                }
            }

            // Instructions de branchement conditionnel
            Opcode::Jmp
            | Opcode::JmpIf
            | Opcode::JmpIfNot
            | Opcode::JmpIfEqual
            | Opcode::JmpIfNotEqual
            | Opcode::JmpIfGreater
            | Opcode::JmpIfGreaterEqual
            | Opcode::JmpIfLess
            | Opcode::JmpIfLessEqual
            | Opcode::JmpIfAbove
            | Opcode::JmpIfAboveEqual
            | Opcode::JmpIfBelow
            | Opcode::JmpIfBelowEqual
            | Opcode::JmpIfNotZero
            | Opcode::JmpIfZero
            | Opcode::JmpIfOverflow
            | Opcode::JmpIfNotOverflow
            | Opcode::JmpIfPositive
            | Opcode::JmpIfNegative => {
                // Ces instructions n'utilisent pas explicitement de registres,
                // mais se basent sur les flags définis par les instructions précédentes
            }

            // Instructions de Mov
            Opcode::Mov => {
                if let Ok(ArgValue::Register(r)) = instruction.get_arg1_value() {
                    rd = Some(r as usize);
                    println!("Registre destination pour MOV: {:?}", rd);
                }

                if let Ok(ArgValue::Immediate(imm)) = instruction.get_arg2_value() {
                    // c'est un "Mov Rd, imm" par exemple
                    // si c'est "create_reg_imm8(Opcode::Mov, reg, imm)"
                    // alors arg1=Register, arg2=Immediate8
                    // => le decode saura stocker l'immediate dans un champ (plus tard).
                    println!("Valeur immédiate pour MOV: {:?}", imm);
                }
            }
            // Instructions de contrôle de flux
            Opcode::Call => {
                // CALL ne nécessite pas de registres, juste une adresse cible
                println!("Instruction CALL détectée");
            }

            Opcode::Ret => {
                // RET ne nécessite pas de registres
                println!("Instruction RET détectée");
            }

            // Instructions d'arret
            Opcode::Halt => {
                // Pas de registre à extraire
                println!("Instruction HALT détectée");
            }

            // Autres instructions (par défaut)
            _ => {
                return Err(format!(
                    "Instruction non prise en charge: {:?}",
                    instruction.opcode
                ));
            }
        }

        Ok((rs1, rs2, rd))
    }

    /// Extrait la valeur immédiate (si présente)
    fn extract_immediate(&self, instruction: &Instruction) -> Result<Option<u64>, String> {
        // Recherche d'une valeur immédiate dans les arguments
        match instruction.get_arg1_value() {
            Ok(ArgValue::Immediate(imm)) => return Ok(Some(imm)),
            _ => {}
        }
        match instruction.get_arg2_value() {
            Ok(ArgValue::Immediate(imm)) => return Ok(Some(imm)),
            _ => {}
        }
        match instruction.get_arg3_value() {
            Ok(ArgValue::Immediate(imm)) => return Ok(Some(imm)),
            _ => {}
        }
        Ok(None)

    }



    // Dans DecodeStage::calculate_branch_address
    fn calculate_branch_address(
        &mut self,
        instruction: &Instruction,
        pc: u32,
    ) -> Result<Option<u32>, String> {
        // Vérifier si c'est une instruction de branchement
        if !instruction.opcode.is_branch() {
            return Ok(None);
        }

        match instruction.opcode {
            Opcode::Jmp | Opcode::JmpIf | Opcode::JmpIfNot |
            Opcode::JmpIfEqual | Opcode::JmpIfNotEqual |
            Opcode::JmpIfGreater | Opcode::JmpIfLess |
            Opcode::JmpIfGreaterEqual | Opcode::JmpIfLessEqual |
            Opcode::JmpIfZero | Opcode::JmpIfNotZero |
            Opcode::JmpIfAbove | Opcode::JmpIfAboveEqual |
            Opcode::JmpIfBelow | Opcode::JmpIfBelowEqual |
            Opcode::JmpIfOverflow | Opcode::JmpIfNotOverflow |
            Opcode::JmpIfPositive | Opcode::JmpIfNegative => {
                match instruction.get_arg2_value() {
                    Ok(ArgValue::RelativeAddr(offset)) => {
                        // IMPORTANT: L'offset est déjà calculé par rapport à PC + taille d'instruction
                        // dans calculate_branch_offset(), donc on doit faire :
                        let next_pc = pc + instruction.total_size() as u32;
                        let target_addr = (next_pc as i32 + offset) as u32;
                        // let target_addr = next_pc;
                        println!("DEBUG: Branch decode - PC=0x{:X}, size={}, next_pc=0x{:X}, offset={}, target=0x{:X}",
                                 pc, instruction.total_size(), next_pc, offset, target_addr);

                        println!("[[[DEBUG: Branch decode ]]] - PC=0x{:X}, size={}, next_pc=0x{:X}, offset={}, target=0x{:X}",
                                 pc, instruction.total_size(), next_pc, offset, target_addr);

                        println!("DEBUG: Instruction: {:?}", instruction);

                        //
                        Ok(Some(target_addr))
                    },
                    Ok(ArgValue::AbsoluteAddr(addr)) => Ok(Some(addr as u32)),
                    _ => Err("Format d'adresse de branchement invalide".to_string()),
                }
            },
            Opcode::Call => {
                match instruction.get_arg2_value() {
                    Ok(ArgValue::RelativeAddr(offset)) => {
                        let next_pc = pc + instruction.total_size() as u32;
                        let target_addr = (next_pc as i32 + offset) as u32;

                        println!("DEBUG: CALL Branch decode calc - PC=0x{:X}, size={}, next_pc=0x{:X}, offset={}, target=0x{:X}",
                                 pc, instruction.total_size(), next_pc, offset, target_addr);
                        Ok(Some(target_addr))
                    },
                    Ok(ArgValue::AbsoluteAddr(addr)) => Ok(Some(addr as u32)),
                    _ => Err("Format d'adresse d'appel invalide".to_string()),
                }
            },
            Opcode::Ret => Ok(None), // Géré par RAS
            _ => Ok(None),
        }
    }


    /// Calcule l'adresse mémoire (si instruction mémoire)
    fn calculate_memory_address(
        &self,
        instruction: &Instruction,
        registers: &[u64],
    ) -> Result<Option<u32>, String> {
        // Vérifier si c'est une instruction mémoire
        match instruction.opcode {
            Opcode::Load
            | Opcode::LoadB
            | Opcode::LoadW
            | Opcode::LoadD
            | Opcode::Store
            | Opcode::StoreB
            | Opcode::StoreW
            | Opcode::StoreD => {
                // On suppose que l'adresse est dans arg2
                match instruction.get_arg2_value() {
                    Ok(ArgValue::AbsoluteAddr(addr)) => Ok(Some(addr as u32)),
                    Ok(ArgValue::RelativeAddr(off)) => {
                        // Pas forcément implémenté
                        Ok(Some(off as u32))
                    }
                    Ok(ArgValue::RegisterOffset(reg, off)) => {
                        if (reg as usize) < registers.len() {
                            let base = registers[reg as usize];
                            let addr = base.wrapping_add(off as u64);
                            Ok(Some(addr as u32))
                        } else {
                            Err(format!("Register R{} out of range", reg))
                        }
                    }
                    Ok(ArgValue::Register(reg)) => {
                        if (reg as usize) < registers.len() {
                            Ok(Some(registers[reg as usize] as u32))
                        } else {
                            Err(format!("Register R{} out of range", reg))
                        }
                    }
                    _ => Err("Adresse mémoire invalide".to_owned()),
                }
            }
            _ => Ok(None),
        }
    }


    /// MEt a jour le RAS lors d'un CALL
    pub fn update_ras_for_call(&mut self,pc:u32,instruction_size:u32) {
        let return_address = pc + instruction_size;
        self.ras.push(return_address);
    }
    /// Met à jour le RAS lors d'un RET
    pub fn update_ras_for_ret(&mut self) -> Option<u32> {
        self.ras.pop()
    }

    /// Retourne öes Statistiques du RAS
    pub fn ras_stats(&self) -> RASStats{
        self.ras.stats()
    }

    /// Réinitialise l'étage Decode
    pub fn reset(&mut self) {
        // Pas d'état interne à réinitialiser pour cet étage
        // maintenant  que le stack est prise en charge par le RAS
        self.ras.reset();
    }
}

fn compute_target(pc: u32, offset: i32) -> u32 {
    println!("DecodeStage: Calcul de l'adresse cible pour le saut");
    // Calculer l'adresse cible
    (pc as i64 + offset as i64) as u32
}


//
// // Test unitaire pour l'étage Decode
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::bytecode::format;
//     use crate::bytecode::format::ArgType;
//     use crate::bytecode::format::InstructionFormat;
//     use crate::bytecode::instructions::Instruction;
//     use crate::bytecode::opcodes::Opcode;
//
//     #[test]
//     fn test_decode_stage_creation() {
//         let decode = DecodeStage::new();
//         // Pas grand-chose à tester pour la création, car l'étage n'a pas d'état interne
//         // Juste s'assurer que la création réussit
//         assert!(true);
//     }
//
//     #[test]
//     fn test_decode_stage_extract_registers_add_two_reg() {
//         let decode = DecodeStage::new();
//
//         // Instruction ADD R0, R1 (format à deux registres)
//         let add_instruction = Instruction::create_reg_reg(Opcode::Add, 0, 1);
//
//         let result = decode.extract_registers(&add_instruction);
//         assert!(result.is_ok());
//
//         let (rs1, rs2, rd) = result.unwrap();
//         assert_eq!(rd, Some(0)); // R0 est le registre destination
//         assert_eq!(rs1, Some(0)); // Dans certaines architectures, rd est aussi rs1
//         assert_eq!(rs2, Some(1)); // R1 est le deuxième registre source
//     }
//
//     #[test]
//     fn test_decode_stage_extract_registers_add_three_reg() {
//         let decode = DecodeStage::new();
//
//         // Instruction ADD R2, R0, R1 (format à trois registres)
//         let add_instruction = Instruction::create_reg_reg_reg(Opcode::Add, 2, 0, 1);
//
//         let result = decode.extract_registers(&add_instruction);
//         assert!(result.is_ok());
//
//         let (rs1, rs2, rd) = result.unwrap();
//         assert_eq!(rd, Some(2)); // R2 est le registre destination
//         assert_eq!(rs1, Some(0)); // R0 est le premier registre source
//         assert_eq!(rs2, Some(1)); // R1 est le deuxième registre source
//     }
//
//     #[test]
//     fn test_decode_stage_extract_immediate() {
//         let decode = DecodeStage::new();
//
//         // Instruction avec valeur immédiate (ADD R0, 5)
//         let add_imm_instruction = Instruction::create_reg_imm8(Opcode::Add, 0, 5);
//
//         let result = decode.extract_immediate(&add_imm_instruction);
//         assert!(result.is_ok());
//
//         let immediate = result.unwrap();
//         assert_eq!(immediate, Some(5));
//     }
//
//     // #[test]
//     // fn test_decode_stage_calculate_branch_address() {
//     //     let mut decode = DecodeStage::new();
//     //
//     //     // Instruction de saut relatif (JMP +8)
//     //     let jmp_instruction = Instruction::new(
//     //         Opcode::Jmp,
//     //         InstructionFormat::new(ArgType::None, ArgType::RelativeAddr, ArgType::None),
//     //         vec![8, 0, 0, 0] // Saut relatif de 8 bytes
//     //     );
//     //
//     //     let pc = 100;
//     //     let instruction_size = jmp_instruction.total_size() as u32;
//     //     let result = decode.calculate_branch_address(&jmp_instruction, pc);
//     //     assert!(result.is_ok());
//     //
//     //     let branch_addr = result.unwrap();
//     //     assert_eq!(branch_addr, Some(pc + instruction_size + 8)); // PC + taille_instruction + 8
//     // }
//
//     #[test]
//     fn test_decode_stage_calculate_branch_address() {
//         let mut decode = DecodeStage::new();
//
//         // Instruction de saut relatif (JMP +8)
//         let jmp_instruction = Instruction::new(
//             Opcode::Jmp,
//             InstructionFormat::new(ArgType::None, ArgType::RelativeAddr, ArgType::None),
//             vec![8, 0, 0, 0], // Saut relatif de 8 bytes
//         );
//
//         let pc = 100;
//         let instruction_size = jmp_instruction.total_size() as u32;
//         println!(
//             "PC: {},Instruction size: {}, Offset:8",
//             pc, instruction_size
//         );
//         let result = decode.calculate_branch_address(&jmp_instruction, pc);
//         assert!(result.is_ok());
//         println!("Result: {:?}", result);
//
//         let branch_addr = result.unwrap();
//         println!("Calculated Branch Address: {:?}", branch_addr);
//
//         //PC(100) + Instruction_size(8) + Offset(8) = 100 + 8 + 8 = 116
//         assert_eq!(branch_addr, Some(116)); // PC + taille_instruction + 8
//     }
//
//     #[test]
//     fn test_decode_stage_calculate_memory_address() {
//         let decode = DecodeStage::new();
//
//         // Instruction LOAD avec offset (LOAD R0, [R1+4])
//         let load_instruction = Instruction::new(
//             Opcode::Load,
//             InstructionFormat::new(ArgType::Register, ArgType::RegisterOffset, ArgType::None),
//             vec![0, 1, 4], // R0 = Mem[R1+4]
//         );
//
//         // Initialiser les registres
//         let mut registers = vec![0; 16];
//         registers[1] = 100; // R1 contient l'adresse 100
//
//         let result = decode.calculate_memory_address(&load_instruction, &registers);
//         assert!(result.is_ok());
//
//         let mem_addr = result.unwrap();
//         assert_eq!(mem_addr, Some(104)); // 100 + 4
//     }
//
//     #[test]
//     fn test_decode_stage_process_direct_two_reg() {
//         let mut decode = DecodeStage::new();
//
//         // Créer une instruction ADD R0, R1 (format à deux registres)
//         let add_instruction = Instruction::create_reg_reg(Opcode::Add, 0, 1);
//
//         // Créer un registre Fetch → Decode
//         let fd_reg = FetchDecodeRegister {
//             instruction: add_instruction,
//             pc: 100,
//         };
//
//         // Initialiser les registres
//         let registers = vec![5, 7, 0, 0, 0, 0, 0, 0];
//
//         // Décoder l'instruction
//         let result = decode.process_direct(&fd_reg, &registers);
//         assert!(result.is_ok());
//
//         // Vérifier le résultat
//         let de_reg = result.unwrap();
//         assert_eq!(de_reg.pc, 100);
//         assert_eq!(de_reg.rs1, Some(0));
//         assert_eq!(de_reg.rs2, Some(1));
//         assert_eq!(de_reg.rd, Some(0));
//         assert_eq!(de_reg.immediate, None);
//         assert_eq!(de_reg.branch_addr, None);
//         assert_eq!(de_reg.mem_addr, None);
//     }
//
//     #[test]
//     fn test_decode_stage_process_direct_three_reg() {
//         let mut decode = DecodeStage::new();
//
//         // Créer une instruction ADD R2, R0, R1 (format à trois registres)
//         let add_instruction = Instruction::create_reg_reg_reg(Opcode::Add, 2, 0, 1);
//
//         // Créer un registre Fetch → Decode
//         let fd_reg = FetchDecodeRegister {
//             instruction: add_instruction,
//             pc: 100,
//         };
//
//         // Initialiser les registres
//         let registers = vec![5, 7, 0, 0, 0, 0, 0, 0];
//
//         // Décoder l'instruction
//         let result = decode.process_direct(&fd_reg, &registers);
//         assert!(result.is_ok());
//
//         // Vérifier le résultat
//         let de_reg = result.unwrap();
//         assert_eq!(de_reg.pc, 100);
//         assert_eq!(de_reg.rs1, Some(0));
//         assert_eq!(de_reg.rs2, Some(1));
//         assert_eq!(de_reg.rd, Some(2));
//         assert_eq!(de_reg.immediate, None);
//         assert_eq!(de_reg.branch_addr, None);
//         assert_eq!(de_reg.mem_addr, None);
//     }
//
//     #[test]
//     fn test_decode_stage_arithmetic_operations_three_reg() {
//         let mut decode = DecodeStage::new();
//         let registers = vec![10, 20, 0, 0, 0, 0, 0, 0]; // R0=10, R1=20
//
//         // Tester plusieurs opérations arithmétiques avec format à trois registres
//         let ops = [Opcode::Add, Opcode::Sub, Opcode::Mul, Opcode::Div];
//
//         for op in &ops {
//             // Instruction arithmétique R2, R0, R1
//             let instruction = Instruction::create_reg_reg_reg(*op, 2, 0, 1);
//
//             // Créer un registre Fetch → Decode
//             let fd_reg = FetchDecodeRegister {
//                 instruction,
//                 pc: 100,
//             };
//
//             // Décodage
//             let result = decode.process_direct(&fd_reg, &registers);
//             assert!(result.is_ok());
//
//             let de_reg = result.unwrap();
//             assert_eq!(de_reg.rs1, Some(0));
//             assert_eq!(de_reg.rs2, Some(1));
//             assert_eq!(de_reg.rd, Some(2));
//             assert_eq!(de_reg.instruction.opcode, *op);
//         }
//     }
//
//     #[test]
//     fn test_decode_stage_mixed_formats() {
//         let mut decode = DecodeStage::new();
//         let registers = vec![5, 10, 0, 0, 0, 0, 0, 0]; // R0=5, R1=10
//
//         // Format à trois registres: ADD R2, R0, R1
//         let add_three_reg = Instruction::create_reg_reg_reg(Opcode::Add, 2, 0, 1);
//         let fd_reg_add3 = FetchDecodeRegister {
//             instruction: add_three_reg,
//             pc: 100,
//         };
//
//         // Format à deux registres: SUB R3, R2
//         let sub_two_reg = Instruction::create_reg_reg(Opcode::Sub, 3, 2);
//         let fd_reg_sub2 = FetchDecodeRegister {
//             instruction: sub_two_reg,
//             pc: 108,
//         };
//
//         // Format à un registre: INC R4
//         let inc_one_reg = Instruction::create_single_reg(Opcode::Inc, 4);
//         let fd_reg_inc1 = FetchDecodeRegister {
//             instruction: inc_one_reg,
//             pc: 112,
//         };
//
//         // Vérifier le décodage des trois formats
//         let result_add3 = decode.process_direct(&fd_reg_add3, &registers);
//         assert!(result_add3.is_ok());
//         let de_reg_add3 = result_add3.unwrap();
//         assert_eq!(de_reg_add3.rd, Some(2));
//         assert_eq!(de_reg_add3.rs1, Some(0));
//         assert_eq!(de_reg_add3.rs2, Some(1));
//
//         let result_sub2 = decode.process_direct(&fd_reg_sub2, &registers);
//         assert!(result_sub2.is_ok());
//         let de_reg_sub2 = result_sub2.unwrap();
//         assert_eq!(de_reg_sub2.rd, Some(3));
//         assert_eq!(de_reg_sub2.rs1, Some(3)); // Dans certaines architectures, rd est aussi rs1
//         assert_eq!(de_reg_sub2.rs2, Some(2));
//
//         let result_inc1 = decode.process_direct(&fd_reg_inc1, &registers);
//         assert!(result_inc1.is_ok());
//         let de_reg_inc1 = result_inc1.unwrap();
//         assert_eq!(de_reg_inc1.rd, Some(4));
//         assert_eq!(de_reg_inc1.rs1, Some(4)); // Le registre est à la fois source et destination
//         assert_eq!(de_reg_inc1.rs2, None); // Pas de second registre source
//     }
//
//     #[test]
//     fn test_decode_stage_reset() {
//         let mut decode = DecodeStage::new();
//
//         // L'étage Decode n'a pas d'état interne, donc reset() ne fait rien
//         // On s'assure juste que la méthode peut être appelée sans erreur
//         decode.reset();
//         assert!(true);
//     }
//
//     #[test]
//     fn test_decode_stage_extract_registers_three_register_instruction() {
//         let decode = DecodeStage::new();
//
//         // Instruction ADD R2, R0, R1 (format à trois registres)
//         let add_instruction = Instruction::create_reg_reg_reg(Opcode::Add, 2, 0, 1);
//
//         let result = decode.extract_registers(&add_instruction);
//         assert!(result.is_ok());
//
//         let (rs1, rs2, rd) = result.unwrap();
//         assert_eq!(rd, Some(2)); // R2 est le registre destination
//         assert_eq!(rs1, Some(0)); // R0 est le premier registre source
//         assert_eq!(rs2, Some(1)); // R1 est le deuxième registre source
//     }
//
//     #[test]
//     fn test_decode_stage_process_direct_three_register_instruction() {
//         let mut decode = DecodeStage::new();
//
//         // Créer une instruction à trois registres: ADD R2, R0, R1
//         let add_instruction = Instruction::create_reg_reg_reg(Opcode::Add, 2, 0, 1);
//
//         // Créer un registre Fetch → Decode
//         let fd_reg = FetchDecodeRegister {
//             instruction: add_instruction,
//             pc: 100,
//         };
//
//         // Initialiser les registres
//         let registers = vec![5, 7, 0, 0, 0, 0, 0, 0];
//
//         // Décoder l'instruction
//         let result = decode.process_direct(&fd_reg, &registers);
//         assert!(result.is_ok());
//
//         // Vérifier le résultat
//         let de_reg = result.unwrap();
//         assert_eq!(de_reg.pc, 100);
//         assert_eq!(de_reg.rs1, Some(0)); // Premier registre source
//         assert_eq!(de_reg.rs2, Some(1)); // Deuxième registre source
//         assert_eq!(de_reg.rd, Some(2)); // Registre destination
//         assert_eq!(de_reg.immediate, None);
//         assert_eq!(de_reg.branch_addr, None);
//         assert_eq!(de_reg.mem_addr, None);
//     }
//
//     #[test]
//     fn test_decode_stage_extract_registers_multiple_formats() {
//         let decode = DecodeStage::new();
//
//         // Format à trois registres: MUL R3, R1, R2
//         let mul_instruction = Instruction::create_reg_reg_reg(Opcode::Mul, 3, 1, 2);
//
//         // Format à deux registres: SUB R4, R3
//         let sub_instruction = Instruction::create_reg_reg(Opcode::Sub, 4, 3);
//
//         // Format à un registre: INC R5
//         let inc_instruction = Instruction::create_single_reg(Opcode::Inc, 5);
//
//         // Tester l'extraction des registres pour chaque format
//         let result_mul = decode.extract_registers(&mul_instruction);
//         assert!(result_mul.is_ok());
//         let (rs1_mul, rs2_mul, rd_mul) = result_mul.unwrap();
//         assert_eq!(rd_mul, Some(3));
//         assert_eq!(rs1_mul, Some(1));
//         assert_eq!(rs2_mul, Some(2));
//
//         let result_sub = decode.extract_registers(&sub_instruction);
//         assert!(result_sub.is_ok());
//         let (rs1_sub, rs2_sub, rd_sub) = result_sub.unwrap();
//         assert_eq!(rd_sub, Some(4));
//         assert_eq!(rs1_sub, Some(4)); // Dans certaines architectures, rd est aussi rs1
//         assert_eq!(rs2_sub, Some(3));
//
//         let result_inc = decode.extract_registers(&inc_instruction);
//         assert!(result_inc.is_ok());
//         let (rs1_inc, rs2_inc, rd_inc) = result_inc.unwrap();
//         assert_eq!(rd_inc, Some(5));
//         assert_eq!(rs1_inc, Some(5)); // Le registre est à la fois source et destination
//         assert_eq!(rs2_inc, None); // Pas de deuxième registre source
//     }
//
//     #[test]
//     fn test_decode_stage_arithmetic_operations_three_registers() {
//         let mut decode = DecodeStage::new();
//         let registers = vec![10, 20, 0, 0, 0, 0, 0, 0]; // R0=10, R1=20
//
//         // Tester les opérations arithmétiques avec trois registres
//         for &op in &[Opcode::Add, Opcode::Sub, Opcode::Mul, Opcode::Div] {
//             let instruction = Instruction::create_reg_reg_reg(op, 2, 0, 1);
//
//             let fd_reg = FetchDecodeRegister {
//                 instruction,
//                 pc: 100,
//             };
//
//             let result = decode.process_direct(&fd_reg, &registers);
//             assert!(result.is_ok());
//
//             let de_reg = result.unwrap();
//             assert_eq!(de_reg.rd, Some(2));
//             assert_eq!(de_reg.rs1, Some(0));
//             assert_eq!(de_reg.rs2, Some(1));
//             assert_eq!(de_reg.instruction.opcode, op);
//         }
//     }
//
//     #[test]
//     fn test_decode_stage_logical_operations_three_registers() {
//         let mut decode = DecodeStage::new();
//         let registers = vec![0xF0, 0x0F, 0, 0, 0, 0, 0, 0]; // R0=0xF0, R1=0x0F
//
//         // Tester les opérations logiques avec trois registres
//         for &op in &[Opcode::And, Opcode::Or, Opcode::Xor] {
//             let instruction = Instruction::create_reg_reg_reg(op, 2, 0, 1);
//
//             let fd_reg = FetchDecodeRegister {
//                 instruction,
//                 pc: 100,
//             };
//
//             let result = decode.process_direct(&fd_reg, &registers);
//             assert!(result.is_ok());
//
//             let de_reg = result.unwrap();
//             assert_eq!(de_reg.rd, Some(2));
//             assert_eq!(de_reg.rs1, Some(0));
//             assert_eq!(de_reg.rs2, Some(1));
//             assert_eq!(de_reg.instruction.opcode, op);
//         }
//     }
//
//     #[test]
//     fn test_decode_stage_mixed_format_program() {
//         let mut decode = DecodeStage::new();
//         let registers = vec![5, 10, 0, 0, 0, 0, 0, 0]; // R0=5, R1=10
//
//         // Simuler un petit programme qui calcule: R3 = (R0 + R1) * 2
//
//         // ADD R2, R0, R1  (R2 = R0 + R1, format à trois registres)
//         let add_instruction = Instruction::create_reg_reg_reg(Opcode::Add, 2, 0, 1);
//         let fd_reg_add = FetchDecodeRegister {
//             instruction: add_instruction,
//             pc: 100,
//         };
//
//         // MUL R3, R2, 2   (R3 = R2 * 2, format à deux registres avec immédiat)
//         let mul_instruction = Instruction::create_reg_imm8(Opcode::Mul, 3, 2);
//         let fd_reg_mul = FetchDecodeRegister {
//             instruction: mul_instruction,
//             pc: 108,
//         };
//
//         // Décoder la première instruction
//         let result_add = decode.process_direct(&fd_reg_add, &registers);
//         assert!(result_add.is_ok());
//         let de_reg_add = result_add.unwrap();
//         assert_eq!(de_reg_add.rd, Some(2));
//         assert_eq!(de_reg_add.rs1, Some(0));
//         assert_eq!(de_reg_add.rs2, Some(1));
//
//         // Décoder la deuxième instruction
//         let result_mul = decode.process_direct(&fd_reg_mul, &registers);
//         assert!(result_mul.is_ok());
//         let de_reg_mul = result_mul.unwrap();
//         assert_eq!(de_reg_mul.rd, Some(3));
//         assert_eq!(de_reg_mul.rs1, Some(3)); // Dans certaines architectures, rd est aussi rs1
//         assert_eq!(de_reg_mul.immediate, Some(2));
//     }
// }
//
// //
// // impl<'a> PipelineStage<'a> for DecodeStage {
// //     type Input = (FetchDecodeRegister, &'a [u64]);
// //     type Output = DecodeExecuteRegister;
// //
// //     fn process(&mut self, input: &Self::Input) -> Result<Self::Output, String> {
// //         let (fd_reg, registers) = input;
// //         self.process(fd_reg, *registers)
// //     }
// //
// //     fn reset(&mut self) {
// //         // Reset direct sans appel récursif
// //     }
// // }
// //
//
//
//
//
// ///////////////////////////////////////////////////////////////////////////////////////////////
// //// Calcule l'adresse de branchement (si instruction de branchement)
// // fn calculate_branch_address(
// //     &mut self,
// //     instruction: &Instruction,
// //     pc: u32,
// // ) -> Result<Option<u32>, String> {
// //     // Vérifier si c'est une instruction de branchement
// //     if !instruction.opcode.is_branch() {
// //         return Ok(None);
// //     }
// //
// //     match instruction.opcode {
// //         // Saut absolu
// //         Opcode::Jmp => {
// //             match instruction.get_arg2_value() {
// //                 Ok(ArgValue::AbsoluteAddr(addr)) => Ok(Some(addr as u32)),
// //                 // Ok(ArgValue::RelativeAddr(offset)) => Ok(Some((pc as i64  + offset as i64) as u32)),
// //                 Ok(ArgValue::RelativeAddr(offset)) => {
// //                     // Attention: pour un saut relatif, l'offset doit être calculé
// //                     // à partir de l'adresse de l'instruction SUIVANTE (pc + instruction.total_size())
// //                     let pc = pc + instruction.total_size() as u32;
// //                     let next_pc = pc + offset as u32;
// //                     println!("Branch address: next_pc = {}, offset = {}", next_pc, offset);
// //                     // Ok(Some((next_pc as i64 + offset as i64) as u32))
// //                     Ok(Some(next_pc))
// //                 }
// //
// //                 _ => Err("Format d'adresse de saut invalide".to_string()),
// //             }
// //         }
// //         // Saut conditionnel
// //         Opcode::JmpIf
// //         | Opcode::JmpIfNot
// //         |Opcode::JmpIfEqual
// //         | Opcode::JmpIfNotEqual
// //         | Opcode::JmpIfGreater
// //         | Opcode::JmpIfGreaterEqual
// //         | Opcode::JmpIfLess
// //         | Opcode::JmpIfLessEqual
// //         | Opcode::JmpIfAbove
// //         | Opcode::JmpIfAboveEqual
// //         | Opcode::JmpIfBelow
// //         | Opcode::JmpIfBelowEqual
// //         | Opcode::JmpIfZero
// //         | Opcode::JmpIfNotZero
// //         | Opcode::JmpIfOverflow
// //         | Opcode::JmpIfNotOverflow
// //         | Opcode::JmpIfPositive
// //         | Opcode::JmpIfNegative => {
// //             match instruction.get_arg2_value() {
// //                 Ok(ArgValue::AbsoluteAddr(addr)) => Ok(Some(addr as u32)),
// //                 Ok(ArgValue::RelativeAddr(offset)) => {
// //                     // Attention: pour un saut relatif, l'offset doit être calculé
// //                     // à partir de l'adresse de l'instruction SUIVANTE (pc + instruction.total_size())
// //                     let pc = pc + instruction.total_size() as u32;
// //                     let next_pc = pc + offset as u32;
// //                     println!("Branch address: next_pc = {}, offset = {}", next_pc, offset);
// //                     // Ok(Some((next_pc as i64 + offset as i64) as u32))
// //                     Ok(Some(next_pc))
// //                 }
// //                 _ => Err("Format d'adresse de saut conditionnel invalide".to_string()),
// //             }
// //         }
// //
// //         // Appel de fonction
// //         Opcode::Call => match instruction.get_arg2_value() {
// //             Ok(ArgValue::AbsoluteAddr(addr)) => Ok(Some(addr as u32)),
// //             Ok(ArgValue::RelativeAddr(offset)) => Ok(Some((pc as i64 + offset as i64) as u32)),
// //             _ => Err("Format d'adresse d'appel invalide".to_string()),
// //         },
// //
// //         // Retour de fonction (pas d'adresse explicite)
// //         Opcode::Ret => Ok(None),
// //
// //         // Autres instructions de branchement (si ajoutées à l'avenir)
// //         _ => Ok(None),
// //     }
// // }
//
// /////////////////////////////////////////////////////////////////////////////////////////////////////////////////

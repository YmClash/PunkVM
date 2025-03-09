//src/pipeline/execute.rs

use crate::alu::alu::{ALU, ALUOperation, BranchCondition};
use crate::bytecode::opcodes::Opcode;
use crate::pipeline::{DecodeExecuteRegister, ExecuteMemoryRegister,};



/// Implementation de l'étage Execute du pipeline
pub struct ExecuteStage {
    // Unité ALU
    // Aucun état interne pour l'instant
}

impl ExecuteStage{
    /// Crée un nouvel étage Execute
    pub fn new() -> Self {
        Self {}
    }

    /// Traite l'étage Execute directement
    pub fn process_direct(&mut self, ex_reg: &DecodeExecuteRegister, alu: &mut ALU) -> Result<ExecuteMemoryRegister, String> {
        // Valeurs par défaut
        let mut alu_result = 0;
        let mut branch_taken = false;
        let mut branch_target = None;
        let mut store_value = None;
        let mut mem_addr = ex_reg.mem_addr;

        // Récupérer les valeurs des registres sources (si présents)
        // let rs1_value = match ex_reg.rs1 {
        //     Some(reg) => {
        //         if reg < ex_reg.instruction.args.len() {
        //             ex_reg.instruction.args[reg] as u64
        //         } else {
        //             0
        //         }
        //     },
        //     None => 0,
        // };
        let is_three_reg_format = ex_reg.instruction.args.len() >= 3 &&
            ex_reg.rs1.is_some() &&
            ex_reg.rs2.is_some() &&
            ex_reg.rd.is_some();

        // Récupérer les valeurs des registres sources (si présents)
        let rs1_value = match ex_reg.rs1 {
            Some(reg) => {
                // Format à trois registres
                if is_three_reg_format {
                    if ex_reg.instruction.args.len() > 1 {
                        ex_reg.instruction.args[1] as u64
                    } else {
                        0
                    }
                }
                // Format à un seul registre (INC, DEC, NEG, NOT, etc.)
                else if ex_reg.instruction.args.len() == 1 && ex_reg.rs2.is_none() {
                    // La valeur est directement dans args[0]
                    ex_reg.instruction.args[0] as u64
                }
                // Autres formats
                else {
                    if reg < ex_reg.instruction.args.len() {
                        ex_reg.instruction.args[reg] as u64
                    } else {
                        0
                    }
                }
            },
            None => 0,
        };

        let rs2_value = if is_three_reg_format {
            ex_reg.instruction.args[2] as u64
        } else {
            match ex_reg.rs2 {
                Some(reg) if reg < ex_reg.instruction.args.len() =>
                    ex_reg.instruction.args[reg] as u64,
                _ => match ex_reg.immediate {
                    Some(imm) => imm,
                    None => 0,
                }
            }
        };

        // Exécuter l'opération en fonction de l'opcode
        match ex_reg.instruction.opcode {
            // Instructions arithmétiques et logiques
            Opcode::Add => {
                alu_result = alu.execute(ALUOperation::Add, rs1_value, rs2_value)?;
                println!("Execute ADD: rs1_value={}, rs2_value={}, alu_result={}", rs1_value, rs2_value, alu_result);
            },

            Opcode::Sub => {
                alu_result = alu.execute(ALUOperation::Sub, rs1_value, rs2_value)?;
                println!("Execute SUB: rs1_value={}, rs2_value={}, alu_result={}", rs1_value, rs2_value, alu_result);
            },

            Opcode::Mul => {
                alu_result = alu.execute(ALUOperation::Mul, rs1_value, rs2_value)?;
                println!("Execute MUL: rs1_value={}, rs2_value={}, alu_result={}", rs1_value, rs2_value, alu_result);
            },

            Opcode::Div => {
                alu_result = alu.execute(ALUOperation::Div, rs1_value, rs2_value)?;
                println!("Execute DIV: rs1_value={}, rs2_value={}, alu_result={}", rs1_value, rs2_value, alu_result);
            },

            Opcode::Mod => {
                alu_result = alu.execute(ALUOperation::Mod, rs1_value, rs2_value)?;
                println!("Execute MOD: rs1_value={}, rs2_value={}, alu_result={}", rs1_value, rs2_value, alu_result);
            },

            Opcode::Inc => {
                alu_result = alu.execute(ALUOperation::Inc, rs1_value, 0)?;
                println!("Execute INC: rs1_value={}, alu_result={}", rs1_value, alu_result);
            },

            Opcode::Dec => {
                alu_result = alu.execute(ALUOperation::Dec, rs1_value, 0)?;
                println!("Execute DEC: rs1_value={}, alu_result={}", rs1_value, alu_result);
            },

            Opcode::Neg => {
                alu_result = alu.execute(ALUOperation::Neg, rs1_value, 0)?;
                println!("Execute NEG: rs1_value={}, alu_result={}", rs1_value, alu_result);
            },

            Opcode::And => {
                alu_result = alu.execute(ALUOperation::And, rs1_value, rs2_value)?;
                println!("Execute AND: rs1_value={}, rs2_value={}, alu_result={}", rs1_value, rs2_value, alu_result);
            },

            Opcode::Or => {
                alu_result = alu.execute(ALUOperation::Or, rs1_value, rs2_value)?;
                println!("Execute OR: rs1_value={}, rs2_value={}, alu_result={}", rs1_value, rs2_value, alu_result);
            },

            Opcode::Xor => {
                alu_result = alu.execute(ALUOperation::Xor, rs1_value, rs2_value)?;
                println!("Execute XOR: rs1_value={}, rs2_value={}, alu_result={}", rs1_value, rs2_value, alu_result);
            },

            Opcode::Not => {
                alu_result = alu.execute(ALUOperation::Not, rs1_value, 0)?;
                println!("Execute NOT: rs1_value={}, alu_result={}", rs1_value, alu_result);
            },

            Opcode::Nop => {
                // Pas d'opération
                alu_result = 0; // Pas utilisé
                println!("Execute NOP");
            },

            Opcode::Shl => {
                alu_result = alu.execute(ALUOperation::Shl, rs1_value, rs2_value)?;
                println!("Execute SHL: rs1_value={}, rs2_value={}, alu_result={}", rs1_value, rs2_value, alu_result);
            },

            Opcode::Shr => {
                alu_result = alu.execute(ALUOperation::Shr, rs1_value, rs2_value)?;
                println!("Execute SHR: rs1_value={}, rs2_value={}, alu_result={}", rs1_value, rs2_value, alu_result);
            },

            Opcode::Sar => {
                alu_result = alu.execute(ALUOperation::Sar, rs1_value, rs2_value)?;
                println!("Execute SAR: rs1_value={}, rs2_value={}, alu_result={}", rs1_value, rs2_value, alu_result);
            },

            Opcode::Rol => {
                alu_result = alu.execute(ALUOperation::Rol, rs1_value, rs2_value)?;
                println!("Execute ROL: rs1_value={}, rs2_value={}, alu_result={}", rs1_value, rs2_value, alu_result);
            },

            Opcode::Ror => {
                alu_result = alu.execute(ALUOperation::Ror, rs1_value, rs2_value)?;
                println!("Execute ROR: rs1_value={}, rs2_value={}, alu_result={}", rs1_value, rs2_value, alu_result);
            },

            // Instructions de comparaison
            Opcode::Cmp => {
                // Compare mais ne stocke pas le résultat
                alu.execute(ALUOperation::Cmp, rs1_value, rs2_value)?;
                alu_result = 0; // Pas utilisé
                println!("Execute CMP: rs1_value={}, rs2_value={}", rs1_value, rs2_value);
            },

            Opcode::Test => {
                // Test (AND logique) mais ne stocke pas le résultat
                alu.execute(ALUOperation::Test, rs1_value, rs2_value)?;
                alu_result = 0; // Pas utilisé
                println!("Execute TEST: rs1_value={}, rs2_value={}", rs1_value, rs2_value);
            },

            // Instructions de contrôle de flux
            Opcode::Jmp => {
                // Saut inconditionnel
                branch_taken = true;
                branch_target = ex_reg.branch_addr;
                println!("Execute JMP: branch_target={:?}", branch_target);
            },

            Opcode::JmpIf => {
                // Saut conditionnel si la condition est vraie
                branch_taken = alu.check_condition(BranchCondition::Equal);
                branch_target = ex_reg.branch_addr;
                println!("Execute JMP_IF: branch_taken={}, branch_target={:?}", branch_taken, branch_target);
            },

            Opcode::JmpIfNot => {
                // Saut conditionnel si la condition est fausse
                branch_taken = alu.check_condition(BranchCondition::NotEqual);
                branch_target = ex_reg.branch_addr;
                println!("Execute JMP_IF_NOT: branch_taken={}, branch_target={:?}", branch_taken, branch_target);
            },

            // Instructions d'accès mémoire
            Opcode::Load | Opcode::LoadB | Opcode::LoadW | Opcode::LoadD => {
                // Ces instructions finalisent leur exécution dans l'étage Memory
                alu_result = 0; // Sera remplacé par la valeur chargée
                println!("Execute LOAD: rs1_value={}, mem_addr={:?}", rs1_value, mem_addr);
            },

            Opcode::Store | Opcode::StoreB | Opcode::StoreW | Opcode::StoreD => {
                // Préparer la valeur à stocker
                store_value = Some(rs1_value);
                println!("Execute STORE: rs1_value={}, mem_addr={:?}", rs1_value, mem_addr);
            },

            Opcode::Push => {
                // Préparer la valeur à empiler
                store_value = Some(rs1_value);
                // L'adresse est calculée dans l'étage Memory
                println!("Execute PUSH: rs1_value={}, mem_addr={:?}", rs1_value, mem_addr);
            },

            Opcode::Pop => {
                // L'adresse est calculée dans l'étage Memory
                // La valeur sera chargée dans l'étage Memory
            },

            // Instructions spéciales
            Opcode::Syscall => {
                // Traitées séparément (pas implémenté pour l'instant)
                return Err("Syscall non implémenté".to_string());
            },

            Opcode::Break => {
                // Instruction de débogage, ne fait rien dans le simulateur
                println!("Execute BREAK");
            },

            Opcode::Halt => {
                // Instruction spéciale pour terminer l'exécution
                // Gérée au niveau du pipeline
                println!("Execute HALT");
            },

            // Instructions étendues et autres
            _ => {
                return Err(format!("Opcode non supporté: {:?}", ex_reg.instruction.opcode));
            },
        }

        Ok(ExecuteMemoryRegister {
            instruction: ex_reg.instruction.clone(),
            alu_result,
            rd: ex_reg.rd,
            store_value,
            mem_addr,
            branch_target,
            branch_taken,
        })
    }

    /// Réinitialise l'étage Execute
    pub fn reset(&mut self) {
        // Pas d'état interne à réinitialiser
    }
}


// Test unitaire pour l'étage Execute
#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::opcodes::Opcode;
    use crate::bytecode::instructions::Instruction;
    use crate::bytecode::format::InstructionFormat;
    use crate::bytecode::format::ArgType;
    use crate::pipeline::DecodeExecuteRegister;

    #[test]
    fn test_execute_stage_creation() {
        let execute = ExecuteStage::new();
        // Vérifier que la création réussit
        assert!(true);
    }

    #[test]
    fn test_execute_stage_reset() {
        let mut execute = ExecuteStage::new();
        execute.reset();
        // L'étage Execute n'a pas d'état interne, donc reset() ne fait rien
        // On s'assure juste que la méthode peut être appelée sans erreur
        assert!(true);
    }

    #[test]
    fn test_execute_add_instruction_two_reg() {
        let mut execute = ExecuteStage::new();
        let mut alu = ALU::new();

        // Créer une instruction ADD R0, R1 (format à deux registres)
        let add_instruction = Instruction::create_reg_reg(Opcode::Add, 0, 1);

        // Créer un registre Decode → Execute
        let de_reg = DecodeExecuteRegister {
            instruction: add_instruction,
            pc: 100,
            rs1: Some(0),
            rs2: Some(1),
            rd: Some(0),
            immediate: None,
            branch_addr: None,
            mem_addr: None,
        };

        // Ajuster les valeurs des registres dans les arguments
        // Normalement ces valeurs seraient lues des registres réels
        let mut instr = de_reg.instruction.clone();
        instr.args = vec![5, 7]; // R0 = 5, R1 = 7

        let de_reg_with_values = DecodeExecuteRegister {
            instruction: instr,
            ..de_reg
        };

        // Exécuter l'instruction
        let result = execute.process_direct(&de_reg_with_values, &mut alu);
        assert!(result.is_ok());

        // Vérifier le résultat
        let em_reg = result.unwrap();
        assert_eq!(em_reg.alu_result, 12); // 5 + 7 = 12
        assert_eq!(em_reg.rd, Some(0));
        assert_eq!(em_reg.branch_taken, false);
        assert_eq!(em_reg.branch_target, None);
    }

    #[test]
    fn test_execute_add_instruction_three_reg() {
        let mut execute = ExecuteStage::new();
        let mut alu = ALU::new();

        // Créer une instruction ADD R2, R0, R1 (format à trois registres)
        let add_instruction = Instruction::create_reg_reg_reg(Opcode::Add, 2, 0, 1);

        // Créer un registre Decode → Execute
        let de_reg = DecodeExecuteRegister {
            instruction: add_instruction,
            pc: 100,
            rs1: Some(0),  // Premier registre source
            rs2: Some(1),  // Deuxième registre source
            rd: Some(2),   // Registre destination
            immediate: None,
            branch_addr: None,
            mem_addr: None,
        };

        // Ajuster les valeurs des registres dans les arguments
        let mut instr = de_reg.instruction.clone();
        instr.args = vec![0, 5, 7]; // R0 = 5, R1 = 7, R2 n'est pas encore défini

        let de_reg_with_values = DecodeExecuteRegister {
            instruction: instr,
            ..de_reg
        };

        // Exécuter l'instruction
        let result = execute.process_direct(&de_reg_with_values, &mut alu);
        assert!(result.is_ok());

        // Vérifier le résultat
        let em_reg = result.unwrap();
        assert_eq!(em_reg.alu_result, 12); // 5 + 7 = 12
        assert_eq!(em_reg.rd, Some(2));    // Le résultat va dans R2
        assert_eq!(em_reg.branch_taken, false);
        assert_eq!(em_reg.branch_target, None);
    }

    #[test]
    fn test_execute_sub_instruction_two_reg() {
        let mut execute = ExecuteStage::new();
        let mut alu = ALU::new();

        // Créer une instruction SUB R0, R1 (format à deux registres)
        let sub_instruction = Instruction::create_reg_reg(Opcode::Sub, 0, 1);

        // Créer un registre Decode → Execute
        let de_reg = DecodeExecuteRegister {
            instruction: sub_instruction,
            pc: 100,
            rs1: Some(0),
            rs2: Some(1),
            rd: Some(0),
            immediate: None,
            branch_addr: None,
            mem_addr: None,
        };

        // Ajuster les valeurs des registres dans les arguments
        let mut instr = de_reg.instruction.clone();
        instr.args = vec![10, 7]; // R0 = 10, R1 = 7

        let de_reg_with_values = DecodeExecuteRegister {
            instruction: instr,
            ..de_reg
        };

        // Exécuter l'instruction
        let result = execute.process_direct(&de_reg_with_values, &mut alu);
        assert!(result.is_ok());

        // Vérifier le résultat
        let em_reg = result.unwrap();
        assert_eq!(em_reg.alu_result, 3); // 10 - 7 = 3
    }

    #[test]
    fn test_execute_sub_instruction_three_reg() {
        let mut execute = ExecuteStage::new();
        let mut alu = ALU::new();

        // Créer une instruction SUB R2, R0, R1 (format à trois registres)
        let sub_instruction = Instruction::create_reg_reg_reg(Opcode::Sub, 2, 0, 1);

        // Créer un registre Decode → Execute
        let de_reg = DecodeExecuteRegister {
            instruction: sub_instruction,
            pc: 100,
            rs1: Some(0),  // Premier registre source
            rs2: Some(1),  // Deuxième registre source
            rd: Some(2),   // Registre destination
            immediate: None,
            branch_addr: None,
            mem_addr: None,
        };

        // Ajuster les valeurs des registres dans les arguments
        let mut instr = de_reg.instruction.clone();
        instr.args = vec![0, 10, 7]; // R0 = 10, R1 = 7, R2 n'est pas encore défini

        let de_reg_with_values = DecodeExecuteRegister {
            instruction: instr,
            ..de_reg
        };

        // Exécuter l'instruction
        let result = execute.process_direct(&de_reg_with_values, &mut alu);
        assert!(result.is_ok());

        // Vérifier le résultat
        let em_reg = result.unwrap();
        assert_eq!(em_reg.alu_result, 3); // 10 - 7 = 3
        assert_eq!(em_reg.rd, Some(2));   // Le résultat va dans R2
    }

    #[test]
    fn test_execute_arithmetic_operations_three_reg() {
        let mut execute = ExecuteStage::new();
        let mut alu = ALU::new();

        // Tester plusieurs opérations arithmétiques avec format à trois registres
        let operations = [
            (Opcode::Add, 5, 7, 12),     // 5 + 7 = 12
            (Opcode::Sub, 10, 3, 7),     // 10 - 3 = 7
            (Opcode::Mul, 4, 5, 20),     // 4 * 5 = 20
            (Opcode::Div, 20, 4, 5),     // 20 / 4 = 5
            (Opcode::Mod, 10, 3, 1),     // 10 % 3 = 1
        ];

        for (op, val1, val2, expected) in &operations {
            // Créer une instruction à trois registres: OP R2, R0, R1
            let instruction = Instruction::create_reg_reg_reg(*op, 2, 0, 1);

            // Créer un registre Decode → Execute
            let de_reg = DecodeExecuteRegister {
                instruction,
                pc: 100,
                rs1: Some(0),
                rs2: Some(1),
                rd: Some(2),
                immediate: None,
                branch_addr: None,
                mem_addr: None,
            };

            // Ajuster les valeurs des registres
            let mut instr = de_reg.instruction.clone();
            instr.args = vec![0, *val1, *val2]; // R0 = val1, R1 = val2

            let de_reg_with_values = DecodeExecuteRegister {
                instruction: instr,
                ..de_reg
            };

            // Exécuter et vérifier
            let result = execute.process_direct(&de_reg_with_values, &mut alu);
            assert!(result.is_ok());

            let em_reg = result.unwrap();
            assert_eq!(em_reg.alu_result, *expected, "Opération {:?} avec {} et {} devrait donner {}", op, val1, val2, expected);
            assert_eq!(em_reg.rd, Some(2));
        }
    }

    #[test]
    fn test_execute_logical_operations_three_reg() {
        let mut execute = ExecuteStage::new();
        let mut alu = ALU::new();

        // Tester les opérations logiques avec format à trois registres
        let operations = [
            (Opcode::And, 0xF0, 0x0F, 0x00),   // F0 & 0F = 00
            (Opcode::Or, 0xF0, 0x0F, 0xFF),    // F0 | 0F = FF
            (Opcode::Xor, 0xF0, 0x0F, 0xFF),   // F0 ^ 0F = FF
        ];

        for (op, val1, val2, expected) in &operations {
            // Créer une instruction à trois registres: OP R2, R0, R1
            let instruction = Instruction::create_reg_reg_reg(*op, 2, 0, 1);

            // Créer un registre Decode → Execute
            let de_reg = DecodeExecuteRegister {
                instruction,
                pc: 100,
                rs1: Some(0),
                rs2: Some(1),
                rd: Some(2),
                immediate: None,
                branch_addr: None,
                mem_addr: None,
            };

            // Ajuster les valeurs des registres
            let mut instr = de_reg.instruction.clone();
            instr.args = vec![0, *val1, *val2]; // R0 = val1, R1 = val2

            let de_reg_with_values = DecodeExecuteRegister {
                instruction: instr,
                ..de_reg
            };

            // Exécuter et vérifier
            let result = execute.process_direct(&de_reg_with_values, &mut alu);
            assert!(result.is_ok());

            let em_reg = result.unwrap();
            assert_eq!(em_reg.alu_result, *expected, "Opération {:?} avec {:X} et {:X} devrait donner {:X}", op, val1, val2, expected);
            assert_eq!(em_reg.rd, Some(2));
        }
    }

    #[test]
    fn test_execute_complex_instruction_sequence() {
        let mut execute = ExecuteStage::new();
        let mut alu = ALU::new();
        let mut result_registers = vec![0u64; 8]; // R0-R7

        // Simuler une séquence d'instructions qui calcule (A + B) * C
        // R0 = 5 (A)
        // R1 = 7 (B)
        // R2 = 3 (C)
        // On veut calculer (5 + 7) * 3 = 36

        // 1. ADD R3, R0, R1 (R3 = R0 + R1)
        let add_instruction = Instruction::create_reg_reg_reg(Opcode::Add, 3, 0, 1);
        let de_reg_add = DecodeExecuteRegister {
            instruction: add_instruction,
            pc: 100,
            rs1: Some(0),
            rs2: Some(1),
            rd: Some(3),
            immediate: None,
            branch_addr: None,
            mem_addr: None,
        };

        // Initialiser R0=5, R1=7
        let mut instr_add = de_reg_add.instruction.clone();
        instr_add.args = vec![0, 5, 7];

        let de_reg_add_with_values = DecodeExecuteRegister {
            instruction: instr_add,
            ..de_reg_add
        };

        // Exécuter ADD R3, R0, R1
        let result_add = execute.process_direct(&de_reg_add_with_values, &mut alu);
        assert!(result_add.is_ok());
        let em_reg_add = result_add.unwrap();
        assert_eq!(em_reg_add.alu_result, 12); // 5 + 7 = 12

        // Stocker le résultat dans R3
        result_registers[3] = em_reg_add.alu_result;

        // 2. MUL R4, R3, R2 (R4 = R3 * R2)
        let mul_instruction = Instruction::create_reg_reg_reg(Opcode::Mul, 4, 3, 2);
        let de_reg_mul = DecodeExecuteRegister {
            instruction: mul_instruction,
            pc: 104,
            rs1: Some(3),
            rs2: Some(2),
            rd: Some(4),
            immediate: None,
            branch_addr: None,
            mem_addr: None,
        };

        // R3=12 (résultat précédent), R2=3
        let mut instr_mul = de_reg_mul.instruction.clone();
        instr_mul.args = vec![0, result_registers[3] as u8, 3];

        let de_reg_mul_with_values = DecodeExecuteRegister {
            instruction: instr_mul,
            ..de_reg_mul
        };

        // Exécuter MUL R4, R3, R2
        let result_mul = execute.process_direct(&de_reg_mul_with_values, &mut alu);
        assert!(result_mul.is_ok());
        let em_reg_mul = result_mul.unwrap();
        assert_eq!(em_reg_mul.alu_result, 36); // 12 * 3 = 36

        // Le résultat final devrait être 36 dans R4
        result_registers[4] = em_reg_mul.alu_result;
        assert_eq!(result_registers[4], 36);
    }

    #[test]
    fn test_execute_jump_instruction() {
        let mut execute = ExecuteStage::new();
        let mut alu = ALU::new();

        // Créer une instruction JMP à l'adresse absolue 0x1000
        let jmp_instruction = Instruction::new(
            Opcode::Jmp,
            InstructionFormat::new(ArgType::None, ArgType::AbsoluteAddr, ArgType::None),
            vec![0, 16, 0, 0] // Adresse 0x1000 (little-endian)
        );

        // Créer un registre Decode → Execute avec adresse de branchement
        let de_reg = DecodeExecuteRegister {
            instruction: jmp_instruction,
            pc: 100,
            rs1: None,
            rs2: None,
            rd: None,
            immediate: None,
            branch_addr: Some(0x1000),
            mem_addr: None,
        };

        // Exécuter l'instruction
        let result = execute.process_direct(&de_reg, &mut alu);
        assert!(result.is_ok());

        // Vérifier le résultat
        let em_reg = result.unwrap();
        assert_eq!(em_reg.branch_taken, true);
        assert_eq!(em_reg.branch_target, Some(0x1000));
    }

    #[test]
    fn test_execute_conditional_jump() {
        let mut execute = ExecuteStage::new();
        let mut alu = ALU::new();

        // Préparer l'ALU avec des flags
        alu.flags.zero = true; // Condition égalité vraie

        // Créer une instruction JMP_IF (saut si égal)
        let jmp_if_instruction = Instruction::new(
            Opcode::JmpIf,
            InstructionFormat::new(ArgType::None, ArgType::AbsoluteAddr, ArgType::None),
            vec![0, 16, 0, 0] // Adresse 0x1000
        );

        // Créer un registre Decode → Execute
        let de_reg = DecodeExecuteRegister {
            instruction: jmp_if_instruction,
            pc: 100,
            rs1: None,
            rs2: None,
            rd: None,
            immediate: None,
            branch_addr: Some(0x1000),
            mem_addr: None,
        };

        // Exécuter l'instruction
        let result = execute.process_direct(&de_reg, &mut alu);
        assert!(result.is_ok());

        // Vérifier le résultat - devrait prendre le branchement car ZF=1
        let em_reg = result.unwrap();
        assert_eq!(em_reg.branch_taken, true);
        assert_eq!(em_reg.branch_target, Some(0x1000));
    }

    #[test]
    fn test_execute_load_instruction() {
        let mut execute = ExecuteStage::new();
        let mut alu = ALU::new();

        // Créer une instruction LOAD R0, [0x2000]
        let load_instruction = Instruction::new(
            Opcode::Load,
            InstructionFormat::new(ArgType::Register, ArgType::AbsoluteAddr, ArgType::None),
            vec![0, 0, 32, 0, 0] // R0 = Mem[0x2000]
        );

        // Créer un registre Decode → Execute
        let de_reg = DecodeExecuteRegister {
            instruction: load_instruction,
            pc: 100,
            rs1: None,
            rs2: None,
            rd: Some(0),
            immediate: None,
            branch_addr: None,
            mem_addr: Some(0x2000),
        };

        // Exécuter l'instruction
        let result = execute.process_direct(&de_reg, &mut alu);
        assert!(result.is_ok());

        // Vérifier le résultat - l'étage Execute ne charge pas la valeur, il prépare juste l'accès mémoire
        let em_reg = result.unwrap();
        assert_eq!(em_reg.mem_addr, Some(0x2000));
        assert_eq!(em_reg.rd, Some(0));
        assert_eq!(em_reg.alu_result, 0); // Pas de calcul ALU pour LOAD
    }

    #[test]
    fn test_execute_store_instruction() {
        let mut execute = ExecuteStage::new();
        let mut alu = ALU::new();

        // Créer une instruction STORE R0, [0x2000]
        let store_instruction = Instruction::new(
            Opcode::Store,
            InstructionFormat::new(ArgType::Register, ArgType::AbsoluteAddr, ArgType::None),
            vec![0, 0, 32, 0, 0] // Mem[0x2000] = R0
        );

        // Créer un registre Decode → Execute
        let de_reg = DecodeExecuteRegister {
            instruction: store_instruction.clone(),
            pc: 100,
            rs1: Some(0), // Registre source
            rs2: None,
            rd: None, // Pas de registre destination pour STORE
            immediate: None,
            branch_addr: None,
            mem_addr: Some(0x2000),
        };

        // Mettre une valeur dans R0
        let mut instr = de_reg.instruction.clone();
        instr.args = vec![42]; // R0 = 42

        let de_reg_with_values = DecodeExecuteRegister {
            instruction: instr,
            ..de_reg
        };

        // Exécuter l'instruction
        let result = execute.process_direct(&de_reg_with_values, &mut alu);
        assert!(result.is_ok());

        // Vérifier le résultat
        let em_reg = result.unwrap();
        assert_eq!(em_reg.mem_addr, Some(0x2000));
        assert_eq!(em_reg.store_value, Some(42)); // La valeur à stocker
    }

    #[test]
    fn test_execute_mixed_format_program() {
        let mut execute = ExecuteStage::new();
        let mut alu = ALU::new();
        let mut result_registers = vec![0u64; 8]; // R0-R7

        // Simuler une séquence d'instructions qui calcule:
        // 1. Calculer R3 = R0 + R1 (format à trois registres)
        // 2. Incrémenter R3 (format à un registre)
        // 3. Comparer R3 avec R2 (format à deux registres)

        // Valeurs initiales: R0=5, R1=7, R2=13

        // 1. ADD R3, R0, R1 (R3 = R0 + R1)
        let add_instruction = Instruction::create_reg_reg_reg(Opcode::Add, 3, 0, 1);
        let de_reg_add = DecodeExecuteRegister {
            instruction: add_instruction,
            pc: 100,
            rs1: Some(0),
            rs2: Some(1),
            rd: Some(3),
            immediate: None,
            branch_addr: None,
            mem_addr: None,
        };

        // Initialiser R0=5, R1=7
        let mut instr_add = de_reg_add.instruction.clone();
        instr_add.args = vec![0, 5, 7];

        let de_reg_add_with_values = DecodeExecuteRegister {
            instruction: instr_add,
            ..de_reg_add
        };

        // Exécuter ADD R3, R0, R1
        let result_add = execute.process_direct(&de_reg_add_with_values, &mut alu);
        assert!(result_add.is_ok());
        let em_reg_add = result_add.unwrap();
        assert_eq!(em_reg_add.alu_result, 12); // 5 + 7 = 12

        // Stocker le résultat dans R3
        result_registers[3] = em_reg_add.alu_result;

        // 2. INC R3 (R3 = R3 + 1) - format à un registre
        let inc_instruction = Instruction::create_single_reg(Opcode::Inc, 3);
        let de_reg_inc = DecodeExecuteRegister {
            instruction: inc_instruction,
            pc: 104,
            rs1: Some(3),
            rs2: None,
            rd: Some(3),
            immediate: None,
            branch_addr: None,
            mem_addr: None,
        };

        // R3=12 (résultat précédent)
        let mut instr_inc = de_reg_inc.instruction.clone();
        instr_inc.args = vec![result_registers[3] as u8];

        let de_reg_inc_with_values = DecodeExecuteRegister {
            instruction: instr_inc,
            ..de_reg_inc
        };

        // Exécuter INC R3
        let result_inc = execute.process_direct(&de_reg_inc_with_values, &mut alu);
        assert!(result_inc.is_ok());
        let em_reg_inc = result_inc.unwrap();
        assert_eq!(em_reg_inc.alu_result, 13); // 12 + 1 = 13

        // Stocker le résultat dans R3
        result_registers[3] = em_reg_inc.alu_result;

        // 3. CMP R3, R2 (format à deux registres)
        let cmp_instruction = Instruction::create_reg_reg(Opcode::Cmp, 3, 2);
        let de_reg_cmp = DecodeExecuteRegister {
            instruction: cmp_instruction,
            pc: 106,
            rs1: Some(3),
            rs2: Some(2),
            rd: None,
            immediate: None,
            branch_addr: None,
            mem_addr: None,
        };

        // R3=13, R2=13
        let mut instr_cmp = de_reg_cmp.instruction.clone();
        instr_cmp.args = vec![result_registers[3] as u8, 13];

        let de_reg_cmp_with_values = DecodeExecuteRegister {
            instruction: instr_cmp,
            ..de_reg_cmp
        };

        // Exécuter CMP R3, R2
        let result_cmp = execute.process_direct(&de_reg_cmp_with_values, &mut alu);
        assert!(result_cmp.is_ok());

        // Vérifier que les flags sont correctement positionnés (égalité)
        assert!(alu.flags.zero);  // Les valeurs sont égales, donc ZF=1
        assert!(!alu.flags.negative);
        assert!(!alu.flags.carry);
    }
}
//src/pipeline/execute.rs

use crate::alu::alu::{ALU, ALUOperation, BranchCondition};
// use crate::bytecode::Opcode;
// use crate::alu::{ALU, ALUOperation, BranchCondition};
use crate::bytecode::opcodes::Opcode;
use crate::pipeline::{DecodeExecuteRegister, ExecuteMemoryRegister, /* stage::PipelineStage*/};
// use crate::PunkVM::alu::{ALU,ALUOperation, BranchCondition};


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
        let rs1_value = match ex_reg.rs1 {
            Some(reg) => {
                if reg < ex_reg.instruction.args.len() {
                    ex_reg.instruction.args[reg] as u64
                } else {
                    0
                }
            },
            None => 0,
        };

        let rs2_value = match ex_reg.rs2 {
            Some(reg) => {
                if reg < ex_reg.instruction.args.len() {
                    ex_reg.instruction.args[reg] as u64
                } else {
                    0
                }
            },
            None => match ex_reg.immediate {
                Some(imm) => imm,
                None => 0,
            },
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
            },

            Opcode::Mul => {
                alu_result = alu.execute(ALUOperation::Mul, rs1_value, rs2_value)?;
            },

            Opcode::Div => {
                alu_result = alu.execute(ALUOperation::Div, rs1_value, rs2_value)?;
            },

            Opcode::Mod => {
                alu_result = alu.execute(ALUOperation::Mod, rs1_value, rs2_value)?;
            },

            Opcode::Inc => {
                alu_result = alu.execute(ALUOperation::Inc, rs1_value, 0)?;
            },

            Opcode::Dec => {
                alu_result = alu.execute(ALUOperation::Dec, rs1_value, 0)?;
            },

            Opcode::Neg => {
                alu_result = alu.execute(ALUOperation::Neg, rs1_value, 0)?;
            },

            Opcode::And => {
                alu_result = alu.execute(ALUOperation::And, rs1_value, rs2_value)?;
            },

            Opcode::Or => {
                alu_result = alu.execute(ALUOperation::Or, rs1_value, rs2_value)?;
            },

            Opcode::Xor => {
                alu_result = alu.execute(ALUOperation::Xor, rs1_value, rs2_value)?;
            },

            Opcode::Not => {
                alu_result = alu.execute(ALUOperation::Not, rs1_value, 0)?;
            },

            Opcode::Nop => {
                // Pas d'opération
                alu_result = 0; // Pas utilisé
            },


            Opcode::Shl => {
                alu_result = alu.execute(ALUOperation::Shl, rs1_value, rs2_value)?;
            },

            Opcode::Shr => {
                alu_result = alu.execute(ALUOperation::Shr, rs1_value, rs2_value)?;
            },

            Opcode::Sar => {
                alu_result = alu.execute(ALUOperation::Sar, rs1_value, rs2_value)?;
            },

            Opcode::Rol => {
                alu_result = alu.execute(ALUOperation::Rol, rs1_value, rs2_value)?;
            },

            Opcode::Ror => {
                alu_result = alu.execute(ALUOperation::Ror, rs1_value, rs2_value)?;
            },

            // Instructions de comparaison
            Opcode::Cmp => {
                // Compare mais ne stocke pas le résultat
                alu.execute(ALUOperation::Cmp, rs1_value, rs2_value)?;
                alu_result = 0; // Pas utilisé
            },

            Opcode::Test => {
                // Test (AND logique) mais ne stocke pas le résultat
                alu.execute(ALUOperation::Test, rs1_value, rs2_value)?;
                alu_result = 0; // Pas utilisé
            },

            // Instructions de contrôle de flux
            Opcode::Jmp => {
                // Saut inconditionnel
                branch_taken = true;
                branch_target = ex_reg.branch_addr;
            },

            Opcode::JmpIf => {
                // Saut conditionnel si la condition est vraie
                branch_taken = alu.check_condition(BranchCondition::Equal);
                branch_target = ex_reg.branch_addr;
            },

            Opcode::JmpIfNot => {
                // Saut conditionnel si la condition est fausse
                branch_taken = alu.check_condition(BranchCondition::NotEqual);
                branch_target = ex_reg.branch_addr;
            },

            // Instructions d'accès mémoire
            Opcode::Load | Opcode::LoadB | Opcode::LoadW | Opcode::LoadD => {
                // Ces instructions finalisent leur exécution dans l'étage Memory
                alu_result = 0; // Sera remplacé par la valeur chargée
            },

            Opcode::Store | Opcode::StoreB | Opcode::StoreW | Opcode::StoreD => {
                // Préparer la valeur à stocker
                store_value = Some(rs1_value);
            },

            Opcode::Push => {
                // Préparer la valeur à empiler
                store_value = Some(rs1_value);
                // L'adresse est calculée dans l'étage Memory
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
            },

            Opcode::Halt => {
                // Instruction spéciale pour terminer l'exécution
                // Gérée au niveau du pipeline
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
    fn test_execute_add_instruction() {
        let mut execute = ExecuteStage::new();
        let mut alu = ALU::new();

        // Créer une instruction ADD R0, R1
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
    fn test_execute_sub_instruction() {
        let mut execute = ExecuteStage::new();
        let mut alu = ALU::new();

        // Créer une instruction SUB R0, R1
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
    fn test_execute_jump_instruction() {
        let mut execute = ExecuteStage::new();
        let mut alu = ALU::new();

        // Créer une instruction JMP à l'adresse absolue 0x1000
        let jmp_instruction = Instruction::new(
            Opcode::Jmp,
            InstructionFormat::new(ArgType::None, ArgType::AbsoluteAddr),
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
            InstructionFormat::new(ArgType::None, ArgType::AbsoluteAddr),
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
            InstructionFormat::new(ArgType::Register, ArgType::AbsoluteAddr),
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
            InstructionFormat::new(ArgType::Register, ArgType::AbsoluteAddr),
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
}
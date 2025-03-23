//src/pipeline/execute.rs

use crate::alu::alu::{ALU, ALUOperation, BranchCondition};
use crate::bytecode::opcodes::Opcode;
use crate::pipeline::{DecodeExecuteRegister, ExecuteMemoryRegister, PipelineState};



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
        // let mut branch_taken = false;
        // let mut branch_target = None;
        // on récupère les valeur calcule en decode

        let rs1_value = ex_reg.rs1_value;
        let rs2_value = ex_reg.rs2_value;
        let mut alu_result = 0;
        let mem_addr = ex_reg.mem_addr;

        let mut branch_taken = false;
        let mut branch_target = ex_reg.branch_addr;

        let mut store_value = None;


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
            Opcode::Mov => {
                let value = ex_reg.immediate.unwrap_or(ex_reg.rs2_value);
                alu_result = value;
                println!("Execute MOV: rs1_value={}, immediate={:?}, alu_result={}", rs1_value, ex_reg.immediate, alu_result);
            }

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
                println!("DEBUG: Processing branch instruction: {:?}", ex_reg.instruction);
                println!("DEBUG: Branch address: {:?}", ex_reg.branch_addr);
                println!("DEBUG: Format: {:?}", ex_reg.instruction.format);
                println!("DEBUG: Args: {:?}", ex_reg.instruction.args);

                println!("Execute JMP_IF: branch_taken={}, branch_target={:?}", branch_taken, branch_target);
            },

            Opcode::JmpIfNot => {
                // Saut conditionnel si la condition est fausse
                branch_taken = alu.check_condition(BranchCondition::NotEqual);
                branch_target = ex_reg.branch_addr;
                println!("Execute JMP_IF_NOT: branch_taken={}, branch_target={:?}", branch_taken, branch_target);
            },

            Opcode::JmpIfEqual => {
                branch_taken = alu.check_condition(BranchCondition::Equal);
                branch_target = ex_reg.branch_addr;
                println!("DEBUG: Processing branch instruction: {:?}", ex_reg.instruction);
                println!("DEBUG: Branch address: {:?}", ex_reg.branch_addr);
                println!("DEBUG: Format: {:?}", ex_reg.instruction.format);
                println!("DEBUG: Args: {:?}", ex_reg.instruction.args);
                println!("Execute JMP_IF_EQUAL: branch_taken={}, branch_target={:?}", branch_taken, branch_target);
            },

            Opcode::JmpIfNotEqual => {
                branch_taken = alu.check_condition(BranchCondition::NotEqual);
                branch_target = ex_reg.branch_addr;
                println!("DEBUG: Processing branch instruction: {:?}", ex_reg.instruction);
                println!("DEBUG: Branch address: {:?}", ex_reg.branch_addr);
                println!("DEBUG: Format: {:?}", ex_reg.instruction.format);
                println!("DEBUG: Args: {:?}", ex_reg.instruction.args);
                println!("Execute JMP_IF_NOT_EQUAL: branch_taken={}, branch_target={:?}", branch_taken, branch_target);
            },

            Opcode::JmpIfGreater => {
                branch_taken = alu.check_condition(BranchCondition::Greater);
                branch_target = ex_reg.branch_addr;
                println!("DEBUG: Processing branch instruction: {:?}", ex_reg.instruction);
                println!("DEBUG: Branch address: {:?}", ex_reg.branch_addr);
                println!("DEBUG: Format: {:?}", ex_reg.instruction.format);
                println!("DEBUG: Args: {:?}", ex_reg.instruction.args);
                println!("Execute JMP_IF_GREATER: branch_taken={}, branch_target={:?}", branch_taken, branch_target);
            },

            Opcode::JmpIfGreaterEqual => {
                branch_taken = alu.check_condition(BranchCondition::GreaterEqual);
                branch_target = ex_reg.branch_addr;
                println!("DEBUG: Processing branch instruction: {:?}", ex_reg.instruction);
                println!("DEBUG: Branch address: {:?}", ex_reg.branch_addr);
                println!("DEBUG: Format: {:?}", ex_reg.instruction.format);
                println!("DEBUG: Args: {:?}", ex_reg.instruction.args);
                println!("Execute JMP_IF_GREATER_EQUAL: branch_taken={}, branch_target={:?}", branch_taken, branch_target);
            },

            Opcode::JmpIfLess => {
                // Saut conditionnel si pas égal
                branch_taken = alu.check_condition(BranchCondition::Less);
                branch_target = ex_reg.branch_addr;
                println!("DEBUG: Processing branch instruction: {:?}", ex_reg.instruction);
                println!("DEBUG: Branch address: {:?}", ex_reg.branch_addr);
                println!("DEBUG: Format: {:?}", ex_reg.instruction.format);
                println!("DEBUG: Args: {:?}", ex_reg.instruction.args);
                println!("Execute JMP_IF_LESS: branch_taken={}, branch_target={:?}", branch_taken, branch_target);

            },

            Opcode::JmpIfLessEqual => {
                branch_taken = alu.check_condition(BranchCondition::LessEqual);
                branch_target = ex_reg.branch_addr;
                println!("DEBUG: Processing branch instruction: {:?}", ex_reg.instruction);
                println!("DEBUG: Branch address: {:?}", ex_reg.branch_addr);
                println!("DEBUG: Format: {:?}", ex_reg.instruction.format);
                println!("DEBUG: Args: {:?}", ex_reg.instruction.args);
                println!("Execute JMP_IF_LESS_EQUAL: branch_taken={}, branch_target={:?}", branch_taken, branch_target);
            },

            Opcode::JmpIfAbove =>{
                branch_taken = alu.check_condition(BranchCondition::Above);
                branch_target = ex_reg.branch_addr;
                println!("DEBUG: Processing branch instruction: {:?}", ex_reg.instruction);
                println!("DEBUG: Branch address: {:?}", ex_reg.branch_addr);
                println!("DEBUG: Format: {:?}", ex_reg.instruction.format);
                println!("DEBUG: Args: {:?}", ex_reg.instruction.args);
                println!("Execute JMP_IF_ABOVE: branch_taken={}, branch_target={:?}", branch_taken, branch_target);
            },

            Opcode::JmpIfAboveEqual => {
                branch_taken = alu.check_condition(BranchCondition::AboveEqual);
                branch_target = ex_reg.branch_addr;
                println!("DEBUG: Processing branch instruction: {:?}", ex_reg.instruction);
                println!("DEBUG: Branch address: {:?}", ex_reg.branch_addr);
                println!("DEBUG: Format: {:?}", ex_reg.instruction.format);
                println!("DEBUG: Args: {:?}", ex_reg.instruction.args);
                println!("Execute JMP_IF_ABOVE_EQUAL: branch_taken={}, branch_target={:?}", branch_taken, branch_target);
            },

            Opcode::JmpIfBelow => {
                branch_taken = alu.check_condition(BranchCondition::Below);
                branch_target = ex_reg.branch_addr;
                println!("DEBUG: Processing branch instruction: {:?}", ex_reg.instruction);
                println!("DEBUG: Branch address: {:?}", ex_reg.branch_addr);
                println!("DEBUG: Format: {:?}", ex_reg.instruction.format);
                println!("DEBUG: Args: {:?}", ex_reg.instruction.args);
                println!("Execute JMP_IF_BELOW: branch_taken={}, branch_target={:?}", branch_taken, branch_target);
            },

            Opcode::JmpIfBelowEqual => {
                branch_taken = alu.check_condition(BranchCondition::BelowEqual);
                branch_target = ex_reg.branch_addr;
                println!("DEBUG: Processing branch instruction: {:?}", ex_reg.instruction);
                println!("DEBUG: Branch address: {:?}", ex_reg.branch_addr);
                println!("DEBUG: Format: {:?}", ex_reg.instruction.format);
                println!("DEBUG: Args: {:?}", ex_reg.instruction.args);
                println!("Execute JMP_IF_BELOW_EQUAL: branch_taken={}, branch_target={:?}", branch_taken, branch_target);
            },

            Opcode::JmpIfZero => {
                branch_taken = alu.check_condition(BranchCondition::Zero);
                branch_target = ex_reg.branch_addr;
                println!("DEBUG: Processing branch instruction: {:?}", ex_reg.instruction);
                println!("DEBUG: Branch address: {:?}", ex_reg.branch_addr);
                println!("DEBUG: Format: {:?}", ex_reg.instruction.format);
                println!("DEBUG: Args: {:?}", ex_reg.instruction.args);
                println!("Execute JMP_IF_ZERO: branch_taken={}, branch_target={:?}", branch_taken, branch_target);

            },

            Opcode::JmpIfNotZero => {
                branch_taken = alu.check_condition(BranchCondition::NotZero);
                branch_target =ex_reg.branch_addr;
                println!("DEBUG: Processing branch instruction: {:?}", ex_reg.instruction);
                println!("DEBUG: Branch address: {:?}", ex_reg.branch_addr);
                println!("DEBUG: Format: {:?}", ex_reg.instruction.format);
                println!("DEBUG: Args: {:?}", ex_reg.instruction.args);
                println!("Execute JMP_IF_NOT_ZERO: branch_taken={}, branch_target={:?}", branch_taken, branch_target);
            },

            Opcode::JmpIfOverflow => {
                branch_taken = alu.check_condition(BranchCondition::Overflow);
                branch_target = ex_reg.branch_addr;
                println!("DEBUG: Processing branch instruction: {:?}", ex_reg.instruction);
                println!("DEBUG: Branch address: {:?}", ex_reg.branch_addr);
                println!("DEBUG: Format: {:?}", ex_reg.instruction.format);
                println!("DEBUG: Args: {:?}", ex_reg.instruction.args);
                println!("Execute JMP_IF_OVERFLOW: branch_taken={}, branch_target={:?}", branch_taken, branch_target);
            },

            Opcode::JmpIfNotOverflow => {
                branch_taken = alu.check_condition(BranchCondition::NotOverflow);
                branch_target = ex_reg.branch_addr;
                println!("DEBUG: Processing branch instruction: {:?}", ex_reg.instruction);
                println!("DEBUG: Branch address: {:?}", ex_reg.branch_addr);
                println!("DEBUG: Format: {:?}", ex_reg.instruction.format);
                println!("DEBUG: Args: {:?}", ex_reg.instruction.args);
                println!("Execute JMP_IF_NOT_OVERFLOW: branch_taken={}, branch_target={:?}", branch_taken, branch_target);
            },

            Opcode::JmpIfPositive => {
                branch_taken = alu.check_condition(BranchCondition::Positive);
                branch_target = ex_reg.branch_addr;
                println!("DEBUG: Processing branch instruction: {:?}", ex_reg.instruction);
                println!("DEBUG: Branch address: {:?}", ex_reg.branch_addr);
                println!("DEBUG: Format: {:?}", ex_reg.instruction.format);
                println!("DEBUG: Args: {:?}", ex_reg.instruction.args);
                println!("Execute JMP_IF_POSITIVE: branch_taken={}, branch_target={:?}", branch_taken, branch_target);
            },

            Opcode::JmpIfNegative => {
                branch_taken = alu.check_condition(BranchCondition::Negative);
                branch_target = ex_reg.branch_addr;
                println!("DEBUG: Processing branch instruction: {:?}", ex_reg.instruction);
                println!("DEBUG: Branch address: {:?}", ex_reg.branch_addr);
                println!("DEBUG: Format: {:?}", ex_reg.instruction.format);
                println!("DEBUG: Args: {:?}", ex_reg.instruction.args);
                println!("Execute JMP_IF_NEGATIVE: branch_taken={}, branch_target={:?}", branch_taken, branch_target);
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
                println!("Execute HALT");
                return Ok(ExecuteMemoryRegister {
                    instruction: ex_reg.instruction.clone(),
                    alu_result: 0,
                    rd: ex_reg.rd,
                    store_value: None,
                    mem_addr: None,
                    branch_target: None,
                    branch_taken: false,
                    halted: true, // un champ qu’il faut rajouter (voir ci-dessous)
                });
            },

            // Instructions étendues et autres
            _ => {
                return Err(format!("Opcode non supporté: {:?}", ex_reg.instruction.opcode));
            },
        }

        println!("Executed Instruction : {:?}", ex_reg.instruction);

        Ok(ExecuteMemoryRegister {
            instruction: ex_reg.instruction.clone(),
            alu_result,
            rd: ex_reg.rd,
            store_value,   // pour CMP
            mem_addr,
            branch_target,
            branch_taken,
            halted: false, // Pas de halt ici
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

    /// Test d'une instruction ADD (2 registres) où on veut R0 = R0 + R1
    #[test]
    fn test_execute_add_instruction_two_reg() {
        let mut execute = ExecuteStage::new();
        let mut alu = ALU::new();

        // Suppose qu'on a fait "Decode" et trouvé que c'est "Add R0, R1"
        let add_instruction = Instruction::create_reg_reg(Opcode::Add, 0, 1);

        // On crée un decode->execute register
        // On positionne rs1_value=5, rs2_value=7
        let de_reg = DecodeExecuteRegister {
            instruction: add_instruction,
            pc: 100,
            rs1: Some(0),    // index
            rs2: Some(1),
            rd: Some(0),
            rs1_value: 5,    // R0=5
            rs2_value: 7,    // R1=7
            immediate: None,
            branch_addr: None,
            mem_addr: None,
        };

        // Exécuter l'instruction
        let result = execute.process_direct(&de_reg, &mut alu);
        assert!(result.is_ok());

        let em_reg = result.unwrap();
        // 5 + 7 = 12
        assert_eq!(em_reg.alu_result, 12);
        assert_eq!(em_reg.rd, Some(0));
        assert!(!em_reg.branch_taken);
        assert_eq!(em_reg.branch_target, None);
    }

    /// Test d'une instruction ADD (3 registres) style R2 = R0 + R1
    #[test]
    fn test_execute_add_instruction_three_reg() {
        let mut execute = ExecuteStage::new();
        let mut alu = ALU::new();

        // Instruction "Add R2, R0, R1"
        let add_instruction = Instruction::create_reg_reg_reg(Opcode::Add, 2, 0, 1);

        // On simule "Decode" qui a trouvé rs1=0, rs2=1, rd=2,
        // et lit la banque de registres => R0=5, R1=7
        let de_reg = DecodeExecuteRegister {
            instruction: add_instruction,
            pc: 100,
            rs1: Some(0),
            rs2: Some(1),
            rd: Some(2),
            rs1_value: 5,  // R0=5
            rs2_value: 7,  // R1=7
            immediate: None,
            branch_addr: None,
            mem_addr: None,
        };

        let result = execute.process_direct(&de_reg, &mut alu);
        assert!(result.is_ok());

        let em_reg = result.unwrap();
        assert_eq!(em_reg.alu_result, 12);
        assert_eq!(em_reg.rd, Some(2));
        assert!(!em_reg.branch_taken);
        assert_eq!(em_reg.branch_target, None);
    }

    #[test]
    fn test_execute_sub_instruction_two_reg() {
        let mut execute = ExecuteStage::new();
        let mut alu = ALU::new();

        // SUB R0, R1 => R0 = R0 - R1
        let sub_instruction = Instruction::create_reg_reg(Opcode::Sub, 0, 1);

        let de_reg = DecodeExecuteRegister {
            instruction: sub_instruction,
            pc: 100,
            rs1: Some(0),
            rs2: Some(1),
            rd: Some(0),
            rs1_value: 10,  // R0=10
            rs2_value: 7,   // R1=7
            immediate: None,
            branch_addr: None,
            mem_addr: None,
        };

        let result = execute.process_direct(&de_reg, &mut alu);
        assert!(result.is_ok());

        let em_reg = result.unwrap();
        // 10 - 7 = 3
        assert_eq!(em_reg.alu_result, 3);
    }

    #[test]
    fn test_execute_sub_instruction_three_reg() {
        let mut execute = ExecuteStage::new();
        let mut alu = ALU::new();

        // SUB R2, R0, R1 => R2 = R0 - R1
        let sub_instruction = Instruction::create_reg_reg_reg(Opcode::Sub, 2, 0, 1);

        let de_reg = DecodeExecuteRegister {
            instruction: sub_instruction,
            pc: 100,
            rs1: Some(0),
            rs2: Some(1),
            rd: Some(2),
            rs1_value: 10,  // R0=10
            rs2_value: 7,   // R1=7
            immediate: None,
            branch_addr: None,
            mem_addr: None,
        };

        let result = execute.process_direct(&de_reg, &mut alu);
        assert!(result.is_ok());

        let em_reg = result.unwrap();
        // 10 - 7 = 3
        assert_eq!(em_reg.alu_result, 3);
        assert_eq!(em_reg.rd, Some(2));
    }

    #[test]
    fn test_execute_arithmetic_operations_three_reg() {
        let mut execute = ExecuteStage::new();
        let mut alu = ALU::new();

        // Tester plusieurs opérations arithmétiques
        let operations = [
            (Opcode::Add,  5,  7,  12),
            (Opcode::Sub, 10,  3,   7),
            (Opcode::Mul,  4,  5,  20),
            (Opcode::Div, 20,  4,   5),
            (Opcode::Mod, 10,  3,   1),
        ];

        for (op, val1, val2, expected) in operations {
            // ex: OP R2, R0, R1
            let instruction = Instruction::create_reg_reg_reg(op, 2, 0, 1);

            let de_reg = DecodeExecuteRegister {
                instruction,
                pc: 100,
                rs1: Some(0),
                rs2: Some(1),
                rd: Some(2),
                rs1_value: val1,
                rs2_value: val2,
                immediate: None,
                branch_addr: None,
                mem_addr: None,
            };

            let result = execute.process_direct(&de_reg, &mut alu);
            assert!(result.is_ok());

            let em_reg = result.unwrap();
            assert_eq!(
                em_reg.alu_result, expected,
                "Opération {:?} avec {} et {} devrait donner {}",
                op, val1, val2, expected
            );
            assert_eq!(em_reg.rd, Some(2));
        }
    }

    #[test]
    fn test_execute_logical_operations_three_reg() {
        let mut execute = ExecuteStage::new();
        let mut alu = ALU::new();

        // Tester les opérations logiques
        let operations = [
            (Opcode::And, 0xF0, 0x0F, 0x00),
            (Opcode::Or,  0xF0, 0x0F, 0xFF),
            (Opcode::Xor, 0xF0, 0x0F, 0xFF),
        ];

        for (op, val1, val2, expected) in operations {
            let instruction = Instruction::create_reg_reg_reg(op, 2, 0, 1);

            let de_reg = DecodeExecuteRegister {
                instruction,
                pc: 100,
                rs1: Some(0),
                rs2: Some(1),
                rd: Some(2),
                rs1_value: val1,
                rs2_value: val2,
                immediate: None,
                branch_addr: None,
                mem_addr: None,
            };

            let result = execute.process_direct(&de_reg, &mut alu);
            assert!(result.is_ok());

            let em_reg = result.unwrap();
            assert_eq!(
                em_reg.alu_result, expected,
                "Opération {:?} avec {:X} et {:X} devrait donner {:X}",
                op, val1, val2, expected
            );
            assert_eq!(em_reg.rd, Some(2));
        }
    }

    #[test]
    fn test_execute_store_instruction() {
        let mut execute = ExecuteStage::new();
        let mut alu = ALU::new();

        // STORE R0, [0x2000]
        let store_instruction = Instruction::new(
            Opcode::Store,
            InstructionFormat::new(ArgType::Register, ArgType::AbsoluteAddr, ArgType::None),
            vec![]
        );

        let de_reg = DecodeExecuteRegister {
            instruction: store_instruction,
            pc: 100,
            rs1: Some(0), // R0 => source
            rs2: None,
            rd: None,
            rs1_value: 42, // On veut stocker 42
            rs2_value: 0,
            immediate: None,
            branch_addr: None,
            mem_addr: Some(0x2000),
        };

        let result = execute.process_direct(&de_reg, &mut alu);
        assert!(result.is_ok());

        let em_reg = result.unwrap();
        assert_eq!(em_reg.mem_addr, Some(0x2000));
        // store_value = rs1_value => 42
        assert_eq!(em_reg.store_value, Some(42));
    }

    #[test]
    fn test_execute_complex_instruction_sequence() {
        let mut execute = ExecuteStage::new();
        let mut alu = ALU::new();

        // On veut calculer (5 + 7) * 3
        // 1) ADD R3, R0, R1 => R3 = R0 + R1
        let add_instr = Instruction::create_reg_reg_reg(Opcode::Add, 3, 0, 1);
        let de_reg_add = DecodeExecuteRegister {
            instruction: add_instr,
            pc: 100,
            rs1: Some(0),
            rs2: Some(1),
            rd: Some(3),
            rs1_value: 5,   // R0=5
            rs2_value: 7,   // R1=7
            immediate: None,
            branch_addr: None,
            mem_addr: None,
        };
        let res_add = execute.process_direct(&de_reg_add, &mut alu).unwrap();
        assert_eq!(res_add.alu_result, 12);

        // 2) MUL R4, R3, R2 => R4 = R3 * R2
        // Suppose R2=3
        let mul_instr = Instruction::create_reg_reg_reg(Opcode::Mul, 4, 3, 2);
        let de_reg_mul = DecodeExecuteRegister {
            instruction: mul_instr,
            pc: 104,
            rs1: Some(3),
            rs2: Some(2),
            rd: Some(4),
            rs1_value: res_add.alu_result,  // R3=12
            rs2_value: 3,                   // R2=3
            immediate: None,
            branch_addr: None,
            mem_addr: None,
        };
        let res_mul = execute.process_direct(&de_reg_mul, &mut alu).unwrap();
        assert_eq!(res_mul.alu_result, 36);

        // On a 36 dans R4 => c'est le résultat final
    }
    #[test]
    fn test_execute_mixed_format_program() {
        let mut execute = ExecuteStage::new();
        let mut alu = ALU::new();

        // 1) ADD R3, R0, R1 => 3-op => R3=R0+R1
        let add_instr = Instruction::create_reg_reg_reg(Opcode::Add, 3, 0, 1);
        let de_reg_add = DecodeExecuteRegister {
            instruction: add_instr,
            pc: 100,
            rs1: Some(0),
            rs2: Some(1),
            rd: Some(3),
            rs1_value: 5,  // R0=5
            rs2_value: 7,  // R1=7
            immediate: None,
            branch_addr: None,
            mem_addr: None,
        };
        let em_reg_add = execute.process_direct(&de_reg_add, &mut alu).unwrap();
        assert_eq!(em_reg_add.alu_result, 12);

        // 2) INC R3 => format 1 reg => "rd=3, rs1=3"
        // => R3 = R3 + 1 => 12 + 1 => 13
        let inc_instr = Instruction::create_single_reg(Opcode::Inc, 3);
        let de_reg_inc = DecodeExecuteRegister {
            instruction: inc_instr,
            pc: 104,
            rs1: Some(3),
            rs2: None,
            rd: Some(3),
            rs1_value: em_reg_add.alu_result,  // R3=12
            rs2_value: 0,
            immediate: None,
            branch_addr: None,
            mem_addr: None,
        };
        let em_reg_inc = execute.process_direct(&de_reg_inc, &mut alu).unwrap();
        assert_eq!(em_reg_inc.alu_result, 13);

        // 3) CMP R3, R2 => 2-reg => "rs1=3, rs2=2"
        // Suppose R3=13, R2=13
        let cmp_instr = Instruction::create_reg_reg(Opcode::Cmp, 3, 2);
        let de_reg_cmp = DecodeExecuteRegister {
            instruction: cmp_instr,
            pc: 106,
            rs1: Some(3),
            rs2: Some(2),
            rd: None,
            rs1_value: 13,  // R3=13
            rs2_value: 13,  // R2=13
            immediate: None,
            branch_addr: None,
            mem_addr: None,
        };
        let em_reg_cmp = execute.process_direct(&de_reg_cmp, &mut alu).unwrap();
        // On attend ZF=1 => alu.flags.zero = true
        assert!(alu.flags.zero);
        assert!(!alu.flags.negative);
        assert!(!alu.flags.carry);
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
            rs1_value: 0,
            rs2_value: 0,
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
            rs1_value: 0,
            rs2_value: 0,
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
            rs1_value: 0,
            rs2_value: 0,
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


}
//src/pipeline/execute.rs

use crate::alu::alu::{ALUOperation, BranchCondition, ALU};
use crate::bytecode::opcodes::Opcode;
use crate::pipeline::{DecodeExecuteRegister, ExecuteMemoryRegister};
use crate::pvm::branch_predictor::{BranchPrediction, BranchPredictor, PredictorType};
use crate::pipeline::decode::StackOperation;

/// Implementation de l'étage Execute du pipeline
pub struct ExecuteStage {
    // Unité ALU
    branch_predictor: BranchPredictor,
    /// Stats Locales
    branch_predictions:u64,
    branch_hits:u64,
}

impl ExecuteStage {
    /// Crée un nouvel étage Execute
    pub fn new() -> Self {
        Self {
            branch_predictor: BranchPredictor::new(PredictorType::Dynamic),
            branch_predictions: 0,
            branch_hits: 0,
        }
    }




    /// Traite l'étage Execute directement
    pub fn process_direct(
        &mut self,
        ex_reg: &DecodeExecuteRegister,
        alu: &mut ALU,
    ) -> Result<ExecuteMemoryRegister, String> {
        // on récupère les valeur calcule en decode
        let rs1_value = ex_reg.rs1_value;
        let rs2_value = ex_reg.rs2_value;
        let mut alu_result = 0;
        let mut mem_addr = ex_reg.mem_addr;
        let mut branch_taken = false;
        let mut branch_target = ex_reg.branch_addr;
        let mut store_value = None;

        // pour les opérations de pile
        let mut stack_operation = None;
        let mut stack_result = None;



        // Exécuter l'opération en fonction de l'opcode

        match ex_reg.instruction.opcode {
            // Instructions arithmétiques et logiques
            Opcode::Add => {
                alu_result = alu.execute(ALUOperation::Add, rs1_value, rs2_value)?;
                println!(
                    "Execute ADD: rs1_value={}, rs2_value={}, alu_result={}",
                    rs1_value, rs2_value, alu_result
                );
            }

            Opcode::Sub => {
                alu_result = alu.execute(ALUOperation::Sub, rs1_value, rs2_value)?;
                println!(
                    "Execute SUB: rs1_value={}, rs2_value={}, alu_result={}",
                    rs1_value, rs2_value, alu_result
                );
            }

            Opcode::Mul => {
                alu_result = alu.execute(ALUOperation::Mul, rs1_value, rs2_value)?;
                println!(
                    "Execute MUL: rs1_value={}, rs2_value={}, alu_result={}",
                    rs1_value, rs2_value, alu_result
                );
            }

            Opcode::Div => {
                alu_result = alu.execute(ALUOperation::Div, rs1_value, rs2_value)?;
                println!(
                    "Execute DIV: rs1_value={}, rs2_value={}, alu_result={}",
                    rs1_value, rs2_value, alu_result
                );
            }

            Opcode::Mod => {
                alu_result = alu.execute(ALUOperation::Mod, rs1_value, rs2_value)?;
                println!(
                    "Execute MOD: rs1_value={}, rs2_value={}, alu_result={}",
                    rs1_value, rs2_value, alu_result
                );
            }
            Opcode::Mov => {
                let value = ex_reg.immediate.unwrap_or(ex_reg.rs2_value);
                alu_result = value;
                println!(
                    "Execute MOV: rs1_value={}, immediate={:?}, alu_result={}",
                    rs1_value, ex_reg.immediate, alu_result
                );
            }

            Opcode::Inc => {
                alu_result = alu.execute(ALUOperation::Inc, rs1_value, 0)?;
                println!(
                    "Execute INC: rs1_value={}, alu_result={}",
                    rs1_value, alu_result
                );
            }

            Opcode::Dec => {
                alu_result = alu.execute(ALUOperation::Dec, rs1_value, 0)?;
                println!(
                    "Execute DEC: rs1_value={}, alu_result={}",
                    rs1_value, alu_result
                );
            }

            Opcode::Neg => {
                alu_result = alu.execute(ALUOperation::Neg, rs1_value, 0)?;
                println!(
                    "Execute NEG: rs1_value={}, alu_result={}",
                    rs1_value, alu_result
                );
            }

            Opcode::And => {
                alu_result = alu.execute(ALUOperation::And, rs1_value, rs2_value)?;
                println!(
                    "Execute AND: rs1_value={}, rs2_value={}, alu_result={}",
                    rs1_value, rs2_value, alu_result
                );
            }

            Opcode::Or => {
                alu_result = alu.execute(ALUOperation::Or, rs1_value, rs2_value)?;
                println!(
                    "Execute OR: rs1_value={}, rs2_value={}, alu_result={}",
                    rs1_value, rs2_value, alu_result
                );
            }

            Opcode::Xor => {
                alu_result = alu.execute(ALUOperation::Xor, rs1_value, rs2_value)?;
                println!(
                    "Execute XOR: rs1_value={}, rs2_value={}, alu_result={}",
                    rs1_value, rs2_value, alu_result
                );
            }

            Opcode::Not => {
                alu_result = alu.execute(ALUOperation::Not, rs1_value, 0)?;
                println!(
                    "Execute NOT: rs1_value={}, alu_result={}",
                    rs1_value, alu_result
                );
            }

            Opcode::Nop => {
                // Pas d'opération
                alu_result = 0; // Pas utilisé
                println!("Execute NOP");
            }

            Opcode::Shl => {
                alu_result = alu.execute(ALUOperation::Shl, rs1_value, rs2_value)?;
                println!(
                    "Execute SHL: rs1_value={}, rs2_value={}, alu_result={}",
                    rs1_value, rs2_value, alu_result
                );
            }

            Opcode::Shr => {
                alu_result = alu.execute(ALUOperation::Shr, rs1_value, rs2_value)?;
                println!(
                    "Execute SHR: rs1_value={}, rs2_value={}, alu_result={}",
                    rs1_value, rs2_value, alu_result
                );
            }

            Opcode::Sar => {
                alu_result = alu.execute(ALUOperation::Sar, rs1_value, rs2_value)?;
                println!(
                    "Execute SAR: rs1_value={}, rs2_value={}, alu_result={}",
                    rs1_value, rs2_value, alu_result
                );
            }

            Opcode::Rol => {
                alu_result = alu.execute(ALUOperation::Rol, rs1_value, rs2_value)?;
                println!(
                    "Execute ROL: rs1_value={}, rs2_value={}, alu_result={}",
                    rs1_value, rs2_value, alu_result
                );
            }

            Opcode::Ror => {
                alu_result = alu.execute(ALUOperation::Ror, rs1_value, rs2_value)?;
                println!(
                    "Execute ROR: rs1_value={}, rs2_value={}, alu_result={}",
                    rs1_value, rs2_value, alu_result
                );
            }

            // Instructions de comparaison
            Opcode::Cmp => {
                // Compare mais ne stocke pas le résultat
                alu.execute(ALUOperation::Cmp, rs1_value, rs2_value)?;
                alu_result = 0; // Pas utilisé
                println!(
                    "Execute CMP: rs1_value={} vs rs2_value={}",
                    rs1_value, rs2_value
                );
            }

            Opcode::Test => {
                // Test (AND logique) mais ne stocke pas le résultat
                alu.execute(ALUOperation::Test, rs1_value, rs2_value)?;
                alu_result = 0; // Pas utilisé
                println!(
                    "Execute TEST: rs1_value={}, rs2_value={}",
                    rs1_value, rs2_value
                );
            }
////////////////////////////////////////////CONTROLE FLOW////////////////////////////////////////////////////////
            Opcode::Jmp|
            Opcode::JmpIf
            |Opcode::JmpIfNot
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
            | Opcode::JmpIfZero
            | Opcode::JmpIfNotZero
            | Opcode::JmpIfOverflow
            | Opcode::JmpIfNotOverflow
            | Opcode::JmpIfPositive
            | Opcode::JmpIfNegative => {
                branch_taken = alu.check_condition(match ex_reg.instruction.opcode {
                    Opcode::JmpIf => BranchCondition::Equal,
                    Opcode::JmpIfEqual => BranchCondition::Equal,
                    Opcode::JmpIfNotEqual => BranchCondition::NotEqual,
                    Opcode::JmpIfGreater => BranchCondition::Greater,
                    Opcode::JmpIfGreaterEqual => BranchCondition::GreaterEqual,
                    Opcode::JmpIfLess => BranchCondition::Less,
                    Opcode::JmpIfLessEqual => BranchCondition::LessEqual,
                    Opcode::JmpIfAbove => BranchCondition::Above,
                    Opcode::JmpIfAboveEqual => BranchCondition::AboveEqual,
                    Opcode::JmpIfBelow => BranchCondition::Below,
                    Opcode::JmpIfBelowEqual => BranchCondition::BelowEqual,
                    Opcode::JmpIfZero => BranchCondition::Zero,
                    Opcode::JmpIfNotZero => BranchCondition::NotZero,
                    Opcode::JmpIfOverflow => BranchCondition::Overflow,
                    Opcode::JmpIfNotOverflow => BranchCondition::NotOverflow,
                    Opcode::JmpIfPositive => BranchCondition::Positive,
                    Opcode::JmpIfNegative => BranchCondition::Negative,
                    //pour tous les autres opcodes
                    _ => BranchCondition::Always, // Ne devrait pas arriver
                });

                branch_target = ex_reg.branch_addr;

                if let Some(prediction) = ex_reg.branch_prediction {
                    // le PC devrai etre passé au predicteur ou stocké dans ex_reg
                    // self.update_predictor(ex_reg.pc, prediction, branch_taken);
                    self.update_branch_predictor(ex_reg.pc as u64, branch_taken, prediction);
                }

                println!(
                    "DEBUG: Processing branch instruction: {:?}",
                    ex_reg.instruction
                );

                println!("DEBUG: Branch address: {:?}", ex_reg.branch_addr);
                println!("DEBUG: Format: {:?}", ex_reg.instruction.format);
                println!("DEBUG: Args: {:?}", ex_reg.instruction.args);
                println!(
                    "Execute branch instruction: {:?}, branch_taken={}, branch_target={:?}",
                    ex_reg.instruction.opcode, branch_taken, branch_target
                );
                println!("[[[DEBUG: Traitement d'un Jmp -]]] PC = 0x{:08X}, Target = {:?}", ex_reg.pc, branch_target);

            }
////////////////////////////////////Control des FLOW////////////////////////////////////////////////////////
            // Instructions d'accès mémoire
            Opcode::Load | Opcode::LoadB | Opcode::LoadW | Opcode::LoadD => {
                // Ces instructions finalisent leur exécution dans l'étage Memory
                alu_result = 0; // Sera remplacé par la valeur chargée
                println!(
                    "Execute LOAD: rs1_value={}, mem_addr={:?}",
                    rs1_value, mem_addr
                );
            }

            Opcode::Store | Opcode::StoreB | Opcode::StoreW | Opcode::StoreD => {
                // Préparer la valeur à stocker
                store_value = Some(rs1_value);
                println!(
                    "Execute STORE: rs1_value={}, mem_addr={:?}",
                    rs1_value, mem_addr
                );
            }

            Opcode::Call => {
                println!("Execute CALL: PC=0x{:X}, target={:?}", ex_reg.pc, branch_target);

                // 1. Calculer l'adresse de retour
                let return_address = ex_reg.pc + ex_reg.instruction.total_size() as u32;

                // // 2. Empiler l'adresse de retour
                // vm.stack_push(return_address as u64)
                //     .map_err(|e| format!("CALL stack push failed: {}", e))?;

                // 3. Préparer le saut
                branch_taken = true;

                println!("CALL executed: return_addr=0x{:X}, target={:?}",
                         return_address, branch_target);

            },
            Opcode::Ret => {
                // println!("Execute RET: PC=0x{:X}", ex_reg.pc);
                //
                // // // 1. Dépiler l'adresse de retour
                // // let return_address = vm.stack_pop()
                // //     .map_err(|e| format!("RET stack pop failed: {}", e))?;
                //
                // // 2. Préparer le saut vers l'adresse de retour
                // branch_taken = true;
                // branch_target = Some(return_address as u32);
                //
                // println!("RET executed: return_addr=0x{:X}", return_address);
            },


            Opcode::Push => {
                // Préparer la valeur à empiler
                store_value = Some(rs1_value);
                // L'adresse est calculée dans l'étage Memory
                println!(
                    "Execute PUSH: rs1_value={}, mem_addr={:?}",
                    rs1_value, mem_addr
                );
            },

            Opcode::Pop => {
                // println!("Execute POP");
                //
                // // Dépiler la valeur
                // let value = vm.stack_pop()
                //     .map_err(|e| format!("POP failed: {}", e))?;
                //
                // // Le résultat sera écrit dans le registre de destination
                // alu_result = value;
                // stack_operation = Some(StackOperation::Pop);
                // stack_result = Some(value);
                //
                // println!("POP executed: value=0x{:X}", value);

            },

            // Instructions spéciales
            Opcode::Syscall => {
                // Traitées séparément (pas implémenté pour l'instant)
                return Err("Syscall non implémenté".to_string());
            },

            Opcode::Break => {
                // Instruction de débogage, ne fait rien dans la PunkVM
                println!("Execute BREAK");
            }

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
                    branch_prediction_correct: None,
                    stack_operation: None,
                    stack_result: None,
                    ras_prediction_correct: None,
                    halted: true, // un champ qu’il faut rajouter (voir ci-dessous)
                });
            }

            // Instructions étendues et autres
            _ => {
                return Err(format!(
                    "Opcode non supporté: {:?}",
                    ex_reg.instruction.opcode
                ));
            }

        }


        // Debug pour les instructions de branchement
        if ex_reg.instruction.opcode.is_branch() {
            if let Some(prediction) = ex_reg.branch_prediction {
                // Cette ligne était commentée - c'est un problème majeur !
                self.update_branch_predictor(ex_reg.pc as u64, branch_taken, prediction);
            }

            println!("DEBUG: Processing branch instruction: {:?}", ex_reg.instruction);
            println!("DEBUG: Branch address: {:?}", branch_target);
            println!("DEBUG: Format: {:?}", ex_reg.instruction.format);
            println!("DEBUG: Args: {:?}", ex_reg.instruction.args);
            println!("Execute branch instruction: {:?}, branch_taken={}, branch_target={:?}",
                     ex_reg.instruction.opcode, branch_taken, branch_target);
        }

        println!("Executed Instruction : {:?}", ex_reg.instruction);



        // Calculer si la prédiction était correcte (pour les branches)
        let branch_prediction_correct = if ex_reg.instruction.opcode.is_branch() {
            ex_reg.branch_prediction.map(|pred| {
                let predicted_taken = pred == BranchPrediction::Taken;
                predicted_taken == branch_taken
            })
        } else {
            None
        };

        // Gestion spéciale pour RET avec validation RAS
        let ras_prediction_correct = if ex_reg.instruction.opcode == Opcode::Ret {
            if let Some(predicted_target) = ex_reg.branch_addr {
                if let Some(actual_target) = branch_target {
                    Some(predicted_target == actual_target)
                } else {
                    Some(false)
                }
            } else {
                None
            }
        } else {
            None
        };


        Ok(ExecuteMemoryRegister {
            instruction: ex_reg.instruction.clone(),
            alu_result,
            rd: ex_reg.rd,
            store_value, // pour CMP
            mem_addr,
            branch_target,
            branch_taken,
            // branch_prediction_correct: ex_reg
            //     .branch_prediction
            //     .map(|pred| (pred == BranchPrediction::Taken) == branch_taken), // Pas encore implémenté
            branch_prediction_correct,
            stack_operation,
            stack_result,
            ras_prediction_correct,
            halted: false, // Pas de halt ici
        })
    }

    pub fn update_branch_predictor(&mut self, pc: u64, taken: bool, prediction: BranchPrediction) {
        println!("Updating branch predictor: PC=0x{:X}, taken={}, prediction={:?}",
                 pc, taken, prediction);

        // Utiliser le prédicteur persistant
        self.branch_predictor.update(pc, taken, prediction);

        // Mise à jour des statistiques locales
        self.branch_predictions += 1;
        if (prediction == BranchPrediction::Taken) == taken {
            self.branch_hits += 1;
        }

        let accuracy = self.get_prediction_accuracy();
        println!("Branch predictor accuracy: {:.2}%", accuracy);
    }

    /// Retourne le taux de réussite du prédicteur
    pub fn get_prediction_accuracy(&self) -> f64 {
        if self.branch_predictions > 0 {
            (self.branch_hits as f64 / self.branch_predictions as f64) * 100.0
        } else {
            0.0
        }

    }

    //methode pour

    /// Réinitialise l'étage Execute
    pub fn reset(&mut self) {
        self.branch_predictor = BranchPredictor::new(PredictorType::Dynamic);
        self.branch_predictions = 0;
        self.branch_hits = 0;

        // Pas d'état interne à réinitialiser
    }
}

//
//
// // Test unitaire pour l'étage Execute
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::bytecode::format::ArgType;
//     use crate::bytecode::format::InstructionFormat;
//     use crate::bytecode::instructions::Instruction;
//     use crate::bytecode::opcodes::Opcode;
//     use crate::pipeline::DecodeExecuteRegister;
//
//     #[test]
//     fn test_execute_stage_creation() {
//         let execute = ExecuteStage::new();
//         // Vérifier que la création réussit
//         assert!(true);
//     }
//
//     #[test]
//     fn test_execute_stage_reset() {
//         let mut execute = ExecuteStage::new();
//         execute.reset();
//         // L'étage Execute n'a pas d'état interne, donc reset() ne fait rien
//         // On s'assure juste que la méthode peut être appelée sans erreur
//         assert!(true);
//     }
//
//     /// Test d'une instruction ADD (2 registres) où on veut R0 = R0 + R1
//     #[test]
//     fn test_execute_add_instruction_two_reg() {
//         let mut execute = ExecuteStage::new();
//         let mut alu = ALU::new();
//
//         // Suppose qu'on a fait "Decode" et trouvé que c'est "Add R0, R1"
//         let add_instruction = Instruction::create_reg_reg(Opcode::Add, 0, 1);
//
//         // On crée un decode->execute register
//         // On positionne rs1_value=5, rs2_value=7
//         let de_reg = DecodeExecuteRegister {
//             instruction: add_instruction,
//             pc: 100,
//             rs1: Some(0), // index
//             rs2: Some(1),
//             rd: Some(0),
//             rs1_value: 5, // R0=5
//             rs2_value: 7, // R1=7
//             immediate: None,
//             branch_addr: None,
//             branch_prediction: None,
//             mem_addr: None,
//         };
//
//         // Exécuter l'instruction
//         let result = execute.process_direct(&de_reg, &mut alu);
//         assert!(result.is_ok());
//
//         let em_reg = result.unwrap();
//         // 5 + 7 = 12
//         assert_eq!(em_reg.alu_result, 12);
//         assert_eq!(em_reg.rd, Some(0));
//         assert!(!em_reg.branch_taken);
//         assert_eq!(em_reg.branch_target, None);
//     }
//
//     /// Test d'une instruction ADD (3 registres) style R2 = R0 + R1
//     #[test]
//     fn test_execute_add_instruction_three_reg() {
//         let mut execute = ExecuteStage::new();
//         let mut alu = ALU::new();
//
//         // Instruction "Add R2, R0, R1"
//         let add_instruction = Instruction::create_reg_reg_reg(Opcode::Add, 2, 0, 1);
//
//         // On simule "Decode" qui a trouvé rs1=0, rs2=1, rd=2,
//         // et lit la banque de registres => R0=5, R1=7
//         let de_reg = DecodeExecuteRegister {
//             instruction: add_instruction,
//             pc: 100,
//             rs1: Some(0),
//             rs2: Some(1),
//             rd: Some(2),
//             rs1_value: 5, // R0=5
//             rs2_value: 7, // R1=7
//             immediate: None,
//             branch_addr: None,
//             branch_prediction: None,
//             mem_addr: None,
//         };
//
//         let result = execute.process_direct(&de_reg, &mut alu);
//         assert!(result.is_ok());
//
//         let em_reg = result.unwrap();
//         assert_eq!(em_reg.alu_result, 12);
//         assert_eq!(em_reg.rd, Some(2));
//         assert!(!em_reg.branch_taken);
//         assert_eq!(em_reg.branch_target, None);
//     }
//
//     #[test]
//     fn test_execute_sub_instruction_two_reg() {
//         let mut execute = ExecuteStage::new();
//         let mut alu = ALU::new();
//
//         // SUB R0, R1 => R0 = R0 - R1
//         let sub_instruction = Instruction::create_reg_reg(Opcode::Sub, 0, 1);
//
//         let de_reg = DecodeExecuteRegister {
//             instruction: sub_instruction,
//             pc: 100,
//             rs1: Some(0),
//             rs2: Some(1),
//             rd: Some(0),
//             rs1_value: 10, // R0=10
//             rs2_value: 7,  // R1=7
//             immediate: None,
//             branch_addr: None,
//             branch_prediction: None,
//             mem_addr: None,
//         };
//
//         let result = execute.process_direct(&de_reg, &mut alu);
//         assert!(result.is_ok());
//
//         let em_reg = result.unwrap();
//         // 10 - 7 = 3
//         assert_eq!(em_reg.alu_result, 3);
//     }
//
//     #[test]
//     fn test_execute_sub_instruction_three_reg() {
//         let mut execute = ExecuteStage::new();
//         let mut alu = ALU::new();
//
//         // SUB R2, R0, R1 => R2 = R0 - R1
//         let sub_instruction = Instruction::create_reg_reg_reg(Opcode::Sub, 2, 0, 1);
//
//         let de_reg = DecodeExecuteRegister {
//             instruction: sub_instruction,
//             pc: 100,
//             rs1: Some(0),
//             rs2: Some(1),
//             rd: Some(2),
//             rs1_value: 10, // R0=10
//             rs2_value: 7,  // R1=7
//             immediate: None,
//             branch_addr: None,
//             branch_prediction: None,
//             mem_addr: None,
//         };
//
//         let result = execute.process_direct(&de_reg, &mut alu);
//         assert!(result.is_ok());
//
//         let em_reg = result.unwrap();
//         // 10 - 7 = 3
//         assert_eq!(em_reg.alu_result, 3);
//         assert_eq!(em_reg.rd, Some(2));
//     }
//
//     #[test]
//     fn test_execute_arithmetic_operations_three_reg() {
//         let mut execute = ExecuteStage::new();
//         let mut alu = ALU::new();
//
//         // Tester plusieurs opérations arithmétiques
//         let operations = [
//             (Opcode::Add, 5, 7, 12),
//             (Opcode::Sub, 10, 3, 7),
//             (Opcode::Mul, 4, 5, 20),
//             (Opcode::Div, 20, 4, 5),
//             (Opcode::Mod, 10, 3, 1),
//         ];
//
//         for (op, val1, val2, expected) in operations {
//             // ex: OP R2, R0, R1
//             let instruction = Instruction::create_reg_reg_reg(op, 2, 0, 1);
//
//             let de_reg = DecodeExecuteRegister {
//                 instruction,
//                 pc: 100,
//                 rs1: Some(0),
//                 rs2: Some(1),
//                 rd: Some(2),
//                 rs1_value: val1,
//                 rs2_value: val2,
//                 immediate: None,
//                 branch_addr: None,
//                 branch_prediction: None,
//                 mem_addr: None,
//             };
//
//             let result = execute.process_direct(&de_reg, &mut alu);
//             assert!(result.is_ok());
//
//             let em_reg = result.unwrap();
//             assert_eq!(
//                 em_reg.alu_result, expected,
//                 "Opération {:?} avec {} et {} devrait donner {}",
//                 op, val1, val2, expected
//             );
//             assert_eq!(em_reg.rd, Some(2));
//         }
//     }
//
//     #[test]
//     fn test_execute_logical_operations_three_reg() {
//         let mut execute = ExecuteStage::new();
//         let mut alu = ALU::new();
//
//         // Tester les opérations logiques
//         let operations = [
//             (Opcode::And, 0xF0, 0x0F, 0x00),
//             (Opcode::Or, 0xF0, 0x0F, 0xFF),
//             (Opcode::Xor, 0xF0, 0x0F, 0xFF),
//         ];
//
//         for (op, val1, val2, expected) in operations {
//             let instruction = Instruction::create_reg_reg_reg(op, 2, 0, 1);
//
//             let de_reg = DecodeExecuteRegister {
//                 instruction,
//                 pc: 100,
//                 rs1: Some(0),
//                 rs2: Some(1),
//                 rd: Some(2),
//                 rs1_value: val1,
//                 rs2_value: val2,
//                 immediate: None,
//                 branch_addr: None,
//                 branch_prediction: None,
//                 mem_addr: None,
//             };
//
//             let result = execute.process_direct(&de_reg, &mut alu);
//             assert!(result.is_ok());
//
//             let em_reg = result.unwrap();
//             assert_eq!(
//                 em_reg.alu_result, expected,
//                 "Opération {:?} avec {:X} et {:X} devrait donner {:X}",
//                 op, val1, val2, expected
//             );
//             assert_eq!(em_reg.rd, Some(2));
//         }
//     }
//
//     #[test]
//     fn test_execute_store_instruction() {
//         let mut execute = ExecuteStage::new();
//         let mut alu = ALU::new();
//
//         // STORE R0, [0x2000]
//         let store_instruction = Instruction::new(
//             Opcode::Store,
//             InstructionFormat::new(ArgType::Register, ArgType::AbsoluteAddr, ArgType::None),
//             vec![],
//         );
//
//         let de_reg = DecodeExecuteRegister {
//             instruction: store_instruction,
//             pc: 100,
//             rs1: Some(0), // R0 => source
//             rs2: None,
//             rd: None,
//             rs1_value: 42, // On veut stocker 42
//             rs2_value: 0,
//             immediate: None,
//             branch_addr: None,
//             branch_prediction: None,
//             mem_addr: Some(0x2000),
//         };
//
//         let result = execute.process_direct(&de_reg, &mut alu);
//         assert!(result.is_ok());
//
//         let em_reg = result.unwrap();
//         assert_eq!(em_reg.mem_addr, Some(0x2000));
//         // store_value = rs1_value => 42
//         assert_eq!(em_reg.store_value, Some(42));
//     }
//
//     #[test]
//     fn test_execute_complex_instruction_sequence() {
//         let mut execute = ExecuteStage::new();
//         let mut alu = ALU::new();
//
//         // On veut calculer (5 + 7) * 3
//         // 1) ADD R3, R0, R1 => R3 = R0 + R1
//         let add_instr = Instruction::create_reg_reg_reg(Opcode::Add, 3, 0, 1);
//         let de_reg_add = DecodeExecuteRegister {
//             instruction: add_instr,
//             pc: 100,
//             rs1: Some(0),
//             rs2: Some(1),
//             rd: Some(3),
//             rs1_value: 5, // R0=5
//             rs2_value: 7, // R1=7
//             immediate: None,
//             branch_addr: None,
//             branch_prediction: None,
//             mem_addr: None,
//         };
//         let res_add = execute.process_direct(&de_reg_add, &mut alu).unwrap();
//         assert_eq!(res_add.alu_result, 12);
//
//         // 2) MUL R4, R3, R2 => R4 = R3 * R2
//         // Suppose R2=3
//         let mul_instr = Instruction::create_reg_reg_reg(Opcode::Mul, 4, 3, 2);
//         let de_reg_mul = DecodeExecuteRegister {
//             instruction: mul_instr,
//             pc: 104,
//             rs1: Some(3),
//             rs2: Some(2),
//             rd: Some(4),
//             rs1_value: res_add.alu_result, // R3=12
//             rs2_value: 3,                  // R2=3
//             immediate: None,
//             branch_addr: None,
//             branch_prediction: None,
//             mem_addr: None,
//         };
//         let res_mul = execute.process_direct(&de_reg_mul, &mut alu).unwrap();
//         assert_eq!(res_mul.alu_result, 36);
//
//         // On a 36 dans R4 => c'est le résultat final
//     }
//     #[test]
//     fn test_execute_mixed_format_program() {
//         let mut execute = ExecuteStage::new();
//         let mut alu = ALU::new();
//
//         // 1) ADD R3, R0, R1 => 3-op => R3=R0+R1
//         let add_instr = Instruction::create_reg_reg_reg(Opcode::Add, 3, 0, 1);
//         let de_reg_add = DecodeExecuteRegister {
//             instruction: add_instr,
//             pc: 100,
//             rs1: Some(0),
//             rs2: Some(1),
//             rd: Some(3),
//             rs1_value: 5, // R0=5
//             rs2_value: 7, // R1=7
//             immediate: None,
//             branch_addr: None,
//             branch_prediction: None,
//             mem_addr: None,
//         };
//         let em_reg_add = execute.process_direct(&de_reg_add, &mut alu).unwrap();
//         assert_eq!(em_reg_add.alu_result, 12);
//
//         // 2) INC R3 => format 1 reg => "rd=3, rs1=3"
//         // => R3 = R3 + 1 => 12 + 1 => 13
//         let inc_instr = Instruction::create_single_reg(Opcode::Inc, 3);
//         let de_reg_inc = DecodeExecuteRegister {
//             instruction: inc_instr,
//             pc: 104,
//             rs1: Some(3),
//             rs2: None,
//             rd: Some(3),
//             rs1_value: em_reg_add.alu_result, // R3=12
//             rs2_value: 0,
//             immediate: None,
//             branch_addr: None,
//             branch_prediction: None,
//             mem_addr: None,
//         };
//         let em_reg_inc = execute.process_direct(&de_reg_inc, &mut alu).unwrap();
//         assert_eq!(em_reg_inc.alu_result, 13);
//
//         // 3) CMP R3, R2 => 2-reg => "rs1=3, rs2=2"
//         // Suppose R3=13, R2=13
//         let cmp_instr = Instruction::create_reg_reg(Opcode::Cmp, 3, 2);
//         let de_reg_cmp = DecodeExecuteRegister {
//             instruction: cmp_instr,
//             pc: 106,
//             rs1: Some(3),
//             rs2: Some(2),
//             rd: None,
//             rs1_value: 13, // R3=13
//             rs2_value: 13, // R2=13
//             immediate: None,
//             branch_addr: None,
//             branch_prediction: None,
//             mem_addr: None,
//         };
//         let em_reg_cmp = execute.process_direct(&de_reg_cmp, &mut alu).unwrap();
//         // On attend ZF=1 => alu.flags.zero = true
//         assert!(alu.flags.zero);
//         assert!(!alu.flags.negative);
//         assert!(!alu.flags.carry);
//     }
//
//     #[test]
//     fn test_execute_jump_instruction() {
//         let mut execute = ExecuteStage::new();
//         let mut alu = ALU::new();
//
//         // Créer une instruction JMP à l'adresse absolue 0x1000
//         let jmp_instruction = Instruction::new(
//             Opcode::Jmp,
//             InstructionFormat::new(ArgType::None, ArgType::AbsoluteAddr, ArgType::None),
//             vec![0, 16, 0, 0], // Adresse 0x1000 (little-endian)
//         );
//
//         // Créer un registre Decode → Execute avec adresse de branchement
//         let de_reg = DecodeExecuteRegister {
//             instruction: jmp_instruction,
//             pc: 100,
//             rs1: None,
//             rs2: None,
//             rd: None,
//             rs1_value: 0,
//             rs2_value: 0,
//             immediate: None,
//             branch_addr: Some(0x1000),
//             branch_prediction: None,
//             mem_addr: None,
//         };
//
//         // Exécuter l'instruction
//         let result = execute.process_direct(&de_reg, &mut alu);
//         assert!(result.is_ok());
//
//         // Vérifier le résultat
//         let em_reg = result.unwrap();
//         assert_eq!(em_reg.branch_taken, true);
//         assert_eq!(em_reg.branch_target, Some(0x1000));
//     }
//
//     #[test]
//     fn test_execute_conditional_jump() {
//         let mut execute = ExecuteStage::new();
//         let mut alu = ALU::new();
//
//         // Préparer l'ALU avec des flags
//         alu.flags.zero = true; // Condition égalité vraie
//
//         // Créer une instruction JMP_IF (saut si égal)
//         let jmp_if_instruction = Instruction::new(
//             Opcode::JmpIf,
//             InstructionFormat::new(ArgType::None, ArgType::AbsoluteAddr, ArgType::None),
//             vec![0, 16, 0, 0], // Adresse 0x1000
//         );
//
//         // Créer un registre Decode → Execute
//         let de_reg = DecodeExecuteRegister {
//             instruction: jmp_if_instruction,
//             pc: 100,
//             rs1: None,
//             rs2: None,
//             rd: None,
//             rs1_value: 0,
//             rs2_value: 0,
//             immediate: None,
//             branch_addr: Some(0x1000),
//             branch_prediction: None,
//             mem_addr: None,
//         };
//
//         // Exécuter l'instruction
//         let result = execute.process_direct(&de_reg, &mut alu);
//         assert!(result.is_ok());
//
//         // Vérifier le résultat - devrait prendre le branchement car ZF=1
//         let em_reg = result.unwrap();
//         assert_eq!(em_reg.branch_taken, true);
//         assert_eq!(em_reg.branch_target, Some(0x1000));
//     }
//
//     #[test]
//     fn test_execute_load_instruction() {
//         let mut execute = ExecuteStage::new();
//         let mut alu = ALU::new();
//
//         // Créer une instruction LOAD R0, [0x2000]
//         let load_instruction = Instruction::new(
//             Opcode::Load,
//             InstructionFormat::new(ArgType::Register, ArgType::AbsoluteAddr, ArgType::None),
//             vec![0, 0, 32, 0, 0], // R0 = Mem[0x2000]
//         );
//
//         // Créer un registre Decode → Execute
//         let de_reg = DecodeExecuteRegister {
//             instruction: load_instruction,
//             pc: 100,
//             rs1: None,
//             rs2: None,
//             rd: Some(0),
//             rs1_value: 0,
//             rs2_value: 0,
//             immediate: None,
//             branch_addr: None,
//             branch_prediction: None,
//             mem_addr: Some(0x2000),
//         };
//
//         // Exécuter l'instruction
//         let result = execute.process_direct(&de_reg, &mut alu);
//         assert!(result.is_ok());
//
//         // Vérifier le résultat - l'étage Execute ne charge pas la valeur, il prépare juste l'accès mémoire
//         let em_reg = result.unwrap();
//         assert_eq!(em_reg.mem_addr, Some(0x2000));
//         assert_eq!(em_reg.rd, Some(0));
//         assert_eq!(em_reg.alu_result, 0); // Pas de calcul ALU pour LOAD
//     }
// }
//
//
// /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// //
// //
// //             // Instructions de contrôle de flux
// //             Opcode::Jmp => {
// //                 // Saut inconditionnel
// //                 branch_taken = true;
// //                 branch_target = ex_reg.branch_addr;
// //                 println!("Execute JMP: branch_target={:?}", branch_target);
// //                 println!(
// //                     "DEBUG: Traitement d'un Jmp - PC = {}, Target = {:?}",
// //                     ex_reg.pc, branch_target
// //                 );
// //             }
// //
// //             Opcode::JmpIf => {
// //                 // Saut conditionnel si la condition est vraie
// //                 branch_taken = alu.check_condition(BranchCondition::Equal);
// //                 branch_target = ex_reg.branch_addr;
// //                 println!("DEBUG: Processing branch instruction: {:?}", ex_reg.instruction);
// //                 println!("DEBUG: Branch address: {:?}", ex_reg.branch_addr);
// //                 println!("DEBUG: Format: {:?}", ex_reg.instruction.format);
// //                 println!("DEBUG: Args: {:?}", ex_reg.instruction.args);
// //
// //                 println!("Execute JMP_IF: branch_taken={}, branch_target={:?}", branch_taken, branch_target);
// //             },
// //
// //             Opcode::JmpIfNot => {
// //                 // Saut conditionnel si la condition est fausse
// //                 branch_taken = alu.check_condition(BranchCondition::NotEqual);
// //                 branch_target = ex_reg.branch_addr;
// //                 println!("Execute JMP_IF_NOT: branch_taken={}, branch_target={:?}", branch_taken, branch_target);
// //             },
// //
// //             Opcode::JmpIfEqual => {
// //                 branch_taken = alu.check_condition(BranchCondition::Equal);
// //                 branch_target = ex_reg.branch_addr;
// //                 println!("DEBUG: Processing branch instruction: {:?}", ex_reg.instruction);
// //                 println!("DEBUG: Branch address: {:?}", ex_reg.branch_addr);
// //                 println!("DEBUG: Format: {:?}", ex_reg.instruction.format);
// //                 println!("DEBUG: Args: {:?}", ex_reg.instruction.args);
// //                 println!("Execute JMP_IF_EQUAL: branch_taken={}, branch_target={:?}", branch_taken, branch_target);
// //             },
// //
// //             Opcode::JmpIfNotEqual => {
// //                 branch_taken = alu.check_condition(BranchCondition::NotEqual);
// //                 branch_target = ex_reg.branch_addr;
// //                 println!("DEBUG: Processing branch instruction: {:?}", ex_reg.instruction);
// //                 println!("DEBUG: Branch address: {:?}", ex_reg.branch_addr);
// //                 println!("DEBUG: Format: {:?}", ex_reg.instruction.format);
// //                 println!("DEBUG: Args: {:?}", ex_reg.instruction.args);
// //                 println!("Execute JMP_IF_NOT_EQUAL: branch_taken={}, branch_target={:?}", branch_taken, branch_target);
// //             },
// //
// //             Opcode::JmpIfGreater => {
// //                 branch_taken = alu.check_condition(BranchCondition::Greater);
// //                 branch_target = ex_reg.branch_addr;
// //                 println!("DEBUG: Processing branch instruction: {:?}", ex_reg.instruction);
// //                 println!("DEBUG: Branch address: {:?}", ex_reg.branch_addr);
// //                 println!("DEBUG: Format: {:?}", ex_reg.instruction.format);
// //                 println!("DEBUG: Args: {:?}", ex_reg.instruction.args);
// //                 println!("Execute JMP_IF_GREATER: branch_taken={}, branch_target={:?}", branch_taken, branch_target);
// //             },
// //
// //             Opcode::JmpIfGreaterEqual => {
// //                 branch_taken = alu.check_condition(BranchCondition::GreaterEqual);
// //                 branch_target = ex_reg.branch_addr;
// //                 println!("DEBUG: Processing branch instruction: {:?}", ex_reg.instruction);
// //                 println!("DEBUG: Branch address: {:?}", ex_reg.branch_addr);
// //                 println!("DEBUG: Format: {:?}", ex_reg.instruction.format);
// //                 println!("DEBUG: Args: {:?}", ex_reg.instruction.args);
// //                 println!("Execute JMP_IF_GREATER_EQUAL: branch_taken={}, branch_target={:?}", branch_taken, branch_target);
// //             },
// //
// //             Opcode::JmpIfLess => {
// //                 // Saut conditionnel si pas égal
// //                 branch_taken = alu.check_condition(BranchCondition::Less);
// //                 branch_target = ex_reg.branch_addr;
// //                 println!("DEBUG: Processing branch instruction: {:?}", ex_reg.instruction);
// //                 println!("DEBUG: Branch address: {:?}", ex_reg.branch_addr);
// //                 println!("DEBUG: Format: {:?}", ex_reg.instruction.format);
// //                 println!("DEBUG: Args: {:?}", ex_reg.instruction.args);
// //                 println!("Execute JMP_IF_LESS: branch_taken={}, branch_target={:?}", branch_taken, branch_target);
// //
// //             },
// //
// //             Opcode::JmpIfLessEqual => {
// //                 branch_taken = alu.check_condition(BranchCondition::LessEqual);
// //                 branch_target = ex_reg.branch_addr;
// //                 println!("DEBUG: Processing branch instruction: {:?}", ex_reg.instruction);
// //                 println!("DEBUG: Branch address: {:?}", ex_reg.branch_addr);
// //                 println!("DEBUG: Format: {:?}", ex_reg.instruction.format);
// //                 println!("DEBUG: Args: {:?}", ex_reg.instruction.args);
// //                 println!("Execute JMP_IF_LESS_EQUAL: branch_taken={}, branch_target={:?}", branch_taken, branch_target);
// //             },
// //
// //             Opcode::JmpIfAbove =>{
// //                 branch_taken = alu.check_condition(BranchCondition::Above);
// //                 branch_target = ex_reg.branch_addr;
// //                 println!("DEBUG: Processing branch instruction: {:?}", ex_reg.instruction);
// //                 println!("DEBUG: Branch address: {:?}", ex_reg.branch_addr);
// //                 println!("DEBUG: Format: {:?}", ex_reg.instruction.format);
// //                 println!("DEBUG: Args: {:?}", ex_reg.instruction.args);
// //                 println!("Execute JMP_IF_ABOVE: branch_taken={}, branch_target={:?}", branch_taken, branch_target);
// //             },
// //
// //             Opcode::JmpIfAboveEqual => {
// //                 branch_taken = alu.check_condition(BranchCondition::AboveEqual);
// //                 branch_target = ex_reg.branch_addr;
// //                 println!("DEBUG: Processing branch instruction: {:?}", ex_reg.instruction);
// //                 println!("DEBUG: Branch address: {:?}", ex_reg.branch_addr);
// //                 println!("DEBUG: Format: {:?}", ex_reg.instruction.format);
// //                 println!("DEBUG: Args: {:?}", ex_reg.instruction.args);
// //                 println!("Execute JMP_IF_ABOVE_EQUAL: branch_taken={}, branch_target={:?}", branch_taken, branch_target);
// //             },
// //
// //             Opcode::JmpIfBelow => {
// //                 branch_taken = alu.check_condition(BranchCondition::Below);
// //                 branch_target = ex_reg.branch_addr;
// //                 println!("DEBUG: Processing branch instruction: {:?}", ex_reg.instruction);
// //                 println!("DEBUG: Branch address: {:?}", ex_reg.branch_addr);
// //                 println!("DEBUG: Format: {:?}", ex_reg.instruction.format);
// //                 println!("DEBUG: Args: {:?}", ex_reg.instruction.args);
// //                 println!("Execute JMP_IF_BELOW: branch_taken={}, branch_target={:?}", branch_taken, branch_target);
// //             },
// //
// //             Opcode::JmpIfBelowEqual => {
// //                 branch_taken = alu.check_condition(BranchCondition::BelowEqual);
// //                 branch_target = ex_reg.branch_addr;
// //                 println!("DEBUG: Processing branch instruction: {:?}", ex_reg.instruction);
// //                 println!("DEBUG: Branch address: {:?}", ex_reg.branch_addr);
// //                 println!("DEBUG: Format: {:?}", ex_reg.instruction.format);
// //                 println!("DEBUG: Args: {:?}", ex_reg.instruction.args);
// //                 println!("Execute JMP_IF_BELOW_EQUAL: branch_taken={}, branch_target={:?}", branch_taken, branch_target);
// //             },
// //
// //             Opcode::JmpIfZero => {
// //                 branch_taken = alu.check_condition(BranchCondition::Zero);
// //                 branch_target = ex_reg.branch_addr;
// //                 println!("DEBUG: Processing branch instruction: {:?}", ex_reg.instruction);
// //                 println!("DEBUG: Branch address: {:?}", ex_reg.branch_addr);
// //                 println!("DEBUG: Format: {:?}", ex_reg.instruction.format);
// //                 println!("DEBUG: Args: {:?}", ex_reg.instruction.args);
// //                 println!("Execute JMP_IF_ZERO: branch_taken={}, branch_target={:?}", branch_taken, branch_target);
// //
// //             },
// //
// //             Opcode::JmpIfNotZero => {
// //                 branch_taken = alu.check_condition(BranchCondition::NotZero);
// //                 branch_target =ex_reg.branch_addr;
// //                 println!("DEBUG: Processing branch instruction: {:?}", ex_reg.instruction);
// //                 println!("DEBUG: Branch address: {:?}", ex_reg.branch_addr);
// //                 println!("DEBUG: Format: {:?}", ex_reg.instruction.format);
// //                 println!("DEBUG: Args: {:?}", ex_reg.instruction.args);
// //                 println!("Execute JMP_IF_NOT_ZERO: branch_taken={}, branch_target={:?}", branch_taken, branch_target);
// //             },
// //
// //             Opcode::JmpIfOverflow => {
// //                 branch_taken = alu.check_condition(BranchCondition::Overflow);
// //                 branch_target = ex_reg.branch_addr;
// //                 println!("DEBUG: Processing branch instruction: {:?}", ex_reg.instruction);
// //                 println!("DEBUG: Branch address: {:?}", ex_reg.branch_addr);
// //                 println!("DEBUG: Format: {:?}", ex_reg.instruction.format);
// //                 println!("DEBUG: Args: {:?}", ex_reg.instruction.args);
// //                 println!("Execute JMP_IF_OVERFLOW: branch_taken={}, branch_target={:?}", branch_taken, branch_target);
// //             },
// //
// //             Opcode::JmpIfNotOverflow => {
// //                 branch_taken = alu.check_condition(BranchCondition::NotOverflow);
// //                 branch_target = ex_reg.branch_addr;
// //                 println!("DEBUG: Processing branch instruction: {:?}", ex_reg.instruction);
// //                 println!("DEBUG: Branch address: {:?}", ex_reg.branch_addr);
// //                 println!("DEBUG: Format: {:?}", ex_reg.instruction.format);
// //                 println!("DEBUG: Args: {:?}", ex_reg.instruction.args);
// //                 println!("Execute JMP_IF_NOT_OVERFLOW: branch_taken={}, branch_target={:?}", branch_taken, branch_target);
// //             },
// //
// //             Opcode::JmpIfPositive => {
// //                 branch_taken = alu.check_condition(BranchCondition::Positive);
// //                 branch_target = ex_reg.branch_addr;
// //                 println!("DEBUG: Processing branch instruction: {:?}", ex_reg.instruction);
// //                 println!("DEBUG: Branch address: {:?}", ex_reg.branch_addr);
// //                 println!("DEBUG: Format: {:?}", ex_reg.instruction.format);
// //                 println!("DEBUG: Args: {:?}", ex_reg.instruction.args);
// //                 println!("Execute JMP_IF_POSITIVE: branch_taken={}, branch_target={:?}", branch_taken, branch_target);
// //             },
// //
// //             Opcode::JmpIfNegative => {
// //                 branch_taken = alu.check_condition(BranchCondition::Negative);
// //                 branch_target = ex_reg.branch_addr;
// //                 println!("DEBUG: Processing branch instruction: {:?}", ex_reg.instruction);
// //                 println!("DEBUG: Branch address: {:?}", ex_reg.branch_addr);
// //                 println!("DEBUG: Format: {:?}", ex_reg.instruction.format);
// //                 println!("DEBUG: Args: {:?}", ex_reg.instruction.args);
// //                 println!("Execute JMP_IF_NEGATIVE: branch_taken={}, branch_target={:?}", branch_taken, branch_target);
// //             },
// //
//
// //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

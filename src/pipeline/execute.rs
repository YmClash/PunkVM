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



    // /// Traite l'étage Execute
    // pub fn process(&mut self, ex_reg: &DecodeExecuteRegister, alu: &mut ALU) -> Result<ExecuteMemoryRegister, String> {
    //     // Valeurs par défaut
    //     let mut alu_result = 0;
    //     let mut branch_taken = false;
    //     let mut branch_target = None;
    //     let mut store_value = None;
    //     let mem_addr = ex_reg.mem_addr;
    //
    //     // Récupérer les valeurs des registres sources (si présents)
    //     let rs1_value = match ex_reg.rs1 {
    //         Some(reg) => {
    //             if reg < ex_reg.instruction.args.len() {
    //                 ex_reg.instruction.args[reg] as u64
    //             } else {
    //                 0
    //             }
    //         },
    //         None => 0,
    //     };
    //
    //     // let rs2_value = match ex_reg.rs2 {
    //     //     Some(reg) => {
    //     //         if reg < ex_reg.instruction.args.len() {
    //     //             ex_reg.instruction.args[reg] as u64
    //     //         } else {
    //     //             0
    //     //         }
    //     //     },
    //     //     None => match ex_reg.immediate {
    //     //         Some(imm) => imm,
    //     //         None => 0,
    //     //     },
    //     // };
    //     let rs2_value = match ex_reg.rs2 {
    //         Some(reg) => {
    //             if reg < ex_reg.instruction.args.len() {
    //                 ex_reg.instruction.args[reg] as u64
    //             } else {
    //                 0
    //             }
    //         },
    //         None => ex_reg.immediate.unwrap_or_else(|| 0),
    //     };
    //
    //     // Exécuter l'opération en fonction de l'opcode
    //     match ex_reg.instruction.opcode {
    //         // Instructions arithmétiques et logiques
    //         Opcode::Add => {
    //             alu_result = alu.execute(ALUOperation::Add, rs1_value, rs2_value)?;
    //         },
    //
    //         Opcode::Sub => {
    //             alu_result = alu.execute(ALUOperation::Sub, rs1_value, rs2_value)?;
    //         },
    //
    //         Opcode::Mul => {
    //             alu_result = alu.execute(ALUOperation::Mul, rs1_value, rs2_value)?;
    //         },
    //
    //         Opcode::Div => {
    //             alu_result = alu.execute(ALUOperation::Div, rs1_value, rs2_value)?;
    //         },
    //
    //         Opcode::Mod => {
    //             alu_result = alu.execute(ALUOperation::Mod, rs1_value, rs2_value)?;
    //         },
    //
    //         Opcode::Inc => {
    //             alu_result = alu.execute(ALUOperation::Inc, rs1_value, 0)?;
    //         },
    //
    //         Opcode::Dec => {
    //             alu_result = alu.execute(ALUOperation::Dec, rs1_value, 0)?;
    //         },
    //
    //         Opcode::Neg => {
    //             alu_result = alu.execute(ALUOperation::Neg, rs1_value, 0)?;
    //         },
    //
    //         Opcode::And => {
    //             alu_result = alu.execute(ALUOperation::And, rs1_value, rs2_value)?;
    //         },
    //
    //         Opcode::Or => {
    //             alu_result = alu.execute(ALUOperation::Or, rs1_value, rs2_value)?;
    //         },
    //
    //         Opcode::Xor => {
    //             alu_result = alu.execute(ALUOperation::Xor, rs1_value, rs2_value)?;
    //         },
    //
    //         Opcode::Not => {
    //             alu_result = alu.execute(ALUOperation::Not, rs1_value, 0)?;
    //         },
    //
    //         Opcode::Shl => {
    //             alu_result = alu.execute(ALUOperation::Shl, rs1_value, rs2_value)?;
    //         },
    //
    //         Opcode::Shr => {
    //             alu_result = alu.execute(ALUOperation::Shr, rs1_value, rs2_value)?;
    //         },
    //
    //         Opcode::Sar => {
    //             alu_result = alu.execute(ALUOperation::Sar, rs1_value, rs2_value)?;
    //         },
    //
    //         Opcode::Rol => {
    //             alu_result = alu.execute(ALUOperation::Rol, rs1_value, rs2_value)?;
    //         },
    //
    //         Opcode::Ror => {
    //             alu_result = alu.execute(ALUOperation::Ror, rs1_value, rs2_value)?;
    //         },
    //         Opcode::Cmp => {
    //             // Compare mais ne stocke pas le résultat
    //             alu.execute(ALUOperation::Cmp, rs1_value, rs2_value)?;
    //             alu_result = 0; // Pas utilisé
    //         }
    //
    //         // Instructions de comparaison
    //         // Opcode::Cmp => {
    //         //     // Compare mais ne stocke pas le résultat
    //         //     alu.execute(ALUOperation::Cmp, rs1_value, rs2_value)?;
    //         //     alu_result = 0; // Pas utilisé
    //         // },
    //         Opcode::Test => {
    //             // Test (AND logique) mais ne stocke pas le résultat
    //             alu.execute(ALUOperation::Test, rs1_value, rs2_value)?;
    //             alu_result = 0; // Pas utilisé
    //         },
    //
    //         // Instructions de contrôle de flux
    //         Opcode::Jmp => {
    //             // Saut inconditionnel
    //             branch_taken = true;
    //             branch_target = ex_reg.branch_addr;
    //         },
    //
    //         Opcode::JmpIf => {
    //             // Saut conditionnel si la condition est vraie
    //             branch_taken = alu.check_condition(BranchCondition::Equal);
    //             branch_target = ex_reg.branch_addr;
    //         },
    //
    //         Opcode::JmpIfNot => {
    //             // Saut conditionnel si la condition est fausse
    //             branch_taken = alu.check_condition(BranchCondition::NotEqual);
    //             branch_target = ex_reg.branch_addr;
    //         },
    //
    //         // Instructions d'accès mémoire
    //         Opcode::Load | Opcode::LoadB | Opcode::LoadW | Opcode::LoadD => {
    //             // Ces instructions finalisent leur exécution dans l'étage Memory
    //             alu_result = 0; // Sera remplacé par la valeur chargée
    //         },
    //
    //         Opcode::Store | Opcode::StoreB | Opcode::StoreW | Opcode::StoreD => {
    //             // Préparer la valeur à stocker
    //             store_value = Some(rs1_value);
    //         },
    //
    //         Opcode::Push => {
    //             // Préparer la valeur à empiler
    //             store_value = Some(rs1_value);
    //             // L'adresse est calculée dans l'étage Memory
    //         },
    //
    //         Opcode::Pop => {
    //             // L'adresse est calculée dans l'étage Memory
    //             // La valeur sera chargée dans l'étage Memory
    //         },
    //
    //         // Instructions spéciales
    //         Opcode::Syscall => {
    //             // Traitées séparément (pas implémenté pour l'instant)
    //             return Err("Syscall non implémenté".to_string());
    //         },
    //
    //         Opcode::Break => {
    //             // Instruction de débogage, ne fait rien dans le simulateur
    //         },
    //
    //         Opcode::Halt => {
    //             // Instruction spéciale pour terminer l'exécution
    //             // Gérée au niveau du pipeline
    //         },
    //
    //         // Instructions étendues et autres
    //         _ => {
    //             return Err(format!("Opcode non supporté: {:?}", ex_reg.instruction.opcode));
    //         },
    //     }
    //
    //     Ok(ExecuteMemoryRegister {
    //         instruction: ex_reg.instruction.clone(),
    //         alu_result,
    //         rd: ex_reg.rd,
    //         store_value,
    //         mem_addr,
    //         branch_target,
    //         branch_taken,
    //     })
    // }

    /// Réinitialise l'étage Execute
    pub fn reset(&mut self) {
        // Pas d'état interne à réinitialiser
    }
}





//
// impl<'a> PipelineStage<'a> for ExecuteStage {
//     type Input = (DecodeExecuteRegister, &'a mut ALU);
//     type Output = ExecuteMemoryRegister;
//
//     fn process(&mut self, input: &Self::Input) -> Result<Self::Output, String> {
//         let (ex_reg, alu) = input;
//         self.process(ex_reg, alu)
//     }
//
//     fn reset(&mut self) {
//         // Reset direct sans appel récursif
//     }
// }
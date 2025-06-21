//src/pipeline/execute.rs

use crate::alu::alu::{ALUOperation, BranchCondition, ALU};
use crate::alu::v_alu::{VectorALU, VectorOperation, VectorResult};
use crate::alu::fpu::{FPU, FPUOperation, FloatPrecision};
use crate::bytecode::opcodes::{Opcode, OpcodeCategory};
use crate::bytecode::simds::{Vector128, Vector256, VectorDataType, Vector256DataType};
use crate::pipeline::{DecodeExecuteRegister, ExecuteMemoryRegister};
use crate::pvm::branch_predictor::{BranchPrediction, BranchPredictor, PredictorType};
use crate::pipeline::decode::StackOperation;
use crate::pvm::vm_errors::VMResult;

/// Implementation de l'étage Execute du pipeline
pub struct ExecuteStage {
    // Unité ALU
    branch_predictor: BranchPredictor,
    /// Unité ALU vectorielle SIMD
    vector_alu: VectorALU,
    /// Unité de calcul flottant FPU
    fpu: FPU,
    /// Stats Locales
    branch_predictions:u64,
    branch_hits:u64,
}

impl ExecuteStage {
    /// Crée un nouvel étage Execute
    pub fn new() -> Self {
        Self {
            branch_predictor: BranchPredictor::new(PredictorType::Dynamic),
            vector_alu: VectorALU::new(),
            fpu: FPU::new(),
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

                // 2. Préparer l'adresse de retour pour être empilée sur la pile
                store_value = Some(return_address as u64);
                stack_operation = Some(StackOperation::Push);
                stack_result = Some(return_address as u64);

                // 3. Préparer le saut vers la fonction appelée
                branch_taken = true;

                println!("CALL executed: return_addr=0x{:X}, target={:?}",
                         return_address, branch_target);

            },
            Opcode::Ret => {
                println!("Execute RET: PC=0x{:X}", ex_reg.pc);

                // 1. Indiquer qu'on veut dépiler une valeur de la pile
                stack_operation = Some(StackOperation::Pop);
                
                // 2. Préparer le saut vers l'adresse de retour
                // L'adresse de retour sera fournie par le RAS prediction ou par la pile
                branch_taken = true;
                // branch_target sera défini par le RAS dans decode ou par la pile dans memory

                println!("RET executed: branch_target={:?}", branch_target);
            },


            Opcode::Push => {
                // Préparer la valeur à empiler
                // Prioriser la valeur immédiate si présente (PUSH immédiat)
                let value_to_push = if let Some(imm) = ex_reg.immediate {
                    imm
                } else {
                    rs1_value
                };
                store_value = Some(value_to_push);
                // L'adresse est calculée dans l'étage Memory
                println!(
                    "Execute PUSH: rs1_value={}, immediate={:?}, value_to_push={}, mem_addr={:?}",
                    rs1_value, ex_reg.immediate, value_to_push, mem_addr
                );
            },

            Opcode::Pop => {
                println!("Execute POP");

                // Indiquer qu'on veut dépiler une valeur de la pile
                stack_operation = Some(StackOperation::Pop);
                
                // Le résultat sera fourni par l'étage Memory après dépilage
                // et sera écrit dans le registre de destination via alu_result

                println!("POP executed: will pop value into register");
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

            // Instructions SIMD 128-bit
            Opcode::Simd128Add | Opcode::Simd128Sub | Opcode::Simd128Mul | Opcode::Simd128Div |
            Opcode::Simd128And | Opcode::Simd128Or | Opcode::Simd128Xor | Opcode::Simd128Not |
            Opcode::Simd128Load | Opcode::Simd128Store | Opcode::Simd128Mov |
            Opcode::Simd128Cmp | Opcode::Simd128Min | Opcode::Simd128Max |
            Opcode::Simd128Sqrt | Opcode::Simd128Shuffle => {
                self.execute_simd_128(&ex_reg.instruction.opcode, ex_reg)?;
                // Pour les instructions SIMD, on retourne 0 car le résultat est dans les registres vectoriels
                alu_result = 0;
                println!("Execute SIMD128 {:?}: completed", ex_reg.instruction.opcode);
            }

            // Instructions SIMD 256-bit
            Opcode::Simd256Add | Opcode::Simd256Sub | Opcode::Simd256Mul | Opcode::Simd256Div |
            Opcode::Simd256And | Opcode::Simd256Or | Opcode::Simd256Xor | Opcode::Simd256Not |
            Opcode::Simd256Load | Opcode::Simd256Store | Opcode::Simd256Mov |
            Opcode::Simd256Cmp | Opcode::Simd256Min | Opcode::Simd256Max |
            Opcode::Simd256Sqrt | Opcode::Simd256Shuffle => {
                self.execute_simd_256(&ex_reg.instruction.opcode, ex_reg)?;
                // Pour les instructions SIMD, on retourne 0 car le résultat est dans les registres vectoriels
                alu_result = 0;
                println!("Execute SIMD256 {:?}: completed", ex_reg.instruction.opcode);
            }

            // Instructions FPU
            Opcode::FpuAdd | Opcode::FpuSub | Opcode::FpuMul | Opcode::FpuDiv |
            Opcode::FpuSqrt | Opcode::FpuCmp | Opcode::FpuLoad | Opcode::FpuStore |
            Opcode::FpuMov | Opcode::FpuConvert | Opcode::FpuRound |
            Opcode::FpuMin | Opcode::FpuMax => {
                let fpu_result = self.execute_fpu(&ex_reg.instruction.opcode, ex_reg)?;
                alu_result = fpu_result.to_bits();
                println!("Execute FPU {:?}: result={}", ex_reg.instruction.opcode, fpu_result);
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
                self.update_branch_predictor(ex_reg.pc as u64, branch_taken, prediction);
            }
            
            // Mise à jour du BTB avec la cible réelle du branchement
            if branch_taken {
                if let Some(target) = branch_target {
                    // Récupérer la cible prédite par le BTB depuis le decode stage
                    let predicted_target = ex_reg.branch_addr;
                    self.branch_predictor.update_btb(ex_reg.pc as u64, target, predicted_target);
                    println!("Updated BTB: PC=0x{:X}, actual_target=0x{:X}, predicted_target={:?}",
                             ex_reg.pc, target, predicted_target);


                }
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

    /// Exécute une instruction SIMD 128-bit
    fn execute_simd_128(&mut self, opcode: &Opcode, ex_reg: &DecodeExecuteRegister) -> Result<(), String> {
        let src1_reg = ex_reg.rs1.unwrap_or(0) as u8;
        let src2_reg = ex_reg.rs2.unwrap_or(0) as u8;
        let dst_reg = ex_reg.rd.unwrap_or(0) as u8;

        let operation = match opcode {
            Opcode::Simd128Add => VectorOperation::Add,
            Opcode::Simd128Sub => VectorOperation::Sub,
            Opcode::Simd128Mul => VectorOperation::Mul,
            Opcode::Simd128Div => VectorOperation::Div,
            Opcode::Simd128And => VectorOperation::And,
            Opcode::Simd128Or => VectorOperation::Or,
            Opcode::Simd128Xor => VectorOperation::Xor,
            Opcode::Simd128Not => VectorOperation::Not,
            Opcode::Simd128Min => VectorOperation::Min,
            Opcode::Simd128Max => VectorOperation::Max,
            Opcode::Simd128Sqrt => VectorOperation::Sqrt,
            Opcode::Simd128Cmp => VectorOperation::Cmp,
            Opcode::Simd128Shuffle => VectorOperation::Shuffle,
            Opcode::Simd128Mov => {
                // Mov vectoriel simple
                let src_vector = self.vector_alu.read_v128(src1_reg)
                    .map_err(|e| format!("Erreur lecture registre V128: {}", e))?;
                self.vector_alu.write_v128(dst_reg, src_vector)
                    .map_err(|e| format!("Erreur écriture registre V128: {}", e))?;
                return Ok(());
            }
            _ => return Err(format!("Opération SIMD 128-bit non supportée: {:?}", opcode)),
        };

        self.vector_alu.execute_v128(
            operation,
            dst_reg,
            src1_reg,
            Some(src2_reg),
            VectorDataType::I32x4, // Type par défaut
        ).map_err(|e| format!("Erreur exécution SIMD 128-bit: {}", e))?;

        Ok(())
    }

    /// Exécute une instruction SIMD 256-bit
    fn execute_simd_256(&mut self, opcode: &Opcode, ex_reg: &DecodeExecuteRegister) -> Result<(), String> {
        let src1_reg = ex_reg.rs1.unwrap_or(0) as u8;
        let src2_reg = ex_reg.rs2.unwrap_or(0) as u8;
        let dst_reg = ex_reg.rd.unwrap_or(0) as u8;

        let operation = match opcode {
            Opcode::Simd256Add => VectorOperation::Add,
            Opcode::Simd256Sub => VectorOperation::Sub,
            Opcode::Simd256Mul => VectorOperation::Mul,
            Opcode::Simd256Div => VectorOperation::Div,
            Opcode::Simd256And => VectorOperation::And,
            Opcode::Simd256Or => VectorOperation::Or,
            Opcode::Simd256Xor => VectorOperation::Xor,
            Opcode::Simd256Not => VectorOperation::Not,
            Opcode::Simd256Min => VectorOperation::Min,
            Opcode::Simd256Max => VectorOperation::Max,
            Opcode::Simd256Sqrt => VectorOperation::Sqrt,
            Opcode::Simd256Cmp => VectorOperation::Cmp,
            Opcode::Simd256Shuffle => VectorOperation::Shuffle,
            Opcode::Simd256Mov => {
                // Mov vectoriel simple
                let src_vector = self.vector_alu.read_v256(src1_reg)
                    .map_err(|e| format!("Erreur lecture registre V256: {}", e))?;
                self.vector_alu.write_v256(dst_reg, src_vector)
                    .map_err(|e| format!("Erreur écriture registre V256: {}", e))?;
                return Ok(());
            }
            _ => return Err(format!("Opération SIMD 256-bit non supportée: {:?}", opcode)),
        };

        self.vector_alu.execute_v256(
            operation,
            dst_reg,
            src1_reg,
            Some(src2_reg),
            Vector256DataType::I32x8, // Type par défaut
        ).map_err(|e| format!("Erreur exécution SIMD 256-bit: {}", e))?;

        Ok(())
    }

    /// Exécute une instruction FPU
    fn execute_fpu(&mut self, opcode: &Opcode, ex_reg: &DecodeExecuteRegister) -> Result<f64, String> {
        let src1_reg = ex_reg.rs1.unwrap_or(0) as u8;
        let src2_reg = ex_reg.rs2.map(|r| r as u8);
        let dst_reg = ex_reg.rd.unwrap_or(0) as u8;
        let precision = FloatPrecision::Double; // Précision par défaut

        let operation = match opcode {
            Opcode::FpuAdd => FPUOperation::Add,
            Opcode::FpuSub => FPUOperation::Sub,
            Opcode::FpuMul => FPUOperation::Mul,
            Opcode::FpuDiv => FPUOperation::Div,
            Opcode::FpuSqrt => FPUOperation::Sqrt,
            Opcode::FpuCmp => FPUOperation::Cmp,
            Opcode::FpuMin => FPUOperation::Min,
            Opcode::FpuMax => FPUOperation::Max,
            Opcode::FpuConvert => FPUOperation::Convert,
            Opcode::FpuRound => FPUOperation::Round,
            Opcode::FpuMov => {
                // Mov FPU simple
                let src_value = self.fpu.read_fp_register(src1_reg)
                    .map_err(|e| format!("Erreur lecture registre FPU: {}", e))?;
                self.fpu.write_fp_register(dst_reg, src_value)
                    .map_err(|e| format!("Erreur écriture registre FPU: {}", e))?;
                return Ok(src_value);
            }
            _ => return Err(format!("Opération FPU non supportée: {:?}", opcode)),
        };

        self.fpu.execute(operation, dst_reg, src1_reg, src2_reg, precision)
            .map_err(|e| format!("Erreur exécution FPU: {}", e))?;

        // Retourner la valeur du registre de destination
        self.fpu.read_fp_register(dst_reg)
            .map_err(|e| format!("Erreur lecture résultat FPU: {}", e))
    }

    /// Réinitialise l'étage Execute
    pub fn reset(&mut self) {
        self.branch_predictor = BranchPredictor::new(PredictorType::Dynamic);
        self.vector_alu.reset();
        self.fpu.reset();
        self.branch_predictions = 0;
        self.branch_hits = 0;
    }

    /// Accès en lecture seule au VectorALU
    pub fn get_vector_alu(&self) -> &VectorALU {
        &self.vector_alu
    }

    /// Accès en écriture au VectorALU
    pub fn get_vector_alu_mut(&mut self) -> &mut VectorALU {
        &mut self.vector_alu
    }

    /// Accès en lecture seule au FPU
    pub fn get_fpu(&self) -> &FPU {
        &self.fpu
    }

    /// Accès en écriture au FPU
    pub fn get_fpu_mut(&mut self) -> &mut FPU {
        &mut self.fpu
    }
}










// Test unitaire pour l'étage Execute
#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::format::ArgType;
    use crate::bytecode::format::InstructionFormat;
    use crate::bytecode::instructions::Instruction;
    use crate::bytecode::opcodes::Opcode;
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
            rs1: Some(0), // index
            rs2: Some(1),
            rd: Some(0),
            rs1_value: 5, // R0=5
            rs2_value: 7, // R1=7
            immediate: None,
            branch_addr: None,
            branch_prediction: None,
            stack_operation: None,
            mem_addr: None,
            stack_value: None,
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
            rs1_value: 5, // R0=5
            rs2_value: 7, // R1=7
            immediate: None,
            branch_addr: None,
            branch_prediction: None,
            stack_operation: None,
            mem_addr: None,
            stack_value: None,
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
            rs1_value: 10, // R0=10
            rs2_value: 7,  // R1=7
            immediate: None,
            branch_addr: None,
            branch_prediction: None,
            stack_operation: None,
            mem_addr: None,
            stack_value: None,
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
            rs1_value: 10, // R0=10
            rs2_value: 7,  // R1=7
            immediate: None,
            branch_addr: None,
            branch_prediction: None,
            stack_operation: None,
            mem_addr: None,
            stack_value: None,
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
            (Opcode::Add, 5, 7, 12),
            (Opcode::Sub, 10, 3, 7),
            (Opcode::Mul, 4, 5, 20),
            (Opcode::Div, 20, 4, 5),
            (Opcode::Mod, 10, 3, 1),
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
                branch_prediction: None,
                stack_operation: None,
                mem_addr: None,
                stack_value: None,
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
            (Opcode::Or, 0xF0, 0x0F, 0xFF),
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
                branch_prediction: None,
                stack_operation: None,
                mem_addr: None,
                stack_value: None,
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
            vec![],
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
            branch_prediction: None,
            stack_operation: None,
            mem_addr: Some(0x2000),
            stack_value: None,
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
            rs1_value: 5, // R0=5
            rs2_value: 7, // R1=7
            immediate: None,
            branch_addr: None,
            branch_prediction: None,
            stack_operation: None,
            mem_addr: None,
            stack_value: None,
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
            rs1_value: res_add.alu_result, // R3=12
            rs2_value: 3,                  // R2=3
            immediate: None,
            branch_addr: None,
            branch_prediction: None,
            stack_operation: None,
            mem_addr: None,
            stack_value: None,
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
            rs1_value: 5, // R0=5
            rs2_value: 7, // R1=7
            immediate: None,
            branch_addr: None,
            branch_prediction: None,
            stack_operation: None,
            mem_addr: None,
            stack_value: None,
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
            rs1_value: em_reg_add.alu_result, // R3=12
            rs2_value: 0,
            immediate: None,
            branch_addr: None,
            branch_prediction: None,
            stack_operation: None,
            mem_addr: None,
            stack_value: None,
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
            rs1_value: 13, // R3=13
            rs2_value: 13, // R2=13
            immediate: None,
            branch_addr: None,
            branch_prediction: None,
            stack_operation: None,
            mem_addr: None,
            stack_value: None,
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
            vec![0, 16, 0, 0], // Adresse 0x1000 (little-endian)
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
            branch_prediction: None,
            stack_operation: None,
            mem_addr: None,
            stack_value: None,
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
            vec![0, 16, 0, 0], // Adresse 0x1000
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
            branch_prediction: None,
            stack_operation: None,
            mem_addr: None,
            stack_value: None,
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
            vec![0, 0, 32, 0, 0], // R0 = Mem[0x2000]
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
            branch_prediction: None,
            stack_operation: None,
            mem_addr: Some(0x2000),
            stack_value: None,
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


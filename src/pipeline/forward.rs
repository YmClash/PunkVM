// // src/pipeline/forward.rs
//
// use crate::bytecode::opcodes::Opcode;
// use crate::pipeline::{DecodeExecuteRegister, ExecuteMemoryRegister, MemoryWritebackRegister};
//
// /// Unité de forwarding
// pub struct ForwardingUnit {
//     /// Compteur de forwarding
//     pub forwards_count: u64,
// }
//
// /// Représente une source de forwarding
// #[derive(Debug, Clone, Copy, PartialEq)]
// pub enum ForwardingSource {
//     // Execute,
//     // Memory,
//     ExecuteMemory,
//     Writeback,
//     // None,
// }
//
// #[derive(Debug, Clone)]
// pub struct ForwardingInfo {
//     pub source: ForwardingSource,
//     pub value: u64,
//     pub register: usize,
// }
//
// impl ForwardingUnit {
//     /// Crée une nouvelle unité de forwarding
//     pub fn new() -> Self {
//         Self { forwards_count: 0 }
//     }
//
//     /// Effectue le forwarding des données et retourne les informations sur le forwarding effectué
//     /// Méthode principale de forwarding
//     /// On renvoie un Vec<ForwardingInfo> pour tracer ce qui a été forwardé
//     pub fn forward_with_info(
//         &mut self,
//         decode_reg: &mut DecodeExecuteRegister,
//         mem_reg: &Option<ExecuteMemoryRegister>,
//         wb_reg: &Option<MemoryWritebackRegister>,
//     ) -> Vec<ForwardingInfo> {
//         let mut info_list = Vec::new();
//
//         let rs1_idx = decode_reg.rs1;
//         let rs2_idx = decode_reg.rs2;
//
//         // S'il n'y a pas de registres source, on ne fait rien
//         if rs1_idx.is_none() && rs2_idx.is_none() {
//             return info_list;
//         }
//
//         // 1. Forwarding depuis l'étage Memory (Execute->MemoryRegister)
//         if let Some(mem) = mem_reg {
//             if let Some(rd_mem) = mem.rd {
//                 // On veut forwarder la valeur mem.alu_result
//                 let mem_val = mem.alu_result;
//
//                 // si decode_reg.rs1 == rd_mem => forward
//                 if rs1_idx == Some(rd_mem) {
//                     decode_reg.rs1_value = mem_val;
//                     self.forwards_count += 1;
//                     info_list.push(ForwardingInfo {
//                         source: ForwardingSource::ExecuteMemory,
//                         register: rd_mem,
//                         value: mem_val,
//                     });
//                     println!("Forwarding: MEM->rs1  R{} = {}", rd_mem, mem_val);
//                 }
//                 // si decode_reg.rs2 == rd_mem => forward
//                 if rs2_idx == Some(rd_mem) {
//                     decode_reg.rs2_value = mem_val;
//                     self.forwards_count += 1;
//                     info_list.push(ForwardingInfo {
//                         source: ForwardingSource::ExecuteMemory,
//                         register: rd_mem,
//                         value: mem_val,
//                     });
//                     println!("Forwarding: MEM->rs2  R{} = {}", rd_mem, mem_val);
//                 }
//             }
//         }
//
//         // 2. Forwarding depuis Writeback
//         //    On ne forward que si on n'a pas déjà forwardé depuis MEM
//         //    (sinon on écraserait la valeur plus récente)
//         if let Some(wb) = wb_reg {
//             if let Some(rd_wb) = wb.rd {
//                 let wb_val = wb.result;
//
//                 // Vérifier rs1
//                 if rs1_idx == Some(rd_wb) {
//                     // S'assurer qu'on n'a pas déjà forwardé pour rs1
//                     let already_forwarded = info_list.iter().any(|fi| {
//                         fi.register == rd_wb && fi.source == ForwardingSource::ExecuteMemory
//                     });
//                     if !already_forwarded {
//                         decode_reg.rs1_value = wb_val;
//                         self.forwards_count += 1;
//                         info_list.push(ForwardingInfo {
//                             source: ForwardingSource::Writeback,
//                             register: rd_wb,
//                             value: wb_val,
//                         });
//                         println!("Forwarding: WB->rs1 R{} = {}", rd_wb, wb_val);
//                     }
//                 }
//                 // Vérifier rs2
//                 if rs2_idx == Some(rd_wb) {
//                     let already_forwarded = info_list.iter().any(|fi| {
//                         fi.register == rd_wb && fi.source == ForwardingSource::ExecuteMemory
//                     });
//                     if !already_forwarded {
//                         decode_reg.rs2_value = wb_val;
//                         self.forwards_count += 1;
//                         info_list.push(ForwardingInfo {
//                             source: ForwardingSource::Writeback,
//                             register: rd_wb,
//                             value: wb_val,
//                         });
//                         println!("Forwarding: WB->rs2 R{} = {}", rd_wb, wb_val);
//                     }
//                 }
//             }
//         }
//
//         // 3. Cas spécial : si l'étage MEM est un load => la donnée n'est pas encore disponible
//         //    On peut décider qu'on retire le forwarding
//         //    (ou on laisse la detection d'un hazard "LoadUse" forcer un stall).
//         if let Some(mem) = mem_reg {
//             if let Some(rd) = mem.rd {
//                 let is_load = matches!(
//                 mem.instruction.opcode,
//                 Opcode::Load | Opcode::LoadB | Opcode::LoadW | Opcode::LoadD
//             );
//                 if is_load && (rs1_idx == Some(rd) || rs2_idx == Some(rd)) {
//                     // On retire les entries correspondantes
//                     info_list.retain(|fi| {
//                         !(fi.register == rd && fi.source == ForwardingSource::ExecuteMemory)
//                     });
//                     println!(
//                         "LoadUse hazard : cannot forward from MEM for a load not yet finished"
//                     );
//                 }
//             }
//         }
//
//         info_list
//     }
//
//     /// Effectue le forwarding des données (version simplifiée)
//     /// Version simplifiée, on jette la liste
//     pub fn forward(
//         &mut self,
//         decode_reg: &mut DecodeExecuteRegister,
//         mem_reg: &Option<ExecuteMemoryRegister>,
//         wb_reg: &Option<MemoryWritebackRegister>,
//     ) {
//         let _ = self.forward_with_info(decode_reg, mem_reg, wb_reg);
//
//     }
//
//     /// Réinitialise l'unité de forwarding
//     pub fn reset(&mut self) {
//         self.forwards_count = 0;
//     }
//
//     /// Retourne le nombre de forwards effectués
//     pub fn get_forwards_count(&self) -> u64 {
//         self.forwards_count
//     }
// }
//

/////////////////////////////////////////////
// src/pipeline/forward.rs

use crate::bytecode::opcodes::Opcode;
use crate::pipeline::{DecodeExecuteRegister, ExecuteMemoryRegister, MemoryWritebackRegister};

/// Unité de forwarding
#[derive(Debug)] // Ajout de Debug pour l'affichage
pub struct ForwardingUnit {
    /// Compteur de forwarding (nombre de fois où une valeur a été forwardée)
    pub forwards_count: u64,
}

/// Représente une source de forwarding pour une valeur spécifique
/// Conservé tel quel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ForwardingSource {
    ExecuteMemory, // Donnée venant du résultat ALU (dispo après EX) ou adresse calculée
    Writeback,     // Donnée venant de la fin de l'étage MEM (load complété ou ALU plus ancienne)
}

/// Informations sur une opération de forwarding effectuée
/// Conservé tel quel.
#[derive(Debug, Clone)]
pub struct ForwardingInfo {
    pub source: ForwardingSource,
    pub value: u64,
    pub register: usize, // Le registre rd qui a produit la valeur
}

impl ForwardingUnit {
    /// Crée une nouvelle unité de forwarding
    pub fn new() -> Self {
        Self { forwards_count: 0 }
    }

    /// Effectue le forwarding des données et retourne les informations sur le forwarding effectué.
    /// Modifie `decode_reg` directement.
    /// Fonction conservée pour la compatibilité (même si on n'utilise pas forcément le retour).
    pub fn forward_with_info(
        &mut self,
        decode_reg: &mut DecodeExecuteRegister, // Instruction en Decode (Destination)
        mem_reg: &Option<ExecuteMemoryRegister>, // Instruction en Execute (Source potentielle 1)
        wb_reg: &Option<MemoryWritebackRegister>, // Instruction en Memory (Source potentielle 2)
    ) -> Vec<ForwardingInfo> {
        let mut info_list = Vec::new();

        let rs1_needed = decode_reg.rs1;
        let rs2_needed = decode_reg.rs2;

        // Si Decode n'a pas besoin de registres source, on sort
        if rs1_needed.is_none() && rs2_needed.is_none() {
            return info_list;
        }

        // --- Priorité 1: Forwarding depuis Execute (EX/MEM Register) ---
        if let Some(mem) = mem_reg {
            if let Some(rd_ex) = mem.rd {
                // IMPORTANT: On ne forward PAS depuis EX si l'instruction est un Load,
                // car la donnée n'est pas encore disponible (elle le sera après MEM).
                let is_load_in_ex = matches!(
                    mem.instruction.opcode,
                    Opcode::Load | Opcode::LoadB | Opcode::LoadW | Opcode::LoadD | Opcode::Pop
                );

                if !is_load_in_ex {
                    // La valeur à forwarder est le résultat ALU
                    let forward_val = mem.alu_result;

                    // Forward vers rs1 ?
                    if rs1_needed == Some(rd_ex) {
                        println!(
                            "   [Forwarding] EX/MEM -> DE (rs1): R{} gets value {} from EX stage (ALU result).",
                            rd_ex, forward_val
                        );
                        decode_reg.rs1_value = forward_val;
                        self.forwards_count += 1;
                        info_list.push(ForwardingInfo {
                            source: ForwardingSource::ExecuteMemory,
                            value: forward_val,
                            register: rd_ex,
                        });
                    }
                    // Forward vers rs2 ? (Attention si rs1 == rs2, déjà fait)
                    if rs2_needed == Some(rd_ex) && rs1_needed != Some(rd_ex) {
                        println!(
                            "   [Forwarding] EX/MEM -> DE (rs2): R{} gets value {} from EX stage (ALU result).",
                            rd_ex, forward_val
                        );
                        decode_reg.rs2_value = forward_val;
                        self.forwards_count += 1;
                        info_list.push(ForwardingInfo {
                            source: ForwardingSource::ExecuteMemory,
                            value: forward_val,
                            register: rd_ex,
                        });
                    }
                }
                // else : c'est un Load en EX, on ne forward pas depuis ici.
            }
        }

        // --- Priorité 2: Forwarding depuis Writeback (MEM/WB Register) ---
        // La donnée vient de l'étage Memory, elle est prête pour Writeback.
        if let Some(wb) = wb_reg {
            if let Some(rd_wb) = wb.rd {
                let forward_val = wb.result; // La valeur finale (ALU ou mémoire)

                // Forward vers rs1 ?
                // Vérifier si rs1 a besoin de rd_wb ET n'a PAS déjà été servi par EX/MEM
                let already_forwarded_rs1 = info_list.iter().any(|info| info.register == rd_wb); // Simplifié: si déjà forwardé pour ce reg

                if rs1_needed == Some(rd_wb) && !already_forwarded_rs1 {
                    println!(
                        "   [Forwarding] MEM/WB -> DE (rs1): R{} gets value {} from MEM stage result.",
                        rd_wb, forward_val
                    );
                    decode_reg.rs1_value = forward_val;
                    self.forwards_count += 1;
                    info_list.push(ForwardingInfo {
                        source: ForwardingSource::Writeback,
                        value: forward_val,
                        register: rd_wb,
                    });
                }

                // Forward vers rs2 ?
                // Vérifier si rs2 a besoin de rd_wb ET n'a PAS déjà été servi par EX/MEM
                // ET n'est pas le même que rs1 qui vient d'être servi par MEM/WB
                let already_forwarded_rs2 = info_list.iter().any(|info| info.register == rd_wb); // Simplifié
                let rs1_just_forwarded_from_wb = rs1_needed == Some(rd_wb) && !already_forwarded_rs1;

                if rs2_needed == Some(rd_wb) && !already_forwarded_rs2 && !(rs1_needed == Some(rd_wb) && rs1_just_forwarded_from_wb) {
                    println!(
                        "   [Forwarding] MEM/WB -> DE (rs2): R{} gets value {} from MEM stage result.",
                        rd_wb, forward_val
                    );
                    decode_reg.rs2_value = forward_val;
                    self.forwards_count += 1;
                    info_list.push(ForwardingInfo {
                        source: ForwardingSource::Writeback,
                        value: forward_val,
                        register: rd_wb,
                    });
                }
            }
        }

        // Note: Le cas spécial pour annuler le forwarding MEM->EX pour un Load-Use a été retiré ici.
        // La raison est que `is_load_use_hazards` dans hazard.rs devrait détecter cela
        // et forcer un stall *avant* que le forwarding ne soit tenté ou appliqué incorrectement.
        // Le forwarding depuis MEM/WB gérera le cas où la donnée du Load est prête.

        info_list
    }

    /// Effectue le forwarding des données (version simplifiée sans retour détaillé).
    /// Fonction conservée pour la compatibilité.
    pub fn forward(
        &mut self,
        decode_reg: &mut DecodeExecuteRegister,
        mem_reg: &Option<ExecuteMemoryRegister>,
        wb_reg: &Option<MemoryWritebackRegister>,
    ) {
        // Appelle la fonction principale mais ignore le résultat détaillé.
        let _ = self.forward_with_info(decode_reg, mem_reg, wb_reg);
    }

    /// Réinitialise l'unité de forwarding
    pub fn reset(&mut self) {
        println!("Resetting forwarding count to 0.");
        self.forwards_count = 0;
    }

    /// Retourne le nombre de forwards effectués
    pub fn get_forwards_count(&self) -> u64 {
        println!("Total forward operations: {}", self.forwards_count);
        self.forwards_count
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////////////////


//
//
//
//
//
//
//
// /// Tests pour l'unité de forwarding
// #[cfg(test)]
// mod forwarding_tests {
//     use super::*;
//     use crate::bytecode::instructions::Instruction;
//     use crate::bytecode::opcodes::Opcode;
//     use crate::pipeline::{DecodeExecuteRegister, ExecuteMemoryRegister, MemoryWritebackRegister};
//
//
//     #[test]
//     fn test_forwarding_unit_creation() {
//         let unit = ForwardingUnit::new();
//         assert_eq!(
//             unit.get_forwards_count(),
//             0,
//             "Unité de forwarding devrait commencer à 0"
//         );
//     }
//
//     #[test]
//     fn test_forwarding_unit_reset() {
//         let mut unit = ForwardingUnit::new();
//         // Simuler un forward
//         unit.forwards_count = 3;
//         unit.reset();
//         assert_eq!(
//             unit.get_forwards_count(),
//             0,
//             "Après reset, le compteur doit être 0"
//         );
//     }
//
//     #[test]
//     fn test_forwarding_no_sources() {
//         let mut unit = ForwardingUnit::new();
//
//         // DecodeExecuteRegister sans rs1/rs2 => pas de forwarding
//         let mut ex_reg = DecodeExecuteRegister {
//             instruction: Instruction::create_no_args(Opcode::Nop),
//             pc: 0,
//             rs1: None,
//             rs2: None,
//             rd: None,
//             rs1_value: 0,
//             rs2_value: 0,
//             immediate: None,
//             branch_addr: None,
//             branch_prediction: None,
//             stack_operation: None,
//             mem_addr: None,
//             stack_value: None,
//         };
//
//         // Pas de mem_reg, pas de wb_reg
//         unit.forward(&mut ex_reg, &None, &None);
//
//         assert_eq!(
//             unit.get_forwards_count(),
//             0,
//             "Aucun forwarding ne doit avoir lieu"
//         );
//         assert_eq!(ex_reg.rs1_value, 0);
//         assert_eq!(ex_reg.rs2_value, 0);
//     }
//
//     #[test]
//     fn test_forwarding_from_memory_stage() {
//         let mut unit = ForwardingUnit::new();
//
//         // ex_reg a besoin de R1
//         let mut ex_reg = DecodeExecuteRegister {
//             instruction: Instruction::create_reg_reg(Opcode::Add, 0, 1),
//             pc: 0,
//             rs1: Some(0),
//             rs2: Some(1),
//             rd: Some(2),
//             rs1_value: 5,  // Valeur "originale" R0
//             rs2_value: 10, // Valeur "originale" R1
//             immediate: None,
//             branch_addr: None,
//             branch_prediction: None,
//             stack_operation: None,
//             mem_addr: None,
//             stack_value: None,
//         };
//
//         // L’étage Memory écrit R1=42
//         let mem_reg = ExecuteMemoryRegister {
//             instruction: Instruction::create_reg_imm8(Opcode::Add, 1, 10),
//             alu_result: 42,
//             rd: Some(1),
//             store_value: None,
//             mem_addr: None,
//             branch_target: None,
//             branch_taken: false,
//             branch_prediction_correct: Option::from(false),
//             stack_operation: None,
//             stack_result: None,
//             ras_prediction_correct: None,
//             halted: false,
//         };
//
//         // On fait le forwarding
//         unit.forward(&mut ex_reg, &Some(mem_reg), &None);
//
//         // Comme ex_reg.rs2 == 1, on met ex_reg.rs2_value à 42
//         assert_eq!(unit.get_forwards_count(), 1);
//         assert_eq!(ex_reg.rs1_value, 5); //"R0 (rs1) ne doit pas etre affecté par le forwarding";
//         assert_eq!(ex_reg.rs2_value, 42);// "R1 (rs2) doit être forwardé depuis EX/MEM"
//     }
//
//     #[test]
//     fn test_forwarding_with_three_register_format() {
//         let mut unit = ForwardingUnit::new();
//
//         // ADD R3, R1, R2 => rd=R3, rs1=R1, rs2=R2
//         let mut ex_reg = DecodeExecuteRegister {
//             instruction: Instruction::create_reg_reg_reg(Opcode::Add, 3, 1, 2),
//             pc: 0,
//             rs1: Some(1),
//             rs2: Some(2),
//             rd: Some(3),
//             rs1_value: 5, // R1
//             rs2_value: 7, // R2
//             immediate: None,
//             branch_addr: None,
//             branch_prediction: None,
//             stack_operation: None,
//             mem_addr: None,
//             stack_value: None,
//         };
//
//         // Memory stage : R1 = 20
//         let mem_reg = ExecuteMemoryRegister {
//             instruction: Instruction::create_reg_imm8(Opcode::Add, 1, 10),
//             alu_result: 20,
//             rd: Some(1),
//             store_value: None,
//             mem_addr: None,
//             branch_target: None,
//             branch_taken: false,
//             branch_prediction_correct: Option::from(false),
//             stack_operation: None,
//             stack_result: None,
//             ras_prediction_correct: None,
//             halted: false,
//         };
//
//         let infos = unit.forward_with_info(&mut ex_reg, &Some(mem_reg), &None);
//
//         assert_eq!(unit.get_forwards_count(), 1);
//         assert_eq!(infos.len(), 1);
//         assert_eq!(infos[0].value, 20);
//         assert_eq!(infos[0].register, 1);
//         assert_eq!(ex_reg.rs1_value, 20); // R1
//         assert_eq!(ex_reg.rs2_value, 7); // R2 inchangé
//     }
//
//     #[test]
//     fn test_forwarding_from_writeback_stage() {
//         let mut unit = ForwardingUnit::new();
//
//         // ex_reg a besoin de R1
//         let mut ex_reg = DecodeExecuteRegister {
//             instruction: Instruction::create_reg_reg(Opcode::Add, 0, 1),
//             pc: 0,
//             rs1: Some(0),
//             rs2: Some(1),
//             rd: Some(2),
//             rs1_value: 5,  // R0
//             rs2_value: 10, // R1
//             immediate: None,
//             branch_addr: None,
//             branch_prediction: None,
//             stack_operation: None,
//             mem_addr: None,
//             stack_value: None,
//         };
//
//         // Writeback stage dit R1 = 42
//         let wb_reg = MemoryWritebackRegister {
//             instruction: Instruction::create_reg_imm8(Opcode::Load, 1, 0),
//             result: 42,
//             rd: Some(1),
//         };
//
//         unit.forward(&mut ex_reg, &None, &Some(wb_reg));
//         assert_eq!(unit.get_forwards_count(), 1);
//         assert_eq!(ex_reg.rs2_value, 42);
//         assert_eq!(ex_reg.rs1_value, 5);
//     }
//
//     #[test]
//     fn test_forwarding_priority() {
//         let mut unit = ForwardingUnit::new();
//
//         // ex_reg veut R1
//         let mut ex_reg = DecodeExecuteRegister {
//             instruction: Instruction::create_reg_reg(Opcode::Add, 0, 1),
//             pc: 0,
//             rs1: Some(0),
//             rs2: Some(1),
//             rd: Some(2),
//             rs1_value: 5,
//             rs2_value: 10,
//             immediate: None,
//             branch_addr: None,
//             branch_prediction: None,
//             stack_operation: None,
//             mem_addr: None,
//             stack_value: None,
//         };
//
//         let mem_reg = ExecuteMemoryRegister {
//             instruction: Instruction::create_no_args(Opcode::Add),
//             alu_result: 42,
//             rd: Some(1),
//             store_value: None,
//             mem_addr: None,
//             branch_target: None,
//             branch_taken: false,
//             branch_prediction_correct: Option::from(false),
//             stack_operation: None,
//             stack_result: None,
//             ras_prediction_correct: None,
//             halted: false,
//         };
//
//         let wb_reg = MemoryWritebackRegister {
//             instruction: Instruction::create_no_args(Opcode::Add),
//             result: 24,
//             rd: Some(1),
//         };
//
//         // memory dit R1=42, writeback dit R1=24 => la source prioritaire est memory
//         let infos = unit.forward_with_info(&mut ex_reg, &Some(mem_reg), &Some(wb_reg));
//         assert_eq!(unit.get_forwards_count(), 1, "Un seul forward prioritaire");
//         assert_eq!(
//             ex_reg.rs2_value, 42,
//             "valeur prioritaire=42 (Memory) et pas 24 (WB)"
//         );
//
//         assert_eq!(infos.len(), 1);
//         assert_eq!(infos[0].value, 42);
//     }
//
//     #[test]
//     fn test_forwarding_load_use_hazard() {
//         let mut unit = ForwardingUnit::new();
//
//         // Decode (DE/EX reg): ADD R3, R2, R1 => utilise R1
//         let mut decode_reg = DecodeExecuteRegister {
//             instruction: Instruction::create_reg_reg_reg(Opcode::Add, 3, 2, 1), // ADD R3, R2, R1
//             pc: 0,
//             rs1: Some(2), // Lit R2
//             rs2: Some(1), // Lit R1
//             rd: Some(3),
//             rs1_value: 5, // R2
//             rs2_value: 0, // R1 initial
//             immediate: None,
//             branch_addr: None,
//             branch_prediction: None,
//             stack_operation: None,
//             mem_addr: None,
//             stack_value: None,
//         };
//
//         // Execute (EX/MEM reg): LOAD R1, [addr] => écrit R1
//         let mem_reg = ExecuteMemoryRegister {
//             instruction: Instruction::create_reg_imm8(Opcode::Load, 1, 0), // LOAD R1, 0(Rx)
//             alu_result: 0x100, // Adresse calculée (pas la valeur chargée!)
//             rd: Some(1),       // Écrit R1
//             store_value: None,
//             mem_addr: Some(0x100),
//             branch_target: None,
//             branch_taken: false,
//             branch_prediction_correct: None, // Modifié ici
//             stack_operation: None,
//             stack_result: None,
//             ras_prediction_correct: None,
//             halted: false,
//         };
//
//         // Memory (MEM/WB reg): (Instruction précédente, écrit R2=20)
//         let wb_reg = MemoryWritebackRegister {
//             instruction: Instruction::create_reg_imm8(Opcode::Add, 2, 5), // ADD R2, 5
//             result: 20, // R2 final = 20
//             rd: Some(2),
//         };
//
//         // --- Appel du Forwarding ---
//         let info = unit.forward_with_info(&mut decode_reg, &Some(mem_reg), &Some(wb_reg));
//
//         // --- Vérifications ---
//         // 1. R2 (rs1) doit être forwardé depuis MEM/WB (valeur 20)
//         // 2. R1 (rs2) dépend d'un LOAD en EX. Le forwarding *depuis EX/MEM* NE doit PAS se produire.
//         //    R1 (rs2) ne doit PAS être forwardé non plus depuis MEM/WB s'il y avait une écriture R1 là.
//         // 3. La valeur de R1 (rs2) doit rester sa valeur initiale (0) car le forwarding est bloqué
//         //    par le type d'instruction (Load) dans EX/MEM.
//
//         // Vérifier R2 (rs1)
//         assert_eq!(decode_reg.rs1_value, 20, "R2 (rs1) doit être forwardé depuis MEM/WB");
//
//         // Vérifier R1 (rs2) - NE DOIT PAS être forwardé depuis EX/MEM car c'est un Load
//         assert_eq!(
//             decode_reg.rs2_value, 0,
//             "R1 (rs2) NE doit PAS être forwardé depuis EX/MEM pour un Load"
//         );
//
//         // Vérifier le nombre de forwards effectués (seulement pour R2)
//         assert_eq!(unit.get_forwards_count(), 1, "Seul R2 doit être forwardé (depuis MEM/WB)");
//
//         // Vérifier les infos retournées (doit contenir uniquement le forward de R2)
//         assert_eq!(info.len(), 1, "Seule l'info pour R2 doit être présente");
//         if !info.is_empty() {
//             assert_eq!(info[0].register, 2, "Le registre forwardé est R2");
//             assert_eq!(info[0].value, 20, "La valeur forwardée est 20");
//             assert_eq!(info[0].source, ForwardingSource::Writeback, "La source est Writeback");
//             // assert_eq!(info[0].producing_register, 2); // R2
//             // assert_eq!(info[0].forwarded_value, 20);
//             // assert_eq!(info[0].source_stage, ForwardingSource::Writeback); // Corrigé: vient de MEM/WB
//         }
//         // Note: Le test initial échouait probablement parce qu'il attendait que la valeur
//         // de R1 (rs2) soit forwardée à 10 (l'alu_result du Load), ce qui ne doit pas arriver.
//         // Ou il attendait que la liste `info` soit vide, ce qui n'est pas correct non plus
//         // car R2 est bien forwardé.
//     }
//
//     #[test]
//     fn test_complex_forwarding_scenario() {
//         let mut unit = ForwardingUnit::new();
//
//         // On veut R5=R1+R2=15, en MEM stage,
//         // ex_reg => MUL R4, R5, R3 => R5=15, R3=3 => R4=45
//         let mem_reg = ExecuteMemoryRegister {
//             instruction: Instruction::create_reg_reg_reg(Opcode::Add, 5, 1, 2),
//             alu_result: 15,
//             rd: Some(5),
//             store_value: None,
//             mem_addr: None,
//             branch_target: None,
//             branch_taken: false,
//             branch_prediction_correct: Option::from(false),
//             stack_operation: None,
//             stack_result: None,
//             ras_prediction_correct: None,
//             halted: false,
//         };
//
//         let mut ex_reg = DecodeExecuteRegister {
//             instruction: Instruction::create_reg_reg_reg(Opcode::Mul, 4, 5, 3),
//             pc: 4,
//             rs1: Some(5),
//             rs2: Some(3),
//             rd: Some(4),
//             rs1_value: 0, // R5 original=?
//             rs2_value: 3, // R3=3
//             immediate: None,
//             branch_addr: None,
//             branch_prediction: None,
//             stack_operation: None,
//             mem_addr: None,
//             stack_value: None,
//         };
//
//         let info = unit.forward_with_info(&mut ex_reg, &Some(mem_reg), &None);
//
//         // R5=15
//         assert_eq!(unit.get_forwards_count(), 1);
//         assert_eq!(info.len(), 1);
//         assert_eq!(ex_reg.rs1_value, 15);
//         assert_eq!(ex_reg.rs2_value, 3);
//         assert_eq!(info[0].register, 5);
//         assert_eq!(info[0].value, 15);
//     }
//
//     #[test]
//     fn test_multiple_forwarding_sources() {
//         let mut unit = ForwardingUnit::new();
//
//         // ex_reg a besoin de R1 et R2
//         let mut ex_reg = DecodeExecuteRegister {
//             instruction: Instruction::create_reg_reg_reg(Opcode::Add, 3, 1, 2),
//             pc: 0,
//             rs1: Some(1),
//             rs2: Some(2),
//             rd: Some(3),
//             rs1_value: 0,
//             rs2_value: 0,
//             immediate: None,
//             branch_addr: None,
//             branch_prediction: None,
//             stack_operation: None,
//             mem_addr: None,
//             stack_value: None,
//         };
//
//         // Memory => R1=42
//         let mem_reg = ExecuteMemoryRegister {
//             instruction: Instruction::create_reg_imm8(Opcode::Add, 1, 10),
//             alu_result: 42,
//             rd: Some(1),
//             store_value: None,
//             mem_addr: None,
//             branch_target: None,
//             branch_taken: false,
//             branch_prediction_correct: Option::from(false),
//             stack_operation: None,
//             stack_result: None,
//             ras_prediction_correct: None,
//             halted: false,
//         };
//
//         // Writeback => R2=24
//         let wb_reg = MemoryWritebackRegister {
//             instruction: Instruction::create_reg_imm8(Opcode::Load, 2, 0),
//             result: 24,
//             rd: Some(2),
//         };
//
//         // On forward R1 depuis Memory, R2 depuis Writeback
//         let info = unit.forward_with_info(&mut ex_reg, &Some(mem_reg), &Some(wb_reg));
//
//         assert_eq!(unit.get_forwards_count(), 2);
//         assert_eq!(info.len(), 2);
//         // R1 => 42, R2 => 24
//         assert_eq!(ex_reg.rs1_value, 42);
//         assert_eq!(ex_reg.rs2_value, 24);
//     }
// }

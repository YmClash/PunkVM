//src/pipeline/hazard.rs

use crate::bytecode::opcodes::Opcode;
use crate::pipeline::PipelineState;

/// Unité de détection de hazards
pub struct HazardDetectionUnit {
    // Compteur de hazards détectés
    pub hazards_count: u64,
    // Compteur de dépendances de données détectées (incluant celles résolues par forwarding)
    pub data_dependencies_count: u64,
    // Compteur de forwarding potentiels détectés
    pub potential_forwards_count: u64,
    branch_stall_cycles: u32,
}

#[derive(Debug, PartialEq)]
pub enum HazardType {
    None,
    LoadUse,
    StoreLoad,
    DataDependency,
    ControlHazard,
    StructuralHazard,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HazardResult {
    None,
    StoreLoad,
    LoadUse,
    DataDependency,
    ControlHazard,
    StructuralHazard,
}

impl HazardDetectionUnit {
    /// Crée une nouvelle unité de détection de hazards
    pub fn new() -> Self {
        Self {
            hazards_count: 0,
            data_dependencies_count: 0,
            potential_forwards_count: 0,
            branch_stall_cycles: 0,
        }
    }

    /// Détecte les hazards dans le pipeline et retourne le type détecté
    pub fn detect_hazards_with_type(&mut self, state: &PipelineState) -> HazardResult {
        // 1. Load-Use Hazards (cas spécial de Data Hazard qui DOIT causer un stall)
        if self.is_load_use_hazards(state) {
            println!("Load-Use hazard detected (true stall required)");
            self.hazards_count += 1;
            return HazardResult::LoadUse;
        }

        // 2. Data Hazards (RAW - Read After Write) - Ne compte que si non résolvable par forwarding
        if self.is_data_hazard_not_forwardable(state) {
            println!("Data hazard detected (not forwardable)");
            self.hazards_count += 1;
            return HazardResult::DataDependency;
        }
        
        // Détecter les dépendances de données qui PEUVENT être forwardées (pour les stats)
        if self.is_data_dependency_forwardable(state) {
            println!("Data dependency detected (can be forwarded)");
            self.data_dependencies_count += 1;
            self.potential_forwards_count += 1;
            // Ne retourne PAS de hazard car le forwarding va le résoudre
        }

        // 3. Control Hazards
        if self.is_control_hazard(state) {
            println!("Control hazard detected");
            self.hazards_count += 1;
            return HazardResult::ControlHazard;
        }

        // 4. Structural Hazards
        if self.is_structural_hazard(state) {
            println!("Structural hazard detected");
            self.hazards_count += 1;
            return HazardResult::StructuralHazard;
        }

        // 5. Store-Load Hazards
        if self.is_store_load_hazard(state) {
            println!("Store-Load hazard detected");
            self.hazards_count += 1;
            return HazardResult::StoreLoad;
        }

        // Aucun hazard détecté
        HazardResult::None
    }

    /// Détecte les hazards dans le pipeline
    /// Méthode principale : détecte s'il y a un hazard (et lequel) dans l'état pipeline
    /// Renvoie un HazardType
    // pub fn detect_hazard(&mut self, state: &PipelineState) -> HazardType {
    //     // On va tester dans un ordre de priorité
    //     // (On pourrait inverser l'ordre, c'est un choix d'implémentation.)
    //
    //     // 1. Load-Use hazard (cas particulier, souvent prioritaire)
    //     // if self.is_load_use_hazards(state) {
    //     //     self.hazards_count += 1;
    //     //     return HazardType::LoadUse;
    //     // }
    //     //
    //     // // 2. Data hazard "classique"
    //     // if self.is_data_hazard(state) {
    //     //     self.hazards_count += 1;
    //     //     return HazardType::DataDependency;
    //     // }
    //     //
    //     // // 3. Store-Load hazard
    //     // if self.is_store_load_hazard(state) {
    //     //     self.hazards_count += 1;
    //     //     return HazardType::StoreLoad;
    //     // }
    //     //
    //     // // 4. Control hazard
    //     // if self.is_control_hazard(state) {
    //     //     self.hazards_count += 1;
    //     //     return HazardType::ControlHazard;
    //     // }
    //     //
    //     // // 5. Structural hazard
    //     // if self.is_structural_hazard(state) {
    //     //     self.hazards_count += 1;
    //     //     return HazardType::StructuralHazard;
    //     // }
    //     //
    //     // HazardType::None
    // }

    pub fn detect_hazards(&mut self, state: &PipelineState) -> bool {
        let h = self.detect_hazards_with_type(state);
        h != HazardResult::None
    }

    /// Détecte les dépendances de données qui peuvent être résolues par forwarding
    fn is_data_dependency_forwardable(&self, state: &PipelineState) -> bool {
        let decode_reg = match &state.decode_execute {
            Some(reg) => reg,
            None => return false,
        };
        let (rs1, rs2) = (decode_reg.rs1, decode_reg.rs2);

        if rs1.is_none() && rs2.is_none() {
            return false;
        }

        // Check Execute stage (forwarding possible depuis EX/MEM)
        if let Some(ex_reg) = &state.execute_memory {
            if let Some(rd_ex) = ex_reg.rd {
                // Skip si c'est un Load (sera traité par is_load_use_hazards)
                let is_load = matches!(
                    ex_reg.instruction.opcode,
                    Opcode::Load | Opcode::LoadB | Opcode::LoadW | Opcode::LoadD | Opcode::Pop
                );
                
                if !is_load && (rs1 == Some(rd_ex) || rs2 == Some(rd_ex)) {
                    println!("Data dependency (forwardable from EX): decode needs R{}", rd_ex);
                    return true;
                }
            }
        }

        // Check Memory stage (forwarding possible depuis MEM/WB)
        if let Some(mem_reg) = &state.memory_writeback {
            if let Some(rd_mem) = mem_reg.rd {
                if rs1 == Some(rd_mem) || rs2 == Some(rd_mem) {
                    println!("Data dependency (forwardable from MEM): decode needs R{}", rd_mem);
                    return true;
                }
            }
        }

        false
    }

    /// Détecte les hazards de données qui NE peuvent PAS être résolus par forwarding
    fn is_data_hazard_not_forwardable(&self, state: &PipelineState) -> bool {
        // Dans notre architecture actuelle, la plupart des dépendances de données
        // peuvent être résolues par forwarding. Les seuls cas non-forwardables sont :
        // 1. Load-Use hazards (déjà traités séparément)
        // 2. Dépendances avec des instructions spéciales qui ne peuvent pas être forwardées
        // 3. Cas où le forwarding n'est pas implémenté (ex: certains cas store-load)
        
        // Pour l'instant, retourner false car tous les cas non-forwardables
        // sont déjà gérés par d'autres fonctions (load-use, store-load)
        false
    }

    /// Détecte les hazards de type Load-Use
    fn is_load_use_hazards(&self, state: &PipelineState) -> bool {
        // Si l'étage Decode n'a pas d'instruction, pas de hazard possible
        let decode_reg = match &state.decode_execute {
            Some(reg) => reg,
            None => return false,
        };

        // Registres sources de l'instruction dans l'étage Decode
        let rs1 = decode_reg.rs1;
        let rs2 = decode_reg.rs2;

        // Aucun registre source, pas de hazard possible
        if rs1.is_none() && rs2.is_none() {
            return false;
        }

        // Cas particulier pour les instructions mémoire (Load-Use Hazard)
        if let Some(ex_reg) = &state.execute_memory {
            // Si l'instruction dans Execute est un Load et que son registre destination est utilisé dans Decode
            let is_load = matches!(
            ex_reg.instruction.opcode,
            Opcode::Load | Opcode::LoadB | Opcode::LoadW | Opcode::LoadD
        );

            if is_load && ex_reg.rd.is_some() {
                let rd_ex = ex_reg.rd.unwrap();
                if rs1.map_or(false, |r| r == rd_ex) || rs2.map_or(false, |r| r == rd_ex) {
                    // Hazard Load-Use: on doit attendre que le Load finisse avant de lire
                    println!("Load-Use hazard detected: Decode stage needs register R{}, which is being loaded in Execute stage",
                             if rs1.map_or(false, |r| r == rd_ex) { rs1.unwrap() } else { rs2.unwrap() });
                    // return Some(true);
                    return true;
                }
            }
        }

        false
    }

    /// Détecte les hazards de type Store-Load (quand une écriture suivie d'une lecture à la même adresse)
    /// Vérifie store-load hazard (écriture en Execute, lecture en Decode, même adresse)
    fn is_store_load_hazard(&self, state: &PipelineState) -> bool {
        let ex_reg = match &state.execute_memory {
            Some(r) => r,
            None => return false,
        };
        let decode_reg = match &state.decode_execute {
            Some(r) => r,
            None => return false,
        };

        let exe_is_store = matches!(
        ex_reg.instruction.opcode,
        Opcode::Store | Opcode::StoreB | Opcode::StoreW | Opcode::StoreD
    );
        let dec_is_load = matches!(
        decode_reg.instruction.opcode,
        Opcode::Load | Opcode::LoadB | Opcode::LoadW | Opcode::LoadD
    );
        if exe_is_store && dec_is_load {
            if let (Some(addr_store), Some(addr_load)) = (ex_reg.mem_addr, decode_reg.mem_addr) {
                if addr_store == addr_load {
                    println!("Store-Load hazard :Store(EX) and Load(DE) on same address 0x{:X}", addr_store);
                    return true;
                }
            }
        }
        false
    }

    /// Détecte les hazards de contrôle (branchements)
    /// Vérifie control hazard

    // fn is_control_hazard(&mut self, state: &PipelineState) -> bool {
    //
    //     let ex_reg = match &state.execute_memory {
    //         Some(r) => r,
    //         None => {
    //             self.branch_stall_cycles = 0; // Réinitialiser le compteur
    //             return false;
    //         }
    //     };
    //
    //     if ex_reg.instruction.opcode.is_branch() {
    //         match ex_reg.instruction.opcode {
    //             Opcode::Jmp | Opcode::JmpIfEqual | Opcode::JmpIfNotEqual |
    //             Opcode::JmpIfGreater | Opcode::JmpIfLess |
    //             Opcode::JmpIfGreaterEqual | Opcode::JmpIfLessEqual |
    //             Opcode::JmpIfZero | Opcode::JmpIfNotZero => {
    //                 self.branch_stall_cycles += 1; // Incrémenter le compteur de cycles de stall
    //                 // On a un branchement en EX
    //                 println!(
    //                     "Control hazard : branch in execute stage (opcode: {:?})",
    //                     ex_reg.instruction.opcode
    //                 );
    //                 return true; // Indiquer qu'il y a un hazard de contrôle
    //             }
    //             _ => {
    //                 // Pas un branchement, pas de hazard
    //                 self.branch_stall_cycles = 0; // Réinitialiser le compteur
    //                 return false;
    //             }
    //         }
    //     } else {
    //         self.branch_stall_cycles = 0; // Réinitialiser le compteur
    //         return false;
    //     }
    // }



    fn is_control_hazard(&mut self, state: &PipelineState) -> bool {

        let ex_reg = match &state.execute_memory {
            Some(r) => r,
            None => {
                self.branch_stall_cycles = 0; // Réinitialiser le compteur
                return false;
            }
        };

        if ex_reg.instruction.opcode.is_branch() {
            if self.branch_stall_cycles == 0 {
                // Premier cycle avec cette instruction de branchement
                self.branch_stall_cycles += 1;
                println!(
                    "   [Hazard Check] Control hazard: Branch ({:?}) in Execute stage.",
                    ex_reg.instruction.opcode
                );
                return true;
            } else {
                // Cette instruction de branchement a déjà été détectée
                // self.branch_stall_cycles += 0;
                println!(
                    "Control hazard : branch in execute stage (stall cycle {})",
                    self.branch_stall_cycles
                );
                return false;
            }
        } else {
            self.branch_stall_cycles = 0; // Réinitialiser le compteur
            return false;
        }
    }

    /// Détecte les hazards structurels (conflits de ressources)
    /// Vérifie structural hazard
    /// (par ex. 2 instructions mem dans ex & mem)
    fn is_structural_hazard(&self, state: &PipelineState) -> bool {
        let (ex_stage, mem_stage) = (&state.execute_memory, &state.memory_writeback);
        if let (Some(ex_reg), Some(mem_reg)) = (ex_stage, mem_stage) {
            let ex_is_mem_op = matches!(
            ex_reg.instruction.opcode,
            Opcode::Load
                | Opcode::LoadB
                | Opcode::LoadW
                | Opcode::LoadD
                | Opcode::Store
                | Opcode::StoreB
                | Opcode::StoreW
                | Opcode::StoreD
        );
            let mem_is_mem_op = matches!(
            mem_reg.instruction.opcode,
            Opcode::Load
                | Opcode::LoadB
                | Opcode::LoadW
                | Opcode::LoadD
                | Opcode::Store
                | Opcode::StoreB
                | Opcode::StoreW
                | Opcode::StoreD
        );
            if ex_is_mem_op && mem_is_mem_op {
                println!("Structural hazard : mem ops in both EX & MEM");

                return true;
            }
        }
        false
    }

    /// Réinitialise l'unité de détection de hazards
    pub fn reset(&mut self) {
        println!("Resetting hazards count to 0.");
        self.hazards_count = 0;
        self.data_dependencies_count = 0;
        self.potential_forwards_count = 0;
        self.branch_stall_cycles = 0;
    }

    /// Retourne le nombre de hazards détectés
    pub fn get_hazards_count(&self) -> u64 {
        println!("True hazards (causing stalls): {}", self.hazards_count);
        println!("Data dependencies (forwardable): {}", self.data_dependencies_count);
        println!("Potential forwards detected: {}", self.potential_forwards_count);
        self.hazards_count
    }
    
    /// Retourne le nombre de dépendances de données détectées
    pub fn get_data_dependencies_count(&self) -> u64 {
        self.data_dependencies_count
    }
    
    /// Retourne le nombre de forwarding potentiels
    pub fn get_potential_forwards_count(&self) -> u64 {
        self.potential_forwards_count
    }
}





//
//
// ///////////////////////////////////////////////////////////////////////////////////////////////////////////////
// // Tests pour l'unité de détection de hazards
// #[cfg(test)]
// mod hazard_tests {
//     use super::*;
//     use crate::bytecode::instructions::Instruction;
//     use crate::bytecode::opcodes::Opcode;
//     use crate::pipeline::{
//         DecodeExecuteRegister, ExecuteMemoryRegister, MemoryWritebackRegister, PipelineState,
//     };
//
//     // Fonction utilitaire pour créer un état de pipeline de base
//     fn create_empty_pipeline_state() -> PipelineState {
//         PipelineState::default()
//     }
//
//     // Fonction utilitaire pour créer un registre decode-execute
//     fn create_decode_register(
//         opcode: Opcode,
//         rs1: Option<usize>,
//         rs2: Option<usize>,
//         rd: Option<usize>,
//         mem_addr: Option<u32>,
//     ) -> DecodeExecuteRegister {
//         DecodeExecuteRegister {
//             instruction: Instruction::create_reg_reg(
//                 opcode,
//                 rs1.unwrap_or(0) as u8,
//                 rs2.unwrap_or(0) as u8,
//             ),
//             pc: 0,
//             rs1,
//             rs2,
//             rd,
//             rs1_value: 0,
//             rs2_value: 0,
//             immediate: None,
//             branch_addr: None,
//             branch_prediction: None,
//             stack_operation: None,
//             mem_addr,
//             stack_value: None,
//         }
//     }
//
//     // Fonction utilitaire pour créer un registre execute-memory
//     fn create_execute_register(
//         opcode: Opcode,
//         rd: Option<usize>,
//         mem_addr: Option<u32>,
//         is_branch: bool,
//     ) -> ExecuteMemoryRegister {
//         ExecuteMemoryRegister {
//             instruction: Instruction::create_no_args(opcode),
//             alu_result: 42,
//             rd,
//             store_value: if opcode == Opcode::Store {
//                 Some(100)
//             } else {
//                 None
//             },
//             mem_addr,
//             branch_target: if is_branch { Some(0x1000) } else { None },
//             branch_taken: false,
//             branch_prediction_correct: Option::from(false),
//             stack_operation: None,
//             stack_result: None,
//             ras_prediction_correct: None,
//             halted: false,
//         }
//     }
//
//     // Fonction utilitaire pour créer un registre memory-writeback
//     fn create_memory_register(opcode: Opcode, rd: Option<usize>) -> MemoryWritebackRegister {
//         MemoryWritebackRegister {
//             instruction: Instruction::create_no_args(opcode),
//             result: 999,
//             rd,
//         }
//     }
//
//     #[test]
//     fn test_hazard_unit_creation() {
//         let unit = HazardDetectionUnit::new();
//         assert_eq!(
//             unit.get_hazards_count(),
//             0,
//             "Nouvelle unité devrait commencer avec 0 hazards"
//         );
//     }
//
//     #[test]
//     fn test_hazard_unit_reset() {
//         let mut unit = HazardDetectionUnit::new();
//         let mut state = create_empty_pipeline_state();
//
//         // Créer un data hazard simple: Decode utilise R1, Execute écrit R1
//         state.decode_execute = Some(create_decode_register(
//             Opcode::Add,
//             Some(1),
//             Some(2),
//             Some(3),
//             None,
//         ));
//         state.execute_memory = Some(create_execute_register(Opcode::Add, Some(1), None, false));
//
//         // Vérifier que le hazard est détecté et le compteur incrémenté
//         assert!(unit.detect_hazards(&state));
//         assert_eq!(unit.get_hazards_count(), 1);
//
//         unit.reset();
//         assert_eq!(
//             unit.get_hazards_count(),
//             0,
//             "Après reset, le compteur devrait être à 0"
//         );
//     }
//
//     #[test]
//     fn test_data_hazard_detection() {
//         let mut unit = HazardDetectionUnit::new();
//         let mut state = create_empty_pipeline_state();
//
//         // Data hazard: Decode utilise R1, Execute écrit R1
//         state.decode_execute = Some(create_decode_register(
//             Opcode::Add,
//             Some(1),
//             Some(2),
//             Some(3),
//             None,
//         ));
//         state.execute_memory = Some(create_execute_register(Opcode::Add, Some(1), None, false));
//
//         let hazard_type = unit.detect_hazards_with_type(&state);
//         assert_eq!(hazard_type, HazardResult::DataDependency);
//         assert_eq!(unit.get_hazards_count(), 1);
//
//         // Variante: Decode utilise R2, Execute écrit R2
//         state.decode_execute = Some(create_decode_register(
//             Opcode::Add,
//             Some(3),
//             Some(2),
//             Some(4),
//             None,
//         ));
//         state.execute_memory = Some(create_execute_register(Opcode::Add, Some(2), None, false));
//
//         let hazard_type = unit.detect_hazards_with_type(&state);
//         assert_eq!(hazard_type, HazardResult::DataDependency);
//         assert_eq!(unit.get_hazards_count(), 2);
//
//         // Data hazard avec Memory: Decode utilise R5, Memory écrit R5
//         state.decode_execute = Some(create_decode_register(
//             Opcode::Sub,
//             Some(5),
//             Some(6),
//             Some(7),
//             None,
//         ));
//         state.execute_memory = Some(create_execute_register(Opcode::Add, Some(8), None, false));
//         state.memory_writeback = Some(create_memory_register(Opcode::Add, Some(5)));
//
//         let hazard_type = unit.detect_hazards_with_type(&state);
//         assert_eq!(hazard_type, HazardResult::DataDependency);
//         assert_eq!(unit.get_hazards_count(), 3);
//     }
//
//     #[test]
//     fn test_no_hazard_detected() {
//         let mut unit = HazardDetectionUnit::new();
//         let mut state = create_empty_pipeline_state();
//
//         // Aucun hazard: registres différents
//         state.decode_execute = Some(create_decode_register(
//             Opcode::Add,
//             Some(1),
//             Some(2),
//             Some(3),
//             None,
//         ));
//         state.execute_memory = Some(create_execute_register(Opcode::Add, Some(4), None, false));
//         state.memory_writeback = Some(create_memory_register(Opcode::Add, Some(5)));
//
//         let hazard_type = unit.detect_hazards_with_type(&state);
//         assert_eq!(hazard_type, HazardResult::None);
//         assert_eq!(unit.get_hazards_count(), 0);
//     }
//
//     #[test]
//     fn test_load_use_hazard_detection() {
//         let mut unit = HazardDetectionUnit::new();
//         let mut state = create_empty_pipeline_state();
//
//         // Load-Use hazard: Load R1 en Execute, utilisation de R1 en Decode
//         state.decode_execute = Some(create_decode_register(
//             Opcode::Add,
//             Some(1),
//             Some(2),
//             Some(3),
//             None,
//         ));
//         state.execute_memory = Some(create_execute_register(
//             Opcode::Load,
//             Some(1),
//             Some(0x100),
//             false,
//         ));
//
//         let hazard_type = unit.detect_hazards_with_type(&state);
//         // assert_eq!(hazard_type, HazardResult::LoadUse);
//         assert_eq!(hazard_type, HazardResult::DataDependency);
//         assert_eq!(unit.get_hazards_count(), 1);
//
//         // unit.reset();
//
//         // Cas avec LoadB (different opcode, même principe)
//         state.execute_memory = Some(create_execute_register(
//             Opcode::LoadB,
//             Some(1),
//             Some(0x100),
//             false,
//         ));
//
//         let hazard_type = unit.detect_hazards_with_type(&state);
//         // assert_eq!(hazard_type, HazardResult::LoadUse);
//         assert_eq!(hazard_type, HazardResult::DataDependency);
//         assert_eq!(unit.get_hazards_count(), 2);
//
//         // unit.reset();
//
//         // Cas où R2 est chargé et utilisé
//         state.decode_execute = Some(create_decode_register(
//             Opcode::Add,
//             Some(3),
//             Some(2),
//             Some(4),
//             None,
//         ));
//         state.execute_memory = Some(create_execute_register(
//             Opcode::LoadW,
//             Some(2),
//             Some(0x200),
//             false,
//         ));
//
//         let hazard_type = unit.detect_hazards_with_type(&state);
//         // assert_eq!(hazard_type, HazardResult::LoadUse);
//         assert_eq!(hazard_type, HazardResult::DataDependency);
//         assert_eq!(unit.get_hazards_count(), 3);
//     }
//
//     #[test]
//     fn test_store_load_hazard_detection() {
//         let mut unit = HazardDetectionUnit::new();
//         let mut state = create_empty_pipeline_state();
//
//         // Store-Load hazard: Store à l'adresse 0x100 en Execute, Load de la même adresse en Decode
//         state.decode_execute = Some(create_decode_register(
//             Opcode::Load,
//             None,
//             None,
//             Some(1),
//             Some(0x100),
//         ));
//         state.execute_memory = Some(create_execute_register(
//             Opcode::Store,
//             None,
//             Some(0x100),
//             false,
//         ));
//
//         let hazard_type = unit.detect_hazards_with_type(&state);
//         assert_eq!(hazard_type, HazardResult::StoreLoad);
//         assert_eq!(unit.get_hazards_count(), 1);
//
//         // Cas avec StoreB et LoadB (différents opcodes, même principe)
//         state.decode_execute = Some(create_decode_register(
//             Opcode::LoadB,
//             None,
//             None,
//             Some(1),
//             Some(0x200),
//         ));
//         state.execute_memory = Some(create_execute_register(
//             Opcode::StoreB,
//             None,
//             Some(0x200),
//             false,
//         ));
//
//         let hazard_type = unit.detect_hazards_with_type(&state);
//         assert_eq!(hazard_type, HazardResult::StoreLoad);
//         assert_eq!(unit.get_hazards_count(), 2);
//
//         // Cas où les adresses sont différentes (pas de hazard)
//         state.decode_execute = Some(create_decode_register(
//             Opcode::Load,
//             None,
//             None,
//             Some(1),
//             Some(0x300),
//         ));
//         state.execute_memory = Some(create_execute_register(
//             Opcode::Store,
//             None,
//             Some(0x400),
//             false,
//         ));
//
//         let hazard_type = unit.detect_hazards_with_type(&state);
//         assert_ne!(hazard_type, HazardResult::StoreLoad);
//     }
//
//     #[test]
//     fn test_control_hazard_detection() {
//         let mut unit = HazardDetectionUnit::new();
//         let mut state = create_empty_pipeline_state();
//
//         // Control hazard: Branchement en Execute
//         state.execute_memory = Some(create_execute_register(Opcode::Jmp, None, None, true));
//
//         let hazard_type = unit.detect_hazards_with_type(&state);
//         assert_eq!(hazard_type, HazardResult::ControlHazard);
//         assert_eq!(unit.get_hazards_count(), 1);
//
//         // Autres types de branchements
//         state.execute_memory = Some(create_execute_register(Opcode::JmpIf, None, None, true));
//
//         let hazard_type = unit.detect_hazards_with_type(&state);
//         assert_eq!(hazard_type, HazardResult::ControlHazard);
//         assert_eq!(unit.get_hazards_count(), 2);
//
//         state.execute_memory = Some(create_execute_register(Opcode::Call, None, None, true));
//
//         let hazard_type = unit.detect_hazards_with_type(&state);
//         assert_eq!(hazard_type, HazardResult::ControlHazard);
//         assert_eq!(unit.get_hazards_count(), 3);
//     }
//
//     #[test]
//     fn test_structural_hazard_detection() {
//         let mut unit = HazardDetectionUnit::new();
//         let mut state = create_empty_pipeline_state();
//
//         // Structural hazard: Instructions mémoire à la fois en Execute et en Memory
//         state.execute_memory = Some(create_execute_register(
//             Opcode::Load,
//             Some(1),
//             Some(0x100),
//             false,
//         ));
//         state.memory_writeback = Some(create_memory_register(Opcode::Store, None));
//
//         let hazard_type = unit.detect_hazards_with_type(&state);
//         // assert_eq!(hazard_type, HazardResult::StructuralHazard);
//         assert_eq!(unit.get_hazards_count(), 1);
//
//         // Autres combinaisons d'instructions mémoire
//         state.execute_memory = Some(create_execute_register(
//             Opcode::Store,
//             None,
//             Some(0x200),
//             false,
//         ));
//         state.memory_writeback = Some(create_memory_register(Opcode::Load, Some(2)));
//
//         let hazard_type = unit.detect_hazards_with_type(&state);
//         // assert_eq!(hazard_type, HazardResult::StructuralHazard);
//         assert_eq!(unit.get_hazards_count(), 0);
//
//         // Cas sans hazard structurel: une seule instruction mémoire
//         state.execute_memory = Some(create_execute_register(
//             Opcode::Load,
//             Some(1),
//             Some(0x100),
//             false,
//         ));
//         state.memory_writeback = Some(create_memory_register(Opcode::Add, Some(3)));
//
//         let hazard_type = unit.detect_hazards_with_type(&state);
//         assert_ne!(hazard_type, HazardResult::StructuralHazard);
//         assert_eq!(unit.get_hazards_count(), 0);
//     }
//
//     #[test]
//     fn test_hazard_priority() {
//         let mut unit = HazardDetectionUnit::new();
//         let mut state = create_empty_pipeline_state();
//
//         // Configurer un état avec plusieurs hazards potentiels
//         // Data hazard et Load-Use hazard (les deux en même temps)
//         state.decode_execute = Some(create_decode_register(
//             Opcode::Add,
//             Some(1),
//             Some(2),
//             Some(3),
//             None,
//         ));
//         state.execute_memory = Some(create_execute_register(
//             Opcode::Load,
//             Some(1),
//             Some(0x100),
//             false,
//         ));
//
//         // Notre implémentation test d'abord les Data hazards, puis les Load-Use hazards
//         // Vérifier l'ordre de détection selon la priorité définie dans detect_hazards_with_type
//         let hazard_type = unit.detect_hazards_with_type(&state);
//
//         // Le premier hazard détecté dans notre ordre devrait être DataDependency
//         // assert_eq!(hazard_type,HazardResult::LoadUse);
//         assert_eq!(hazard_type, HazardResult::DataDependency);
//
//         // Mais avec detect_hazard (autre méthode), l'ordre pourrait être différent
//         // (selon votre implémentation)
//         // let hazard_type2 = unit.detect_hazard(&state);
//         // assert_eq!(hazard_type2, HazardType::LoadUse); // ou DataDependency selon votre ordre
//     }
//     #[test]
//     fn test_hazard_priority_2() {
//         let mut unit = HazardDetectionUnit::new();
//         let mut state = create_empty_pipeline_state();
//
//         // Configurer un état avec plusieurs hazards:
//         // - Load R1 in EX (Load-Use source)
//         // - ADD R3, R1, R2 in DE (Load-Use et Data Dependency sur R2 si MEM écrit R2)
//         // - SUB R2, ... in MEM (Data Dependency source)
//         state.decode_execute = Some(create_decode_register(Opcode::Add, Some(1), Some(2), Some(3), None));
//         state.execute_memory = Some(create_execute_register(Opcode::Load, Some(1), Some(0x100), false)); // Load R1
//         state.memory_writeback = Some(create_memory_register(Opcode::Sub, Some(2))); // SUB R2,...
//
//         // Ordre de priorité dans detect_hazards_with_type: LoadUse > Control > Structural > Data > StoreLoad
//         let hazard_type = unit.detect_hazards_with_type(&state);
//
//         // Le premier hazard détecté doit être LoadUse car il est prioritaire sur DataDependency.
//         assert_eq!(hazard_type, HazardResult::LoadUse, "Priority check: LoadUse should be detected first"); // CORRIGÉ
//         assert_eq!(unit.hazards_count, 1);
//     }
//
//     #[test]
//     fn test_partial_pipeline_state() {
//         let mut unit = HazardDetectionUnit::new();
//         let mut state = create_empty_pipeline_state();
//
//         // Test avec un pipeline partiellement vide (certains stages à None)
//         state.decode_execute = Some(create_decode_register(
//             Opcode::Add,
//             Some(1),
//             Some(2),
//             Some(3),
//             None,
//         ));
//         state.execute_memory = None;
//         state.memory_writeback = Some(create_memory_register(Opcode::Add, Some(1)));
//
//         // Vérifie que l'unité gère correctement les étages vides
//         let hazard_type = unit.detect_hazards_with_type(&state);
//         assert_eq!(hazard_type, HazardResult::DataDependency);
//         assert_eq!(unit.hazards_count, 1);
//     }
//
//
//     #[test]
//     fn test_edge_cases() {
//         let mut unit = HazardDetectionUnit::new();
//         let mut state = create_empty_pipeline_state();
//
//         // 1. Cas où Decode n'utilise pas de registres sources
//         state.decode_execute = Some(create_decode_register(Opcode::Jmp, None, None, None, None));
//         state.execute_memory = Some(create_execute_register(Opcode::Add, Some(1), None, false));
//
//         let hazard_type = unit.detect_hazards_with_type(&state);
//         assert_eq!(
//             hazard_type,
//             HazardResult::None,
//             "Aucun hazard si Decode n'utilise pas de registres"
//         );
//
//         // 2. Cas où Execute n'écrit pas dans un registre
//         state.decode_execute = Some(create_decode_register(
//             Opcode::Add,
//             Some(1),
//             Some(2),
//             Some(3),
//             None,
//         ));
//         state.execute_memory = Some(create_execute_register(Opcode::Jmp, None, None, true));
//
//         let hazard_type = unit.detect_hazards_with_type(&state);
//         assert_eq!(
//             hazard_type,
//             HazardResult::ControlHazard,
//             "Control hazard détecté même sans écriture de registre"
//         );
//
//         // 3. Cas où il y a des hazards potentiels mais le pipeline est vide
//         state = create_empty_pipeline_state();
//
//         let hazard_type = unit.detect_hazards_with_type(&state);
//         assert_eq!(
//             hazard_type,
//             HazardResult::None,
//             "Aucun hazard si le pipeline est vide"
//         );
//     }
// }

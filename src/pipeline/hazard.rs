//src/pipeline/hazard.rs

use crate::bytecode::opcodes::Opcode;
use crate::pipeline::PipelineState;

/// Unité de détection de hazards
pub struct HazardDetectionUnit {
    // Compteur de hazards détectés
    pub hazards_count: u64,
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
            branch_stall_cycles: 0,
        }
    }

    /// Détecte les hazards dans le pipeline et retourne le type détecté
    pub fn detect_hazards_with_type(&mut self, state: &PipelineState) -> HazardResult {
        // 1. Data Hazards (RAW - Read After Write)
        if self.is_data_hazard(state) {
            println!("Data hazard detected");
            self.hazards_count += 1;
            return HazardResult::DataDependency;
        }

        // 2. Load-Use Hazards (cas spécial de Data Hazard)
        if self.is_load_use_hazards(state) {
            println!("Load-Use hazard detected");
            self.hazards_count += 1;
            return HazardResult::LoadUse;
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

    /// Détecte les hazards de données (RAW - Read After Write)
    // Vérifie un hazard data "classique" (RAW)
    fn is_data_hazard(&self, state: &PipelineState) -> bool {
        let decode_reg = match &state.decode_execute {
            Some(reg) => reg,

            None => return false,
        };
        let (rs1, rs2) = (decode_reg.rs1, decode_reg.rs2);

        if rs1.is_none() && rs2.is_none() {
            return false;
        }

        // Check Execute stage
        if let Some(ex_reg) = &state.execute_memory {
            if let Some(rd_ex) = ex_reg.rd {
                // Si decode a besoin du rd_ex de execute
                if rs1 == Some(rd_ex) || rs2 == Some(rd_ex) {
                    println!("Data hazard (RAW) : decode needs R{}", rd_ex);

                    return true;
                }
            }
        }

        // Check Memory stage
        if let Some(mem_reg) = &state.memory_writeback {
            if let Some(rd_mem) = mem_reg.rd {
                if rs1 == Some(rd_mem) || rs2 == Some(rd_mem) {
                    println!(
                        "Data hazard (RAW) : decode needs R{} (written in Memory stage)",
                        rd_mem
                    );

                    return true;
                }
            }
        }

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
        self.branch_stall_cycles = 0;
    }

    /// Retourne le nombre de hazards détectés
    pub fn get_hazards_count(&self) -> u64 {
        // println!("Hazards count: {}", self.hazards_count);
        println!("Total hazards detected (may include forwarded): {}", self.hazards_count);
        self.hazards_count
    }

}


/////////////////////////////////////////////
// // src/pipeline/hazard.rs
//
// use crate::bytecode::opcodes::Opcode;
// use crate::pipeline::PipelineState;
//
// /// Unité de détection de hazards
// #[derive(Debug)] // Ajout de Debug pour l'affichage
// pub struct HazardDetectionUnit {
//     /// Compteur de hazards détectés (peut indiquer un stall ou juste une dépendance)
//     pub hazards_count: u64,
//     // Retrait de branch_stall_cycles, la logique est simplifiée
// }
//
// // Note: HazardType est conservé tel quel, car vous l'utilisiez.
// #[derive(Debug, PartialEq)]
// pub enum HazardType {
//     None,
//     LoadUse,
//     StoreLoad,
//     DataDependency,
//     ControlHazard,
//     StructuralHazard,
// }
//
// // HazardResult est conservé tel quel
// #[derive(Debug, Clone, Copy, PartialEq)]
// pub enum HazardResult {
//     None,
//     StoreLoad,
//     LoadUse,
//     DataDependency,
//     ControlHazard,
//     StructuralHazard,
// }
//
// impl HazardDetectionUnit {
//     /// Crée une nouvelle unité de détection de hazards
//     pub fn new() -> Self {
//         Self {
//             hazards_count: 0,
//             // branch_stall_cycles: 0, // Retiré
//         }
//     }
//
//     /// Détecte les hazards dans le pipeline et retourne le type détecté.
//     /// L'ordre des vérifications définit la priorité si plusieurs hazards sont présents.
//     pub fn detect_hazards_with_type(&mut self, state: &PipelineState) -> HazardResult {
//         // Priorité 1: Load-Use Hazard (nécessite quasi toujours un stall)
//         if self.is_load_use_hazards(state) {
//             println!("Hazard Detected: Load-Use");
//             self.hazards_count += 1; // Incrémenter lors de la détection
//             return HazardResult::LoadUse; // Le contrôleur décidera du stall
//
//         }
//
//         // Priorité 2: Control Hazard (branchement non résolu dans EX)
//         // Si un branchement est en EX, on a un hazard potentiel jusqu'à sa résolution.
//         if self.is_control_hazard(state) {
//             println!("Hazard Detected: Control (Branch in EX stage)");
//             self.hazards_count += 1; // Incrémenter lors de la détection
//             return HazardResult::ControlHazard; // Le contrôleur gérera (stall/flush si mispredict)
//         }
//
//         // Priorité 3: Structural Hazard (conflit de ressource)
//         if self.is_structural_hazard(state) {
//             println!("Hazard Detected: Structural");
//             self.hazards_count += 1; // Incrémenter lors de la détection
//             return HazardResult::StructuralHazard; // Nécessite un stall
//         }
//
//         // Priorité 4: Data Hazard (RAW classique - peut être résolu par forwarding)
//         // On le détecte après les stalls obligatoires car le forwarding pourrait le masquer.
//         if self.is_data_hazard(state) {
//             println!("Hazard Detected: Data Dependency (RAW)");
//             // Note: On incrémente hazards_count ici, même si le forwarding peut le résoudre.
//             // C'est discutable, on pourrait vouloir ne compter que les hazards *non résolus*.
//             // Pour l'instant, on suit la logique précédente: compter à la détection.
//             self.hazards_count += 1;
//             return HazardResult::DataDependency;
//         }
//
//         // Priorité 5: Store-Load Hazard (peut être résolu par forwarding mémoire/SB)
//         if self.is_store_load_hazard(state) {
//             println!("Hazard Detected: Store-Load");
//             self.hazards_count += 1; // Compter à la détection
//             return HazardResult::StoreLoad;
//         }
//
//         // Aucun hazard détecté
//         HazardResult::None
//     }
//
//     /// Détecte s'il y a *un* hazard (quel qu'il soit).
//     /// Fonction conservée pour la compatibilité.
//     pub fn detect_hazards(&mut self, state: &PipelineState) -> bool {
//         // On réutilise detect_hazards_with_type pour la logique
//         let hazard_type = self.detect_hazards_with_type(state);
//         // On retourne true si un hazard (autre que None) a été détecté.
//         hazard_type != HazardResult::None
//     }
//
//     /// Détecte les hazards de données (RAW - Read After Write).
//     /// Vérifie si l'instruction en Decode lit un registre écrit par Execute ou Memory.
//     fn is_data_hazard(&self, state: &PipelineState) -> bool {
//         let decode_reg = match &state.decode_execute {
//             Some(reg) => reg,
//             None => return false, // Pas d'instruction en Decode
//         };
//         let (rs1, rs2) = (decode_reg.rs1, decode_reg.rs2);
//
//         // Si pas de registres source, pas de dépendance
//         if rs1.is_none() && rs2.is_none() {
//             return false;
//         }
//
//         // Vérifier dépendance avec Execute (EX/MEM)
//         if let Some(ex_reg) = &state.execute_memory {
//             if let Some(rd_ex) = ex_reg.rd {
//                 // Si l'instruction en EX est un Load, ce n'est pas un hazard RAW classique
//                 // géré par forwarding ALU, c'est un Load-Use (vérifié ailleurs).
//                 let is_load_in_ex = matches!(
//                     ex_reg.instruction.opcode,
//                     Opcode::Load | Opcode::LoadB | Opcode::LoadW | Opcode::LoadD | Opcode::Pop
//                 );
//                 if !is_load_in_ex { // Ne vérifier que pour les non-loads
//                     if rs1 == Some(rd_ex) || rs2 == Some(rd_ex) {
//                         println!("   [Hazard Check] Data hazard (RAW): Decode needs R{} written by non-Load in Execute", rd_ex);
//                         return true;
//                     }
//                 }
//             }
//         }
//
//         // Vérifier dépendance avec Memory (MEM/WB)
//         if let Some(mem_reg) = &state.memory_writeback {
//             if let Some(rd_mem) = mem_reg.rd {
//                 // Éviter double détection si EX écrit aussi rd_mem (priorité à EX)
//                 if let Some(ex_reg) = &state.execute_memory {
//                     if ex_reg.rd == Some(rd_mem) {
//                         return false; // Conflit déjà géré/détecté avec EX
//                     }
//                 }
//                 // Si conflit avec MEM/WB
//                 if rs1 == Some(rd_mem) || rs2 == Some(rd_mem) {
//                     println!("   [Hazard Check] Data hazard (RAW): Decode needs R{} written by Memory stage", rd_mem);
//                     return true;
//                 }
//             }
//         }
//
//         false
//     }
//
//     /// Détecte les hazards de type Load-Use.
//     /// Vérifie si Decode lit le résultat d'un Load en Execute.
//     fn is_load_use_hazards(&self, state: &PipelineState) -> bool {
//         if let Some(ex_reg) = &state.execute_memory {
//             let is_load_in_ex = matches!(
//                 ex_reg.instruction.opcode,
//                 Opcode::Load | Opcode::LoadB | Opcode::LoadW | Opcode::LoadD | Opcode::Pop
//             );
//
//             if is_load_in_ex {
//                 if let Some(rd_load) = ex_reg.rd {
//                     // Vérifier si Decode utilise rd_load
//                     if let Some(decode_reg) = &state.decode_execute {
//                         if decode_reg.rs1 == Some(rd_load) || decode_reg.rs2 == Some(rd_load) {
//                             println!(
//                                 "   [Hazard Check] Load-Use hazard: Decode reads R{} from Load/Pop in Execute.",
//                                 rd_load
//                             );
//                             return true; // Stall nécessaire
//                         }
//                     }
//                 }
//             }
//         }
//         false
//     }
//
//     /// Détecte les hazards de type Store-Load.
//     /// Vérifie si Decode (Load) lit une adresse écrite par Execute (Store).
//     fn is_store_load_hazard(&self, state: &PipelineState) -> bool {
//         let ex_reg = match &state.execute_memory {
//             Some(r) => r,
//             None => return false,
//         };
//         let decode_reg = match &state.decode_execute {
//             Some(r) => r,
//             None => return false,
//         };
//
//         let exe_is_store = matches!(
//             ex_reg.instruction.opcode,
//             Opcode::Store | Opcode::StoreB | Opcode::StoreW | Opcode::StoreD | Opcode::Push
//         );
//         let dec_is_load = matches!(
//             decode_reg.instruction.opcode,
//             Opcode::Load | Opcode::LoadB | Opcode::LoadW | Opcode::LoadD | Opcode::Pop
//         );
//
//         if exe_is_store && dec_is_load {
//             // Comparaison d'adresse possible seulement pour Store/Load explicites
//             // car l'adresse pour Push/Pop n'est calculée qu'en MEM.
//             if ex_reg.mem_addr.is_some() && decode_reg.mem_addr.is_some() {
//                 if ex_reg.mem_addr == decode_reg.mem_addr {
//                     println!(
//                         "   [Hazard Check] Store-Load hazard: Store(EX) and Load(DE) on same address 0x{:X}",
//                         ex_reg.mem_addr.unwrap()
//                     );
//                     return true; // Dépendance détectée
//                 }
//             }
//             // On pourrait considérer une dépendance si les adresses sont inconnues,
//             // mais cela peut être trop pessimiste.
//         }
//         false
//     }
//
//     /// Détecte les hazards de contrôle (branchement en cours d'exécution).
//     /// Simplifié : retourne true si une instruction de branchement est dans EX/MEM.
//     fn is_control_hazard(&mut self, state: &PipelineState) -> bool {
//         // Le hazard principal survient quand une instruction de branchement est
//         // dans l'étage Execute (registre EX/MEM) car son issue (taken/not taken)
//         // et sa cible ne sont déterminées qu'à la fin de cet étage.
//         // Le Fetch a pu continuer (spéculativement ou non) sans connaître le bon chemin.
//         if let Some(ex_reg) = &state.execute_memory {
//             if ex_reg.instruction.opcode.is_branch() {
//                 println!(
//                     "   [Hazard Check] Control hazard: Branch ({:?}) in Execute stage.",
//                     ex_reg.instruction.opcode
//                 );
//                 return true; // Signalement du hazard de contrôle potentiel
//             }
//         }
//         false
//     }
//
//     /// Détecte les hazards structurels.
//     /// Dans ce pipeline, on suppose que seule l'unité Memory est une ressource critique unique.
//     /// Le conflit survient si EX et MEM tentent d'accéder simultanément (ce qui n'arrive pas ici).
//     fn is_structural_hazard(&self, state: &PipelineState) -> bool {
//         // Voir les commentaires dans la version précédente. Dans ce design de pipeline
//         // où l'accès mémoire est strictement dans l'étage MEM, il n'y a pas de
//         // conflit structurel entre EX et MEM sur le port mémoire.
//         // La fonction est conservée pour la structure, mais retourne false.
//         false
//     }
//
//     /// Réinitialise l'unité de détection de hazards
//     pub fn reset(&mut self) {
//         println!("Resetting hazards count to 0.");
//         self.hazards_count = 0;
//     }
//
//     /// Retourne le nombre de hazards détectés.
//     /// Note: Ce nombre reflète les détections, pas nécessairement les stalls finaux.
//     pub fn get_hazards_count(&self) -> u64 {
//         println!("Total hazards detected (may include forwarded): {}", self.hazards_count);
//         self.hazards_count
//     }
// }


///////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Tests pour l'unité de détection de hazards
#[cfg(test)]
mod hazard_tests {
    use super::*;
    use crate::bytecode::instructions::Instruction;
    use crate::bytecode::opcodes::Opcode;
    use crate::pipeline::{
        DecodeExecuteRegister, ExecuteMemoryRegister, MemoryWritebackRegister, PipelineState,
    };

    // Fonction utilitaire pour créer un état de pipeline de base
    fn create_empty_pipeline_state() -> PipelineState {
        PipelineState::default()
    }

    // Fonction utilitaire pour créer un registre decode-execute
    fn create_decode_register(
        opcode: Opcode,
        rs1: Option<usize>,
        rs2: Option<usize>,
        rd: Option<usize>,
        mem_addr: Option<u32>,
    ) -> DecodeExecuteRegister {
        DecodeExecuteRegister {
            instruction: Instruction::create_reg_reg(
                opcode,
                rs1.unwrap_or(0) as u8,
                rs2.unwrap_or(0) as u8,
            ),
            pc: 0,
            rs1,
            rs2,
            rd,
            rs1_value: 0,
            rs2_value: 0,
            immediate: None,
            branch_addr: None,
            branch_prediction: None,
            mem_addr,
        }
    }

    // Fonction utilitaire pour créer un registre execute-memory
    fn create_execute_register(
        opcode: Opcode,
        rd: Option<usize>,
        mem_addr: Option<u32>,
        is_branch: bool,
    ) -> ExecuteMemoryRegister {
        ExecuteMemoryRegister {
            instruction: Instruction::create_no_args(opcode),
            alu_result: 42,
            rd,
            store_value: if opcode == Opcode::Store {
                Some(100)
            } else {
                None
            },
            mem_addr,
            branch_target: if is_branch { Some(0x1000) } else { None },
            branch_taken: false,
            branch_prediction_correct: Option::from(false),
            halted: false,
        }
    }

    // Fonction utilitaire pour créer un registre memory-writeback
    fn create_memory_register(opcode: Opcode, rd: Option<usize>) -> MemoryWritebackRegister {
        MemoryWritebackRegister {
            instruction: Instruction::create_no_args(opcode),
            result: 999,
            rd,
        }
    }

    #[test]
    fn test_hazard_unit_creation() {
        let unit = HazardDetectionUnit::new();
        assert_eq!(
            unit.get_hazards_count(),
            0,
            "Nouvelle unité devrait commencer avec 0 hazards"
        );
    }

    #[test]
    fn test_hazard_unit_reset() {
        let mut unit = HazardDetectionUnit::new();
        let mut state = create_empty_pipeline_state();

        // Créer un data hazard simple: Decode utilise R1, Execute écrit R1
        state.decode_execute = Some(create_decode_register(
            Opcode::Add,
            Some(1),
            Some(2),
            Some(3),
            None,
        ));
        state.execute_memory = Some(create_execute_register(Opcode::Add, Some(1), None, false));

        // Vérifier que le hazard est détecté et le compteur incrémenté
        assert!(unit.detect_hazards(&state));
        assert_eq!(unit.get_hazards_count(), 1);

        unit.reset();
        assert_eq!(
            unit.get_hazards_count(),
            0,
            "Après reset, le compteur devrait être à 0"
        );
    }

    #[test]
    fn test_data_hazard_detection() {
        let mut unit = HazardDetectionUnit::new();
        let mut state = create_empty_pipeline_state();

        // Data hazard: Decode utilise R1, Execute écrit R1
        state.decode_execute = Some(create_decode_register(
            Opcode::Add,
            Some(1),
            Some(2),
            Some(3),
            None,
        ));
        state.execute_memory = Some(create_execute_register(Opcode::Add, Some(1), None, false));

        let hazard_type = unit.detect_hazards_with_type(&state);
        assert_eq!(hazard_type, HazardResult::DataDependency);
        assert_eq!(unit.get_hazards_count(), 1);

        // Variante: Decode utilise R2, Execute écrit R2
        state.decode_execute = Some(create_decode_register(
            Opcode::Add,
            Some(3),
            Some(2),
            Some(4),
            None,
        ));
        state.execute_memory = Some(create_execute_register(Opcode::Add, Some(2), None, false));

        let hazard_type = unit.detect_hazards_with_type(&state);
        assert_eq!(hazard_type, HazardResult::DataDependency);
        assert_eq!(unit.get_hazards_count(), 2);

        // Data hazard avec Memory: Decode utilise R5, Memory écrit R5
        state.decode_execute = Some(create_decode_register(
            Opcode::Sub,
            Some(5),
            Some(6),
            Some(7),
            None,
        ));
        state.execute_memory = Some(create_execute_register(Opcode::Add, Some(8), None, false));
        state.memory_writeback = Some(create_memory_register(Opcode::Add, Some(5)));

        let hazard_type = unit.detect_hazards_with_type(&state);
        assert_eq!(hazard_type, HazardResult::DataDependency);
        assert_eq!(unit.get_hazards_count(), 3);
    }

    #[test]
    fn test_no_hazard_detected() {
        let mut unit = HazardDetectionUnit::new();
        let mut state = create_empty_pipeline_state();

        // Aucun hazard: registres différents
        state.decode_execute = Some(create_decode_register(
            Opcode::Add,
            Some(1),
            Some(2),
            Some(3),
            None,
        ));
        state.execute_memory = Some(create_execute_register(Opcode::Add, Some(4), None, false));
        state.memory_writeback = Some(create_memory_register(Opcode::Add, Some(5)));

        let hazard_type = unit.detect_hazards_with_type(&state);
        assert_eq!(hazard_type, HazardResult::None);
        assert_eq!(unit.get_hazards_count(), 0);
    }

    #[test]
    fn test_load_use_hazard_detection() {
        let mut unit = HazardDetectionUnit::new();
        let mut state = create_empty_pipeline_state();

        // Load-Use hazard: Load R1 en Execute, utilisation de R1 en Decode
        state.decode_execute = Some(create_decode_register(
            Opcode::Add,
            Some(1),
            Some(2),
            Some(3),
            None,
        ));
        state.execute_memory = Some(create_execute_register(
            Opcode::Load,
            Some(1),
            Some(0x100),
            false,
        ));

        let hazard_type = unit.detect_hazards_with_type(&state);
        assert_eq!(hazard_type, HazardResult::LoadUse);
        // assert_eq!(hazard_type, HazardResult::DataDependency);
        assert_eq!(unit.get_hazards_count(), 1);

        // unit.reset();

        // Cas avec LoadB (different opcode, même principe)
        state.execute_memory = Some(create_execute_register(
            Opcode::LoadB,
            Some(1),
            Some(0x100),
            false,
        ));

        let hazard_type = unit.detect_hazards_with_type(&state);
        assert_eq!(hazard_type, HazardResult::LoadUse);
        // assert_eq!(hazard_type, HazardResult::DataDependency);
        assert_eq!(unit.get_hazards_count(), 2);

        // unit.reset();

        // Cas où R2 est chargé et utilisé
        state.decode_execute = Some(create_decode_register(
            Opcode::Add,
            Some(3),
            Some(2),
            Some(4),
            None,
        ));
        state.execute_memory = Some(create_execute_register(
            Opcode::LoadW,
            Some(2),
            Some(0x200),
            false,
        ));

        let hazard_type = unit.detect_hazards_with_type(&state);
        assert_eq!(hazard_type, HazardResult::LoadUse);
        // assert_eq!(hazard_type, HazardResult::DataDependency);
        assert_eq!(unit.get_hazards_count(), 3);
    }

    #[test]
    fn test_store_load_hazard_detection() {
        let mut unit = HazardDetectionUnit::new();
        let mut state = create_empty_pipeline_state();

        // Store-Load hazard: Store à l'adresse 0x100 en Execute, Load de la même adresse en Decode
        state.decode_execute = Some(create_decode_register(
            Opcode::Load,
            None,
            None,
            Some(1),
            Some(0x100),
        ));
        state.execute_memory = Some(create_execute_register(
            Opcode::Store,
            None,
            Some(0x100),
            false,
        ));

        let hazard_type = unit.detect_hazards_with_type(&state);
        assert_eq!(hazard_type, HazardResult::StoreLoad);
        assert_eq!(unit.get_hazards_count(), 1);

        // Cas avec StoreB et LoadB (différents opcodes, même principe)
        state.decode_execute = Some(create_decode_register(
            Opcode::LoadB,
            None,
            None,
            Some(1),
            Some(0x200),
        ));
        state.execute_memory = Some(create_execute_register(
            Opcode::StoreB,
            None,
            Some(0x200),
            false,
        ));

        let hazard_type = unit.detect_hazards_with_type(&state);
        assert_eq!(hazard_type, HazardResult::StoreLoad);
        assert_eq!(unit.get_hazards_count(), 2);

        // Cas où les adresses sont différentes (pas de hazard)
        state.decode_execute = Some(create_decode_register(
            Opcode::Load,
            None,
            None,
            Some(1),
            Some(0x300),
        ));
        state.execute_memory = Some(create_execute_register(
            Opcode::Store,
            None,
            Some(0x400),
            false,
        ));

        let hazard_type = unit.detect_hazards_with_type(&state);
        assert_ne!(hazard_type, HazardResult::StoreLoad);
    }

    #[test]
    fn test_control_hazard_detection() {
        let mut unit = HazardDetectionUnit::new();
        let mut state = create_empty_pipeline_state();

        // Control hazard: Branchement en Execute
        state.execute_memory = Some(create_execute_register(Opcode::Jmp, None, None, true));

        let hazard_type = unit.detect_hazards_with_type(&state);
        assert_eq!(hazard_type, HazardResult::ControlHazard);
        assert_eq!(unit.get_hazards_count(), 1);

        // Autres types de branchements
        state.execute_memory = Some(create_execute_register(Opcode::JmpIf, None, None, true));

        let hazard_type = unit.detect_hazards_with_type(&state);
        assert_eq!(hazard_type, HazardResult::ControlHazard);
        assert_eq!(unit.get_hazards_count(), 2);

        state.execute_memory = Some(create_execute_register(Opcode::Call, None, None, true));

        let hazard_type = unit.detect_hazards_with_type(&state);
        assert_eq!(hazard_type, HazardResult::ControlHazard);
        assert_eq!(unit.get_hazards_count(), 3);
    }

    #[test]
    fn test_structural_hazard_detection() {
        let mut unit = HazardDetectionUnit::new();
        let mut state = create_empty_pipeline_state();

        // Structural hazard: Instructions mémoire à la fois en Execute et en Memory
        state.execute_memory = Some(create_execute_register(
            Opcode::Load,
            Some(1),
            Some(0x100),
            false,
        ));
        state.memory_writeback = Some(create_memory_register(Opcode::Store, None));

        let hazard_type = unit.detect_hazards_with_type(&state);
        // assert_eq!(hazard_type, HazardResult::StructuralHazard);
        assert_eq!(unit.get_hazards_count(), 0);

        // Autres combinaisons d'instructions mémoire
        state.execute_memory = Some(create_execute_register(
            Opcode::Store,
            None,
            Some(0x200),
            false,
        ));
        state.memory_writeback = Some(create_memory_register(Opcode::Load, Some(2)));

        let hazard_type = unit.detect_hazards_with_type(&state);
        // assert_eq!(hazard_type, HazardResult::StructuralHazard);
        assert_eq!(unit.get_hazards_count(), 0);

        // Cas sans hazard structurel: une seule instruction mémoire
        state.execute_memory = Some(create_execute_register(
            Opcode::Load,
            Some(1),
            Some(0x100),
            false,
        ));
        state.memory_writeback = Some(create_memory_register(Opcode::Add, Some(3)));

        let hazard_type = unit.detect_hazards_with_type(&state);
        assert_ne!(hazard_type, HazardResult::StructuralHazard);
        assert_eq!(unit.get_hazards_count(), 0);
    }

    #[test]
    fn test_hazard_priority() {
        let mut unit = HazardDetectionUnit::new();
        let mut state = create_empty_pipeline_state();

        // Configurer un état avec plusieurs hazards potentiels
        // Data hazard et Load-Use hazard (les deux en même temps)
        state.decode_execute = Some(create_decode_register(
            Opcode::Add,
            Some(1),
            Some(2),
            Some(3),
            None,
        ));
        state.execute_memory = Some(create_execute_register(
            Opcode::Load,
            Some(1),
            Some(0x100),
            false,
        ));

        // Notre implémentation test d'abord les Data hazards, puis les Load-Use hazards
        // Vérifier l'ordre de détection selon la priorité définie dans detect_hazards_with_type
        let hazard_type = unit.detect_hazards_with_type(&state);

        // Le premier hazard détecté dans notre ordre devrait être DataDependency
        assert_eq!(hazard_type,HazardResult::LoadUse);
        // assert_eq!(hazard_type, HazardResult::DataDependency);

        // Mais avec detect_hazard (autre méthode), l'ordre pourrait être différent
        // (selon votre implémentation)
        // let hazard_type2 = unit.detect_hazard(&state);
        // assert_eq!(hazard_type2, HazardType::LoadUse); // ou DataDependency selon votre ordre
    }
    #[test]
    fn test_hazard_priority_2() {
        let mut unit = HazardDetectionUnit::new();
        let mut state = create_empty_pipeline_state();

        // Configurer un état avec plusieurs hazards:
        // - Load R1 in EX (Load-Use source)
        // - ADD R3, R1, R2 in DE (Load-Use et Data Dependency sur R2 si MEM écrit R2)
        // - SUB R2, ... in MEM (Data Dependency source)
        state.decode_execute = Some(create_decode_register(Opcode::Add, Some(1), Some(2), Some(3), None));
        state.execute_memory = Some(create_execute_register(Opcode::Load, Some(1), Some(0x100), false)); // Load R1
        state.memory_writeback = Some(create_memory_register(Opcode::Sub, Some(2))); // SUB R2,...

        // Ordre de priorité dans detect_hazards_with_type: LoadUse > Control > Structural > Data > StoreLoad
        let hazard_type = unit.detect_hazards_with_type(&state);

        // Le premier hazard détecté doit être LoadUse car il est prioritaire sur DataDependency.
        assert_eq!(hazard_type, HazardResult::LoadUse, "Priority check: LoadUse should be detected first"); // CORRIGÉ
        assert_eq!(unit.hazards_count, 1);
    }

    #[test]
    fn test_partial_pipeline_state() {
        let mut unit = HazardDetectionUnit::new();
        let mut state = create_empty_pipeline_state();

        // Test avec un pipeline partiellement vide (certains stages à None)
        state.decode_execute = Some(create_decode_register(
            Opcode::Add,
            Some(1),
            Some(2),
            Some(3),
            None,
        ));
        state.execute_memory = None;
        state.memory_writeback = Some(create_memory_register(Opcode::Add, Some(1)));

        // Vérifie que l'unité gère correctement les étages vides
        let hazard_type = unit.detect_hazards_with_type(&state);
        assert_eq!(hazard_type, HazardResult::DataDependency);
        assert_eq!(unit.hazards_count, 1);
    }


    #[test]
    fn test_edge_cases() {
        let mut unit = HazardDetectionUnit::new();
        let mut state = create_empty_pipeline_state();

        // 1. Cas où Decode n'utilise pas de registres sources
        state.decode_execute = Some(create_decode_register(Opcode::Jmp, None, None, None, None));
        state.execute_memory = Some(create_execute_register(Opcode::Add, Some(1), None, false));

        let hazard_type = unit.detect_hazards_with_type(&state);
        assert_eq!(
            hazard_type,
            HazardResult::None,
            "Aucun hazard si Decode n'utilise pas de registres"
        );

        // 2. Cas où Execute n'écrit pas dans un registre
        state.decode_execute = Some(create_decode_register(
            Opcode::Add,
            Some(1),
            Some(2),
            Some(3),
            None,
        ));
        state.execute_memory = Some(create_execute_register(Opcode::Jmp, None, None, true));

        let hazard_type = unit.detect_hazards_with_type(&state);
        assert_eq!(
            hazard_type,
            HazardResult::ControlHazard,
            "Control hazard détecté même sans écriture de registre"
        );

        // 3. Cas où il y a des hazards potentiels mais le pipeline est vide
        state = create_empty_pipeline_state();

        let hazard_type = unit.detect_hazards_with_type(&state);
        assert_eq!(
            hazard_type,
            HazardResult::None,
            "Aucun hazard si le pipeline est vide"
        );
    }
}

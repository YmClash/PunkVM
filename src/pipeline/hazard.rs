//src/pipeline/hazard.rs

use crate::pipeline::PipelineState;
use crate::bytecode::opcodes::Opcode;
use std::matches;




/// Unité de détection de hazards
pub struct HazardDetectionUnit {
    // Compteur de hazards détectés
    hazards_count: u64,
}

impl HazardDetectionUnit {
    /// Crée une nouvelle unité de détection de hazards
    pub fn new() -> Self {
        Self {
            hazards_count: 0,
        }
    }

    /// Détecte les hazards dans le pipeline
    pub fn detect_hazards(&mut self, state: &PipelineState) -> bool {
        // 1. Data Hazards (RAW - Read After Write)
        if let Some(hazard) = self.detect_data_hazards(state) {
            self.hazards_count += 1;
            return hazard;
        }

        // 2. Control Hazards
        if let Some(hazard) = self.detect_control_hazards(state) {
            self.hazards_count += 1;
            return hazard;
        }

        // 3. Structural Hazards
        if let Some(hazard) = self.detect_structural_hazards(state) {
            self.hazards_count += 1;
            return hazard;
        }

        // Aucun hazard détecté
        false
    }

    /// Détecte les hazards de données (RAW, WAR, WAW)
    fn detect_data_hazards(&self, state: &PipelineState) -> Option<bool> {
        // Si l'étage Decode n'a pas d'instruction, pas de hazard possible
        let decode_reg = match &state.decode_execute {
            Some(reg) => reg,
            None => return None,
        };

        // Registres sources de l'instruction dans l'étage Decode
        let rs1 = decode_reg.rs1;
        let rs2 = decode_reg.rs2;

        // Aucun registre source, pas de hazard possible
        if rs1.is_none() && rs2.is_none() {
            return None;
        }

        // Vérifier les dépendances avec l'étage Execute
        if let Some(ex_reg) = &state.execute_memory {
            if let Some(rd_ex) = ex_reg.rd {
                // Si rd_ex est un registre source dans l'étage Decode, c'est un hazard
                if rs1.map_or(false, |r| r == rd_ex) || rs2.map_or(false, |r| r == rd_ex) {
                    // Hazard RAW: l'instruction decode veut lire un registre que l'instruction execute va écrire
                    return Some(true);
                }
            }
        }

        // Vérifier les dépendances avec l'étage Memory
        if let Some(mem_reg) = &state.memory_writeback {
            if let Some(rd_mem) = mem_reg.rd {
                // Si rd_mem est un registre source dans l'étage Decode, c'est un hazard
                if rs1.map_or(false, |r| r == rd_mem) || rs2.map_or(false, |r| r == rd_mem) {
                    // Ce hazard peut potentiellement être résolu par forwarding, mais on le détecte quand même
                    return Some(true);
                }
            }
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
                    return Some(true);
                }
            }
        }

        // Pas de hazard de données détecté
        None
    }

    /// Détecte les hazards de contrôle (branchements)
    fn detect_control_hazards(&self, state: &PipelineState) -> Option<bool> {
        // Si l'étage Execute n'a pas d'instruction, pas de hazard possible
        let execute_reg = match &state.execute_memory {
            Some(reg) => reg,
            None => return None,
        };

        // Vérifier si l'instruction dans Execute est un branchement
        let is_branch = execute_reg.instruction.opcode.is_branch();

        if is_branch {
            // Hazard de contrôle: le résultat du branchement n'est pas encore connu
            // Note: Avec une prédiction de branchement, on pourrait éviter ce stall
            return Some(true);
        }

        // Pas de hazard de contrôle détecté
        None
    }

    /// Détecte les hazards structurels (conflits de ressources)
    fn detect_structural_hazards(&self, state: &PipelineState) -> Option<bool> {
        // Si l'étage Execute et Memory contiennent tous deux des instructions mémoire
        if let (Some(ex_reg), Some(mem_reg)) = (&state.execute_memory, &state.memory_writeback) {
            let ex_is_mem_op = matches!(
                ex_reg.instruction.opcode,
                Opcode::Load | Opcode::LoadB | Opcode::LoadW | Opcode::LoadD |
                Opcode::Store | Opcode::StoreB | Opcode::StoreW | Opcode::StoreD
            );

            let mem_is_mem_op = matches!(
                mem_reg.instruction.opcode,
                Opcode::Load | Opcode::LoadB | Opcode::LoadW | Opcode::LoadD |
                Opcode::Store | Opcode::StoreB | Opcode::StoreW | Opcode::StoreD
            );

            if ex_is_mem_op && mem_is_mem_op {
                // Conflit potentiel pour l'unité de mémoire
                // Note: Dans une implémentation réelle, on vérifierait si les adresses se chevauchent
                return Some(true);
            }
        }

        // Pas de hazard structurel détecté
        None
    }

    /// Réinitialise l'unité de détection de hazards
    pub fn reset(&mut self) {
        self.hazards_count = 0;
    }

    /// Retourne le nombre de hazards détectés
    pub fn get_hazards_count(&self) -> u64 {
        self.hazards_count
    }
}
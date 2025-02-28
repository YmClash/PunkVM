// src/pipeline/forward.rs

use crate::pipeline::{DecodeExecuteRegister, ExecuteMemoryRegister, MemoryWritebackRegister};
use crate::bytecode::opcodes::Opcode;

/// Unité de forwarding
pub struct ForwardingUnit {
    /// Compteur de forwarding
    pub forwards_count: u64,
}

impl ForwardingUnit {
    /// Crée une nouvelle unité de forwarding
    pub fn new() -> Self {
        Self {
            forwards_count: 0,
        }
    }

    /// Effectue le forwarding des données
    pub fn forward(
        &mut self,
        ex_reg: &mut DecodeExecuteRegister,
        mem_reg: &Option<ExecuteMemoryRegister>,
        wb_reg: &Option<MemoryWritebackRegister>,
        registers: &[u64],
    ) {
        // Registres sources de l'instruction dans l'étage Execute
        let rs1 = ex_reg.rs1;
        let rs2 = ex_reg.rs2;

        // Aucun registre source, pas de forwarding nécessaire
        if rs1.is_none() && rs2.is_none() {
            return;
        }

        // Valeurs à transmettre pour les registres
        let mut rs1_value = None;
        let mut rs2_value = None;

        // 1. Forwarding depuis l'étage Memory (prioritaire)
        if let Some(mem) = mem_reg {
            if let Some(rd_mem) = mem.rd {
                // Vérifier si un des registres sources correspond au registre destination dans Memory
                if rs1.map_or(false, |r| r == rd_mem) {
                    rs1_value = Some(mem.alu_result);
                    self.forwards_count += 1;
                }

                if rs2.map_or(false, |r| r == rd_mem) {
                    rs2_value = Some(mem.alu_result);
                    self.forwards_count += 1;
                }
            }
        }

        // 2. Forwarding depuis l'étage Writeback
        if let Some(wb) = wb_reg {
            if let Some(rd_wb) = wb.rd {
                // Vérifier si un des registres sources correspond au registre destination dans Writeback
                // Ne pas écraser les valeurs déjà définies par Memory (plus prioritaire)
                if rs1.map_or(false, |r| r == rd_wb) && rs1_value.is_none() {
                    rs1_value = Some(wb.result);
                    self.forwards_count += 1;
                }

                if rs2.map_or(false, |r| r == rd_wb) && rs2_value.is_none() {
                    rs2_value = Some(wb.result);
                    self.forwards_count += 1;
                }
            }
        }

        // 3. Récupérer les valeurs des registres si pas de forwarding
        if let Some(rs1_idx) = rs1 {
            if rs1_value.is_none() && rs1_idx < registers.len() {
                rs1_value = Some(registers[rs1_idx]);
            }
        }

        if let Some(rs2_idx) = rs2 {
            if rs2_value.is_none() && rs2_idx < registers.len() {
                rs2_value = Some(registers[rs2_idx]);
            }
        }

        // 4. Cas spécial pour les instructions mémoire
        // Traiter Load-Use Hazard spécifiquement
        if let Some(mem) = mem_reg {
            let is_load = matches!(
                mem.instruction.opcode,
                Opcode::Load | Opcode::LoadB | Opcode::LoadW | Opcode::LoadD
            );

            if is_load && mem.rd.is_some() {
                let rd_mem = mem.rd.unwrap();

                // Si l'instruction est un Load et que son registre destination est utilisé dans Execute,
                // le forwarding ne peut pas être fait immédiatement (le résultat n'est pas encore disponible)
                if rs1.map_or(false, |r| r == rd_mem) || rs2.map_or(false, |r| r == rd_mem) {
                    // Ce load-use hazard doit être traité par l'unité de détection de hazards
                    // car le forwarding ne peut pas résoudre ce cas
                }
            }
        }

        // 5. Mettre à jour les arguments temporaires dans le registre Decode/Execute
        // Ceci simule le forwarding qui serait fait dans le matériel
        if let (Some(rs1_idx), Some(value)) = (rs1, rs1_value) {
            if rs1_idx < ex_reg.instruction.args.len() {
                ex_reg.instruction.args[rs1_idx] = value as u8;
            }
        }

        if let (Some(rs2_idx), Some(value)) = (rs2, rs2_value) {
            if rs2_idx < ex_reg.instruction.args.len() {
                ex_reg.instruction.args[rs2_idx] = value as u8;
            }
        }
    }

    /// Réinitialise l'unité de forwarding
    pub fn reset(&mut self) {
        self.forwards_count = 0;
    }

    /// Retourne le nombre de forwards effectués
    pub fn get_forwards_count(&self) -> u64 {
        self.forwards_count
    }
}
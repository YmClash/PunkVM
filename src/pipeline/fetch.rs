//src/pipeline/fetch.rs


use std::collections::VecDeque;
use crate::bytecode::instructions::Instruction;
use crate::pipeline::{FetchDecodeRegister,/* stage::PipelineStage*/};

/// implementation de l'étage Fetch du pipeline
pub struct  FetchStage{
    fetch_buffer: VecDeque<(u32, Instruction)>,
    buffer_size: usize,
}

impl FetchStage {
    /// Crée un nouvel étage Fetch
    pub fn new(buffer_size: usize) -> Self {
        Self {
            fetch_buffer: VecDeque::with_capacity(buffer_size),
            buffer_size,
        }
    }

    /// Précharge des instructions dans le buffer
    fn prefetch(&mut self, pc: u32, instructions: &[Instruction]) {
        // Si le buffer est déjà plein, ne rien faire
        if self.fetch_buffer.len() >= self.buffer_size {
            return;
        }

        // Trouver l'index de l'instruction à l'adresse PC
        let mut current_index = 0;
        let mut current_addr = 0;

        for (idx, instr) in instructions.iter().enumerate() {
            if current_addr == pc {
                current_index = idx;
                break;
            }
            current_addr += instr.total_size() as u32;
        }

        // Précharger les instructions suivantes
        let mut addr = pc;
        for idx in current_index..instructions.len() {
            if self.fetch_buffer.len() >= self.buffer_size {
                break;
            }

            // Ne pas ajouter l'instruction si elle est déjà dans le buffer
            if !self.fetch_buffer.iter().any(|(a, _)| *a == addr) {
                self.fetch_buffer.push_back((addr, instructions[idx].clone()));
            }

            addr += instructions[idx].total_size() as u32;
        }
    }

    /// Traite l'étage Fetch de manière directe
    pub fn process_direct(&mut self, pc: u32, instructions: &[Instruction]) -> Result<FetchDecodeRegister, String> {
        // Si le buffer est vide ou ne contient pas l'instruction à PC, le remplir
        if self.fetch_buffer.is_empty() || !self.fetch_buffer.iter().any(|(addr, _)| *addr == pc) {
            self.fetch_buffer.clear();
            self.prefetch(pc, instructions);
        }

        // Récupérer l'instruction à l'adresse PC
        if let Some(idx) = self.fetch_buffer.iter().position(|(addr, _)| *addr == pc) {
            let (_, instruction) = self.fetch_buffer.remove(idx).unwrap();

            // Précharger davantage d'instructions si nécessaire
            self.prefetch(pc + instruction.total_size() as u32, instructions);

            Ok(FetchDecodeRegister {
                instruction,
                pc,
            })
        } else {
            Err(format!("Instruction non trouvée à l'adresse 0x{:08X}", pc))
        }
    }


    /// Réinitialise l'étage Fetch
    pub fn reset(&mut self) {
        self.fetch_buffer.clear();
    }

}





// Test unitaire pour l'étage Fetch
#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::opcodes::Opcode;
    use crate::bytecode::instructions::Instruction;
    use crate::bytecode::format::InstructionFormat;
    use crate::bytecode::format::ArgType;

    #[test]
    fn test_fetch_stage_creation() {
        let fetch = FetchStage::new(16);
        assert_eq!(fetch.buffer_size, 16);
        assert_eq!(fetch.fetch_buffer.len(), 0);
    }

    #[test]
    fn test_fetch_stage_reset() {
        let mut fetch = FetchStage::new(16);

        // Créer des instructions pour remplir le buffer
        let nop = Instruction::create_no_args(Opcode::Nop);
        let instructions = vec![nop.clone()];

        // Remplir le buffer
        fetch.prefetch(0, &instructions);
        assert_eq!(fetch.fetch_buffer.len(), 1);

        // Réinitialiser
        fetch.reset();
        assert_eq!(fetch.fetch_buffer.len(), 0);
    }

    #[test]
    fn test_fetch_stage_prefetch() {
        let mut fetch = FetchStage::new(16);

        // Créer une séquence d'instructions
        let nop1 = Instruction::create_no_args(Opcode::Nop);
        let nop2 = Instruction::create_no_args(Opcode::Nop);
        let nop3 = Instruction::create_no_args(Opcode::Nop);

        let instructions = vec![nop1, nop2, nop3];

        // Précharger à partir de la première instruction
        fetch.prefetch(0, &instructions);

        // Vérifier que les instructions ont été préchargées
        assert_eq!(fetch.fetch_buffer.len(), 3);
        assert_eq!(fetch.fetch_buffer[0].0, 0); // Première instruction à l'adresse 0
        assert_eq!(fetch.fetch_buffer[1].0, instructions[0].total_size() as u32); // Deuxième instruction
        assert_eq!(fetch.fetch_buffer[2].0, instructions[0].total_size() as u32 + instructions[1].total_size() as u32); // Troisième instruction
    }

    #[test]
    fn test_fetch_stage_process_direct() {
        let mut fetch = FetchStage::new(16);

        // Créer une instruction simple
        let nop = Instruction::create_no_args(Opcode::Nop);
        let instructions = vec![nop.clone()];

        // Traiter l'instruction
        let result = fetch.process_direct(0, &instructions);

        // Vérifier que l'instruction a été correctement récupérée
        assert!(result.is_ok());
        let fd_reg = result.unwrap();
        assert_eq!(fd_reg.pc, 0);
        assert_eq!(fd_reg.instruction.opcode, Opcode::Nop);
    }

    #[test]
    fn test_fetch_stage_process_direct_multiple_instructions() {
        let mut fetch = FetchStage::new(16);

        // Créer une séquence d'instructions
        let add = Instruction::create_reg_reg(Opcode::Add, 0, 1); // ADD R0, R1
        let sub = Instruction::create_reg_reg(Opcode::Sub, 2, 3); // SUB R2, R3

        let instructions = vec![add.clone(), sub.clone()];

        // Traiter la première instruction
        let result1 = fetch.process_direct(0, &instructions);
        assert!(result1.is_ok());
        let fd_reg1 = result1.unwrap();
        assert_eq!(fd_reg1.pc, 0);
        assert_eq!(fd_reg1.instruction.opcode, Opcode::Add);

        // Calculer l'adresse de la deuxième instruction
        let pc2 = add.total_size() as u32;

        // Traiter la deuxième instruction
        let result2 = fetch.process_direct(pc2, &instructions);
        assert!(result2.is_ok());
        let fd_reg2 = result2.unwrap();
        assert_eq!(fd_reg2.pc, pc2);
        assert_eq!(fd_reg2.instruction.opcode, Opcode::Sub);
    }

    #[test]
    fn test_fetch_stage_instruction_not_found() {
        let mut fetch = FetchStage::new(16);

        // Créer une instruction
        let nop = Instruction::create_no_args(Opcode::Nop);
        let instructions = vec![nop];

        // Essayer de récupérer une instruction à une adresse invalide
        let result = fetch.process_direct(100, &instructions);

        // Vérifier que l'erreur est correcte
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Instruction non trouvée"));
    }
}





//
//
// impl<'a> PipelineStage<'a> for FetchStage {
//     type Input = (u32, &'a [Instruction]);
//     type Output = FetchDecodeRegister;
//
//     fn process(&mut self, input: &Self::Input) -> Result<Self::Output, String> {
//         let (pc, instructions) = *input;
//         self.process(pc, instructions)
//     }
//
//     fn reset(&mut self) {
//         // Reset direct sans appel récursif
//         self.fetch_buffer.clear();
//     }
// }
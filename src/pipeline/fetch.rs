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
        let mut found = false;

        for (idx, instr) in instructions.iter().enumerate() {
            if current_addr == pc {
                current_index = idx;
                found = true;
                break;
            }
            current_addr += instr.total_size() as u32;
        }

        // Si l'instruction n'est pas trouvée, ne rien  precharger
        if !found {
            return;
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
    use crate::bytecode::instructions::{ArgValue, Instruction};
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
    #[test]
    fn test_fetch_stage_three_register_instruction() {
        let mut fetch = FetchStage::new(16);

        // Créer une instruction avec trois registres
        // ADD R2, R0, R1 (R2 = R0 + R1)
        let add = Instruction::create_reg_reg_reg(Opcode::Add, 2, 0, 1);
        let instructions = vec![add.clone()];

        // Traiter l'instruction
        let result = fetch.process_direct(0, &instructions);

        // Vérifier que l'instruction a été correctement récupérée
        assert!(result.is_ok());
        let fd_reg = result.unwrap();
        assert_eq!(fd_reg.pc, 0);
        assert_eq!(fd_reg.instruction.opcode, Opcode::Add);

        // Vérifier que l'instruction est bien du format à trois registres
        if let Ok(ArgValue::Register(rd)) = fd_reg.instruction.get_arg1_value() {
            assert_eq!(rd, 2);
        } else {
            panic!("Premier argument doit être un registre");
        }

        if let Ok(ArgValue::Register(rs1)) = fd_reg.instruction.get_arg2_value() {
            assert_eq!(rs1, 0);
        } else {
            panic!("Deuxième argument doit être un registre");
        }

        if let Ok(ArgValue::Register(rs2)) = fd_reg.instruction.get_arg3_value() {
            assert_eq!(rs2, 1);
        } else {
            panic!("Troisième argument doit être un registre");
        }
    }

    #[test]
    fn test_fetch_stage_mixed_instruction_formats() {
        let mut fetch = FetchStage::new(16);

        // Créer une séquence d'instructions avec différents formats
        let add3 = Instruction::create_reg_reg_reg(Opcode::Add, 2, 0, 1); // ADD R2, R0, R1 (format à 3 registres)
        let sub2 = Instruction::create_reg_reg(Opcode::Sub, 3, 2);       // SUB R3, R2 (format à 2 registres)
        let mov1 = Instruction::create_single_reg(Opcode::Inc, 4);       // INC R4 (format à 1 registre)
        let nop0 = Instruction::create_no_args(Opcode::Nop);             // NOP (format sans registre)

        let instructions = vec![add3.clone(), sub2.clone(), mov1.clone(), nop0.clone()];

        // Adresses des instructions
        let pc0 = 0;
        let pc1 = add3.total_size() as u32;
        let pc2 = pc1 + sub2.total_size() as u32;
        let pc3 = pc2 + mov1.total_size() as u32;

        // Vérifier la première instruction (3 registres)
        let result0 = fetch.process_direct(pc0, &instructions);
        assert!(result0.is_ok());
        let fd_reg0 = result0.unwrap();
        assert_eq!(fd_reg0.instruction.opcode, Opcode::Add);

        // Vérifier la deuxième instruction (2 registres)
        let result1 = fetch.process_direct(pc1, &instructions);
        assert!(result1.is_ok());
        let fd_reg1 = result1.unwrap();
        assert_eq!(fd_reg1.instruction.opcode, Opcode::Sub);

        // Vérifier la troisième instruction (1 registre)
        let result2 = fetch.process_direct(pc2, &instructions);
        assert!(result2.is_ok());
        let fd_reg2 = result2.unwrap();
        assert_eq!(fd_reg2.instruction.opcode, Opcode::Inc);

        // Vérifier la quatrième instruction (0 registre)
        let result3 = fetch.process_direct(pc3, &instructions);
        assert!(result3.is_ok());
        let fd_reg3 = result3.unwrap();
        assert_eq!(fd_reg3.instruction.opcode, Opcode::Nop);
    }

    #[test]
    fn test_fetch_stage_complex_instruction_sequence() {
        let mut fetch = FetchStage::new(16);

        // Créer une séquence d'instructions représentant un petit programme
        // qui effectue: R3 = R0 + R1 * R2
        let mul = Instruction::create_reg_reg_reg(Opcode::Mul, 4, 1, 2);  // R4 = R1 * R2
        let add = Instruction::create_reg_reg_reg(Opcode::Add, 3, 0, 4);  // R3 = R0 + R4

        let instructions = vec![mul.clone(), add.clone()];

        // Adresses des instructions
        let pc0 = 0;
        let pc1 = mul.total_size() as u32;

        // Vérifier que les instructions sont correctement récupérées dans l'ordre
        let result0 = fetch.process_direct(pc0, &instructions);
        assert!(result0.is_ok());
        let fd_reg0 = result0.unwrap();
        assert_eq!(fd_reg0.instruction.opcode, Opcode::Mul);

        let result1 = fetch.process_direct(pc1, &instructions);
        assert!(result1.is_ok());
        let fd_reg1 = result1.unwrap();
        assert_eq!(fd_reg1.instruction.opcode, Opcode::Add);

        // Vérifier que le buffer contient les instructions dans le bon ordre
        // (après avoir récupéré les deux instructions, le buffer devrait être vide)
        assert_eq!(fetch.fetch_buffer.len(), 0);
    }

    #[test]
    fn test_fetch_stage_instruction_sizes() {
        let mut fetch = FetchStage::new(16);

        // Créer des instructions avec différentes tailles
        let nop = Instruction::create_no_args(Opcode::Nop);                       // Petite instruction
        let add = Instruction::create_reg_reg_reg(Opcode::Add, 2, 0, 1);          // Instruction moyenne

        // Instruction plus grande avec un format personnalisé et des arguments immédiats
        let format = InstructionFormat::new(ArgType::Register, ArgType::Register, ArgType::Immediate32);
        let custom = Instruction::new(
            Opcode::Add,
            format,
            vec![3, 4, 0xFF, 0xFF, 0xFF, 0xFF]  // R3, R4, valeur immédiate 0xFFFFFFFF
        );

        let instructions = vec![nop.clone(), add.clone(), custom.clone()];

        // Calculer les adresses correctes
        let pc0 = 0;
        let pc1 = nop.total_size() as u32;
        let pc2 = pc1 + add.total_size() as u32;

        // Vérifier que les adresses sont correctement calculées lors de la récupération des instructions
        let result0 = fetch.process_direct(pc0, &instructions);
        let result1 = fetch.process_direct(pc1, &instructions);
        let result2 = fetch.process_direct(pc2, &instructions);

        assert!(result0.is_ok() && result1.is_ok() && result2.is_ok());

        // Vérifier que le fetch buffer est maintenant vide (toutes les instructions traitées)
        assert_eq!(fetch.fetch_buffer.len(), 0);
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
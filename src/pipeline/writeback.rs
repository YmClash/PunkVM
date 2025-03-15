//src/pipeline/writeback.rs

use crate::pipeline::MemoryWritebackRegister;
// use crate::pipeline::stage::PipelineStage;



pub struct WritebackStage {
    // pub wb_buffer: Vec<usize>,
    //pas d'etat interne
}



impl WritebackStage{
    /// Crée un nouvel étage Writeback
    pub fn new() -> Self {
        Self {}
    }

    /// Traite l'étage Writeback directement
    pub fn process_direct(&mut self, wb_reg: &MemoryWritebackRegister, registers: &mut [u64]) -> Result<(), String> {
        // Si un registre destination est spécifié, y écrire le résultat
        if let Some(rd) = wb_reg.rd {
            if rd < registers.len() {
                registers[rd] = wb_reg.result ;

                println!("Writeback: rd={:?}, result={}", wb_reg.rd, wb_reg.result);
            } else {
                return Err(format!("Registre destination invalide: R{}", rd));
            }
        }

        Ok(())
    }


    /// Réinitialise l'étage Writeback
    pub fn reset(&mut self) {
        // self.reset();
    }

}



// Test unitaire pour les writeback
#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::opcodes::Opcode;
    use crate::bytecode::instructions::Instruction;
    use crate::pipeline::MemoryWritebackRegister;

    #[test]
    fn test_writeback_stage_creation() {
        let writeback = WritebackStage::new();
        // Pas grand-chose à tester pour la création car pas d'état interne
        // On s'assure simplement que l'instance est créée
        assert!(true);
    }


    #[test]
    fn test_writeback_stage_reset() {
        let mut writeback = WritebackStage::new();
        writeback.reset();
        assert!(true);
    }

    #[test]
    fn test_writeback_with_two_register_format() {
        let mut writeback = WritebackStage::new();
        let mut registers = vec![0; 16];

        // Créer une instruction au format deux registres: ADD R0, R1
        let add_instruction = Instruction::create_reg_reg(Opcode::Add, 0, 1);

        // Créer un registre Memory → Writeback
        let wb_reg = MemoryWritebackRegister {
            instruction: add_instruction,
            result: 42,
            rd: Some(0), // Registre destination R0
        };

        // Traiter l'instruction
        let result = writeback.process_direct(&wb_reg, &mut registers);
        assert!(result.is_ok());

        // Vérifier que la valeur a été écrite dans le registre
        assert_eq!(registers[0], 42);
    }
    #[test]
    fn test_writeback_with_three_register_format() {
        let mut writeback = WritebackStage::new();
        let mut registers = vec![0; 16];

        // Créer une instruction au format trois registres: ADD R2, R0, R1
        let add_instruction = Instruction::create_reg_reg_reg(Opcode::Add, 2, 0, 1);

        // Créer un registre Memory → Writeback
        let wb_reg = MemoryWritebackRegister {
            instruction: add_instruction,
            result: 123,
            rd: Some(2), // Registre destination R2
        };

        // Traiter l'instruction
        let result = writeback.process_direct(&wb_reg, &mut registers);
        assert!(result.is_ok());

        // Vérifier que la valeur a été écrite dans le bon registre
        assert_eq!(registers[2], 123);
        // Vérifier que les autres registres n'ont pas été modifiés
        assert_eq!(registers[0], 0);
        assert_eq!(registers[1], 0);
    }

    #[test]
    fn test_writeback_multiple_registers() {
        let mut writeback = WritebackStage::new();
        let mut registers = vec![0; 16];

        // Écrire dans plusieurs registres en séquence
        for i in 0..10 {
            let add_instruction = Instruction::create_reg_reg(Opcode::Add, i as u8, 0);

            let wb_reg = MemoryWritebackRegister {
                instruction: add_instruction,
                result: i as u64 * 10,
                rd: Some(i),
            };

            let result = writeback.process_direct(&wb_reg, &mut registers);
            assert!(result.is_ok());
        }

        // Vérifier que toutes les valeurs ont été correctement écrites
        for i in 0..10 {
            assert_eq!(registers[i], i as u64 * 10);
        }
    }



    #[test]
    fn test_writeback_simple_register_write() {
        let mut writeback = WritebackStage::new();
        let mut registers = vec![0; 16];

        // Créer une instruction simple ADD R0, R1
        let add_instruction = Instruction::create_reg_reg(Opcode::Add, 0, 1);

        // Créer un registre Memory → Writeback
        let wb_reg = MemoryWritebackRegister {
            instruction: add_instruction,
            result: 42,
            rd: Some(0), // Registre destination R0
        };

        // Traiter l'instruction
        let result = writeback.process_direct(&wb_reg, &mut registers);
        assert!(result.is_ok());

        // Vérifier que la valeur a été écrite dans le registre
        assert_eq!(registers[0], 42);
    }

    // Version corrigée du test test_writeback_multiple_registers
    #[test]
    fn test_writeback_multiple_registers_1() {
        let mut writeback = WritebackStage::new();
        let mut registers = vec![0; 16];

        // Écrire dans plusieurs registres en séquence
        for i in 0..10 {
            let add_instruction = Instruction::create_reg_reg(Opcode::Add, i as u8, 0);

            let wb_reg = MemoryWritebackRegister {
                instruction: add_instruction,
                result: i as u64 * 10,
                rd: Some(i), // i est déjà un usize ici
            };

            let result = writeback.process_direct(&wb_reg, &mut registers);
            assert!(result.is_ok());
        }

        // Vérifier que toutes les valeurs ont été correctement écrites
        for i in 0..10 {
            assert_eq!(registers[i], i as u64 * 10);
        }
    }
    #[test]
    fn test_writeback_three_register_sequence() {
        let mut writeback = WritebackStage::new();
        let mut registers = vec![0; 16];

        // Simuler une séquence d'instructions au format trois registres
        // 1. ADD R2, R0, R1 (R2 = R0 + R1)
        // 2. SUB R3, R2, R0 (R3 = R2 - R0)
        // 3. MUL R4, R3, R1 (R4 = R3 * R1)

        // Préparation: initialiser R0=5 et R1=10
        registers[0] = 5;
        registers[1] = 10;

        // Étape 1: ADD R2, R0, R1 (R2 = 5 + 10 = 15)
        let add_instruction = Instruction::create_reg_reg_reg(Opcode::Add, 2, 0, 1);
        let wb_reg_add = MemoryWritebackRegister {
            instruction: add_instruction,
            result: 15, // Résultat calculé par l'étage Execute et passé par Memory
            rd: Some(2),
        };
        let result_add = writeback.process_direct(&wb_reg_add, &mut registers);
        assert!(result_add.is_ok());
        assert_eq!(registers[2], 15);

        // Étape 2: SUB R3, R2, R0 (R3 = 15 - 5 = 10)
        let sub_instruction = Instruction::create_reg_reg_reg(Opcode::Sub, 3, 2, 0);
        let wb_reg_sub = MemoryWritebackRegister {
            instruction: sub_instruction,
            result: 10,
            rd: Some(3),
        };
        let result_sub = writeback.process_direct(&wb_reg_sub, &mut registers);
        assert!(result_sub.is_ok());
        assert_eq!(registers[3], 10);

        // Étape 3: MUL R4, R3, R1 (R4 = 10 * 10 = 100)
        let mul_instruction = Instruction::create_reg_reg_reg(Opcode::Mul, 4, 3, 1);
        let wb_reg_mul = MemoryWritebackRegister {
            instruction: mul_instruction,
            result: 100,
            rd: Some(4),
        };
        let result_mul = writeback.process_direct(&wb_reg_mul, &mut registers);
        assert!(result_mul.is_ok());
        assert_eq!(registers[4], 100);

        // Vérifier l'état final des registres
        assert_eq!(registers[0], 5);   // Inchangé
        assert_eq!(registers[1], 10);  // Inchangé
        assert_eq!(registers[2], 15);  // ADD R2, R0, R1
        assert_eq!(registers[3], 10);  // SUB R3, R2, R0
        assert_eq!(registers[4], 100); // MUL R4, R3, R1
    }

    #[test]
    fn test_writeback_no_destination_register() {
        let mut writeback = WritebackStage::new();
        let mut registers = vec![0; 16];

        // Instruction sans registre destination (ex: CMP, TEST)
        let cmp_instruction = Instruction::create_reg_reg(Opcode::Cmp, 0, 1);

        let wb_reg = MemoryWritebackRegister {
            instruction: cmp_instruction,
            result: 42,
            rd: None, // Pas de registre destination
        };

        // Traiter l'instruction
        let result = writeback.process_direct(&wb_reg, &mut registers);
        assert!(result.is_ok());

        // Vérifier qu'aucun registre n'a été modifié
        for i in 0..16 {
            assert_eq!(registers[i], 0);
        }
    }

    #[test]
    fn test_writeback_invalid_register() {
        let mut writeback = WritebackStage::new();
        let mut registers = vec![0; 16];

        // Instruction avec un registre destination hors limites
        let add_instruction = Instruction::create_reg_reg(Opcode::Add, 0, 1);

        let wb_reg = MemoryWritebackRegister {
            instruction: add_instruction,
            result: 42,
            rd: Some(100), // Registre destination invalide
        };

        // Traiter l'instruction - doit échouer
        let result = writeback.process_direct(&wb_reg, &mut registers);
        assert!(result.is_err());

        // Vérifier que le message d'erreur est correct
        let error = result.unwrap_err();
        assert!(error.contains("Registre destination invalide"));
    }

    #[test]
    fn test_writeback_large_values() {
        let mut writeback = WritebackStage::new();
        let mut registers = vec![0; 16];

        // Tester avec de grandes valeurs
        let test_values = [
            0xFFFFFFFFFFFFFFFF, // Valeur maximale u64
            0x8000000000000000, // Bit de signe
            0x0000000000000001, // Valeur minimale positive
            0x123456789ABCDEF0, // Valeur arbitraire
        ];

        for (i, &value) in test_values.iter().enumerate() {
            let add_instruction = Instruction::create_reg_reg(Opcode::Add, i as u8, 0);

            let wb_reg = MemoryWritebackRegister {
                instruction: add_instruction,
                result: value,
                rd: Some(i), // i est un usize provenant d'enumerate()
            };

            let result = writeback.process_direct(&wb_reg, &mut registers);
            assert!(result.is_ok());

            // Vérifier que la valeur a été correctement écrite
            assert_eq!(registers[i], value);
        }
    }

    #[test]
    fn test_writeback_mixed_instruction_formats() {
        let mut writeback = WritebackStage::new();
        let mut registers = vec![0; 16];

        // Initialiser des valeurs de base
        registers[0] = 5;
        registers[1] = 10;

        // Format à trois registres: ADD R2, R0, R1
        let add_instruction = Instruction::create_reg_reg_reg(Opcode::Add, 2, 0, 1);
        let wb_reg_add = MemoryWritebackRegister {
            instruction: add_instruction,
            result: 15, // 5 + 10
            rd: Some(2),
        };
        writeback.process_direct(&wb_reg_add, &mut registers).unwrap();

        // Format à un registre: INC R2
        let inc_instruction = Instruction::create_single_reg(Opcode::Inc, 2);
        let wb_reg_inc = MemoryWritebackRegister {
            instruction: inc_instruction,
            result: 16, // 15 + 1
            rd: Some(2),
        };
        writeback.process_direct(&wb_reg_inc, &mut registers).unwrap();

        // Format à deux registres: MOV R3, R2
        let mov_instruction = Instruction::create_reg_reg(Opcode::Mov, 3, 2);
        let wb_reg_mov = MemoryWritebackRegister {
            instruction: mov_instruction,
            result: 16, // Valeur de R2
            rd: Some(3),
        };
        writeback.process_direct(&wb_reg_mov, &mut registers).unwrap();

        // Vérifier les résultats
        assert_eq!(registers[2], 16); // Valeur après INC
        assert_eq!(registers[3], 16); // Copie de R2
    }
}
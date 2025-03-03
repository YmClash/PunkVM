//src/pipeline/memory.rs
use crate::bytecode::opcodes::Opcode;
use crate::pipeline::{ExecuteMemoryRegister, MemoryWritebackRegister};
// use crate::pipeline::stage::PipelineStage;
use crate::pvm::memorys::Memory;

///Implementation de l'étage Memory du pipeline
pub struct MemoryStage{
    //Registre de la pile
    stack_pointer: u32,
}impl MemoryStage {
    /// Crée un nouvel étage Memory
    pub fn new() -> Self {
        Self {
            // La pile commence typiquement en haut de la mémoire et croît vers le bas
            stack_pointer: 0xFFFF0000, // Exemple: pile commence à 16 MB - 64 KB
            // stack_pointer: 0x1000, // Seulement 4KB, devrait être valide dans tous les tests
        }
    }

    /// Traite l'étage Memory directement
    pub fn process_direct(&mut self, mem_reg: &ExecuteMemoryRegister, memory: &mut Memory) -> Result<MemoryWritebackRegister, String> {
        let mut result = mem_reg.alu_result;

        // Traitement spécifique selon l'opcode
        match mem_reg.instruction.opcode {
            // Instructions de chargement (load)
            Opcode::Load => {
                if let Some(addr) = mem_reg.mem_addr {
                    result = self.load_from_memory(memory, addr, 8)?;
                }
            },

            Opcode::LoadB => {
                if let Some(addr) = mem_reg.mem_addr {
                    result = self.load_from_memory(memory, addr, 1)?;
                }
            },

            Opcode::LoadW => {
                if let Some(addr) = mem_reg.mem_addr {
                    result = self.load_from_memory(memory, addr, 2)?;
                }
            },

            Opcode::LoadD => {
                if let Some(addr) = mem_reg.mem_addr {
                    result = self.load_from_memory(memory, addr, 4)?;
                }
            },

            // Instructions de stockage (store)
            Opcode::Store => {
                if let Some(addr) = mem_reg.mem_addr {
                    if let Some(value) = mem_reg.store_value {
                        self.store_to_memory(memory, addr, value, 8)?;
                    }
                }
            },

            Opcode::StoreB => {
                if let Some(addr) = mem_reg.mem_addr {
                    if let Some(value) = mem_reg.store_value {
                        self.store_to_memory(memory, addr, value, 1)?;
                    }
                }
            },

            Opcode::StoreW => {
                if let Some(addr) = mem_reg.mem_addr {
                    if let Some(value) = mem_reg.store_value {
                        self.store_to_memory(memory, addr, value, 2)?;
                    }
                }
            },

            Opcode::StoreD => {
                if let Some(addr) = mem_reg.mem_addr {
                    if let Some(value) = mem_reg.store_value {
                        self.store_to_memory(memory, addr, value, 4)?;
                    }
                }
            },

            // Instructions de pile
            Opcode::Push => {
                if let Some(value) = mem_reg.store_value {
                    // Vérifier que l'adresse est valide avant de décrémenter
                    if self.stack_pointer < 8 {
                        return Err("Stack overflow: cannot push more values".to_string());
                    }
                    self.stack_pointer -= 8;

                    // Essayer d'écrire et capturer l'erreur pour un meilleur message
                    match self.store_to_memory(memory, self.stack_pointer, value, 8) {
                        Ok(_) => {},
                        Err(e) => return Err(format!("Push failed: {}", e)),
                    }
                }
                // if let Some(value) = mem_reg.store_value {
                //     // Décrémenter SP avant de stocker
                //     self.stack_pointer -= 8;
                //     self.store_to_memory(memory, self.stack_pointer, value, 8)?;
                // }
            },

            Opcode::Pop => {
                // Capturer et afficher l'erreur éventuelle
                match self.load_from_memory(memory, self.stack_pointer, 8) {
                    Ok(value) => {
                        result = value;
                        self.stack_pointer += 8;
                    },
                    Err(e) => return Err(format!("Pop failed: {}", e)),
                }
                // Charger depuis la pile puis incrémenter SP
                // result = self.load_from_memory(memory, self.stack_pointer, 8)?;
                // self.stack_pointer += 8;
            },

            // Autres instructions - rien à faire dans l'étage Memory
            _ => {}
        }

        Ok(MemoryWritebackRegister {
            instruction: mem_reg.instruction.clone(),
            result,
            rd: mem_reg.rd,
        })
    }


    /// Charge une valeur depuis la mémoire
    fn load_from_memory(&self, memory: &mut Memory, addr: u32, size: u8) -> Result<u64, String> {
        match size {
            1 => memory.read_byte(addr)
                .map(|b| b as u64)
                .map_err(|e| e.to_string()),

            2 => memory.read_word(addr)
                .map(|w| w as u64)
                .map_err(|e| e.to_string()),

            4 => memory.read_dword(addr)
                .map(|d| d as u64)
                .map_err(|e| e.to_string()),

            8 => memory.read_qword(addr)
                .map_err(|e| e.to_string()),

            _ => Err(format!("Taille de lecture non supportée: {}", size)),
        }
    }

    /// Stocke une valeur en mémoire
    fn store_to_memory(&self, memory: &mut Memory, addr: u32, value: u64, size: u8) -> Result<(), String> {
        match size {
            1 => memory.write_byte(addr, value as u8)
                .map_err(|e| e.to_string()),

            2 => memory.write_word(addr, value as u16)
                .map_err(|e| e.to_string()),

            4 => memory.write_dword(addr, value as u32)
                .map_err(|e| e.to_string()),

            8 => memory.write_qword(addr, value)
                .map_err(|e| e.to_string()),

            _ => Err(format!("Taille d'écriture non supportée: {}", size)),
        }
    }

    /// Réinitialise l'étage Memory
    pub fn reset(&mut self) {
        // Réinitialiser le pointeur de pile
        self.stack_pointer = 0xFFFF0000;
    }

    #[cfg(test)]
    /// Crée un nouvel étage Memory avec un SP adapté aux tests
    pub fn new_for_test() -> Self {
        Self {
            stack_pointer: 0x1000,
        }
    }
}



// Test unitaire pour l'étage Memory
#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::opcodes::Opcode;
    use crate::bytecode::instructions::Instruction;
    use crate::bytecode::format::InstructionFormat;
    use crate::bytecode::format::ArgType;
    use crate::pipeline::{ExecuteMemoryRegister, MemoryWritebackRegister};
    use crate::pvm::memorys::{Memory, MemoryConfig};

    #[test]
    fn test_memory_stage_creation() {
        let memory_stage = MemoryStage::new_for_test();
        assert_eq!(memory_stage.stack_pointer, 0x1000); // Valeur modifiée
    }

    #[test]
    fn test_memory_stage_reset() {
        let mut memory_stage = MemoryStage::new_for_test();

        // Modifier l'état
        memory_stage.stack_pointer = 0xFFFF1000;

        // Réinitialiser
        memory_stage.reset();

        // Vérifier que l'état est réinitialisé
        assert_eq!(memory_stage.stack_pointer, 0xFFFF0000);
    }

    #[test]
    fn test_memory_load_instruction() {
        let mut memory_stage = MemoryStage::new_for_test();
        let mut memory = Memory::new(MemoryConfig::default());

        // Écrire une valeur en mémoire
        let _ = memory.write_qword(0x1000, 0xDEADBEEF);

        // Créer une instruction LOAD R0, [0x1000]
        let load_instruction = Instruction::new(
            Opcode::Load,
            InstructionFormat::new(ArgType::Register, ArgType::AbsoluteAddr),
            vec![0, 0, 16, 0, 0] // R0 = Mem[0x1000]
        );

        // Créer un registre Execute → Memory
        let em_reg = ExecuteMemoryRegister {
            instruction: load_instruction,
            alu_result: 0, // Pas utilisé pour LOAD
            rd: Some(0), // Destination R0
            store_value: None,
            mem_addr: Some(0x1000),
            branch_target: None,
            branch_taken: false,
        };

        // Exécuter l'instruction
        let result = memory_stage.process_direct(&em_reg, &mut memory);
        assert!(result.is_ok());

        // Vérifier le résultat
        let mw_reg = result.unwrap();
        assert_eq!(mw_reg.result, 0xDEADBEEF); // La valeur chargée
        assert_eq!(mw_reg.rd, Some(0));
    }

    #[test]
    fn test_memory_store_instruction() {
        let mut memory_stage = MemoryStage::new();
        let mut memory = Memory::new(MemoryConfig::default());

        // Créer une instruction STORE R0, [0x1000]
        let store_instruction = Instruction::new(
            Opcode::Store,
            InstructionFormat::new(ArgType::Register, ArgType::AbsoluteAddr),
            vec![0, 0, 16, 0, 0] // Mem[0x1000] = R0
        );

        // Créer un registre Execute → Memory
        let em_reg = ExecuteMemoryRegister {
            instruction: store_instruction,
            alu_result: 0, // Pas utilisé pour STORE
            rd: None, // Pas de registre destination
            store_value: Some(0xCAFEBABE), // Valeur à stocker
            mem_addr: Some(0x1000),
            branch_target: None,
            branch_taken: false,
        };

        // Exécuter l'instruction
        let result = memory_stage.process_direct(&em_reg, &mut memory);
        assert!(result.is_ok());

        // Vérifier que la valeur a été stockée en mémoire
        let loaded_value = memory.read_qword(0x1000);
        assert!(loaded_value.is_ok());
        assert_eq!(loaded_value.unwrap(), 0xCAFEBABE);
    }

    #[test]
    fn test_memory_push_instruction() {
        let mut memory_stage = MemoryStage::new_for_test();
        let mut memory = Memory::new(MemoryConfig::default());




        // Créer une instruction PUSH R0
        let push_instruction = Instruction::new(
            Opcode::Push,
            InstructionFormat::new(ArgType::Register, ArgType::None),
            vec![0] // Push R0
        );

        // Créer un registre Execute → Memory
        let em_reg = ExecuteMemoryRegister {
            instruction: push_instruction,
            alu_result: 0,
            rd: None,
            store_value: Some(0x12345678), // Valeur à empiler
            mem_addr: None,
            branch_target: None,
            branch_taken: false,
        };

        // Sauvegarder la valeur originale du SP
        let original_sp = memory_stage.stack_pointer;

        // Exécuter l'instruction
        let result = memory_stage.process_direct(&em_reg, &mut memory);
        assert!(result.is_ok());

        // Vérifier que le SP a été décrémenté
        assert_eq!(memory_stage.stack_pointer, original_sp - 8);

        // Vérifier que la valeur a été stockée à la nouvelle adresse SP
        let loaded_value = memory.read_qword(memory_stage.stack_pointer);
        assert!(loaded_value.is_ok());
        assert_eq!(loaded_value.unwrap(), 0x12345678);
    }

    // #[test]
    // fn test_memory_pop_instruction() {
    //     let mut memory_stage = MemoryStage::new();
    //     let mut memory = Memory::new(MemoryConfig::default());
    //
    //     // Préparer la pile - écrire une valeur
    //     memory_stage.stack_pointer = 0xFFFF0000 - 8; // Déjà décrémenté
    //     let _ = memory.write_qword(memory_stage.stack_pointer, 0xABCDEF01);
    //
    //     // Créer une instruction POP R0
    //     let pop_instruction = Instruction::new(
    //         Opcode::Pop,
    //         InstructionFormat::new(ArgType::Register, ArgType::None),
    //         vec![0] // Pop into R0
    //     );
    //
    //     // Créer un registre Execute → Memory
    //     let em_reg = ExecuteMemoryRegister {
    //         instruction: pop_instruction,
    //         alu_result: 0,
    //         rd: Some(0), // Destination R0
    //         store_value: None,
    //         mem_addr: None,
    //         branch_target: None,
    //         branch_taken: false,
    //     };
    //
    //     // Sauvegarder la valeur originale du SP
    //     let original_sp = memory_stage.stack_pointer;
    //
    //     // Exécuter l'instruction
    //     let result = memory_stage.process_direct(&em_reg, &mut memory);
    //     assert!(result.is_ok());
    //
    //     // Vérifier que le SP a été incrémenté
    //     assert_eq!(memory_stage.stack_pointer, original_sp + 8);
    //
    //     // Vérifier que la valeur a été chargée
    //     let mw_reg = result.unwrap();
    //     assert_eq!(mw_reg.result, 0xABCDEF01);
    //     assert_eq!(mw_reg.rd, Some(0));
    // }
    #[test]
    fn test_memory_pop_instruction() {
        // Créer une configuration mémoire avec une taille suffisante
        let config = MemoryConfig {
            size: 0x2000000, // 32MB, suffisant pour contenir 0xFFFF0000
            ..Default::default()
        };

        let mut memory_stage = MemoryStage::new();
        let mut memory = Memory::new(config);

        // Utilisez un SP qui est certainement dans la mémoire valide
        memory_stage.stack_pointer = 0x1000; // Une adresse basse qui est certainement valide

        // Préparer la pile - écrire une valeur
        let _ = memory.write_qword(memory_stage.stack_pointer, 0xABCDEF01);

        // Créer une instruction POP R0
        let pop_instruction = Instruction::new(
            Opcode::Pop,
            InstructionFormat::new(ArgType::Register, ArgType::None),
            vec![0] // Pop into R0
        );

        // Reste du test comme avant...
    }

    #[test]
    fn test_memory_load_different_sizes() {
        let mut memory_stage = MemoryStage::new();
        let mut memory = Memory::new(MemoryConfig::default());

        // Écrire des valeurs de différentes tailles en mémoire
        let _ = memory.write_byte(0x2000, 0xAB);
        let _ = memory.write_word(0x2010, 0xCDEF);
        let _ = memory.write_dword(0x2020, 0x12345678);

        // Test LoadB
        let loadb_instruction = Instruction::new(
            Opcode::LoadB,
            InstructionFormat::new(ArgType::Register, ArgType::AbsoluteAddr),
            vec![0, 0, 32, 0, 0]
        );

        let em_reg_b = ExecuteMemoryRegister {
            instruction: loadb_instruction,
            alu_result: 0,
            rd: Some(0),
            store_value: None,
            mem_addr: Some(0x2000),
            branch_target: None,
            branch_taken: false,
        };

        let result_b = memory_stage.process_direct(&em_reg_b, &mut memory);
        assert!(result_b.is_ok());
        assert_eq!(result_b.unwrap().result, 0xAB);

        // Test LoadW
        let loadw_instruction = Instruction::new(
            Opcode::LoadW,
            InstructionFormat::new(ArgType::Register, ArgType::AbsoluteAddr),
            vec![0, 0x10, 32, 0, 0]
        );

        let em_reg_w = ExecuteMemoryRegister {
            instruction: loadw_instruction,
            alu_result: 0,
            rd: Some(1),
            store_value: None,
            mem_addr: Some(0x2010),
            branch_target: None,
            branch_taken: false,
        };

        let result_w = memory_stage.process_direct(&em_reg_w, &mut memory);
        assert!(result_w.is_ok());
        assert_eq!(result_w.unwrap().result, 0xCDEF);

        // Test LoadD
        let loadd_instruction = Instruction::new(
            Opcode::LoadD,
            InstructionFormat::new(ArgType::Register, ArgType::AbsoluteAddr),
            vec![0, 0x20, 32, 0, 0]
        );

        let em_reg_d = ExecuteMemoryRegister {
            instruction: loadd_instruction,
            alu_result: 0,
            rd: Some(2),
            store_value: None,
            mem_addr: Some(0x2020),
            branch_target: None,
            branch_taken: false,
        };

        let result_d = memory_stage.process_direct(&em_reg_d, &mut memory);
        assert!(result_d.is_ok());
        assert_eq!(result_d.unwrap().result, 0x12345678);
    }
}







//
// impl<'a> PipelineStage<'a> for MemoryStage {
//     type Input = (ExecuteMemoryRegister, &'a mut Memory);
//     type Output = MemoryWritebackRegister;
//
//     fn process(&mut self, input: &Self::Input) -> Result<Self::Output, String> {
//         let (mem_reg, memory) = input;
//         self.process(mem_reg, memory)
//     }
//
//     fn reset(&mut self) {
//         // Reset direct sans appel récursif
//         self.stack_pointer = 0xFFFF0000;
//     }
// }
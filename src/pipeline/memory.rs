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
                    // Décrémenter SP avant de stocker
                    self.stack_pointer -= 8;
                    self.store_to_memory(memory, self.stack_pointer, value, 8)?;
                }
            },

            Opcode::Pop => {
                // Charger depuis la pile puis incrémenter SP
                result = self.load_from_memory(memory, self.stack_pointer, 8)?;
                self.stack_pointer += 8;
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

    // /// Traite l'étage Memory
    // pub fn process(&mut self, mem_reg: &ExecuteMemoryRegister, memory: &mut Memory) -> Result<MemoryWritebackRegister, String> {
    //     let mut result = mem_reg.alu_result;
    //
    //     // Traitement spécifique selon l'opcode
    //     match mem_reg.instruction.opcode {
    //         // Instructions de chargement (load)
    //         Opcode::Load => {
    //             if let Some(addr) = mem_reg.mem_addr {
    //                 result = self.load_from_memory(memory, addr, 8)?;
    //             }
    //         },
    //
    //         Opcode::LoadB => {
    //             if let Some(addr) = mem_reg.mem_addr {
    //                 result = self.load_from_memory(memory, addr, 1)?;
    //             }
    //         },
    //
    //         Opcode::LoadW => {
    //             if let Some(addr) = mem_reg.mem_addr {
    //                 result = self.load_from_memory(memory, addr, 2)?;
    //             }
    //         },
    //
    //         Opcode::LoadD => {
    //             if let Some(addr) = mem_reg.mem_addr {
    //                 result = self.load_from_memory(memory, addr, 4)?;
    //             }
    //         },
    //
    //         // Instructions de stockage (store)
    //         Opcode::Store => {
    //             if let Some(addr) = mem_reg.mem_addr {
    //                 if let Some(value) = mem_reg.store_value {
    //                     self.store_to_memory(memory, addr, value, 8)?;
    //                 }
    //             }
    //         },
    //
    //         Opcode::StoreB => {
    //             if let Some(addr) = mem_reg.mem_addr {
    //                 if let Some(value) = mem_reg.store_value {
    //                     self.store_to_memory(memory, addr, value, 1)?;
    //                 }
    //             }
    //         },
    //
    //         Opcode::StoreW => {
    //             if let Some(addr) = mem_reg.mem_addr {
    //                 if let Some(value) = mem_reg.store_value {
    //                     self.store_to_memory(memory, addr, value, 2)?;
    //                 }
    //             }
    //         },
    //
    //         Opcode::StoreD => {
    //             if let Some(addr) = mem_reg.mem_addr {
    //                 if let Some(value) = mem_reg.store_value {
    //                     self.store_to_memory(memory, addr, value, 4)?;
    //                 }
    //             }
    //         },
    //
    //         // Instructions de pile
    //         Opcode::Push => {
    //             if let Some(value) = mem_reg.store_value {
    //                 // Décrémenter SP avant de stocker
    //                 self.stack_pointer -= 8;
    //                 self.store_to_memory(memory, self.stack_pointer, value, 8)?;
    //             }
    //         },
    //
    //         Opcode::Pop => {
    //             // Charger depuis la pile puis incrémenter SP
    //             result = self.load_from_memory(memory, self.stack_pointer, 8)?;
    //             self.stack_pointer += 8;
    //         },
    //
    //         // Autres instructions - rien à faire dans l'étage Memory
    //         _ => {}
    //     }
    //
    //     Ok(MemoryWritebackRegister {
    //         instruction: mem_reg.instruction.clone(),
    //         result,
    //         rd: mem_reg.rd,
    //     })
    // }

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
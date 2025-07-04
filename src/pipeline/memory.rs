//src/pipeline/memory.rs
use crate::bytecode::opcodes::Opcode;
use crate::pipeline::{ExecuteMemoryRegister, MemoryWritebackRegister};
// use crate::pipeline::stage::PipelineStage;
use crate::pvm::memorys::Memory;

/// Entrée du store buffer pour Store-Load forwarding
#[derive(Debug, Clone)]
pub struct StoreBufferEntry {
    pub address: u32,
    pub value: u64,
    pub size: u8,
    pub cycle: u64,
}

///Implementation de l'étage Memory du pipeline
pub struct MemoryStage {
    //Registre de la pile
    stack_pointer: u32,
    
    /// Store buffer pour Store-Load forwarding
    store_buffer: Vec<StoreBufferEntry>,
    
    /// Statistiques de Store-Load forwarding
    pub store_load_forwards: u64,
    pub store_load_attempts: u64,
    
    // Compteurs pour les statistiques de pile
    pub stack_pushes: u64,
    pub stack_pops: u64,
    pub stack_overflow_attempts: u64,
    pub stack_underflow_attempts: u64,
}
impl MemoryStage {
    /// Crée un nouvel étage Memory
    pub fn new() -> Self {
        Self {
            // La pile commence typiquement en haut de la mémoire et croît vers le bas
            stack_pointer: 0xFFFF0000, // Exemple: pile commence à 16 MB - 64 KB
                                       // stack_pointer: 0x1000, // Seulement 4KB, devrait être valide dans tous les tests
            
            // Initialiser le store buffer
            store_buffer: Vec::with_capacity(8),
            
            // Statistiques de Store-Load forwarding
            store_load_forwards: 0,
            store_load_attempts: 0,
            
            // Initialiser les compteurs de statistiques
            stack_pushes: 0,
            stack_pops: 0,
            stack_overflow_attempts: 0,
            stack_underflow_attempts: 0,
        }
    }

    /// Traite l'étage Memory directement
    pub fn process_direct(
        &mut self,
        mem_reg: &ExecuteMemoryRegister,
        memory: &mut Memory,
        registers: &mut [u64],
    ) -> Result<MemoryWritebackRegister, String> {
        let mut result = mem_reg.alu_result;
        
        // Nettoyage du store buffer (supprime les entrées anciennes)
        self.cleanup_store_buffer();

        // Traitement spécifique selon l'opcode
        match mem_reg.instruction.opcode {
            // Instructions de chargement (load)
            Opcode::Load => {
                if let Some(addr) = mem_reg.mem_addr {
                    // Tenter Store-Load forwarding d'abord
                    if let Some(forwarded_value) = self.check_store_load_forwarding(addr, 8) {
                        result = forwarded_value;
                        self.store_load_forwards += 1;
                        println!("[Store-Load Forwarding] Load from address: {:#X}, forwarded value: {:#X}", addr, result);
                    } else {
                        result = self.load_from_memory(memory, addr, 8)?;
                        println!("Load from address: {:#X}, result: {:#X}", addr, result);
                    }
                    self.store_load_attempts += 1;
                    println!(
                        "MemoryStage: Instruction opcode={:?}, mem_addr={:?}, store_value={:?}",
                        mem_reg.instruction.opcode, mem_reg.mem_addr, mem_reg.store_value
                    );
                }
            }

            Opcode::LoadB => {
                if let Some(addr) = mem_reg.mem_addr {
                    if let Some(forwarded_value) = self.check_store_load_forwarding(addr, 1) {
                        result = forwarded_value;
                        self.store_load_forwards += 1;
                        println!("[Store-Load Forwarding] LoadB from address: {:#X}, forwarded value: {:#X}", addr, result);
                    } else {
                        result = self.load_from_memory(memory, addr, 1)?;
                        println!("LoadB from address: {:#X}, result: {:#X}", addr, result);
                    }
                    self.store_load_attempts += 1;
                    println!(
                        "MemoryStage: Instruction opcode={:?}, mem_addr={:?}, store_value={:?}",
                        mem_reg.instruction.opcode, mem_reg.mem_addr, mem_reg.store_value
                    );
                }
            }

            Opcode::LoadW => {
                if let Some(addr) = mem_reg.mem_addr {
                    if let Some(forwarded_value) = self.check_store_load_forwarding(addr, 2) {
                        result = forwarded_value;
                        self.store_load_forwards += 1;
                        println!("[Store-Load Forwarding] LoadW from address: {:#X}, forwarded value: {:#X}", addr, result);
                    } else {
                        result = self.load_from_memory(memory, addr, 2)?;
                        println!("LoadW from address: {:#X}, result: {:#X}", addr, result);
                    }
                    self.store_load_attempts += 1;
                    println!(
                        "MemoryStage: Instruction opcode={:?}, mem_addr={:?}, store_value={:?}",
                        mem_reg.instruction.opcode, mem_reg.mem_addr, mem_reg.store_value
                    );
                }
            }

            Opcode::LoadD => {
                if let Some(addr) = mem_reg.mem_addr {
                    if let Some(forwarded_value) = self.check_store_load_forwarding(addr, 4) {
                        result = forwarded_value;
                        self.store_load_forwards += 1;
                        println!("[Store-Load Forwarding] LoadD from address: {:#X}, forwarded value: {:#X}", addr, result);
                    } else {
                        result = self.load_from_memory(memory, addr, 4)?;
                        println!("LoadD from address: {:#X}, result: {:#X}", addr, result);
                    }
                    self.store_load_attempts += 1;
                    println!(
                        "MemoryStage: Instruction opcode={:?}, mem_addr={:?}, store_value={:?}",
                        mem_reg.instruction.opcode, mem_reg.mem_addr, mem_reg.store_value
                    );
                }
            }

            // Instructions de stockage (store)
            Opcode::Store => {
                if let Some(addr) = mem_reg.mem_addr {
                    if let Some(value) = mem_reg.store_value {
                        // Ajouter au store buffer pour Store-Load forwarding
                        self.add_to_store_buffer(addr, value, 8);
                        self.store_to_memory(memory, addr, value, 8)?;
                        println!("Store to address: {:#X}, value: {:#X}", addr, value);
                        println!(
                            "MemoryStage: Instruction opcode={:?}, mem_addr={:?}, store_value={:?}",
                            mem_reg.instruction.opcode, mem_reg.mem_addr, mem_reg.store_value
                        );
                    }
                }
            }

            Opcode::StoreB => {
                if let Some(addr) = mem_reg.mem_addr {
                    if let Some(value) = mem_reg.store_value {
                        self.add_to_store_buffer(addr, value, 1);
                        self.store_to_memory(memory, addr, value, 1)?;
                        println!("StoreB to address: {:#X}, value: {:#X}", addr, value);
                        println!(
                            "MemoryStage: Instruction opcode={:?}, mem_addr={:?}, store_value={:?}",
                            mem_reg.instruction.opcode, mem_reg.mem_addr, mem_reg.store_value
                        );
                    }
                }
            }

            Opcode::StoreW => {
                if let Some(addr) = mem_reg.mem_addr {
                    if let Some(value) = mem_reg.store_value {
                        self.add_to_store_buffer(addr, value, 2);
                        self.store_to_memory(memory, addr, value, 2)?;
                        println!("StoreW to address: {:#X}, value: {:#X}", addr, value);
                        println!(
                            "MemoryStage: Instruction opcode={:?}, mem_addr={:?}, store_value={:?}",
                            mem_reg.instruction.opcode, mem_reg.mem_addr, mem_reg.store_value
                        );
                    }
                }
            }

            Opcode::StoreD => {
                if let Some(addr) = mem_reg.mem_addr {
                    if let Some(value) = mem_reg.store_value {
                        self.add_to_store_buffer(addr, value, 4);
                        self.store_to_memory(memory, addr, value, 4)?;
                        println!("StoreD to address: {:#X}, value: {:#X}", addr, value);
                        println!(
                            "MemoryStage: Instruction opcode={:?}, mem_addr={:?}, store_value={:?}",
                            mem_reg.instruction.opcode, mem_reg.mem_addr, mem_reg.store_value
                        );
                    }
                }
            }

            // Instructions de pile
            Opcode::Push => {
                if let Some(value) = mem_reg.store_value {
                    // Utiliser le Stack Pointer des registres (SP = registre 16)
                    let sp = registers[16] as u32;
                    
                    // Vérifier stack overflow
                    if sp < 8 {
                        self.stack_overflow_attempts += 1;
                        return Err("Stack overflow: cannot push more values".to_string());
                    }
                    
                    // Décrémenter SP de 8 (64-bit values)
                    let new_sp = sp - 8;
                    registers[16] = new_sp as u64;
                    
                    println!(
                        "Push to address: {:#X}, value: {:#X}",
                        new_sp, value
                    );
                    println!(
                        "MemoryStage: Instruction opcode={:?}, mem_addr={:?}, store_value={:?}",
                        mem_reg.instruction.opcode, mem_reg.mem_addr, mem_reg.store_value
                    );

                    // Essayer d'écrire et capturer l'erreur pour un meilleur message
                    match self.store_to_memory(memory, new_sp, value, 8) {
                        Ok(_) => {
                            // Incrémenter le compteur de push réussis
                            self.stack_pushes += 1;
                        }
                        Err(e) => return Err(format!("Push failed: {}", e)),
                    }
                }
            }

            Opcode::Pop => {
                // Utiliser le Stack Pointer des registres (SP = registre 16)
                let sp = registers[16] as u32;
                
                // Capturer et afficher l'erreur éventuelle
                match self.load_from_memory(memory, sp, 8) {
                    Ok(value) => {
                        result = value;
                        // Incrémenter SP de 8
                        registers[16] = (sp + 8) as u64;
                        
                        // Incrémenter le compteur de pop réussis
                        self.stack_pops += 1;
                        
                        println!(
                            "Pop from address: {:#X}, result: {:#X}",
                            sp, result
                        );
                        println!(
                            "MemoryStage: Instruction opcode={:?}, mem_addr={:?}, store_value={:?}",
                            mem_reg.instruction.opcode, mem_reg.mem_addr, mem_reg.store_value
                        );
                    }
                    Err(e) => {
                        self.stack_underflow_attempts += 1;
                        return Err(format!("Pop failed: {}", e));
                    }
                }
            }

            Opcode::Halt => {
                // Si l'instruction est un halt, on ne fait rien
                println!("Halt instruction encountered");
                println!(
                    "MemoryStage: Instruction opcode={:?}, mem_addr={:?}, store_value={:?}",
                    mem_reg.instruction.opcode, mem_reg.mem_addr, mem_reg.store_value
                );
            }

            // Autres instructions - rien à faire dans l'étage Memory
            _ => {}
        }

        // println!("MemoryStage: Instruction opcode={:?}, mem_addr={:?}, store_value={:?}",
        //          mem_reg.instruction.opcode, mem_reg.mem_addr, mem_reg.store_value
        // );

        Ok(MemoryWritebackRegister {
            instruction: mem_reg.instruction.clone(),
            result,
            rd: mem_reg.rd,
        })
    }

    /// Charge une valeur depuis la mémoire
    fn load_from_memory(&self, memory: &mut Memory, addr: u32, size: u8) -> Result<u64, String> {
        match size {
            1 => memory
                .read_byte(addr)
                .map(|b| b as u64)
                .map_err(|e| e.to_string()),

            2 => memory
                .read_word(addr)
                .map(|w| w as u64)
                .map_err(|e| e.to_string()),

            4 => memory
                .read_dword(addr)
                .map(|d| d as u64)
                .map_err(|e| e.to_string()),

            8 => memory.read_qword(addr).map_err(|e| e.to_string()),

            _ => Err(format!("Taille de lecture non supportée: {}", size)),
        }
    }

    /// Stocke une valeur en mémoire
    fn store_to_memory(
        &self,
        memory: &mut Memory,
        addr: u32,
        value: u64,
        size: u8,
    ) -> Result<(), String> {
        match size {
            1 => memory
                .write_byte(addr, value as u8)
                .map_err(|e| e.to_string()),

            2 => memory
                .write_word(addr, value as u16)
                .map_err(|e| e.to_string()),

            4 => memory
                .write_dword(addr, value as u32)
                .map_err(|e| e.to_string()),

            8 => memory.write_qword(addr, value).map_err(|e| e.to_string()),

            _ => Err(format!("Taille d'écriture non supportée: {}", size)),
        }
    }

    /// Ajoute une entrée au store buffer
    fn add_to_store_buffer(&mut self, address: u32, value: u64, size: u8) {
        // Maintenir la taille du buffer (FIFO)
        if self.store_buffer.len() >= 8 {
            self.store_buffer.remove(0);
        }
        
        self.store_buffer.push(StoreBufferEntry {
            address,
            value,
            size,
            cycle: 0, // Sera mis à jour si nécessaire
        });
        
        println!("[Store Buffer] Added entry: addr={:#X}, value={:#X}, size={}", address, value, size);
    }
    
    /// Vérifie si un load peut être forwardé depuis le store buffer
    fn check_store_load_forwarding(&self, load_addr: u32, load_size: u8) -> Option<u64> {
        // Chercher dans le store buffer en ordre inverse (plus récent en premier)
        for entry in self.store_buffer.iter().rev() {
            if entry.address == load_addr {
                // Vérifier la compatibilité des tailles
                if entry.size >= load_size {
                    // Extraire la valeur selon la taille du load
                    let forwarded_value = match load_size {
                        1 => entry.value & 0xFF,
                        2 => entry.value & 0xFFFF,
                        4 => entry.value & 0xFFFFFFFF,
                        8 => entry.value,
                        _ => return None,
                    };
                    
                    println!("[Store-Load Forwarding] Hit: load_addr={:#X}, forwarded={:#X}", 
                            load_addr, forwarded_value);
                    return Some(forwarded_value);
                }
            }
        }
        None
    }
    
    /// Nettoie le store buffer (supprime les entrées anciennes)
    fn cleanup_store_buffer(&mut self) {
        // Pour l'instant, on garde toutes les entrées récentes
        // Une implémentation plus sophistiquée pourrait utiliser un TTL
    }
    
    /// Réinitialise l'étage Memory
    pub fn reset(&mut self) {
        // Réinitialiser le pointeur de pile
        self.stack_pointer = 0xFFFF0000;
        
        // Vider le store buffer
        self.store_buffer.clear();
        
        // Réinitialiser les compteurs de statistiques
        self.store_load_forwards = 0;
        self.store_load_attempts = 0;
        self.stack_pushes = 0;
        self.stack_pops = 0;
        self.stack_overflow_attempts = 0;
        self.stack_underflow_attempts = 0;
    }

    /// Retourne les statistiques de Store-Load forwarding
    pub fn get_store_load_stats(&self) -> (u64, u64) {
        (self.store_load_forwards, self.store_load_attempts)
    }
    
    #[cfg(test)]
    /// Crée un nouvel étage Memory avec un SP adapté aux tests
    pub fn new_for_test() -> Self {
        Self {
            stack_pointer: 0x1000,
            store_buffer: Vec::with_capacity(8),
            store_load_forwards: 0,
            store_load_attempts: 0,
            stack_pushes: 0,
            stack_pops: 0,
            stack_overflow_attempts: 0,
            stack_underflow_attempts: 0,
        }
    }
}
//
// // // Test unitaire pour l'étage Memory
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::bytecode::format::{ArgType, InstructionFormat};
//     use crate::bytecode::instructions::Instruction;
//     use crate::bytecode::opcodes::Opcode;
//     use crate::pipeline::{ExecuteMemoryRegister, MemoryWritebackRegister};
//     use crate::pvm::memorys::{Memory, MemoryConfig};
//
//     #[test]
//     fn test_memory_stage_creation() {
//         let memory_stage = MemoryStage::new_for_test();
//         assert_eq!(memory_stage.stack_pointer, 0x1000);
//     }
//
//     #[test]
//     fn test_memory_stage_reset() {
//         let mut memory_stage = MemoryStage::new_for_test();
//         memory_stage.stack_pointer = 0x2000;
//         memory_stage.reset();
//         assert_eq!(memory_stage.stack_pointer, 0xFFFF0000);
//     }
//
//     #[test]
//     fn test_memory_load_with_three_register_format() {
//         let mut memory_stage = MemoryStage::new_for_test();
//         let mut memory = Memory::new(MemoryConfig::default());
//
//         // Écrire une valeur à l'adresse 0x2000
//         let _ = memory.write_qword(0x2000, 0x0123456789ABCDEF);
//
//         // Créer une instruction LOAD R2, [R0+R1] (format à trois registres)
//         let load_instruction = Instruction::create_reg_reg_reg(
//             Opcode::Load,
//             2, // Rd  (destination)
//             0, // Rs1 (base)
//             1, // Rs2 (offset)
//         );
//
//         // Dans l'étage Execute, les adresses sont calculées et transmises à l'étage Memory
//         let em_reg = ExecuteMemoryRegister {
//             instruction: load_instruction,
//             alu_result: 0, // Non utilisé pour LOAD
//             rd: Some(2),   // Registre destination R2
//             store_value: None,
//             mem_addr: Some(0x2000), // Adresse calculée (R0+R1)
//             branch_target: None,
//             branch_taken: false,
//             branch_prediction_correct: Option::from(false),
//
//             stack_operation: None,
//             stack_result: None,
//             ras_prediction_correct: None,
//             halted: false,
//         };
//
//         // Exécuter l'instruction dans l'étage Memory
//         let result = memory_stage.process_direct(&em_reg, &mut memory);
//         assert!(result.is_ok());
//
//         // Vérifier le résultat
//         let mw_reg = result.unwrap();
//         assert_eq!(mw_reg.result, 0x0123456789ABCDEF); // Valeur chargée depuis la mémoire
//         assert_eq!(mw_reg.rd, Some(2)); // Destination R2
//     }
//
//     #[test]
//     fn test_memory_store_with_three_register_format() {
//         let mut memory_stage = MemoryStage::new_for_test();
//         let mut memory = Memory::new(MemoryConfig::default());
//
//         // Créer une instruction STORE R0, [R1+R2] (format à trois registres)
//         let store_instruction = Instruction::create_reg_reg_reg(
//             Opcode::Store,
//             0, // Rs (source de la valeur)
//             1, // Rd (base de l'adresse)
//             2, // Rt (offset de l'adresse)
//         );
//
//         // Dans l'étage Execute, les adresses sont calculées et la valeur à stocker est préparée
//         let em_reg = ExecuteMemoryRegister {
//             instruction: store_instruction,
//             alu_result: 0,                         // Non utilisé pour STORE
//             rd: None,                              // Pas de registre destination pour STORE
//             store_value: Some(0xFEDCBA9876543210), // Valeur de R0 à stocker
//             mem_addr: Some(0x3000),                // Adresse calculée (R1+R2)
//             branch_target: None,
//             branch_taken: false,
//             branch_prediction_correct: Option::from(false),
//
//             stack_operation: None,
//             stack_result: None,
//             ras_prediction_correct: None,
//             halted: false,
//         };
//
//         // Exécuter l'instruction dans l'étage Memory
//         let result = memory_stage.process_direct(&em_reg, &mut memory);
//         assert!(result.is_ok());
//
//         // Vérifier que la valeur a été correctement stockée
//         let loaded_value = memory.read_qword(0x3000);
//         assert!(loaded_value.is_ok());
//         assert_eq!(loaded_value.unwrap(), 0xFEDCBA9876543210);
//     }
//
//     #[test]
//     fn test_memory_load_store_sequence_with_three_register_format() {
//         let mut memory_stage = MemoryStage::new_for_test();
//         let mut memory = Memory::new(MemoryConfig::default());
//
//         // Simuler une séquence d'instructions:
//         // 1. STORE R0, [R1+R2] - Stocker une valeur à l'adresse calculée
//         // 2. LOAD R3, [R1+R2]  - Charger la même valeur dans un autre registre
//
//         // Étape 1: STORE R0, [R1+R2]
//         let store_instruction = Instruction::create_reg_reg_reg(Opcode::Store, 0, 1, 2);
//
//         let em_reg_store = ExecuteMemoryRegister {
//             instruction: store_instruction,
//             alu_result: 0,
//             rd: None,
//             store_value: Some(0xAABBCCDDEEFF0011),
//             mem_addr: Some(0x4000), // Adresse calculée (R1+R2)
//             branch_target: None,
//             branch_taken: false,
//             branch_prediction_correct: Option::from(false),
//             stack_operation: None,
//             stack_result: None,
//             ras_prediction_correct: None,
//             halted: false,
//         };
//
//         // Exécuter STORE
//         let result_store = memory_stage.process_direct(&em_reg_store, &mut memory);
//         assert!(result_store.is_ok());
//
//         // Étape 2: LOAD R3, [R1+R2]
//         let load_instruction = Instruction::create_reg_reg_reg(Opcode::Load, 3, 1, 2);
//
//         let em_reg_load = ExecuteMemoryRegister {
//             instruction: load_instruction,
//             alu_result: 0,
//             rd: Some(3),
//             store_value: None,
//             mem_addr: Some(0x4000), // Même adresse (R1+R2)
//             branch_target: None,
//             branch_taken: false,
//             branch_prediction_correct: Option::from(false),
//             stack_operation: None,
//             stack_result: None,
//             ras_prediction_correct: None,
//             halted: false,
//         };
//
//         // Exécuter LOAD
//         let result_load = memory_stage.process_direct(&em_reg_load, &mut memory);
//         assert!(result_load.is_ok());
//
//         // Vérifier que la valeur chargée correspond à celle stockée
//         let mw_reg_load = result_load.unwrap();
//         assert_eq!(mw_reg_load.result, 0xAABBCCDDEEFF0011);
//         assert_eq!(mw_reg_load.rd, Some(3));
//     }
//
//     #[test]
//     fn test_memory_different_sizes_with_three_register_format() {
//         let mut memory_stage = MemoryStage::new_for_test();
//         let mut memory = Memory::new(MemoryConfig::default());
//
//         // Écrire des valeurs de différentes tailles
//         let _ = memory.write_byte(0x5000, 0xAB);
//         let _ = memory.write_word(0x5100, 0xCDEF);
//         let _ = memory.write_dword(0x5200, 0x01234567);
//
//         // Tester LoadB avec format à trois registres
//         let loadb_instruction = Instruction::create_reg_reg_reg(Opcode::LoadB, 3, 0, 1);
//
//         let em_reg_loadb = ExecuteMemoryRegister {
//             instruction: loadb_instruction,
//             alu_result: 0,
//             rd: Some(3),
//             store_value: None,
//             mem_addr: Some(0x5000), // Adresse calculée
//             branch_target: None,
//             branch_taken: false,
//             branch_prediction_correct: Option::from(false),
//             stack_operation: None,
//             stack_result: None,
//             ras_prediction_correct: None,
//             halted: false,
//         };
//
//         let result_loadb = memory_stage.process_direct(&em_reg_loadb, &mut memory);
//         assert!(result_loadb.is_ok());
//         assert_eq!(result_loadb.unwrap().result, 0xAB);
//
//         // Tester LoadW avec format à trois registres
//         let loadw_instruction = Instruction::create_reg_reg_reg(Opcode::LoadW, 4, 0, 1);
//
//         let em_reg_loadw = ExecuteMemoryRegister {
//             instruction: loadw_instruction,
//             alu_result: 0,
//             rd: Some(4),
//             store_value: None,
//             mem_addr: Some(0x5100), // Adresse calculée
//             branch_target: None,
//             branch_taken: false,
//             branch_prediction_correct: Option::from(false),
//             stack_operation: None,
//             stack_result: None,
//             ras_prediction_correct: None,
//             halted: false,
//         };
//
//         let result_loadw = memory_stage.process_direct(&em_reg_loadw, &mut memory);
//         assert!(result_loadw.is_ok());
//         assert_eq!(result_loadw.unwrap().result, 0xCDEF);
//
//         // Tester LoadD avec format à trois registres
//         let loadd_instruction = Instruction::create_reg_reg_reg(Opcode::LoadD, 5, 0, 1);
//
//         let em_reg_loadd = ExecuteMemoryRegister {
//             instruction: loadd_instruction,
//             alu_result: 0,
//             rd: Some(5),
//             store_value: None,
//             mem_addr: Some(0x5200), // Adresse calculée
//             branch_target: None,
//             branch_taken: false,
//             branch_prediction_correct: Option::from(false),
//             stack_operation: None,
//             stack_result: None,
//             ras_prediction_correct: None,
//             halted: false,
//         };
//
//         let result_loadd = memory_stage.process_direct(&em_reg_loadd, &mut memory);
//         assert!(result_loadd.is_ok());
//         assert_eq!(result_loadd.unwrap().result, 0x01234567);
//     }
//
//     #[test]
//     fn test_memory_store_different_sizes_with_three_register_format() {
//         let mut memory_stage = MemoryStage::new_for_test();
//         let mut memory = Memory::new(MemoryConfig::default());
//
//         // Tester StoreB avec format à trois registres
//         let storeb_instruction = Instruction::create_reg_reg_reg(Opcode::StoreB, 0, 1, 2);
//
//         let em_reg_storeb = ExecuteMemoryRegister {
//             instruction: storeb_instruction,
//             alu_result: 0,
//             rd: None,
//             store_value: Some(0xEF), // Seul l'octet de poids faible sera stocké
//             mem_addr: Some(0x6000),  // Adresse calculée
//             branch_target: None,
//             branch_taken: false,
//             branch_prediction_correct: Option::from(false),
//             stack_operation: None,
//             stack_result: None,
//             ras_prediction_correct: None,
//             halted: false,
//         };
//
//         let result_storeb = memory_stage.process_direct(&em_reg_storeb, &mut memory);
//         assert!(result_storeb.is_ok());
//
//         // Vérifier la valeur stockée
//         let loaded_byte = memory.read_byte(0x6000);
//         assert!(loaded_byte.is_ok());
//         assert_eq!(loaded_byte.unwrap(), 0xEF);
//
//         // Tester StoreW avec format à trois registres
//         let storew_instruction = Instruction::create_reg_reg_reg(Opcode::StoreW, 0, 1, 2);
//
//         let em_reg_storew = ExecuteMemoryRegister {
//             instruction: storew_instruction,
//             alu_result: 0,
//             rd: None,
//             store_value: Some(0xABCD), // Seuls les 16 bits de poids faible seront stockés
//             mem_addr: Some(0x6100),    // Adresse calculée
//             branch_target: None,
//             branch_taken: false,
//             branch_prediction_correct: Option::from(false),
//             stack_operation: None,
//             stack_result: None,
//             ras_prediction_correct: None,
//             halted: false,
//         };
//
//         let result_storew = memory_stage.process_direct(&em_reg_storew, &mut memory);
//         assert!(result_storew.is_ok());
//
//         // Vérifier la valeur stockée
//         let loaded_word = memory.read_word(0x6100);
//         assert!(loaded_word.is_ok());
//         assert_eq!(loaded_word.unwrap(), 0xABCD);
//     }
//
//     #[test]
//     fn test_memory_push_pop_with_registers() {
//         let mut memory_stage = MemoryStage::new_for_test();
//         let mut memory = Memory::new(MemoryConfig::default());
//
//         // Simuler une séquence PUSH/POP
//         // 1. PUSH R0 (R0 contient 0x1122334455667788)
//         // 2. POP R1 (doit récupérer la même valeur)
//
//         // Étape 1: PUSH R0
//         let push_instruction = Instruction::create_single_reg(Opcode::Push, 0);
//
//         let em_reg_push = ExecuteMemoryRegister {
//             instruction: push_instruction,
//             alu_result: 0,
//             rd: None,
//             store_value: Some(0x1122334455667788), // Valeur de R0
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
//         // Sauvegarder le SP initial
//         let initial_sp = memory_stage.stack_pointer;
//
//         // Exécuter PUSH
//         let result_push = memory_stage.process_direct(&em_reg_push, &mut memory);
//         assert!(result_push.is_ok());
//
//         // Vérifier que le SP a été décrémenté
//         assert_eq!(memory_stage.stack_pointer, initial_sp - 8);
//
//         // Étape 2: POP R1
//         let pop_instruction = Instruction::create_single_reg(Opcode::Pop, 1);
//
//         let em_reg_pop = ExecuteMemoryRegister {
//             instruction: pop_instruction,
//             alu_result: 0,
//             rd: Some(1), // Registre destination R1
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
//         // Exécuter POP
//         let result_pop = memory_stage.process_direct(&em_reg_pop, &mut memory);
//         assert!(result_pop.is_ok());
//
//         // Vérifier que le SP est revenu à sa valeur initiale
//         assert_eq!(memory_stage.stack_pointer, initial_sp);
//
//         // Vérifier que la valeur récupérée est correcte
//         let mw_reg_pop = result_pop.unwrap();
//         assert_eq!(mw_reg_pop.result, 0x1122334455667788);
//         assert_eq!(mw_reg_pop.rd, Some(1));
//     }
//
//     #[test]
//     fn test_memory_complex_program() {
//         let mut memory_stage = MemoryStage::new_for_test();
//         let mut memory = Memory::new(MemoryConfig::default());
//
//         // Simuler un programme plus complexe:
//         // 1. STORE R0, [R1+R2]  // Stocker une valeur à une adresse calculée
//         // 2. LOAD R3, [R1+R2]   // Charger cette valeur dans R3
//         // 3. INC R3             // Incrémenter R3 (exécuté dans l'étage Execute)
//         // 4. STORE R3, [R4]     // Stocker la nouvelle valeur à une autre adresse
//         // 5. LOAD R5, [R4]      // Charger la valeur depuis la nouvelle adresse
//
//         // Préparation: Écrire quelques valeurs dans les registres fictifs
//         let r0_value = 0x1000000000000000; // Valeur à stocker
//         let r3_incremented = 0x1000000000000001; // R0_value + 1
//         let addr1 = 0x7000; // [R1+R2]
//         let addr2 = 0x8000; // [R4]
//
//         // Étape 1: STORE R0, [R1+R2]
//         let store1_instruction = Instruction::create_reg_reg_reg(Opcode::Store, 0, 1, 2);
//
//         let em_reg_store1 = ExecuteMemoryRegister {
//             instruction: store1_instruction,
//             alu_result: 0,
//             rd: None,
//             store_value: Some(r0_value),
//             mem_addr: Some(addr1),
//             branch_target: None,
//             branch_taken: false,
//             branch_prediction_correct: Option::from(false),
//             stack_operation: None,
//             stack_result: None,
//             ras_prediction_correct: None,
//             halted: false,
//         };
//
//         let result_store1 = memory_stage.process_direct(&em_reg_store1, &mut memory);
//         assert!(result_store1.is_ok());
//
//         // Étape 2: LOAD R3, [R1+R2]
//         let load1_instruction = Instruction::create_reg_reg_reg(Opcode::Load, 3, 1, 2);
//
//         let em_reg_load1 = ExecuteMemoryRegister {
//             instruction: load1_instruction,
//             alu_result: 0,
//             rd: Some(3),
//             store_value: None,
//             mem_addr: Some(addr1),
//             branch_target: None,
//             branch_taken: false,
//             branch_prediction_correct: Option::from(false),
//             stack_operation: None,
//             stack_result: None,
//             ras_prediction_correct: None,
//             halted: false,
//         };
//
//         let result_load1 = memory_stage.process_direct(&em_reg_load1, &mut memory);
//         assert!(result_load1.is_ok());
//         assert_eq!(result_load1.unwrap().result, r0_value);
//
//         // Étape 3: INC R3 (simulé, car effectué dans l'étage Execute)
//         // Pas besoin de code pour cette étape
//
//         // Étape 4: STORE R3, [R4]
//         let store2_instruction = Instruction::create_reg_reg(Opcode::Store, 3, 4);
//
//         let em_reg_store2 = ExecuteMemoryRegister {
//             instruction: store2_instruction,
//             alu_result: 0,
//             rd: None,
//             store_value: Some(r3_incremented), // Valeur incrémentée de R3
//             mem_addr: Some(addr2),
//             branch_target: None,
//             branch_taken: false,
//             branch_prediction_correct: Option::from(false),
//             stack_operation: None,
//             stack_result: None,
//             ras_prediction_correct: None,
//             halted: false,
//         };
//
//         let result_store2 = memory_stage.process_direct(&em_reg_store2, &mut memory);
//         assert!(result_store2.is_ok());
//
//         // Étape 5: LOAD R5, [R4]
//         let load2_instruction = Instruction::create_reg_reg(Opcode::Load, 5, 4);
//
//         let em_reg_load2 = ExecuteMemoryRegister {
//             instruction: load2_instruction,
//             alu_result: 0,
//             rd: Some(5),
//             store_value: None,
//             mem_addr: Some(addr2),
//             branch_target: None,
//             branch_taken: false,
//             branch_prediction_correct: Option::from(false),
//             stack_operation: None,
//             stack_result: None,
//             ras_prediction_correct: None,
//             halted: false,
//         };
//
//         let result_load2 = memory_stage.process_direct(&em_reg_load2, &mut memory);
//         assert!(result_load2.is_ok());
//
//         // Vérifier que R5 contient la valeur incrémentée
//         let mw_reg_load2 = result_load2.unwrap();
//         assert_eq!(mw_reg_load2.result, r3_incremented);
//         assert_eq!(mw_reg_load2.rd, Some(5));
//     }
// }
//
// //
// // impl<'a> PipelineStage<'a> for MemoryStage {
// //     type Input = (ExecuteMemoryRegister, &'a mut Memory);
// //     type Output = MemoryWritebackRegister;
// //
// //     fn process(&mut self, input: &Self::Input) -> Result<Self::Output, String> {
// //         let (mem_reg, memory) = input;
// //         self.process(mem_reg, memory)
// //     }
// //
// //     fn reset(&mut self) {
// //         // Reset direct sans appel récursif
// //         self.stack_pointer = 0xFFFF0000;
// //     }
// // }

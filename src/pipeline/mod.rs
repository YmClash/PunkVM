//src/pipeline/mod.rs
pub mod fetch;
pub mod decode;
pub mod execute;
pub mod memory;
pub mod writeback;
pub mod hazard;
pub mod forward;



use crate::alu::alu::ALU;
use crate::bytecode::opcodes::Opcode;

use crate::bytecode::instructions::Instruction;
use crate::pvm::memorys::Memory;

/// Structure représentant le pipeline à 5 étages
pub struct Pipeline {
    /// État actuel du pipeline
    pub state: PipelineState,
    /// Module de l'étage Fetch
    fetch: fetch::FetchStage,
    /// Module de l'étage Decode
    decode: decode::DecodeStage,
    /// Module de l'étage Execute
    execute: execute::ExecuteStage,
    /// Module de l'étage Memory
    memory: memory::MemoryStage,
    /// Module de l'étage Writeback
    writeback: writeback::WritebackStage,
    /// Unité de détection de hazards
    pub hazard_detection: hazard::HazardDetectionUnit,
    /// Unité de forwarding
    pub forwarding: forward::ForwardingUnit,
    /// Statistiques du pipeline
    stats: PipelineStats,
    /// Configuration
    enable_forwarding: bool,
    enable_hazard_detection: bool,
}

/// État du pipeline à un instant donné
#[derive(Debug, Clone)]
pub struct PipelineState {
    /// Registre intermédiaire Fetch -> Decode
    pub fetch_decode: Option<FetchDecodeRegister>,
    /// Registre intermédiaire Decode -> Execute
    pub decode_execute: Option<DecodeExecuteRegister>,
    /// Registre intermédiaire Execute -> Memory
    pub execute_memory: Option<ExecuteMemoryRegister>,
    /// Registre intermédiaire Memory -> Writeback
    pub memory_writeback: Option<MemoryWritebackRegister>,
    /// Prochain PC à charger
    pub next_pc: u32,
    /// Indique si le pipeline est stall (bloqué)
    pub stalled: bool,
    /// Indique si l'exécution est terminée (HALT)
    pub halted: bool,
    /// Nombre d'instructions complétées ce cycle
    pub instructions_completed: usize,

}

impl Default for PipelineState {
    fn default() -> Self {
        Self {
            fetch_decode: None,
            decode_execute: None,
            execute_memory: None,
            memory_writeback: None,
            next_pc: 0,
            stalled: false,
            halted: false,
            instructions_completed: 0,
        }
    }
}


/// Registre intermédiaire entre les étages Fetch et Decode
#[derive(Debug, Clone)]
pub struct FetchDecodeRegister {
    /// Instruction brute récupérée
    pub instruction: Instruction,
    /// Adresse de l'instruction
    pub pc: u32,
}

/// Registre intermédiaire entre les étages Decode et Execute
#[derive(Debug, Clone)]
pub struct DecodeExecuteRegister {
    /// Instruction décodée
    pub instruction: Instruction,
    /// Adresse de l'instruction
    pub pc: u32,
    /// Registre source 1
    pub rs1: Option<usize>,
    /// Registre source 2
    pub rs2: Option<usize>,
    /// Registre destination
    pub rd: Option<usize>,

    /// Valeurs des registres source 1 et 2
    pub rs1_value: u64,
    pub rs2_value: u64,

    /// Valeur immédiate (si présente)
    pub immediate: Option<u64>,
    /// Adresse branchement (si instruction de saut)
    pub branch_addr: Option<u32>,
    /// Adresse mémoire (si instruction mémoire)
    pub mem_addr: Option<u32>,
}

/// Registre intermédiaire entre les étages Execute et Memory
#[derive(Debug, Clone)]
pub struct ExecuteMemoryRegister {
    /// Instruction
    pub instruction: Instruction,
    /// Résultat de l'ALU
    pub alu_result: u64,
    /// Registre destination
    pub rd: Option<usize>,
    /// Valeur à écrire en mémoire (si store)
    pub store_value: Option<u64>,
    /// Adresse mémoire (si load/store)
    pub mem_addr: Option<u32>,
    /// PC du branchement (si instruction de saut)
    pub branch_target: Option<u32>,
    /// Branchement pris ou non
    pub branch_taken: bool,

    pub halted: bool,
}

/// Registre intermédiaire entre les étages Memory et Writeback
#[derive(Debug, Clone)]
pub struct MemoryWritebackRegister {
    /// Instruction
    pub instruction: Instruction,
    /// Résultat à écrire dans le registre destination
    pub result: u64,
    /// Registre destination
    pub rd: Option<usize>,
}

/// Statistiques du pipeline
#[derive(Debug, Clone, Copy, Default)]
pub struct PipelineStats {
    /// Nombre total de cycles
    pub cycles: u64,
    /// Nombre d'instructions exécutées
    pub instructions: u64,
    /// Nombre de stalls (cycles où le pipeline est bloqué)
    pub stalls: u64,
    /// Nombre de hazards détectés
    pub hazards: u64,
    /// Nombre de forwards effectués
    pub forwards: u64,
    /// Nombre de prédictions de branchement
    pub branch_predictions: u64,
    /// Nombre de prédictions correctes
    pub branch_hits: u64,
    /// Nombre de prédictions incorrectes
    pub branch_misses: u64,
    /// Nombre de brance flush
    pub branch_flush: u64,

}

impl Pipeline {
    /// Crée un nouveau pipeline
    pub fn new(
        fetch_buffer_size: usize,
        enable_forwarding: bool,
        enable_hazard_detection: bool,
    ) -> Self {
        Self {
            state: PipelineState::default(),
            fetch: fetch::FetchStage::new(fetch_buffer_size),
            decode: decode::DecodeStage::new(),
            execute: execute::ExecuteStage::new(),
            memory: memory::MemoryStage::new(),
            writeback: writeback::WritebackStage::new(),
            hazard_detection: hazard::HazardDetectionUnit::new(),
            forwarding: forward::ForwardingUnit::new(),
            stats: PipelineStats::default(),
            enable_forwarding,
            enable_hazard_detection,
        }
    }

    /// Réinitialise le pipeline
    pub fn reset(&mut self) {
        self.state = PipelineState::default();
        self.fetch.reset();
        self.decode.reset();
        self.execute.reset();
        self.memory.reset();
        self.writeback.reset();
        self.hazard_detection.reset();
        self.forwarding.reset();
        self.stats = PipelineStats::default();
    }

    /// Exécute un cycle du pipeline
    pub fn cycle(
        &mut self,
        pc: u32,
        registers: &mut [u64],
        memory: &mut Memory,
        alu: &mut ALU,
        instructions: &[Instruction],
    ) -> Result<PipelineState, String> {
        // 0) Incrément du compteur de cycles pipeline
        self.stats.cycles += 1;

        // 1) Clone de l’état local
        let mut state = self.state.clone();
        state.stalled = false;
        state.instructions_completed = 0;

        // 2) Détection de hazards
        if self.enable_hazard_detection {
            let any_hazard = self.hazard_detection.detect_hazards(&state);
            if any_hazard {
                self.stats.stalls += 1;
                self.stats.hazards += 1;
                state.stalled = true;
            }
        }

        // ----- (1ᵉʳᵉ étape) FETCH -----
        // Si on n’est pas stalled, on fetch l’instruction à l’adresse `pc`.
        if !state.stalled {
            // On fetch
            let fd_reg = self.fetch.process_direct(pc, instructions)?;
            state.fetch_decode = Some(fd_reg);

            // Mise à jour du next_pc s’il n’est pas déjà modifié par un branch
            if !state.halted && state.next_pc == pc {
                if let Some(fd_reg) = &state.fetch_decode {
                    let size = fd_reg.instruction.total_size() as u32;
                    state.next_pc = pc.wrapping_add(size);
                }
            }
        }

        // ----- (2ᵉ étape) DECODE -----
        if !state.stalled {
            if let Some(fd_reg) = &state.fetch_decode {
                let ex_reg = self.decode.process_direct(fd_reg, registers)?;
                state.decode_execute = Some(ex_reg);
            } else {
                state.decode_execute = None;
            }
        }

        // ----- (3ᵉ étape) EXECUTE -----
        if let Some(de_reg) = &state.decode_execute {
            // Forwarding si activé
            let mut de_reg_mut = de_reg.clone();
            if self.enable_forwarding {
                self.forwarding.forward(
                    &mut de_reg_mut,
                    &state.execute_memory,
                    &state.memory_writeback,
                );
            }

            let mem_reg = self.execute.process_direct(&de_reg_mut, alu)?;

            // Si un branch est pris => flush fetch/decode
            if mem_reg.branch_taken {
                if let Some(target) = mem_reg.branch_target {
                    state.next_pc = target;

                    //  vide la pipeline
                    state.fetch_decode = None;
                    state.decode_execute = None;
                    state.execute_memory = None;    //Optionnel

                    self.stats.branch_flush += 1;

                }
            }
            state.execute_memory = Some(mem_reg);
        } else {
            state.execute_memory = None;
        }

        // ----- (4ᵉ étape) MEMORY -----
        if let Some(ex_mem) = &state.execute_memory {
            let wb_reg = self.memory.process_direct(ex_mem, memory)?;

            // Si c’est un HALT => on arrête tout de suite
            if ex_mem.instruction.opcode == Opcode::Halt {
                state.halted = true;
                // Flush
                state.fetch_decode = None;
                state.decode_execute = None;
                state.execute_memory = None;
                state.memory_writeback = None;

                // Optionnellement, on peut stocker wb_reg si besoin
                state.memory_writeback = Some(wb_reg);

                // On quitte aussitôt ce cycle => pas de Writeback
                self.state = state.clone();
                return Ok(state);
            }

            state.memory_writeback = Some(wb_reg);
        } else {
            state.memory_writeback = None;
        }

        // ----- (5ᵉ étape) WRITEBACK -----
        if let Some(mw_reg) = &state.memory_writeback {
            self.writeback.process_direct(mw_reg, registers)?;
            // On considère qu’une instruction est finalisée ici
            state.instructions_completed += 1;
            // self.stats.instructions += 1;

        }
        state.memory_writeback = None;

        self.stats.hazards = self.hazard_detection.get_hazards_count();
        self.stats.forwards = self.forwarding.get_forwards_count();
        // if state.stalled {
        //     self.stats.stalls += 1;
        // }


        // 9) Mise à jour de self.state
        self.state = state.clone();
        Ok(state)
    }



    /// Retourne les statistiques du pipeline
    pub fn stats(&self) -> PipelineStats {
        self.stats
    }
}


// Test unitaire pour les fichiers de bytecode
#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use super::*;
    use crate::bytecode::opcodes::Opcode;
    use crate::bytecode::format::{ArgType, InstructionFormat};
    use crate::bytecode::instructions::Instruction;
    use std::io::ErrorKind;
    use tempfile::tempdir;
    use crate::bytecode::files::{BytecodeVersion, SegmentMetadata, SegmentType};
    use crate::BytecodeFile;

    #[test]
    fn test_bytecode_version() {
        let version = BytecodeVersion::new(1, 2, 3, 4);

        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
        assert_eq!(version.build, 4);

        // Test encode/decode
        let encoded = version.encode();
        let decoded = BytecodeVersion::decode(encoded);

        assert_eq!(decoded.major, 1);
        assert_eq!(decoded.minor, 2);
        assert_eq!(decoded.patch, 3);
        assert_eq!(decoded.build, 4);

        // Test to_string
        assert_eq!(version.to_string(), "1.2.3.4");
    }

    #[test]
    fn test_segment_type() {
        // Test des conversions valides
        assert_eq!(SegmentType::from_u8(0), Some(SegmentType::Code));
        assert_eq!(SegmentType::from_u8(1), Some(SegmentType::Data));
        assert_eq!(SegmentType::from_u8(2), Some(SegmentType::ReadOnlyData));
        assert_eq!(SegmentType::from_u8(3), Some(SegmentType::Symbols));
        assert_eq!(SegmentType::from_u8(4), Some(SegmentType::Debug));

        // Test avec valeur invalide
        assert_eq!(SegmentType::from_u8(5), None);
    }

    #[test]
    fn test_segment_metadata() {
        let segment = SegmentMetadata::new(SegmentType::Code, 100, 200, 300);

        assert_eq!(segment.segment_type, SegmentType::Code);
        assert_eq!(segment.offset, 100);
        assert_eq!(segment.size, 200);
        assert_eq!(segment.load_addr, 300);

        // Test encode/decode
        let encoded = segment.encode();
        let decoded = SegmentMetadata::decode(&encoded).unwrap();

        assert_eq!(decoded.segment_type, SegmentType::Code);
        assert_eq!(decoded.offset, 100);
        assert_eq!(decoded.size, 200);
        assert_eq!(decoded.load_addr, 300);
    }

    #[test]
    fn test_bytecode_file_simple() {
        // Création d'un fichier bytecode simple
        let mut bytecode = BytecodeFile::new();

        // Ajout de métadonnées
        bytecode.add_metadata("name", "Test");
        bytecode.add_metadata("author", "PunkVM");

        // Vérification
        assert_eq!(bytecode.metadata.get("name"), Some(&"Test".to_string()));
        assert_eq!(bytecode.metadata.get("author"), Some(&"PunkVM".to_string()));

        // Ajout d'instructions
        let instr1 = Instruction::create_no_args(Opcode::Nop);
        let instr2 = Instruction::create_reg_imm8(Opcode::Load, 0, 42);

        bytecode.add_instruction(instr1);
        bytecode.add_instruction(instr2);

        assert_eq!(bytecode.code.len(), 2);
        assert_eq!(bytecode.code[0].opcode, Opcode::Nop);
        assert_eq!(bytecode.code[1].opcode, Opcode::Load);

        // Ajout de données
        let offset = bytecode.add_data(&[1, 2, 3, 4]);
        assert_eq!(offset, 0);
        assert_eq!(bytecode.data, vec![1, 2, 3, 4]);

        // Ajout de données en lecture seule
        let offset = bytecode.add_readonly_data(&[5, 6, 7, 8]);
        assert_eq!(offset, 0);
        assert_eq!(bytecode.readonly_data, vec![5, 6, 7, 8]);

        // Ajout de symboles
        bytecode.add_symbol("main", 0x1000);
        assert_eq!(bytecode.symbols.get("main"), Some(&0x1000));
    }

    #[test]
    fn test_bytecode_file_with_arithmetic_instructions() {
        // Création d'un fichier bytecode avec des instructions arithmétiques
        let mut bytecode = BytecodeFile::new();

        // Ajouter des instructions arithmétiques avec le nouveau format à 3 registres
        let instr1 = Instruction::create_reg_reg_reg(Opcode::Add, 2, 0, 1);  // R2 = R0 + R1
        let instr2 = Instruction::create_reg_reg_reg(Opcode::Sub, 3, 0, 1);  // R3 = R0 - R1
        let instr3 = Instruction::create_reg_reg_reg(Opcode::Mul, 4, 0, 1);  // R4 = R0 * R1

        bytecode.add_instruction(instr1);
        bytecode.add_instruction(instr2);
        bytecode.add_instruction(instr3);

        assert_eq!(bytecode.code.len(), 3);

        // Vérifier le premier opcode
        assert_eq!(bytecode.code[0].opcode, Opcode::Add);

        // Vérifier les types d'arguments du format
        assert_eq!(bytecode.code[0].format.arg1_type, ArgType::Register);
        assert_eq!(bytecode.code[0].format.arg2_type, ArgType::Register);
        assert_eq!(bytecode.code[0].format.arg3_type, ArgType::Register);

        // Vérifier les valeurs des registres
        assert_eq!(bytecode.code[0].args[0], 2);  // Rd (destination)
        assert_eq!(bytecode.code[0].args[1], 0);  // Rs1 (source 1)
        assert_eq!(bytecode.code[0].args[2], 1);  // Rs2 (source 2)
    }

    #[test]
    fn test_bytecode_file_io() {
        // Création d'un répertoire temporaire pour les tests
        let dir = tempdir().expect("Impossible de créer un répertoire temporaire");
        let file_path = dir.path().join("test.punk");

        // Création d'un fichier bytecode à écrire
        let mut bytecode = BytecodeFile::new();
        bytecode.version = BytecodeVersion::new(1, 0, 0, 0);
        bytecode.add_metadata("name", "TestIO");
        bytecode.add_instruction(Instruction::create_no_args(Opcode::Halt));
        bytecode.add_data(&[1, 2, 3]);
        bytecode.add_readonly_data(&[4, 5, 6]);
        bytecode.add_symbol("main", 0);

        // Écrire le fichier
        bytecode.write_to_file(&file_path).expect("Impossible d'écrire le fichier bytecode");

        // Lire le fichier
        let loaded = BytecodeFile::read_from_file(&file_path).expect("Impossible de lire le fichier bytecode");

        // Vérifier que le contenu est identique
        assert_eq!(loaded.version.major, 1);
        assert_eq!(loaded.version.minor, 0);
        assert_eq!(loaded.metadata.get("name"), Some(&"TestIO".to_string()));
        assert_eq!(loaded.code.len(), 1);
        assert_eq!(loaded.code[0].opcode, Opcode::Halt);
        assert_eq!(loaded.data, vec![1, 2, 3]);
        assert_eq!(loaded.readonly_data, vec![4, 5, 6]);
        assert_eq!(loaded.symbols.get("main"), Some(&0));
    }

    #[test]
    fn test_bytecode_file_with_three_register_instructions_io() {
        // Test d'écriture et lecture d'un fichier contenant des instructions à 3 registres
        let dir = tempdir().expect("Impossible de créer un répertoire temporaire");
        let file_path = dir.path().join("test_three_reg.punk");

        // Création du fichier bytecode
        let mut bytecode = BytecodeFile::new();
        bytecode.version = BytecodeVersion::new(1, 0, 0, 0);

        // Ajouter une instruction ADD avec 3 registres
        let add_instr = Instruction::create_reg_reg_reg(Opcode::Add, 2, 0, 1);  // R2 = R0 + R1
        bytecode.add_instruction(add_instr);

        // Écrire le fichier
        bytecode.write_to_file(&file_path).expect("Impossible d'écrire le fichier bytecode");

        // Lire le fichier
        let loaded = BytecodeFile::read_from_file(&file_path).expect("Impossible de lire le fichier bytecode");

        // Vérifier que l'instruction est correctement chargée
        assert_eq!(loaded.code.len(), 1);
        assert_eq!(loaded.code[0].opcode, Opcode::Add);

        // Vérifier les valeurs des registres
        assert_eq!(loaded.code[0].args.len(), 3);
        assert_eq!(loaded.code[0].args[0], 2);  // Rd
        assert_eq!(loaded.code[0].args[1], 0);  // Rs1
        assert_eq!(loaded.code[0].args[2], 1);  // Rs2
    }

    #[test]
    fn test_bytecode_file_extended_size_io() {
        // Test d'écriture et lecture d'un fichier avec une instruction de grande taille
        let dir = tempdir().expect("Impossible de créer un répertoire temporaire");
        let file_path = dir.path().join("test_extended.punk");

        // Création du fichier bytecode
        let mut bytecode = BytecodeFile::new();

        // Créer une instruction avec beaucoup de données pour forcer un size_type Extended
        let large_args = vec![0; 248];  // Suffisant pour dépasser la limite de 255 octets
        let large_instr = Instruction::new(
            Opcode::Add,
            InstructionFormat::double_reg(),
            large_args
        );

        bytecode.add_instruction(large_instr);

        // Écrire le fichier
        bytecode.write_to_file(&file_path).expect("Impossible d'écrire le fichier bytecode");

        // Lire le fichier
        let loaded = BytecodeFile::read_from_file(&file_path).expect("Impossible de lire le fichier bytecode");

        // Vérifier que l'instruction est correctement chargée
        assert_eq!(loaded.code.len(), 1);
        assert_eq!(loaded.code[0].opcode, Opcode::Add);
        assert_eq!(loaded.code[0].args.len(), 248);
    }

    #[test]
    fn test_bytecode_file_complex_program() {
        // Créer un programme complet avec des instructions variées
        let mut bytecode = BytecodeFile::new();

        // Ajouter des métadonnées
        bytecode.add_metadata("name", "Programme de test");
        bytecode.add_metadata("author", "PunkVM Team");
        bytecode.add_metadata("version", "1.0.0");

        // Initialisation des registres
        bytecode.add_instruction(Instruction::create_reg_imm8(Opcode::Load, 0, 10));  // R0 = 10
        bytecode.add_instruction(Instruction::create_reg_imm8(Opcode::Load, 1, 5));   // R1 = 5

        // Opérations arithmétiques
        bytecode.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 2, 0, 1));  // R2 = R0 + R1
        bytecode.add_instruction(Instruction::create_reg_reg_reg(Opcode::Sub, 3, 0, 1));  // R3 = R0 - R1
        bytecode.add_instruction(Instruction::create_reg_reg_reg(Opcode::Mul, 4, 0, 1));  // R4 = R0 * R1
        bytecode.add_instruction(Instruction::create_reg_reg_reg(Opcode::Div, 5, 0, 1));  // R5 = R0 / R1

        // Opérations logiques
        bytecode.add_instruction(Instruction::create_reg_reg_reg(Opcode::And, 6, 0, 1));  // R6 = R0 & R1
        bytecode.add_instruction(Instruction::create_reg_reg_reg(Opcode::Or, 7, 0, 1));   // R7 = R0 | R1

        // Fin du programme
        bytecode.add_instruction(Instruction::create_no_args(Opcode::Halt));

        // Ajouter un symbole pour le début du programme
        bytecode.add_symbol("start", 0);

        // Vérifier le nombre d'instructions
        assert_eq!(bytecode.code.len(), 9);

        // Vérifier les métadonnées
        assert_eq!(bytecode.metadata.len(), 3);
        assert_eq!(bytecode.metadata.get("name"), Some(&"Programme de test".to_string()));

        // Vérifier les symboles
        assert_eq!(bytecode.symbols.len(), 1);
        assert_eq!(bytecode.symbols.get("start"), Some(&0));
    }

    #[test]
    fn test_bytecode_file_io_errors() {
        // Test avec un fichier inexistant
        let result = BytecodeFile::read_from_file("nonexistent_file.punk");
        assert!(result.is_err());

        // Test avec un fichier trop petit
        let dir = tempdir().expect("Impossible de créer un répertoire temporaire");
        let invalid_file_path = dir.path().join("invalid.punk");

        // Créer un fichier invalide avec juste quelques octets
        std::fs::write(&invalid_file_path, &[0, 1, 2]).expect("Impossible d'écrire le fichier de test");

        let result = BytecodeFile::read_from_file(&invalid_file_path);
        assert!(result.is_err());

        // Vérifier le type d'erreur
        match result {
            Err(e) => assert_eq!(e.kind(), ErrorKind::InvalidData),
            _ => panic!("Expected an error but got success"),
        }
    }

    #[test]
    fn test_encode_decode_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("key1".to_string(), "value1".to_string());
        metadata.insert("key2".to_string(), "value2".to_string());

        let mut bytecode = BytecodeFile::new();
        bytecode.metadata = metadata.clone();

        let encoded = bytecode.encode_metadata();
        let decoded = BytecodeFile::decode_metadata(&encoded).expect("Failed to decode metadata");

        assert_eq!(decoded.len(), 2);
        assert_eq!(decoded.get("key1"), Some(&"value1".to_string()));
        assert_eq!(decoded.get("key2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_encode_decode_symbols() {
        let mut symbols = HashMap::new();
        symbols.insert("sym1".to_string(), 0x1000);
        symbols.insert("sym2".to_string(), 0x2000);

        let mut bytecode = BytecodeFile::new();
        bytecode.symbols = symbols.clone();

        let encoded = bytecode.encode_symbols();
        let decoded = BytecodeFile::decode_symbols(&encoded).expect("Failed to decode symbols");

        assert_eq!(decoded.len(), 2);
        assert_eq!(decoded.get("sym1"), Some(&0x1000));
        assert_eq!(decoded.get("sym2"), Some(&0x2000));
    }

    #[test]
    fn test_encode_decode_code() {
        // Créer un ensemble d'instructions de test
        let mut code = Vec::new();
        code.push(Instruction::create_no_args(Opcode::Nop));
        code.push(Instruction::create_reg_imm8(Opcode::Load, 0, 42));
        code.push(Instruction::create_reg_reg_reg(Opcode::Add, 2, 0, 1));

        let mut bytecode = BytecodeFile::new();
        bytecode.code = code.clone();

        let encoded = bytecode.encode_code();
        let decoded = BytecodeFile::decode_code(&encoded).expect("Failed to decode code");

        assert_eq!(decoded.len(), 3);
        assert_eq!(decoded[0].opcode, Opcode::Nop);
        assert_eq!(decoded[1].opcode, Opcode::Load);
        assert_eq!(decoded[2].opcode, Opcode::Add);

        // Vérifier les arguments de l'instruction Add
        assert_eq!(decoded[2].args.len(), 3);
        assert_eq!(decoded[2].args[0], 2);
        assert_eq!(decoded[2].args[1], 0);
        assert_eq!(decoded[2].args[2], 1);
    }
}

// src/pipeline/mod.rs
pub mod decode;
pub mod execute;
pub mod fetch;
pub mod forward;
pub mod hazard;
pub mod memory;
pub mod writeback;

use crate::alu::alu::ALU;
use crate::bytecode::opcodes::Opcode;

use crate::bytecode::instructions::Instruction;
use crate::pvm::branch_predictor::{BranchPrediction, BranchPredictor};
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
    /// Indique si la branche a été traitée
    branch_processed: bool,
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
            branch_processed: false,
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
    ///Prediction de branchement (si instruction de branchement)
    pub branch_prediction: Option<BranchPrediction>,
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

    pub branch_prediction_correct: Option<bool>,

    /// Halt
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
    /// Taux de prédiction de branchement (calculé lors de l'accès)
    pub branch_predictor_rate: f64,
}

impl PipelineStats {
    // pub fn branch_prediction_rate(&self) -> f64 {
    //     if self.branch_predictions == 0 {
    //         0.0
    //     } else {
    //         self.branch_hits as f64 / self.branch_predictions as f64
    //     }
    // }
    pub fn branch_prediction_rate(&self) -> f64 {
        if self.branch_predictions > 0 {
            (self.branch_hits as f64 / self.branch_predictions as f64) * 100.0
        } else { 0.0 }
    }
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
        println!("DEBUG: Debut du cycle - PC = {}", pc);

        // 1) Clone de l’état local
        let mut state = self.state.clone();
        state.stalled = false;
        state.instructions_completed = 0;
        let current_pc_target = self.state.next_pc;

        //Gestion de l'etai des branchement en cours
        if state.execute_memory.is_some()
            && state
                .execute_memory
                .as_ref()
                .unwrap()
                .instruction
                .opcode
                .is_branch()
        {
            if state.stalled {
                // Si on est stalled, on ne traite pas le branchement
                println!("DEBUG: Branchement en cours, mais pipeline est stalled");
                state.branch_processed = true;
            } else {
                // Si on n'est pas stalled, on traite le branchement
                println!("DEBUG: Branchement en cours, mais pipeline n'est pas stalled");
                state.branch_processed = false
            }
        }

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
                    println!("[DEBUG: Fin Fetch -] PC = 0x{:08X}, next_pc = 0x{:08X}", fd_reg.pc, state.next_pc);
                }

            }




        }


        // ----- (2ᵉ étape) DECODE -----
        if !state.stalled {
            if let Some(fd_reg) = &state.fetch_decode {
                let ex_reg = self.decode.process_direct(fd_reg, registers)?;
                state.decode_execute = Some(ex_reg);
                println!("[DEBUG: Fin Decode -] PC = 0x{:08X}, instruction = {:?},next_pc = 0x{:08X}", fd_reg.pc, fd_reg.instruction.opcode, state.next_pc);
            } else {
                state.decode_execute = None;
                println!("DEBUG: Pas d'instruction à décoder (fetch_decode est None)");
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

            // Extraire les valeurs dont nous aurons besoin plus tard
            let branch_pc = de_reg.pc;
            let branch_prediction = de_reg.branch_prediction;

            // Gérer les prédictions de branchement
            if let Some(prediction_correct) = mem_reg.branch_prediction_correct {
                if prediction_correct {
                    // Prédiction correcte - mise à jour des statistiques
                    self.stats.branch_hits += 1;

                } else {
                    // Prédiction incorrecte - flush du pipeline et mise à jour du PC
                    self.stats.branch_misses += 1;


                    if mem_reg.branch_taken {
                        if let Some(target) = mem_reg.branch_target {
                            // Branchement pris mais prédit non pris
                            state.next_pc = target;
                            println!("Branchement pris vers l'adresse: 0x{:08X}", target);
                            state.fetch_decode = None;
                            state.decode_execute = None;
                            self.stats.branch_flush += 1;
                        }else {
                            println!("On ne fait rien ")
                        }
                    }

                }

                /////////////////////////


                //// Gestion des branchements quand la prédiction est incorrecte
                // if !prediction_correct && !mem_reg.branch_taken && mem_reg.instruction.opcode.is_branch() {
                //     // Branchement non pris mais prédit pris
                //     println!("Branchement non pris mais prédit pris");
                //     // On avance le PC
                //     // On n'avancepas le PC ici car il est déjà avancé par l'étape Fetch
                //
                //
                //     // state.next_pc = branch_pc + mem_reg.instruction.total_size() as u32;
                //
                //
                //     println!(
                //         "Branchement non pris, avancement à PC = 0x{:08X}",
                //         state.next_pc
                //     );
                //
                // }

                /////////////////////////


                // Mise à jour du prédicteur
                let pc = branch_pc as u64;
                let taken = mem_reg.branch_taken;
                let prediction = branch_prediction.unwrap_or(BranchPrediction::NotTaken);
                // let prediction = branch_prediction.unwrap_or(BranchPredictor::predict(pc));
                self.decode.branch_predictor.update(pc, taken, prediction);

                self.stats.branch_predictions += 1;
            }

            if !mem_reg.branch_taken && mem_reg.instruction.opcode.is_branch() {
                // Pour un branchement non pris, s'assurer que le PC avance correctement
                let branch_pc = mem_reg.instruction.total_size() as u32;
                let expected_next_pc = pc + branch_pc;// Le PC de l'instruction de branchement + sa taille
                    if let Some(ex_reg) = &state.decode_execute {
                        ex_reg.pc + branch_pc
                    } else {
                        state.next_pc
                    };

                // Ne mettre à jour que si next_pc n'a pas déjà été correctement calculé
                if state.next_pc != expected_next_pc {
                    state.next_pc = expected_next_pc;
                }

                println!(
                    "Branchement non pris, PC avance à 0x{:08X}",
                    state.next_pc
                );
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
                // Flush le pipeline
                state.fetch_decode = None;
                state.decode_execute = None;
                state.execute_memory = None;
                // Optionnellement, on peut stocker wb_reg pour un dernier writeback
                state.memory_writeback = Some(wb_reg);
                // On arrête le cycle ici, sans traiter les étages suivants
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

        // Mise à jour des statistiques
        self.stats.hazards = self.hazard_detection.get_hazards_count();
        self.stats.forwards = self.forwarding.get_forwards_count();

        // 9) Mise à jour de self.state
        self.state = state.clone();

        println!("[[[DEBUG: Fin du cycle ]]] - PC = 0x{:08X}, next_pc = 0x{:08X}", pc, state.next_pc);

        // println!("DEBUG: Fin du cycle - PC = {}", pc);

        Ok(state)
    }

    pub fn update_branch_predictor(&mut self, pc: u64, taken: bool, prediction: BranchPredictor) {
        println!("Updating branch predictor: PC=0x{:X}, taken={}, prediction={:?}",
                 pc, taken, prediction);

    }

    /// Retourne les statistiques du pipeline
    pub fn stats(&self) -> PipelineStats {
        self.stats
    }
}



//
// // Test unitaire pour les fichiers de bytecode
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::bytecode::files::{BytecodeVersion, SegmentMetadata, SegmentType};
//     use crate::bytecode::format::{ArgType, InstructionFormat};
//     use crate::bytecode::instructions::Instruction;
//     use crate::bytecode::opcodes::Opcode;
//     use crate::BytecodeFile;
//     use std::collections::HashMap;
//     use std::io::ErrorKind;
//     use tempfile::tempdir;
//
//     #[test]
//     fn test_bytecode_version() {
//         let version = BytecodeVersion::new(1, 2, 3, 4);
//
//         assert_eq!(version.major, 1);
//         assert_eq!(version.minor, 2);
//         assert_eq!(version.patch, 3);
//         assert_eq!(version.build, 4);
//
//         // Test encode/decode
//         let encoded = version.encode();
//         let decoded = BytecodeVersion::decode(encoded);
//
//         assert_eq!(decoded.major, 1);
//         assert_eq!(decoded.minor, 2);
//         assert_eq!(decoded.patch, 3);
//         assert_eq!(decoded.build, 4);
//
//         // Test to_string
//         assert_eq!(version.to_string(), "1.2.3.4");
//     }
//
//     #[test]
//     fn test_segment_type() {
//         // Test des conversions valides
//         assert_eq!(SegmentType::from_u8(0), Some(SegmentType::Code));
//         assert_eq!(SegmentType::from_u8(1), Some(SegmentType::Data));
//         assert_eq!(SegmentType::from_u8(2), Some(SegmentType::ReadOnlyData));
//         assert_eq!(SegmentType::from_u8(3), Some(SegmentType::Symbols));
//         assert_eq!(SegmentType::from_u8(4), Some(SegmentType::Debug));
//
//         // Test avec valeur invalide
//         assert_eq!(SegmentType::from_u8(5), None);
//     }
//
//     #[test]
//     fn test_segment_metadata() {
//         let segment = SegmentMetadata::new(SegmentType::Code, 100, 200, 300);
//
//         assert_eq!(segment.segment_type, SegmentType::Code);
//         assert_eq!(segment.offset, 100);
//         assert_eq!(segment.size, 200);
//         assert_eq!(segment.load_addr, 300);
//
//         // Test encode/decode
//         let encoded = segment.encode();
//         let decoded = SegmentMetadata::decode(&encoded).unwrap();
//
//         assert_eq!(decoded.segment_type, SegmentType::Code);
//         assert_eq!(decoded.offset, 100);
//         assert_eq!(decoded.size, 200);
//         assert_eq!(decoded.load_addr, 300);
//     }
//
//     #[test]
//     fn test_bytecode_file_simple() {
//         // Création d'un fichier bytecode simple
//         let mut bytecode = BytecodeFile::new();
//
//         // Ajout de métadonnées
//         bytecode.add_metadata("name", "Test");
//         bytecode.add_metadata("author", "PunkVM");
//
//         // Vérification
//         assert_eq!(bytecode.metadata.get("name"), Some(&"Test".to_string()));
//         assert_eq!(bytecode.metadata.get("author"), Some(&"PunkVM".to_string()));
//
//         // Ajout d'instructions
//         let instr1 = Instruction::create_no_args(Opcode::Nop);
//         let instr2 = Instruction::create_reg_imm8(Opcode::Load, 0, 42);
//
//         bytecode.add_instruction(instr1);
//         bytecode.add_instruction(instr2);
//
//         assert_eq!(bytecode.code.len(), 2);
//         assert_eq!(bytecode.code[0].opcode, Opcode::Nop);
//         assert_eq!(bytecode.code[1].opcode, Opcode::Load);
//
//         // Ajout de données
//         let offset = bytecode.add_data(&[1, 2, 3, 4]);
//         assert_eq!(offset, 0);
//         assert_eq!(bytecode.data, vec![1, 2, 3, 4]);
//
//         // Ajout de données en lecture seule
//         let offset = bytecode.add_readonly_data(&[5, 6, 7, 8]);
//         assert_eq!(offset, 0);
//         assert_eq!(bytecode.readonly_data, vec![5, 6, 7, 8]);
//
//         // Ajout de symboles
//         bytecode.add_symbol("main", 0x1000);
//         assert_eq!(bytecode.symbols.get("main"), Some(&0x1000));
//     }
//
//     #[test]
//     fn test_bytecode_file_with_arithmetic_instructions() {
//         // Création d'un fichier bytecode avec des instructions arithmétiques
//         let mut bytecode = BytecodeFile::new();
//
//         // Ajouter des instructions arithmétiques avec le nouveau format à 3 registres
//         let instr1 = Instruction::create_reg_reg_reg(Opcode::Add, 2, 0, 1); // R2 = R0 + R1
//         let instr2 = Instruction::create_reg_reg_reg(Opcode::Sub, 3, 0, 1); // R3 = R0 - R1
//         let instr3 = Instruction::create_reg_reg_reg(Opcode::Mul, 4, 0, 1); // R4 = R0 * R1
//
//         bytecode.add_instruction(instr1);
//         bytecode.add_instruction(instr2);
//         bytecode.add_instruction(instr3);
//
//         assert_eq!(bytecode.code.len(), 3);
//
//         // Vérifier le premier opcode
//         assert_eq!(bytecode.code[0].opcode, Opcode::Add);
//
//         // Vérifier les types d'arguments du format
//         assert_eq!(bytecode.code[0].format.arg1_type, ArgType::Register);
//         assert_eq!(bytecode.code[0].format.arg2_type, ArgType::Register);
//         assert_eq!(bytecode.code[0].format.arg3_type, ArgType::Register);
//
//         // Vérifier les valeurs des registres
//         assert_eq!(bytecode.code[0].args[0], 2); // Rd (destination)
//         assert_eq!(bytecode.code[0].args[1], 0); // Rs1 (source 1)
//         assert_eq!(bytecode.code[0].args[2], 1); // Rs2 (source 2)
//     }
//
//     #[test]
//     fn test_bytecode_file_io() {
//         // Création d'un répertoire temporaire pour les tests
//         let dir = tempdir().expect("Impossible de créer un répertoire temporaire");
//         let file_path = dir.path().join("test.punk");
//
//         // Création d'un fichier bytecode à écrire
//         let mut bytecode = BytecodeFile::new();
//         bytecode.version = BytecodeVersion::new(1, 0, 0, 0);
//         bytecode.add_metadata("name", "TestIO");
//         bytecode.add_instruction(Instruction::create_no_args(Opcode::Halt));
//         bytecode.add_data(&[1, 2, 3]);
//         bytecode.add_readonly_data(&[4, 5, 6]);
//         bytecode.add_symbol("main", 0);
//
//         // Écrire le fichier
//         bytecode
//             .write_to_file(&file_path)
//             .expect("Impossible d'écrire le fichier bytecode");
//
//         // Lire le fichier
//         let loaded = BytecodeFile::read_from_file(&file_path)
//             .expect("Impossible de lire le fichier bytecode");
//
//         // Vérifier que le contenu est identique
//         assert_eq!(loaded.version.major, 1);
//         assert_eq!(loaded.version.minor, 0);
//         assert_eq!(loaded.metadata.get("name"), Some(&"TestIO".to_string()));
//         assert_eq!(loaded.code.len(), 1);
//         assert_eq!(loaded.code[0].opcode, Opcode::Halt);
//         assert_eq!(loaded.data, vec![1, 2, 3]);
//         assert_eq!(loaded.readonly_data, vec![4, 5, 6]);
//         assert_eq!(loaded.symbols.get("main"), Some(&0));
//     }
//
//     #[test]
//     fn test_bytecode_file_with_three_register_instructions_io() {
//         // Test d'écriture et lecture d'un fichier contenant des instructions à 3 registres
//         let dir = tempdir().expect("Impossible de créer un répertoire temporaire");
//         let file_path = dir.path().join("test_three_reg.punk");
//
//         // Création du fichier bytecode
//         let mut bytecode = BytecodeFile::new();
//         bytecode.version = BytecodeVersion::new(1, 0, 0, 0);
//
//         // Ajouter une instruction ADD avec 3 registres
//         let add_instr = Instruction::create_reg_reg_reg(Opcode::Add, 2, 0, 1); // R2 = R0 + R1
//         bytecode.add_instruction(add_instr);
//
//         // Écrire le fichier
//         bytecode
//             .write_to_file(&file_path)
//             .expect("Impossible d'écrire le fichier bytecode");
//
//         // Lire le fichier
//         let loaded = BytecodeFile::read_from_file(&file_path)
//             .expect("Impossible de lire le fichier bytecode");
//
//         // Vérifier que l'instruction est correctement chargée
//         assert_eq!(loaded.code.len(), 1);
//         assert_eq!(loaded.code[0].opcode, Opcode::Add);
//
//         // Vérifier les valeurs des registres
//         assert_eq!(loaded.code[0].args.len(), 3);
//         assert_eq!(loaded.code[0].args[0], 2); // Rd
//         assert_eq!(loaded.code[0].args[1], 0); // Rs1
//         assert_eq!(loaded.code[0].args[2], 1); // Rs2
//     }
//
//     #[test]
//     fn test_bytecode_file_extended_size_io() {
//         // Test d'écriture et lecture d'un fichier avec une instruction de grande taille
//         let dir = tempdir().expect("Impossible de créer un répertoire temporaire");
//         let file_path = dir.path().join("test_extended.punk");
//
//         // Création du fichier bytecode
//         let mut bytecode = BytecodeFile::new();
//
//         // Créer une instruction avec beaucoup de données pour forcer un size_type Extended
//         let large_args = vec![0; 248]; // Suffisant pour dépasser la limite de 255 octets
//         let large_instr =
//             Instruction::new(Opcode::Add, InstructionFormat::double_reg(), large_args);
//
//         bytecode.add_instruction(large_instr);
//
//         // Écrire le fichier
//         bytecode
//             .write_to_file(&file_path)
//             .expect("Impossible d'écrire le fichier bytecode");
//
//         // Lire le fichier
//         let loaded = BytecodeFile::read_from_file(&file_path)
//             .expect("Impossible de lire le fichier bytecode");
//
//         // Vérifier que l'instruction est correctement chargée
//         assert_eq!(loaded.code.len(), 1);
//         assert_eq!(loaded.code[0].opcode, Opcode::Add);
//         assert_eq!(loaded.code[0].args.len(), 248);
//     }
//
//     #[test]
//     fn test_bytecode_file_complex_program() {
//         // Créer un programme complet avec des instructions variées
//         let mut bytecode = BytecodeFile::new();
//
//         // Ajouter des métadonnées
//         bytecode.add_metadata("name", "Programme de test");
//         bytecode.add_metadata("author", "PunkVM Team");
//         bytecode.add_metadata("version", "1.0.0");
//
//         // Initialisation des registres
//         bytecode.add_instruction(Instruction::create_reg_imm8(Opcode::Load, 0, 10)); // R0 = 10
//         bytecode.add_instruction(Instruction::create_reg_imm8(Opcode::Load, 1, 5)); // R1 = 5
//
//         // Opérations arithmétiques
//         bytecode.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 2, 0, 1)); // R2 = R0 + R1
//         bytecode.add_instruction(Instruction::create_reg_reg_reg(Opcode::Sub, 3, 0, 1)); // R3 = R0 - R1
//         bytecode.add_instruction(Instruction::create_reg_reg_reg(Opcode::Mul, 4, 0, 1)); // R4 = R0 * R1
//         bytecode.add_instruction(Instruction::create_reg_reg_reg(Opcode::Div, 5, 0, 1)); // R5 = R0 / R1
//
//         // Opérations logiques
//         bytecode.add_instruction(Instruction::create_reg_reg_reg(Opcode::And, 6, 0, 1)); // R6 = R0 & R1
//         bytecode.add_instruction(Instruction::create_reg_reg_reg(Opcode::Or, 7, 0, 1)); // R7 = R0 | R1
//
//         // Fin du programme
//         bytecode.add_instruction(Instruction::create_no_args(Opcode::Halt));
//
//         // Ajouter un symbole pour le début du programme
//         bytecode.add_symbol("start", 0);
//
//         // Vérifier le nombre d'instructions
//         assert_eq!(bytecode.code.len(), 9);
//
//         // Vérifier les métadonnées
//         assert_eq!(bytecode.metadata.len(), 3);
//         assert_eq!(
//             bytecode.metadata.get("name"),
//             Some(&"Programme de test".to_string())
//         );
//
//         // Vérifier les symboles
//         assert_eq!(bytecode.symbols.len(), 1);
//         assert_eq!(bytecode.symbols.get("start"), Some(&0));
//     }
//
//     #[test]
//     fn test_bytecode_file_io_errors() {
//         // Test avec un fichier inexistant
//         let result = BytecodeFile::read_from_file("nonexistent_file.punk");
//         assert!(result.is_err());
//
//         // Test avec un fichier trop petit
//         let dir = tempdir().expect("Impossible de créer un répertoire temporaire");
//         let invalid_file_path = dir.path().join("invalid.punk");
//
//         // Créer un fichier invalide avec juste quelques octets
//         std::fs::write(&invalid_file_path, &[0, 1, 2])
//             .expect("Impossible d'écrire le fichier de test");
//
//         let result = BytecodeFile::read_from_file(&invalid_file_path);
//         assert!(result.is_err());
//
//         // Vérifier le type d'erreur
//         match result {
//             Err(e) => assert_eq!(e.kind(), ErrorKind::InvalidData),
//             _ => panic!("Expected an error but got success"),
//         }
//     }
//
//     #[test]
//     fn test_encode_decode_metadata() {
//         let mut metadata = HashMap::new();
//         metadata.insert("key1".to_string(), "value1".to_string());
//         metadata.insert("key2".to_string(), "value2".to_string());
//
//         let mut bytecode = BytecodeFile::new();
//         bytecode.metadata = metadata.clone();
//
//         let encoded = bytecode.encode_metadata();
//         let decoded = BytecodeFile::decode_metadata(&encoded).expect("Failed to decode metadata");
//
//         assert_eq!(decoded.len(), 2);
//         assert_eq!(decoded.get("key1"), Some(&"value1".to_string()));
//         assert_eq!(decoded.get("key2"), Some(&"value2".to_string()));
//     }
//
//     #[test]
//     fn test_encode_decode_symbols() {
//         let mut symbols = HashMap::new();
//         symbols.insert("sym1".to_string(), 0x1000);
//         symbols.insert("sym2".to_string(), 0x2000);
//
//         let mut bytecode = BytecodeFile::new();
//         bytecode.symbols = symbols.clone();
//
//         let encoded = bytecode.encode_symbols();
//         let decoded = BytecodeFile::decode_symbols(&encoded).expect("Failed to decode symbols");
//
//         assert_eq!(decoded.len(), 2);
//         assert_eq!(decoded.get("sym1"), Some(&0x1000));
//         assert_eq!(decoded.get("sym2"), Some(&0x2000));
//     }
//
//     #[test]
//     fn test_encode_decode_code() {
//         // Créer un ensemble d'instructions de test
//         let mut code = Vec::new();
//         code.push(Instruction::create_no_args(Opcode::Nop));
//         code.push(Instruction::create_reg_imm8(Opcode::Load, 0, 42));
//         code.push(Instruction::create_reg_reg_reg(Opcode::Add, 2, 0, 1));
//
//         let mut bytecode = BytecodeFile::new();
//         bytecode.code = code.clone();
//
//         let encoded = bytecode.encode_code();
//         let decoded = BytecodeFile::decode_code(&encoded).expect("Failed to decode code");
//
//         assert_eq!(decoded.len(), 3);
//         assert_eq!(decoded[0].opcode, Opcode::Nop);
//         assert_eq!(decoded[1].opcode, Opcode::Load);
//         assert_eq!(decoded[2].opcode, Opcode::Add);
//
//         // Vérifier les arguments de l'instruction Add
//         assert_eq!(decoded[2].args.len(), 3);
//         assert_eq!(decoded[2].args[0], 2);
//         assert_eq!(decoded[2].args[1], 0);
//         assert_eq!(decoded[2].args[2], 1);
//     }
// }
//
//
// #[cfg(test)]
// mod pipeline_mod_tests {
//     use super::*; // Importe Pipeline et les structures du mod.rs
//     use crate::alu::alu::ALU;
//     use crate::bytecode::instructions::Instruction;
//     use crate::bytecode::opcodes::Opcode;
//     use crate::pvm::memorys::{Memory, MemoryConfig}; // Assurez-vous que les chemins sont corrects
//     use crate::pipeline::{FetchDecodeRegister, DecodeExecuteRegister, ExecuteMemoryRegister, MemoryWritebackRegister};
//
//     // Helper pour créer une VM minimale pour les tests de pipeline
//     fn setup_test_pipeline(instructions: Vec<Instruction>) -> (Pipeline, Vec<u64>, Memory, ALU) {
//         let pipeline = Pipeline::new(4, true, true); // Buffersize 4, forwarding/hazard ON
//         let registers = vec![0u64; 16];
//         let memory_config = MemoryConfig { size: 1024, l1_cache_size: 64, store_buffer_size: 4 };
//         let memory = Memory::new(memory_config);
//         let alu = ALU::new();
//         // Note: La mémoire n'est pas pré-chargée avec les instructions ici.
//         // La fonction `cycle` reçoit le slice `instructions` directement.
//         (pipeline, registers, memory, alu)
//     }
//
//     #[test]
//     fn test_pipeline_simple_add() {
//         // Instructions: ADD R2, R0, R1; HALT
//         let instructions = vec![
//             Instruction::create_reg_reg_reg(Opcode::Add, 2, 0, 1), // R2 = R0 + R1
//             Instruction::create_no_args(Opcode::Halt),
//         ];
//         let (mut pipeline, mut registers, mut memory, mut alu) = setup_test_pipeline(instructions.clone());
//
//         // Initialiser les registres
//         registers[0] = 10;
//         registers[1] = 5;
//
//         // Simuler les cycles
//         let mut current_pc: u32 = 0;
//         let max_cycles = 10; // Éviter boucle infinie
//         for _ in 0..max_cycles {
//             let result = pipeline.cycle(current_pc, &mut registers, &mut memory, &mut alu, &instructions);
//             assert!(result.is_ok(), "Le cycle du pipeline a échoué");
//             let state = result.unwrap();
//             current_pc = state.next_pc; // Mettre à jour le PC pour le prochain cycle
//             println!("Cycle: {}, PC: 0x{:X}, Halted: {}, Stalled: {}",
//                      pipeline.stats.cycles, current_pc, state.halted, state.stalled);
//             if state.halted {
//                 break;
//             }
//         }
//
//         assert!(pipeline.state.halted, "Le pipeline devrait être arrêté (halted)");
//         // ADD prend ~5 cycles pour compléter (F-D-E-M-W)
//         assert!(pipeline.stats.cycles >= 5, "Il faut au moins 5 cycles");
//         assert_eq!(registers[2], 15, "R2 devrait contenir 10 + 5"); // Vérifier le résultat
//         assert_eq!(pipeline.stats.instructions, 1, "Une seule instruction (ADD) devrait être complétée");
//     }
//
//     #[test]
//     fn test_pipeline_data_hazard_raw_forwarding() {
//         // Instructions: ADD R1, R0, R0; ADD R2, R1, R1; HALT
//         let instructions = vec![
//             Instruction::create_reg_reg_reg(Opcode::Add, 1, 0, 0), // R1 = R0 + R0 (R1=10)
//             Instruction::create_reg_reg_reg(Opcode::Add, 2, 1, 1), // R2 = R1 + R1 (dépend de R1)
//             Instruction::create_no_args(Opcode::Halt),
//         ];
//         let (mut pipeline, mut registers, mut memory, mut alu) = setup_test_pipeline(instructions.clone());
//         registers[0] = 5; // R0 = 5
//
//         // Simuler les cycles
//         let mut current_pc: u32 = 0;
//         let max_cycles = 15;
//         for _ in 0..max_cycles {
//             let result = pipeline.cycle(current_pc, &mut registers, &mut memory, &mut alu, &instructions);
//             assert!(result.is_ok());
//             let state = result.unwrap();
//             current_pc = state.next_pc;
//             if state.halted { break; }
//         }
//
//         assert!(pipeline.state.halted);
//         assert_eq!(registers[1], 10, "R1 devrait être 5 + 5");
//         assert_eq!(registers[2], 20, "R2 devrait être 10 + 10 (avec forwarding)");
//         assert!(pipeline.forwarding.get_forwards_count() >= 1, "Au moins un forward aurait dû se produire pour R1");
//         // Le nombre exact de cycles dépend si le forwarding évite complètement le stall
//         // S'il y a un stall d'un cycle malgré le forwarding: cycles = 5 (ADD1) + 1 (stall) + 1 (ADD2 complété) = 7?
//         // S'il n'y a pas de stall: cycles = 5 (ADD1) + 1 (ADD2 complété) = 6?
//         println!("Stats: Cycles={}, Instructions={}, Forwards={}, Stalls={}",
//                  pipeline.stats.cycles, pipeline.stats.instructions,
//                  pipeline.forwarding.get_forwards_count(), pipeline.stats.stalls);
//         assert_eq!(pipeline.stats.stalls, 0, "Aucun stall ne devrait être nécessaire avec forwarding EX->DE");
//         assert_eq!(pipeline.stats.instructions, 2);
//
//     }
//     //
    // #[test]
    // fn test_pipeline_load_use_hazard_stall() {
    //     // Instructions: LOAD R1, [0]; ADD R2, R1, R0; HALT
    //     let load_addr = 0x100; // Adresse arbitraire
    //     let instructions = vec![
    //         Instruction::create_reg_imm8(Opcode::Load, 1, 0), // LOAD R1, [0x100] (simplifié: addr dans immediate)
    //         Instruction::create_reg_reg_reg(Opcode::Add, 2, 1, 0), // ADD R2, R1, R0 (utilise R1)
    //         Instruction::create_no_args(Opcode::Halt),
    //     ];
    //     let (mut pipeline, mut registers, mut memory, mut alu) = setup_test_pipeline(instructions.clone());
    //
    //     // Mettre une valeur en mémoire
    //     memory.write_qword(load_addr, 99).unwrap();
    //     registers[0] = 1; // R0 = 1
    //
    //     // Simuler les cycles
    //     let mut current_pc: u32 = 0;
    //     let max_cycles = 15;
    //     for i in 0..max_cycles {
    //         println!("--- Test Cycle {} ---", i + 1);
    //         let result = pipeline.cycle(current_pc, &mut registers, &mut memory, &mut alu, &instructions);
    //         assert!(result.is_ok());
    //         let state = result.unwrap();
    //         current_pc = state.next_pc;
    //         if state.halted { break; }
    //     }
    //
    //     assert!(pipeline.state.halted);
    //     assert_eq!(registers[1], 99, "R1 doit contenir la valeur chargée");
    //     assert_eq!(registers[2], 100, "R2 doit être 99 + 1");
    //     println!("Stats: Cycles={}, Instructions={}, Forwards={}, Stalls={}",
    //              pipeline.stats.cycles, pipeline.stats.instructions,
    //              pipeline.forwarding.get_forwards_count(), pipeline.stats.stalls);
    //     assert!(pipeline.stats.stalls >= 1, "Au moins un stall est attendu pour Load-Use");
    //     assert_eq!(pipeline.stats.instructions, 2);
    //     assert!(pipeline.hazard_detection.hazards_count >= 1, "Au moins un hazard LoadUse détecté");
    // }
    //
    //
    // #[test]
    // fn test_pipeline_branch_taken_correctly_predicted() {
    //     // JMP_IF_EQUAL target ; ADD R0, R0, 1 (ne devrait pas s'exécuter); target: HALT
    //     let target_offset = 8; // Taille JIE + ADD = 4 + 4 = 8 ? Vérifier tailles!
    //     let branch_instr = Instruction::create_jump_if_equal(target_offset);
    //     let nop_instr = Instruction::create_no_args(Opcode::Nop); // Utiliser NOP pour simplicité taille
    //     let halt_instr = Instruction::create_no_args(Opcode::Halt);
    //
    //     // Calcul des tailles réelles
    //     let branch_size = branch_instr.total_size() as u32;
    //     let nop_size = nop_instr.total_size() as u32;
    //     // L'offset doit pointer *après* le NOP vers HALT
    //     let correct_offset = nop_size as i32;
    //     let branch_instr_corrected = Instruction::create_jump_if_equal(correct_offset);
    //
    //
    //     let instructions = vec![
    //         Instruction::create_reg_reg_reg(Opcode::Sub, 0, 0, 0), // SUB R0, R0, R0 => Met Z=1
    //         branch_instr_corrected.clone(),                      // JIE target (devrait être pris)
    //         nop_instr.clone(),                                   // NOP (devrait être sauté)
    //         halt_instr.clone(),                                  // target: HALT
    //     ];
    //     let (mut pipeline, mut registers, mut memory, mut alu) = setup_test_pipeline(instructions.clone());
    //
    //     // Forcer la prédiction initiale à Taken (pour tester prédiction correcte)
    //     let branch_pc = instructions[0].total_size() as u64;
    //     pipeline.decode.branch_predictor.two_bit_states.insert(branch_pc, crate::pvm::branch_predictor::TwoBitState::StronglyTaken);
    //
    //
    //     // Simuler les cycles
    //     let mut current_pc: u32 = 0;
    //     let max_cycles = 15;
    //     for i in 0..max_cycles {
    //         println!("--- Test Cycle {} ---", i + 1);
    //         let result = pipeline.cycle(current_pc, &mut registers, &mut memory, &mut alu, &instructions);
    //         assert!(result.is_ok());
    //         let state = result.unwrap();
    //         current_pc = state.next_pc;
    //         if state.halted { break; }
    //     }
    //
    //     assert!(pipeline.state.halted);
    //     println!("Stats: Cycles={}, Instructions={}, Branches={}, Hits={}, Misses={}, Flushes={}, Stalls={}",
    //              pipeline.stats.cycles, pipeline.stats.instructions,
    //              pipeline.stats.branch_predictions, pipeline.stats.branch_hits,
    //              pipeline.stats.branch_misses, pipeline.stats.branch_flush,
    //              pipeline.stats.stalls);
    //
    //     assert_eq!(pipeline.stats.branch_predictions, 1, "Une seule prédiction de branchement");
    //     assert_eq!(pipeline.stats.branch_hits, 1, "La prédiction doit être correcte");
    //     assert_eq!(pipeline.stats.branch_misses, 0, "Aucune misprediction");
    //     assert_eq!(pipeline.stats.branch_flush, 0, "Aucun flush nécessaire");
    //     // Le nombre d'instructions complétées: SUB, JIE, HALT = 3
    //     assert_eq!(pipeline.stats.instructions, 3);
    // }



///////////////////////////////////////////////////////
//     pub fn cycle(
//         &mut self,
//         pc: u32,
//         registers: &mut [u64],
//         memory: &mut Memory,
//         alu: &mut ALU,
//         instructions: &[Instruction],
//     ) -> Result<PipelineState, String> {
//         self.stats.cycles += 1;
//         println!("DEBUG: Cycle {} Start - Current PC = 0x{:X}", self.stats.cycles, pc);
//
//         // 1. Préparer le nouvel état (basé sur l'ancien)
//         let mut next_pipeline_state = self.state.clone();
//         next_pipeline_state.stalled = false; // Sera peut-être mis à true par les hazards
//         next_pipeline_state.instructions_completed = 0;
//         // Important : le `next_pc` de l'état actuel est le PC *attendu* pour CE cycle.
//         let current_pc_target = self.state.next_pc; // Le PC que Fetch devrait utiliser
//
//         // --- Exécution des étages (ordre logique inverse pour faciliter la propagation) ---
//         // Cela évite d'utiliser les données *juste* calculées dans le même cycle.
//
//         // 5. Writeback Stage
//         let mut completed_in_wb = false;
//         if let Some(mem_wb_reg) = &self.state.memory_writeback { // Lire l'état *précédent*
//             match self.writeback.process_direct(mem_wb_reg, registers) {
//                 Ok(_) => {
//                     println!("DEBUG: WB successful for PC=0x{:X} ({:?})", mem_wb_reg.instruction., mem_wb_reg.instruction.opcode); // Ajout PC si dispo
//                     next_pipeline_state.instructions_completed += 1;
//                     self.stats.instructions += 1; // Compter instruction terminée
//                     completed_in_wb = true;
//                 },
//                 Err(e) => return Err(format!("Writeback Error: {}", e)),
//             }
//         }
//         // Vider le registre MEM/WB pour le prochain cycle
//         next_pipeline_state.memory_writeback = None;
//
//
//         // 4. Memory Stage
//         let mut mem_wb_output: Option<MemoryWritebackRegister> = None;
//         if let Some(ex_mem_reg) = &self.state.execute_memory { // Lire l'état *précédent*
//             match self.memory.process_direct(ex_mem_reg, memory) {
//                 Ok(wb_reg) => {
//                     println!("DEBUG: MEM successful for PC=0x{:X} ({:?})", ex_mem_reg.pc, ex_mem_reg.instruction.opcode); // Ajout PC si dispo
//                     mem_wb_output = Some(wb_reg);
//                     // Gestion HALT spécifique
//                     if ex_mem_reg.halted { // Vérifier le flag Halted venant d'Execute
//                         println!("DEBUG: HALT detected in MEM stage. Setting pipeline to halted.");
//                         next_pipeline_state.halted = true;
//                         // Important: Ne pas flusher ici, laisser le pipeline se vider des étages précédents
//                         // Mais ne plus rien chercher (Fetch sera bloqué par halted).
//                         // On propage quand même le résultat du HALT (qui est vide) vers WB.
//                     }
//                 },
//                 Err(e) => return Err(format!("Memory Error: {}", e)),
//             }
//         }
//         // Mettre à jour le registre MEM/WB pour le prochain cycle
//         next_pipeline_state.memory_writeback = mem_wb_output;
//
//
//         // --- Détection des Hazards & Application du Forwarding ---
//         // Important: Basé sur l'état *avant* l'exécution de DE et EX de ce cycle.
//         let mut hazard_stall_needed = false;
//         if self.enable_hazard_detection {
//             // Utilise l'état *actuel* pour voir s'il y a des dépendances ou conflits
//             // qui nécessitent un stall *pour le prochain cycle*.
//             if self.hazard_detection.detect_stall_hazard(&self.state) { // Vérifie Load-Use, Control simple, Structural
//                 println!("DEBUG: STALL required by Hazard Detection Unit.");
//                 hazard_stall_needed = true;
//                 self.stats.stalls += 1;
//                 // hazards_count est déjà incrémenté dans detect_stall_hazard
//             }
//         }
//         // Appliquer le stall maintenant si nécessaire
//         next_pipeline_state.stalled = hazard_stall_needed;
//
//
//         // --- Préparation pour Execute & Decode ---
//         // On clone le registre DE/EX *avant* le forwarding pour la logique de branchement
//         let de_reg_before_forward = self.state.decode_execute.clone();
//
//         // Appliquer le Forwarding (modifie le contenu du registre DE/EX *pour l'étage Execute*)
//         let mut ex_mem_input = self.state.decode_execute.clone(); // Prendre l'entrée de EX
//         if self.enable_forwarding {
//             if let Some(ref mut de_reg_to_forward) = ex_mem_input {
//                 println!("DEBUG: Applying forwarding for EX stage input (PC=0x{:X})", de_reg_to_forward.pc);
//                 let _forwarding_info = self.forwarding.forward(
//                     de_reg_to_forward,
//                     &self.state.execute_memory, // Source EX/MEM de l'état précédent
//                     &self.state.memory_writeback, // Source MEM/WB de l'état précédent
//                 );
//                 // Mettre à jour les stats de forwarding (déjà fait dans forward())
//                 self.stats.forwards = self.forwarding.get_forwards_count();
//             }
//         }
//
//
//         // 3. Execute Stage
//         let mut ex_mem_output: Option<ExecuteMemoryRegister> = None;
//         let mut branch_mispredicted = false;
//         let mut correct_next_pc_on_mispredict = current_pc_target; // Default, sera écrasé si mispredict
//
//         if !next_pipeline_state.stalled { // Ne pas exécuter si stall général
//             if let Some(de_reg) = &ex_mem_input { // Utiliser l'état potentiellement forwardé
//                 match self.execute.process_direct(de_reg, alu) {
//                     Ok(mut mem_reg) => { // `mut` pour pouvoir potentiellement corriger prediction_correct
//                         println!("DEBUG: EX successful for PC=0x{:X} ({:?})", de_reg.pc, de_reg.instruction.opcode);
//                         ex_mem_output = Some(mem_reg.clone()); // Sauvegarder le résultat
//
//                         // --- Gestion Spécifique des Branchements ICI ---
//                         if de_reg.instruction.opcode.is_branch() {
//                             println!("DEBUG: Branch instruction resolved in EX. PC=0x{:X}, Taken={}, Target={:?}, Predicted={:?}",
//                                      de_reg.pc, mem_reg.branch_taken, mem_reg.branch_target, de_reg.branch_prediction);
//                             self.stats.branch_predictions += 1; // Compter chaque branche résolue
//
//                             let prediction = de_reg.branch_prediction.unwrap_or(BranchPrediction::NotTaken); // Default prediction if None
//                             let actual_taken = mem_reg.branch_taken;
//
//                             // Comparer prédiction et résultat réel
//                             if (prediction == BranchPrediction::Taken) != actual_taken {
//                                 // *** MISPREDICTION ***
//                                 branch_mispredicted = true;
//                                 self.stats.branch_misses += 1;
//                                 self.stats.branch_flush += 1; // Compter le flush causé
//                                 println!("DEBUG: Branch MISPREDICTED! PC=0x{:X}", de_reg.pc);
//
//                                 // Déterminer le PC correct
//                                 if actual_taken {
//                                     correct_next_pc_on_mispredict = mem_reg.branch_target.expect("Branch taken but no target!");
//                                     println!("   >> Mispredict recovery: Branch was TAKEN. Correct PC = 0x{:X}", correct_next_pc_on_mispredict);
//                                 } else {
//                                     correct_next_pc_on_mispredict = de_reg.pc.wrapping_add(de_reg.instruction.total_size() as u32);
//                                     println!("   >> Mispredict recovery: Branch was NOT TAKEN. Correct PC = 0x{:X}", correct_next_pc_on_mispredict);
//                                 }
//                                 // Marquer comme incorrect dans le registre pour info (même si on flush)
//                                 if let Some(ref mut output) = ex_mem_output {
//                                     output.branch_prediction_correct = Some(false);
//                                 }
//
//                             } else {
//                                 // *** PREDICTION CORRECTE ***
//                                 self.stats.branch_hits += 1;
//                                 println!("DEBUG: Branch PREDICTED correctly. PC=0x{:X}", de_reg.pc);
//                                 // Marquer comme correct
//                                 if let Some(ref mut output) = ex_mem_output {
//                                     output.branch_prediction_correct = Some(true);
//                                 }
//                             }
//
//                             // Mettre à jour le prédicteur de branchement (toujours après résolution)
//                             self.decode.branch_predictor.update(de_reg.pc as u64, actual_taken, prediction);
//                         }
//
//                     },
//                     Err(e) => return Err(format!("Execute Error: {}", e)),
//                 }
//             }
//         } else {
//             println!("DEBUG: EX stage stalled.");
//             // Propager la bulle (None)
//             ex_mem_output = None;
//         }
//         // Mettre à jour le registre EX/MEM pour le prochain cycle
//         next_pipeline_state.execute_memory = ex_mem_output;
//
//
//         // 2. Decode Stage
//         let mut de_ex_output: Option<DecodeExecuteRegister> = None;
//         if !next_pipeline_state.stalled { // Ne pas décoder si stall général
//             // Si misprediction => insérer bulle en Decode
//             if branch_mispredicted {
//                 println!("DEBUG: Flushing DE stage due to misprediction.");
//                 de_ex_output = None;
//             } else if let Some(fd_reg) = &self.state.fetch_decode { // Lire l'état *précédent*
//                 match self.decode.process_direct(fd_reg, registers) {
//                     Ok(mut de_reg) => { // `mut` pour injecter prédiction
//                         println!("DEBUG: DE successful for PC=0x{:X} ({:?})", fd_reg.pc, fd_reg.instruction.opcode);
//
//                         // Injecter la prédiction si c'est un branchement
//                         if de_reg.instruction.opcode.is_branch() {
//                             let prediction = self.decode.branch_predictor.predict(de_reg.pc as u64);
//                             de_reg.branch_prediction = Some(prediction);
//                             println!("   >> Branch predicted in DE: {:?} for PC=0x{:X}", prediction, de_reg.pc);
//                             // Si prédit pris, on pourrait mettre à jour next_pc *spéculativement*
//                             // if prediction == BranchPrediction::Taken {
//                             //    if let Some(target) = de_reg.branch_addr { // Utiliser l'adresse calculée en DE
//                             //       next_pipeline_state.next_pc = target; // Mise à jour spéculative
//                             //       println!("   >> Speculative PC update for predicted taken branch: 0x{:X}", target);
//                             //    }
//                             // }
//                         }
//
//                         de_ex_output = Some(de_reg);
//                     },
//                     Err(e) => return Err(format!("Decode Error: {}", e)),
//                 }
//             }
//         } else {
//             println!("DEBUG: DE stage stalled.");
//             // Propager la bulle
//             de_ex_output = None;
//         }
//         // Mettre à jour le registre DE/EX pour le prochain cycle
//         next_pipeline_state.decode_execute = de_ex_output;
//
//
//         // 1. Fetch Stage
//         let mut fd_output: Option<FetchDecodeRegister> = None;
//         let pc_to_fetch = if branch_mispredicted {
//             // Si mispredict, utiliser le PC corrigé
//             correct_next_pc_on_mispredict
//         } else {
//             // Sinon, utiliser le PC cible de ce cycle
//             current_pc_target
//         };
//
//         if !next_pipeline_state.stalled { // Ne pas fetcher si stall général
//             // Si misprediction => insérer bulle en Fetch
//             if branch_mispredicted {
//                 println!("DEBUG: Flushing FD stage due to misprediction.");
//                 fd_output = None;
//             } else if !next_pipeline_state.halted { // Ne pas fetcher si HALT signalé
//                 match self.fetch.process_direct(pc_to_fetch, instructions) {
//                     Ok(fd_reg) => {
//                         println!("DEBUG: IF successful for PC=0x{:X} ({:?})", pc_to_fetch, fd_reg.instruction.opcode);
//                         fd_output = Some(fd_reg.clone());
//
//                         // --- Mise à jour du Prochain PC ---
//                         // Si on vient de fetcher une instruction de branchement prédite prise
//                         let mut predicted_taken_target: Option<u32> = None;
//                         if let Some(de_output) = &next_pipeline_state.decode_execute { // Regarder ce qui vient d'être décodé
//                             if de_output.instruction.opcode.is_branch() && de_output.branch_prediction == Some(BranchPrediction::Taken) {
//                                 predicted_taken_target = de_output.branch_addr;
//                                 println!("   >> Branch TAKEN predicted for instruction at 0x{:X}. Target=0x{:X}", de_output.pc, predicted_taken_target.unwrap_or(0));
//                             }
//                         }
//
//                         // Calculer le PC suivant normal (instruction+taille)
//                         let pc_after_current = pc_to_fetch.wrapping_add(fd_reg.instruction.total_size() as u32);
//
//                         // Choisir le prochain PC
//                         if let Some(target) = predicted_taken_target {
//                             next_pipeline_state.next_pc = target; // Spéculatif basé sur prédiction 'Taken'
//                             println!("   >> NEXT PC set to predicted target: 0x{:X}", target);
//                         } else {
//                             next_pipeline_state.next_pc = pc_after_current; // Normal ou branche prédite 'Not Taken'
//                             println!("   >> NEXT PC set sequentially or predicted not taken: 0x{:X}", pc_after_current);
//                         }
//
//                     },
//                     // Gérer l'erreur de fetch (ex: PC invalide)
//                     Err(e) => {
//                         // Si Fetch échoue (ex: fin du programme sans HALT), on peut vouloir s'arrêter
//                         println!("ERROR: Fetch failed at PC=0x{:X}: {}. Halting.", pc_to_fetch, e);
//                         next_pipeline_state.halted = true; // Considérer comme un arrêt
//                         // Pas besoin de retourner une erreur ici, le prochain cycle verra halted=true
//                         fd_output = None; // Pas d'instruction fetchée
//                         next_pipeline_state.next_pc = pc_to_fetch; // Garder le PC où l'erreur s'est produite
//                     }
//                 }
//             } else {
//                 println!("DEBUG: IF stage halted or stalled.");
//                 // Si halted ou stalled, ne rien fetcher et ne pas changer next_pc
//                 fd_output = None;
//                 next_pipeline_state.next_pc = pc_to_fetch; // Maintient le PC
//             }
//         } else {
//             println!("DEBUG: IF stage stalled.");
//             // Si stall général, ne rien fetcher et ne pas changer next_pc
//             fd_output = None;
//             next_pipeline_state.next_pc = pc_to_fetch; // Maintient le PC
//         }
//         // Mettre à jour le registre IF/ID pour le prochain cycle
//         next_pipeline_state.fetch_decode = fd_output;
//
//
//         // --- Finalisation du Cycle ---
//         // Mettre à jour l'état global du pipeline pour le prochain cycle
//         self.state = next_pipeline_state.clone();
//
//         // Mettre à jour les stats globales à la fin
//         self.stats.branch_predictor_rate = self.decode.branch_predictor.get_accuracy(); // Recalculer le taux
//
//         println!("DEBUG: Cycle {} End - Next PC=0x{:X}, Halted={}, Stalled={}", self.stats.cycles, self.state.next_pc, self.state.halted, self.state.stalled);
//         println!("---"); // Séparateur de cycle
//
//         // Retourner l'état à la fin de CE cycle
//         Ok(self.state.clone())
//     }

//////////////////////////////////////////////////////////////////////////////////////
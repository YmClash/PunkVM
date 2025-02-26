

use crate::bytecode::instructions::{ArgValue, Instruction};
use crate::bytecode::opcodes::Opcode;
use crate::pipeline::{FetchDecodeRegister, DecodeExecuteRegister, stage::PipelineStage};

/// implementation de l'étage Decode du pipeline
pub struct DecodeStage {
    /// Registre intermédiaire Decode -> Execute
    // pub decode_register: Option<DecodeExecuteRegister>,
    //données de l'état interne si nécessaire
}


impl DecodeStage {
    /// Crée un nouvel étage Decode
    pub fn new() -> Self {
        Self {}
    }

    /// Traite l'étage Decode
    pub fn process(&mut self, fd_reg: &FetchDecodeRegister, registers: &[u64]) -> Result<DecodeExecuteRegister, String> {
        let instruction = &fd_reg.instruction;

        // Extraction des registres source et destination
        let (rs1, rs2, rd) = self.extract_registers(instruction)?;

        // Extraction de la valeur immédiate
        let immediate = self.extract_immediate(instruction)?;

        // Calcul de l'adresse de branchement (si instruction de branchement)
        let branch_addr = self.calculate_branch_address(instruction, fd_reg.pc)?;

        // Calcul de l'adresse mémoire (si instruction mémoire)
        let mem_addr = self.calculate_memory_address(instruction, registers)?;

        Ok(DecodeExecuteRegister {
            instruction: instruction.clone(),
            pc: fd_reg.pc,
            rs1,
            rs2,
            rd,
            immediate,
            branch_addr,
            mem_addr,
        })
    }

    /// Extrait les registres source et destination
    fn extract_registers(&self, instruction: &Instruction) -> Result<(Option<usize>, Option<usize>, Option<usize>), String> {
        let mut rs1 = None;
        let mut rs2 = None;
        let mut rd = None;

        // Extraction en fonction du type d'instruction
        match instruction.opcode {
            // Instructions à deux registres (destination = premier argument)
            Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::Div |
            Opcode::And | Opcode::Or | Opcode::Xor | Opcode::Shl |
            Opcode::Shr | Opcode::Sar | Opcode::Rol | Opcode::Ror => {
                if let Ok(ArgValue::Register(r)) = instruction.get_arg1_value() {
                    rd = Some(r as usize);
                    rs1 = Some(r as usize); // Dans certaines architectures, rd est aussi rs1
                }

                if let Ok(ArgValue::Register(r)) = instruction.get_arg2_value() {
                    rs2 = Some(r as usize);
                }
            },

            // Instructions à un registre (destination = premier argument)
            Opcode::Inc | Opcode::Dec | Opcode::Neg | Opcode::Not => {
                if let Ok(ArgValue::Register(r)) = instruction.get_arg1_value() {
                    rd = Some(r as usize);
                    rs1 = Some(r as usize); // Le registre est à la fois source et destination
                }
            },

            // Instructions de comparaison (pas de registre destination)
            Opcode::Cmp | Opcode::Test => {
                if let Ok(ArgValue::Register(r)) = instruction.get_arg1_value() {
                    rs1 = Some(r as usize);
                }

                if let Ok(ArgValue::Register(r)) = instruction.get_arg2_value() {
                    rs2 = Some(r as usize);
                }
            },

            // Instructions de charge (load)
            Opcode::Load | Opcode::LoadB | Opcode::LoadW | Opcode::LoadD => {
                if let Ok(ArgValue::Register(r)) = instruction.get_arg1_value() {
                    rd = Some(r as usize);
                }

                // Extraction du registre base pour les adresses indexées
                if let Ok(ArgValue::RegisterOffset(r, _)) = instruction.get_arg2_value() {
                    rs1 = Some(r as usize);
                }
            },

            // Instructions de stockage (store)
            Opcode::Store | Opcode::StoreB | Opcode::StoreW | Opcode::StoreD => {
                if let Ok(ArgValue::Register(r)) = instruction.get_arg1_value() {
                    rs1 = Some(r as usize); // Registre contenant la valeur à stocker
                }

                // Extraction du registre base pour les adresses indexées
                if let Ok(ArgValue::RegisterOffset(r, _)) = instruction.get_arg2_value() {
                    rs2 = Some(r as usize);
                }
            },

            // Instructions de pile
            Opcode::Push => {
                if let Ok(ArgValue::Register(r)) = instruction.get_arg1_value() {
                    rs1 = Some(r as usize);
                }
            },

            Opcode::Pop => {
                if let Ok(ArgValue::Register(r)) = instruction.get_arg1_value() {
                    rd = Some(r as usize);
                }
            },

            // Instructions de branchement conditionnel
            Opcode::JmpIf | Opcode::JmpIfNot => {
                // Ces instructions n'utilisent pas explicitement de registres,
                // mais se basent sur les flags définis par les instructions précédentes
            },

            // Autres instructions (par défaut)
            _ => {},
        }

        Ok((rs1, rs2, rd))
    }

    /// Extrait la valeur immédiate (si présente)
    fn extract_immediate(&self, instruction: &Instruction) -> Result<Option<u64>, String> {
        // Recherche d'une valeur immédiate dans les arguments
        match instruction.get_arg1_value() {
            Ok(ArgValue::Immediate(imm)) => return Ok(Some(imm)),
            _ => {},
        }

        match instruction.get_arg2_value() {
            Ok(ArgValue::Immediate(imm)) => return Ok(Some(imm)),
            _ => {},
        }

        Ok(None)
    }

    /// Calcule l'adresse de branchement (si instruction de branchement)
    fn calculate_branch_address(&self, instruction: &Instruction, pc: u32) -> Result<Option<u32>, String> {
        // Vérifier si c'est une instruction de branchement
        if !instruction.opcode.is_branch() {
            return Ok(None);
        }

        match instruction.opcode {
            // Saut absolu
            Opcode::Jmp => {
                match instruction.get_arg2_value() {
                    Ok(ArgValue::AbsoluteAddr(addr)) => Ok(Some(addr as u32)),
                    Ok(ArgValue::RelativeAddr(offset)) => Ok(Some((pc as i64 + offset as i64) as u32)),
                    _ => Err("Format d'adresse de saut invalide".to_string()),
                }
            },

            // Saut conditionnel
            Opcode::JmpIf | Opcode::JmpIfNot => {
                match instruction.get_arg2_value() {
                    Ok(ArgValue::AbsoluteAddr(addr)) => Ok(Some(addr as u32)),
                    Ok(ArgValue::RelativeAddr(offset)) => Ok(Some((pc as i64 + offset as i64) as u32)),
                    _ => Err("Format d'adresse de saut conditionnel invalide".to_string()),
                }
            },

            // Appel de fonction
            Opcode::Call => {
                match instruction.get_arg2_value() {
                    Ok(ArgValue::AbsoluteAddr(addr)) => Ok(Some(addr as u32)),
                    Ok(ArgValue::RelativeAddr(offset)) => Ok(Some((pc as i64 + offset as i64) as u32)),
                    _ => Err("Format d'adresse d'appel invalide".to_string()),
                }
            },

            // Retour de fonction (pas d'adresse explicite)
            Opcode::Ret => Ok(None),

            // Autres instructions de branchement (si ajoutées à l'avenir)
            _ => Ok(None),
        }
    }

    /// Calcule l'adresse mémoire (si instruction mémoire)
    fn calculate_memory_address(&self, instruction: &Instruction, registers: &[u64]) -> Result<Option<u32>, String> {
        // Vérifier si c'est une instruction mémoire
        match instruction.opcode {
            Opcode::Load | Opcode::LoadB | Opcode::LoadW | Opcode::LoadD |
            Opcode::Store | Opcode::StoreB | Opcode::StoreW | Opcode::StoreD => {
                // Différents types d'adressage
                match instruction.get_arg2_value() {
                    Ok(ArgValue::AbsoluteAddr(addr)) => {
                        // Adresse absolue
                        Ok(Some(addr as u32))
                    },

                    Ok(ArgValue::RegisterOffset(reg, offset)) => {
                        // Adressage indirect avec offset (registre + offset)
                        if reg as usize >= registers.len() {
                            return Err(format!("Registre R{} hors limites", reg));
                        }

                        let base_addr = registers[reg as usize];
                        let final_addr = (base_addr as i64 + offset as i64) as u32;
                        Ok(Some(final_addr))
                    },

                    Ok(ArgValue::Register(reg)) => {
                        // Adressage indirect (contenu du registre est l'adresse)
                        if reg as usize >= registers.len() {
                            return Err(format!("Registre R{} hors limites", reg));
                        }

                        Ok(Some(registers[reg as usize] as u32))
                    },

                    _ => Err("Format d'adresse mémoire invalide".to_string()),
                }
            },

            // Pas une instruction mémoire
            _ => Ok(None),
        }
    }

    /// Réinitialise l'étage Decode
    pub fn reset(&mut self) {
        // Pas d'état interne à réinitialiser pour cet étage
    }
}

impl PipelineStage for DecodeStage {
    type Input = (FetchDecodeRegister, &'static [u64]);
    type Output = DecodeExecuteRegister;

    fn process(&mut self, input: &Self::Input) -> Result<Self::Output, String> {
        let (fd_reg, registers) = input;
        self.process(fd_reg, registers)
    }

    fn reset(&mut self) {
        self.reset();
    }
}
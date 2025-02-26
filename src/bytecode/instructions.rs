//src/bytecode/instructions.rs


use crate::bytecode::decode_errors::DecodeError;
use super::format::{ArgType, InstructionFormat};
use super::opcodes::Opcode;

/// Represente le type de taille d'instruction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeType{
    Compact,    // Taille sur 1 byte
    Extended,   // Taille sur 2 bytes
}

/// Structure reprensentan une instruction complete de PunkVM
#[derive(Debug, Clone, PartialEq,Eq)]
pub struct Instruction{
    pub opcode: Opcode,
    pub format: InstructionFormat,
    pub size_type: SizeType,
    pub args: Vec<u8>,  // Donnees brutes des arguments
}

/// Valeur extraite d'un argument d'instruction
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArgValue {
    None,
    Register(u8),
    Immediate(u64),
    RelativeAddr(i32),
    AbsoluteAddr(u64),
    RegisterOffset(u8, i8),
}

impl Instruction{
    /// Cree une nouvelle instruction
    pub fn new(opcode: Opcode, format: InstructionFormat, args: Vec<u8>) -> Self {
        // Calcule automatiquement le type de taille en fonction de la longueur totale
        let total_size = 1 + 1 + args.len(); // opcode + format + args
        let size_type = if total_size <= 255 {
            SizeType::Compact
        } else {
            SizeType::Extended
        };

        Self {
            opcode,
            format,
            size_type,
            args,
        }
    }

    /// Calcule la taille totale de l'instruction en bytes
    pub fn total_size(&self) -> usize{
        let header_size = 2 ; // Opcode 1B + format 1B
        let size_field_size = match self.size_type{
            SizeType::Compact => 1,
            SizeType::Extended => 2,
        };
        header_size + size_field_size + self.args.len()
    }

    /// Enccode l'instruction en bytes
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.total_size());

        // Opcode et format
        bytes.push(self.opcode as u8);
        bytes.push(self.format.encode());

        // Champ de taille
        let total_size = self.total_size() as u16;
        match self.size_type {
            SizeType::Compact => {
                bytes.push(total_size as u8);
            }
            SizeType::Extended => {
                bytes.push((total_size & 0xFF) as u8);
                bytes.push((total_size >> 8) as u8);
            }
        }

        // Arguments
        bytes.extend_from_slice(&self.args);

        bytes
    }

    /// Décode une séquence de bytes en instruction
    pub fn decode(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
        if bytes.len() < 3 {
            return Err(DecodeError::InsufficientData);
        }

        let opcode = Opcode::from_u8(bytes[0])
            .ok_or(DecodeError::InvalidOpcode(bytes[0]))?;

        let format = InstructionFormat::decode(bytes[1])
            .ok_or(DecodeError::InvalidFormat(bytes[1]))?;

        // Détermination du champ de taille
        let (size, size_type, size_field_size) = if bytes[2] == 0xFF {
            if bytes.len() < 4 {
                return Err(DecodeError::InsufficientData);
            }
            let size = u16::from_le_bytes([bytes[2], bytes[3]]);
            (size, SizeType::Extended, 2)
        } else {
            (bytes[2] as u16, SizeType::Compact, 1)
        };

        let total_header_size = 2 + size_field_size; // opcode + format + size

        if bytes.len() < size as usize {
            return Err(DecodeError::InsufficientData);
        }

        let args_size = size as usize - total_header_size;
        let args = bytes[total_header_size..total_header_size + args_size].to_vec();

        Ok((
            Self {
                opcode,
                format,
                size_type,
                args,
            },
            size as usize,
        ))
    }

    /// Extrait la valeur du premier argument en fonction de son type
    pub fn get_arg1_value(&self) -> Result<ArgValue, DecodeError> {
        self.get_arg_value(0, self.format.arg1_type)
    }

    /// Extrait la valeur du second argument en fonction de son type
    pub fn get_arg2_value(&self) -> Result<ArgValue, DecodeError> {
        let offset = self.format.arg1_type.size();
        self.get_arg_value(offset, self.format.arg2_type)
    }

    /// Extrait la valeur d'un argument à partir de son offset et de son type
    fn get_arg_value(&self, offset: usize, arg_type: ArgType) -> Result<ArgValue, DecodeError> {
        if arg_type == ArgType::None {
            return Ok(ArgValue::None);
        }

        if offset >= self.args.len() {
            return Err(DecodeError::InvalidArgumentOffset);
        }

        match arg_type {
            ArgType::None => Ok(ArgValue::None),

            ArgType::Register => {
                if offset < self.args.len() {
                    Ok(ArgValue::Register(self.args[offset] & 0x0F))
                } else {
                    Err(DecodeError::InvalidArgumentOffset)
                }
            },

            ArgType::RegisterExt => {
                if offset < self.args.len() {
                    Ok(ArgValue::Register(self.args[offset]))
                } else {
                    Err(DecodeError::InvalidArgumentOffset)
                }
            },

            ArgType::Immediate8 => {
                if offset < self.args.len() {
                    Ok(ArgValue::Immediate(self.args[offset] as u64))
                } else {
                    Err(DecodeError::InvalidArgumentOffset)
                }
            },

            ArgType::Immediate16 => {
                if offset + 1 < self.args.len() {
                    let value = u16::from_le_bytes([
                        self.args[offset],
                        self.args[offset + 1],
                    ]);
                    Ok(ArgValue::Immediate(value as u64))
                } else {
                    Err(DecodeError::InvalidArgumentOffset)
                }
            },

            ArgType::Immediate32 => {
                if offset + 3 < self.args.len() {
                    let value = u32::from_le_bytes([
                        self.args[offset],
                        self.args[offset + 1],
                        self.args[offset + 2],
                        self.args[offset + 3],
                    ]);
                    Ok(ArgValue::Immediate(value as u64))
                } else {
                    Err(DecodeError::InvalidArgumentOffset)
                }
            },

            ArgType::Immediate64 => {
                if offset + 7 < self.args.len() {
                    let value = u64::from_le_bytes([
                        self.args[offset],
                        self.args[offset + 1],
                        self.args[offset + 2],
                        self.args[offset + 3],
                        self.args[offset + 4],
                        self.args[offset + 5],
                        self.args[offset + 6],
                        self.args[offset + 7],
                    ]);
                    Ok(ArgValue::Immediate(value))
                } else {
                    Err(DecodeError::InvalidArgumentOffset)
                }
            },

            ArgType::RelativeAddr => {
                if offset + 3 < self.args.len() {
                    let value = i32::from_le_bytes([
                        self.args[offset],
                        self.args[offset + 1],
                        self.args[offset + 2],
                        self.args[offset + 3],
                    ]);
                    Ok(ArgValue::RelativeAddr(value))
                } else {
                    Err(DecodeError::InvalidArgumentOffset)
                }
            },

            ArgType::AbsoluteAddr => {
                if offset + 3 < self.args.len() {
                    let value = u32::from_le_bytes([
                        self.args[offset],
                        self.args[offset + 1],
                        self.args[offset + 2],
                        self.args[offset + 3],
                    ]);
                    Ok(ArgValue::AbsoluteAddr(value as u64))
                } else {
                    Err(DecodeError::InvalidArgumentOffset)
                }
            },

            ArgType::RegisterOffset => {
                if offset + 1 < self.args.len() {
                    let reg = self.args[offset];
                    let offset_val = self.args[offset + 1] as i8;
                    Ok(ArgValue::RegisterOffset(reg, offset_val))
                } else {
                    Err(DecodeError::InvalidArgumentOffset)
                }
            },
        }
    }

    /// Crée une instruction simple sans arguments
    pub fn create_no_args(opcode: Opcode) -> Self {
        Self::new(opcode, InstructionFormat::no_args(), vec![])
    }

    /// Crée une instruction avec un seul registre en argument
    pub fn create_single_reg(opcode: Opcode, reg: u8) -> Self {
        Self::new(opcode, InstructionFormat::single_reg(), vec![reg & 0x0F])
    }

    /// Crée une instruction avec deux registres en arguments
    pub fn create_reg_reg(opcode: Opcode, reg1: u8, reg2: u8) -> Self {
        Self::new(
            opcode,
            InstructionFormat::reg_reg(),
            vec![(reg1 & 0x0F) | ((reg2 & 0x0F) << 4)]
        )
    }

    /// Crée une instruction avec un registre et une valeur immédiate 8 bits
    pub fn create_reg_imm8(opcode: Opcode, reg: u8, imm: u8) -> Self {
        Self::new(
            opcode,
            InstructionFormat::reg_imm8(),
            vec![reg & 0x0F, imm]
        )
    }

    /// Cree une instruction avec un registre et une valeur immédiate 16 bits
    pub fn create_reg_imm16(opcode: Opcode, reg: u8, imm: u16) -> Self {
        Self::new(
            opcode,
            InstructionFormat::reg_imm16(),
            vec![
                reg & 0x0F,
                (imm & 0xFF) as u8,
                ((imm >> 8) & 0xFF) as u8,
            ]
        )
    }

    /// Crée une instruction de chargement mémoire avec registre + offset
    pub fn create_load_reg_offset(reg_dest: u8, reg_base: u8, offset: i8) -> Self {
        Self::new(
            Opcode::Load,
            InstructionFormat::reg_regoff(),
            vec![
                reg_dest & 0x0F,
                reg_base & 0x0F,
                offset as u8,
            ]
        )
    }



}


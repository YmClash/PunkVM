// src/bytecode/instructions.rs

use crate::bytecode::decode_errors::DecodeError;
use crate::bytecode::format::{ArgType, InstructionFormat};
use crate::bytecode::opcodes::Opcode;

/// Represente le type de taille d'instruction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeType {
    Compact,  // Taille sur 1 byte
    Extended, // Taille sur 3 bytes      0xFF + 2 bytes
}

/// Structure reprensentan une instruction complete de PunkVM
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instruction {
    pub opcode: Opcode,
    pub format: InstructionFormat,
    pub size_type: SizeType,
    pub args: Vec<u8>, // Donnees brutes des arguments

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
    Label(String), // Pour les labels, si besoin
}

impl Instruction {
    /// Cree une nouvelle instruction
    pub fn new(opcode: Opcode, format: InstructionFormat, args: Vec<u8>) -> Self {
        // Calcule la taille potentielle (sans le champ "taille" lui-même)
        //  - 1 octet opcode
        //  - 2 octets de format
        //  - 0 => on ne compte pas encore le champ de taille
        //  - + args.len()
        let overhead = 1 + 2;
        let potential_size = overhead + args.len();

        // Décidons si on est en compact ou extended
        // => si potential_size + 1 (pour champ compact) <= 255 => compact
        // sinon => extended
        let needed_if_compact = potential_size + 1;
        let size_type = if needed_if_compact <= 255 {
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
    pub fn total_size(&self) -> usize {
        let overhead = 1 + 2; // opcode + format
        let size_field_size = match self.size_type {
            SizeType::Compact => 1,
            SizeType::Extended => 3, // 3 octets: 0xFF (marqueur) + 2 octets de taille
        };

        overhead + size_field_size + self.args.len()
    }

    /// Encode l'instruction dans un Vec<u8>
    /// Structure:
    /// 1) opcode (1 octet)
    /// 2) format (2 octets)
    /// 3) taille => 1 octet (compact) ou 3 octets (extended)
    /// 4) args (n octets)

    /// Encode l'instruction en bytes
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.total_size());

        // 1) opcode
        bytes.push(self.opcode as u8);

        // 2) format (2 octets)
        let fmt_bytes = self.format.encode();
        bytes.push(fmt_bytes[0]);
        bytes.push(fmt_bytes[1]);

        // 3) champ de taille
        let tsize = self.total_size() as u16;
        match self.size_type {
            SizeType::Compact => {
                // 1 octet
                bytes.push(tsize as u8);
            }
            SizeType::Extended => {
                // 0xFF + 2 octets
                bytes.push(0xFF);
                let lo = (tsize & 0xFF) as u8;
                let hi = (tsize >> 8) as u8;
                bytes.push(lo);
                bytes.push(hi);
            }
        };
        // 4) args
        bytes.extend_from_slice(&self.args);
        bytes
    }

    /// Décode une séquence de bytes en instruction
    /// /// Décode une instruction depuis un slice
    pub fn decode(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
        if bytes.len() < 3 {
            return Err(DecodeError::InsufficientData);
        }

        let opcode = Opcode::from_u8(bytes[0]).ok_or(DecodeError::InvalidOpcode(bytes[0]))?;

        // Lire le format (2 octets)
        let fmt_lo = bytes[1];
        let fmt_hi = bytes[2];
        let format = InstructionFormat::decode([fmt_lo, fmt_hi])
            .ok_or(DecodeError::InvalidFormat(fmt_lo))?;

        if bytes.len() < 4 {
            return Err(DecodeError::InsufficientData);
        }

        let first_size_byte = bytes[3];

        // Détermination du champ de taille
        let (size, size_type, size_field_size) = if first_size_byte == 0xFF {
            // Format étendu, la taille est stockée sur 3 octets après le marqueur
            if bytes.len() < 6 {
                // Minimum 5 octets: opcode, format, marker, size_lo, size_hi
                return Err(DecodeError::InsufficientData);
            }
            let lo = bytes[4];
            let hi = bytes[5];
            let sz = u16::from_le_bytes([lo, hi]);
            (sz, SizeType::Extended, 3)
        } else {
            // compact => 1 octet
            (first_size_byte as u16, SizeType::Compact, 1)
        };

        let total_header_size = 1 + 2 + size_field_size; // opcode(1), format(2), champ taille (1 ou 3)
        if size as usize > bytes.len() {
            return Err(DecodeError::InsufficientData);
        }

        let args_size = size as usize - total_header_size;
        let args_start = total_header_size;
        let args_end = args_start + args_size;
        if args_end > bytes.len() {
            return Err(DecodeError::InsufficientData);
        }

        let args = bytes[args_start..args_end].to_vec();
        let inst = Instruction {
            opcode,
            format,
            size_type,
            args,
        };
        Ok((inst, size as usize))
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

    /// Extrait la valeur du troisième argument en fonction de son type
    pub fn get_arg3_value(&self) -> Result<ArgValue, DecodeError> {
        let offset = self.format.arg1_type.size() + self.format.arg2_type.size();
        self.get_arg_value(offset, self.format.arg3_type)
    }

    /// Extrait la valeur d'un argument à partir de son offset et de son type
    fn get_arg_value(&self, offset: usize, arg_type: ArgType) -> Result<ArgValue, DecodeError> {
        if arg_type == ArgType::None {
            return Ok(ArgValue::None);
        }

        if offset >= self.args.len() {
            return Err(DecodeError::InvalidArgumentOffset);
        }

        // if offset >= self.args.len() && arg_type != ArgType::Register{
        //     return Err(DecodeError::InvalidArgumentOffset);
        // }

        match arg_type {
            ArgType::None => Ok(ArgValue::None),

            ArgType::Register => {
                let reg = self.args[offset];
                Ok(ArgValue::Register(reg & 0x0F))
            }

            ArgType::RegisterExt => {
                if offset < self.args.len() {
                    Ok(ArgValue::Register(self.args[offset]))
                } else {
                    Err(DecodeError::InvalidArgumentOffset)
                }
            }

            ArgType::Immediate8 => {
                if offset < self.args.len() {
                    Ok(ArgValue::Immediate(self.args[offset] as u64))
                } else {
                    Err(DecodeError::InvalidArgumentOffset)
                }
            }

            ArgType::Immediate16 => {
                if offset + 1 < self.args.len() {
                    let value = u16::from_le_bytes([self.args[offset], self.args[offset + 1]]);
                    Ok(ArgValue::Immediate(value as u64))
                } else {
                    Err(DecodeError::InvalidArgumentOffset)
                }
            }

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
            }

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
            }

            ArgType::RelativeAddr => {
                if offset + 3 < self.args.len() {
                    let value = i32::from_le_bytes([
                        self.args[offset],
                        self.args[offset + 1],
                        self.args[offset + 2],
                        self.args[offset + 3],
                    ]);
                    println!("DEBUG: RelativeAddr => value={}", value);
                    Ok(ArgValue::RelativeAddr(value))
                } else {
                    Err(DecodeError::InvalidArgumentOffset)
                }
            }

            ArgType::AbsoluteAddr => {
                if offset + 3 < self.args.len() {
                    let value = u32::from_le_bytes([
                        self.args[offset],
                        self.args[offset + 1],
                        self.args[offset + 2],
                        self.args[offset + 3],
                    ]);
                    println!("DEBUG: AbsoluteAddr => value={}", value);
                    Ok(ArgValue::AbsoluteAddr(value as u64))
                } else {
                    Err(DecodeError::InvalidArgumentOffset)
                }
            }

            ArgType::RegisterOffset => {
                if offset + 1 < self.args.len() {
                    let reg = self.args[offset];
                    let offset_val = self.args[offset + 1] as i8;
                    println!(
                        "DEBUG: RegisterOffset => reg={}, offset={}",
                        reg, offset_val
                    );
                    Ok(ArgValue::RegisterOffset(reg, offset_val))
                } else {
                    Err(DecodeError::InvalidArgumentOffset)
                }
            }
        }
    }

    /// ici on a les fonctions d'aide pour créer des instructions

    /// Crée une instruction simple sans arguments
    pub fn create_no_args(opcode: Opcode) -> Self {
        // Self::new(opcode, InstructionFormat::no_args(), vec![])
        Self::new(opcode, InstructionFormat::no_args(), Vec::new())
    }

    /// Crée une instruction avec un seul registre en argument
    pub fn create_single_reg(opcode: Opcode, reg: u8) -> Self {
        // Self::new(opcode, InstructionFormat::single_reg(), vec![reg & 0x0F])
        let fmt = InstructionFormat::single_reg();
        let args = vec![reg & 0x0F];
        Self::new(opcode, fmt, args)
    }

    /// Crée une instruction avec deux registres en arguments
    pub fn create_reg_reg(opcode: Opcode, rd: u8, rs1: u8) -> Self {
        // Empaqueter les deux registres dans un seul octet
        // reg1 dans les 4 bits de poids faible, reg2 dans les 4 bits de poids fort
        // ADD R2, R1
        let fmt = InstructionFormat::double_reg();
        let args = vec![rd & 0x0F, rs1 & 0x0F];
        Self::new(opcode, fmt, args)
    }

    /// Crée une instruction avec trois registres en arguments
    pub fn create_reg_reg_reg(opcode: Opcode, rd: u8, rs1: u8, rs2: u8) -> Self {
        // Stocker les trois registres dans args
        // rd (destination), rs1 (source 1), rs2 (source 2)
        // ADD R2, R0, R1
        let fmt = InstructionFormat::reg_reg_reg();
        // [rd, rs1, rs2]
        let args = vec![rd & 0x0F, rs1 & 0x0F, rs2 & 0x0F];
        Self::new(opcode, fmt, args)
    }

    /// Crée une instruction avec un registre et une valeur immédiate 8 bits
    pub fn create_reg_imm8(opcode: Opcode, reg: u8, imm: u8) -> Self {
        let fmt = InstructionFormat::reg_imm8(); // (Register, Immediate8, None)
        let args = vec![reg & 0x0F, imm];
        Self::new(opcode, fmt, args)
    }

    /// Cree une instruction avec un registre et une valeur immédiate 16 bits
    pub fn create_reg_imm16(opcode: Opcode, reg: u8, imm: u16) -> Self {
        let fmt = InstructionFormat::reg_reg_imm16(); // ou un format custom => (Register, None, Immediate16)?
        let mut args = vec![reg & 0x0F];
        args.push((imm & 0x00FF) as u8);
        args.push(((imm >> 8) & 0xFF) as u8);
        Self::new(opcode, fmt, args)
    }

    /// Crée une instruction de chargement mémoire avec registre + offset
    pub fn create_load_reg_offset(reg_dest: u8, reg_base: u8, offset: i8) -> Self {
        let fmt = InstructionFormat::reg_regoff(); // (Register, RegisterOffset, None)?
        let args = vec![reg_dest & 0x0F, reg_base & 0x0F, offset as u8];
        Self::new(Opcode::Load, fmt, args)
    }

    /// Crée une instruction de stockage mémoire avec registre + offset
    pub fn create_reg_reg_offset(opcode: Opcode, reg_src: u8, reg_base: u8, offset: i8) -> Self {
        let fmt = InstructionFormat::reg_reg_imm8(); // (Register, RegisterOffset, None)?
        let  args = vec![reg_src & 0x0F, reg_base & 0x0F, offset as u8];

        Self::new(opcode, fmt, args)
    }


    pub fn create_jump(from_addr: u32, to_addr: u32) -> Self {
        // Calculer la taille de l'instruction de saut
        let temp_instr = Self::new(
            Opcode::Jmp,
            InstructionFormat::jump(),
            vec![0, 0, 0, 0], // Placeholder pour 4 bytes d'offset
        );
        let instr_size = temp_instr.total_size() as u32;

        // Calculer l'offset relatif : target - (current + instruction_size)
        let next_pc = from_addr + instr_size;
        // let offset = (to_addr as i64 - next_pc as i64) as i32;
        let offset = calculate_branch_offset(from_addr, to_addr, instr_size);

        println!("DEBUG: create_jump_to_address - from=0x{:X}, to=0x{:X}, instr_size={}, next_pc=0x{:X}, offset={}",
                 from_addr, to_addr, instr_size, next_pc, offset);

        Self::new(
            Opcode::Jmp,
            InstructionFormat::jump(),
            offset.to_le_bytes().to_vec(),
        )
    }



    pub fn create_jump_if(from_addr: u32, to_addr: u32) -> Self{
        let temp_instr = Self::new(
            Opcode::JmpIf,
            InstructionFormat::jumpif(),
            vec![0, 0, 0, 0],
        );
        let instr_size = temp_instr.total_size() as u32;

        // Calculer l'offset relatif : target - (current + instruction_size)
        let next_pc = from_addr + instr_size;
        // let offset = (to_addr as i64 - next_pc as i64) as i32;
        let offset = calculate_branch_offset(from_addr, to_addr, instr_size);
        println!("DEBUG: create_jump_to_address - from=0x{:X}, to=0x{:X}, instr_size={}, next_pc=0x{:X}, offset={}",
                 from_addr, to_addr, instr_size, next_pc, offset);

        Self::new(
            Opcode::JmpIf,
            InstructionFormat::jumpif(),
            offset.to_le_bytes().to_vec(),
        )

    }


    pub fn create_jump_if_not(from_addr: u32, to_addr: u32) -> Self {
        let temp_instr = Self::new(
            Opcode::JmpIfNot,
            InstructionFormat::jump_if_not(),
            vec![0, 0, 0, 0],
        );
        let instr_size = temp_instr.total_size() as u32;

        // Calculer l'offset relatif : target - (current + instruction_size)
        let next_pc = from_addr + instr_size;
        // let offset = (to_addr as i64 - next_pc as i64) as i32;
        let offset = calculate_branch_offset(from_addr, to_addr, instr_size);
        println!("DEBUG: create_jump_if_not - from=0x{:X}, to=0x{:X}, instr_size={}, next_pc=0x{:X}, offset={}",
                 from_addr, to_addr, instr_size, next_pc, offset);

        println!("DEBUG: create_jump_if_not - offset length={} bytes", offset.to_le_bytes().len());

        Self::new(
            Opcode::JmpIfNot,
            InstructionFormat::jump_if_not(),
            offset.to_le_bytes().to_vec(),
        )
    }

    pub fn create_jump_if_equal(from_addr: u32, to_addr: u32) -> Self {
        let temp_instr = Self::new(
            Opcode::JmpIfEqual,
            InstructionFormat::jump_if_equal(),
            vec![0, 0, 0, 0],
        );
        let instr_size = temp_instr.total_size() as u32;

        // Calculer l'offset relatif : target - (current + instruction_size)
        let next_pc = from_addr + instr_size;
        // let offset = (to_addr as i64 - next_pc as i64) as i32;
        let offset = calculate_branch_offset(from_addr, to_addr, instr_size);
        println!("DEBUG: create_jump_if_equal - from=0x{:X}, to=0x{:X}, instr_size={}, next_pc=0x{:X}, offset={}",
                 from_addr, to_addr, instr_size, next_pc, offset);

        println!("DEBUG: create_jump_if_equal - offset length={} bytes", offset.to_le_bytes().len());

        Self::new(
            Opcode::JmpIfEqual,
            InstructionFormat::jump_if_equal(),
            offset.to_le_bytes().to_vec(),
        )
    }

    pub fn create_jump_if_not_equal(from_addr: u32, to_addr: u32) -> Self {
        let temp_instr = Self::new(
            Opcode::JmpIfNotEqual,
            InstructionFormat::jump_if_notequal(),
            vec![0, 0, 0, 0],
        );
        let instr_size = temp_instr.total_size() as u32;

        // Calculer l'offset relatif : target - (current + instruction_size)
        let next_pc = from_addr + instr_size;
        // let offset = (to_addr as i64 - next_pc as i64) as i32;
        let offset = calculate_branch_offset(from_addr, to_addr, instr_size);
        println!("DEBUG: create_jump_if_not_equal - from=0x{:X}, to=0x{:X}, instr_size={}, next_pc=0x{:X}, offset={}",
                 from_addr, to_addr, instr_size, next_pc, offset);

        println!("DEBUG: create_jump_if_not_equal - offset length={} bytes", offset.to_le_bytes().len());

        Self::new(
            Opcode::JmpIfNotEqual,
            InstructionFormat::jump_if_notequal(),
            offset.to_le_bytes().to_vec(),
        )
    }


    pub fn create_jump_if_greater(from_addr: u32, to_addr: u32) -> Self {
        let temp_instr = Self::new(
            Opcode::JmpIfGreater,
            InstructionFormat::jump_if_greater(),
            vec![0, 0, 0, 0],
        );
        let instr_size = temp_instr.total_size() as u32;

        // Calculer l'offset relatif : target - (current + instruction_size)
        let next_pc = from_addr + instr_size;
        // let offset = (to_addr as i64 - next_pc as i64) as i32;
        let offset = calculate_branch_offset(from_addr, to_addr, instr_size);

        println!("DEBUG: create_jump_if_greater - from=0x{:X}, to=0x{:X}, instr_size={}, next_pc=0x{:X}, offset={}",
                 from_addr, to_addr, instr_size, next_pc, offset);

        println!("DEBUG: create_jump_if_greater - offset length={} bytes", offset.to_le_bytes().len());

        Self::new(
            Opcode::JmpIfGreater,
            InstructionFormat::jump_if_greater(),
            offset.to_le_bytes().to_vec(),
        )
    }


    pub fn create_jump_if_greater_equal(from_addr: u32, to_addr: u32) -> Self {
        let temp_instr = Self::new(
            Opcode::JmpIfGreaterEqual,
            InstructionFormat::jump(),
            vec![0, 0, 0, 0],
        );
        let instr_size = temp_instr.total_size() as u32;

        // Calculer l'offset relatif : target - (current + instruction_size)
        let next_pc = from_addr + instr_size;
        // let offset = (to_addr as i64 - next_pc as i64) as i32;
        let offset = calculate_branch_offset(from_addr, to_addr, instr_size);

        println!("DEBUG: create_jump_if_greater_equal - from=0x{:X}, to=0x{:X}, instr_size={}, next_pc=0x{:X}, offset={}",
                 from_addr, to_addr, instr_size, next_pc, offset);

        println!("DEBUG: create_jump_if_greater_equal - offset length={} bytes", offset.to_le_bytes().len());

        Self::new(
            Opcode::JmpIfGreaterEqual,
            InstructionFormat::jump(),
            offset.to_le_bytes().to_vec(),
        )
    }

    pub fn create_jump_if_less(from_addr: u32, to_addr: u32) -> Self {
        let temp_instr = Self::new(
            Opcode::JmpIfLess,
            InstructionFormat::jump_if_less(),
            vec![0, 0, 0, 0],
        );
        let instr_size = temp_instr.total_size() as u32;

        // Calculer l'offset relatif : target - (current + instruction_size)
        let next_pc = from_addr + instr_size;
        // let offset = (to_addr as i64 - next_pc as i64) as i32;
        let offset = calculate_branch_offset(from_addr, to_addr, instr_size);

        println!("DEBUG: create_jump_if_less - from=0x{:X}, to=0x{:X}, instr_size={}, next_pc=0x{:X}, offset={}",
                 from_addr, to_addr, instr_size, next_pc, offset);

        println!("DEBUG: create_jump_if_less - offset length={} bytes", offset.to_le_bytes().len());

        Self::new(
            Opcode::JmpIfLess,
            InstructionFormat::jump_if_less(),
            offset.to_le_bytes().to_vec(),
        )
    }


    pub fn create_jump_if_less_equal(from_addr: u32, to_addr: u32) -> Self {
        let temp_instr = Self::new(
            Opcode::JmpIfLessEqual,
            InstructionFormat::jump_if_lessequal(),
            vec![0, 0, 0, 0],
        );
        let instr_size = temp_instr.total_size() as u32;

        // Calculer l'offset relatif : target - (current + instruction_size)
        let next_pc = from_addr + instr_size;
        // let offset = (to_addr as i64 - next_pc as i64) as i32;
        let offset = calculate_branch_offset(from_addr, to_addr, instr_size);
        println!("DEBUG: create_jump_if_less_equal - from=0x{:X}, to=0x{:X}, instr_size={}, next_pc=0x{:X}, offset={}",
                 from_addr, to_addr, instr_size, next_pc, offset);

        println!("DEBUG: create_jump_if_less_equal - offset length={} bytes", offset.to_le_bytes().len());

        Self::new(
            Opcode::JmpIfLessEqual,
            InstructionFormat::jump_if_lessequal(),
            offset.to_le_bytes().to_vec(),
        )
    }

    pub fn create_jump_if_above(from_addr: u32, to_addr: u32) -> Self {
        let temp_instr = Self::new(
            Opcode::JmpIfAbove,
            InstructionFormat::jump_if_above(),
            vec![0, 0, 0, 0],
        );
        let instr_size = temp_instr.total_size() as u32;

        // Calculer l'offset relatif : target - (current + instruction_size)
        let next_pc = from_addr + instr_size;
        // let offset = (to_addr as i64 - next_pc as i64) as i32;
        let offset = calculate_branch_offset(from_addr, to_addr, instr_size);
        println!("DEBUG: create_jump_if_less_equal - from=0x{:X}, to=0x{:X}, instr_size={}, next_pc=0x{:X}, offset={}",
                 from_addr, to_addr, instr_size, next_pc, offset);

        println!("DEBUG: create_jump_if_above - offset length={} bytes", offset.to_le_bytes().len());

        Self::new(
            Opcode::JmpIfAbove,
            InstructionFormat::jump_if_lessequal(),
            offset.to_le_bytes().to_vec(),
        )
    }

    pub fn create_jump_if_above_equal(from_addr: u32, to_addr: u32) -> Self {
        let temp_instr = Self::new(
            Opcode::JmpIfAboveEqual,
            InstructionFormat::jump_if_aboveequal(),
            vec![0, 0, 0, 0],
        );
        let instr_size = temp_instr.total_size() as u32;

        // Calculer l'offset relatif : target - (current + instruction_size)
        let next_pc = from_addr + instr_size;
        // let offset = (to_addr as i64 - next_pc as i64) as i32;
        let offset = calculate_branch_offset(from_addr, to_addr, instr_size);
        println!("DEBUG: create_jump_if_above_equal - from=0x{:X}, to=0x{:X}, instr_size={}, next_pc=0x{:X}, offset={}",
                 from_addr, to_addr, instr_size, next_pc, offset);

        println!("DEBUG: create_jump_if_above_equal - offset length={} bytes", offset.to_le_bytes().len());

        Self::new(
            Opcode::JmpIfAboveEqual,
            InstructionFormat::jump_if_aboveequal(),
            offset.to_le_bytes().to_vec(),
        )
    }

    pub fn create_jump_below(from_addr: u32, to_addr: u32) -> Self {
        let temp_instr = Self::new(
            Opcode::JmpIfBelow,
            InstructionFormat::jump_if_below(),
            vec![0, 0, 0, 0],
        );
        let instr_size = temp_instr.total_size() as u32;

        // Calculer l'offset relatif : target - (current + instruction_size)
        let next_pc = from_addr + instr_size;
        // let offset = (to_addr as i64 - next_pc as i64) as i32;
        let offset = calculate_branch_offset(from_addr, to_addr, instr_size);

        println!("DEBUG: create_jump_below - from=0x{:X}, to=0x{:X}, instr_size={}, next_pc=0x{:X}, offset={}",
                 from_addr, to_addr, instr_size, next_pc, offset);

        println!("DEBUG: create_jump_below - offset length={} bytes", offset.to_le_bytes().len());

        Self::new(
            Opcode::JmpIfBelow,
            InstructionFormat::jump_if_below(),
            offset.to_le_bytes().to_vec(),
        )
    }

    pub fn create_jump_if_below_equal(from_addr: u32, to_addr: u32) -> Self {
        let temp_instr = Self::new(
            Opcode::JmpIfBelowEqual,
            InstructionFormat::jump_if_belowequal(),
            vec![0, 0, 0, 0],
        );
        let instr_size = temp_instr.total_size() as u32;

        // Calculer l'offset relatif : target - (current + instruction_size)
        let next_pc = from_addr + instr_size;
        // let offset = (to_addr as i64 - next_pc as i64) as i32;

        let offset = calculate_branch_offset(from_addr, to_addr, instr_size);

        println!("DEBUG: create_jump_if_below_equal - from=0x{:X}, to=0x{:X}, instr_size={}, next_pc=0x{:X}, offset={}",
                 from_addr, to_addr, instr_size, next_pc, offset);

        println!("DEBUG: create_jump_if_below_equal - offset length={} bytes", offset.to_le_bytes().len());

        Self::new(
            Opcode::JmpIfBelowEqual,
            InstructionFormat::jump_if_belowequal(),
            offset.to_le_bytes().to_vec(),
        )
    }

    pub fn create_jump_if_not_zero(from_addr: u32, to_addr: u32) -> Self {
        let temp_instr = Self::new(
            Opcode::JmpIfNotZero,
            InstructionFormat::jump_if_not_zero(),
            vec![0, 0, 0, 0],
        );
        let instr_size = temp_instr.total_size() as u32;

        // Calculer l'offset relatif : target - (current + instruction_size)
        let next_pc = from_addr + instr_size;
        // let offset = (to_addr as i64 - next_pc as i64) as i32;
        let offset = calculate_branch_offset(from_addr, to_addr, instr_size);

        println!("DEBUG: create_jump_if_not_zero - from=0x{:X}, to=0x{:X}, instr_size={}, next_pc=0x{:X}, offset={}",
                 from_addr, to_addr, instr_size, next_pc, offset);

        println!("DEBUG: create_jump_if_not_zero - offset length={} bytes", offset.to_le_bytes().len());

        Self::new(
            Opcode::JmpIfNotZero,
            InstructionFormat::jump_if_not_zero(),
            offset.to_le_bytes().to_vec(),
        )
    }

    pub fn create_jump_if_zero(from_addr: u32, to_addr: u32) -> Self {
        let temp_instr = Self::new(
            Opcode::JmpIfZero,
            InstructionFormat::jump_if_zero(),
            vec![0, 0, 0, 0],
        );
        let instr_size = temp_instr.total_size() as u32;

        // Calculer l'offset relatif : target - (current + instruction_size)
        let next_pc = from_addr + instr_size;
        // let offset = (to_addr as i64 - next_pc as i64) as i32;

        let offset = calculate_branch_offset(from_addr, to_addr, instr_size);
        println!("DEBUG: create_jump_if_zero - from=0x{:X}, to=0x{:X}, instr_size={}, next_pc=0x{:X}, offset={}",
                 from_addr, to_addr, instr_size, next_pc, offset);

        println!("DEBUG: create_jump_if_zero - offset length={} bytes", offset.to_le_bytes().len());

        Self::new(
            Opcode::JmpIfZero,
            InstructionFormat::jump_if_zero(),
            offset.to_le_bytes().to_vec(),
        )
    }

    pub fn create_jump_if_overflow(from_addr: u32, to_addr: u32) -> Self {
        let temp_instr = Self::new(
            Opcode::JmpIfOverflow,
            InstructionFormat::jump_if_overflow(),
            vec![0, 0, 0, 0],
        );
        let instr_size = temp_instr.total_size() as u32;

        // Calculer l'offset relatif : target - (current + instruction_size)
        let next_pc = from_addr + instr_size;
        // let offset = (to_addr as i64 - next_pc as i64) as i32;
        let offset = calculate_branch_offset(from_addr, to_addr, instr_size);

        println!("DEBUG: create_jump_if_overflow - from=0x{:X}, to=0x{:X}, instr_size={}, next_pc=0x{:X}, offset={}",
                 from_addr, to_addr, instr_size, next_pc, offset);

        println!("DEBUG: create_jump_if_overflow - offset length={} bytes", offset.to_le_bytes().len());

        Self::new(
            Opcode::JmpIfOverflow,
            InstructionFormat::jump_if_overflow(),
            offset.to_le_bytes().to_vec(),
        )
    }

    pub fn create_jump_if_not_overflow(from_addr: u32, to_addr: u32) -> Self {
        let temp_instr = Self::new(
            Opcode::JmpIfNotOverflow,
            InstructionFormat::jump_if_not_overflow(),
            vec![0, 0, 0, 0],
        );
        let instr_size = temp_instr.total_size() as u32;

        // Calculer l'offset relatif : target - (current + instruction_size)
        let next_pc = from_addr + instr_size;
        // let offset = (to_addr as i64 - next_pc as i64) as i32;
        let offset = calculate_branch_offset(from_addr, to_addr, instr_size);

        println!("DEBUG: create_jump_if_not_overflow - from=0x{:X}, to=0x{:X}, instr_size={}, next_pc=0x{:X}, offset={}",
                 from_addr, to_addr, instr_size, next_pc, offset);

        println!("DEBUG: create_jump_if_not_overflow - offset length={} bytes", offset.to_le_bytes().len());

        Self::new(
            Opcode::JmpIfNotOverflow,
            InstructionFormat::jump_if_not_overflow(),
            offset.to_le_bytes().to_vec(),
        )
    }

    pub fn create_jump_if_positive(from_addr: u32, to_addr: u32) -> Self {
        let temp_instr = Self::new(
            Opcode::JmpIfPositive,
            InstructionFormat::jump_if_positive(),
            vec![0, 0, 0, 0],
        );
        let instr_size = temp_instr.total_size() as u32;

        // Calculer l'offset relatif : target - (current + instruction_size)
        let next_pc = from_addr + instr_size;
        // let offset = (to_addr as i64 - next_pc as i64) as i32;
        // let offset = calculate_branch_offset(from_addr, to_addr, instr_size);
        let offset = calculate_branch_offset(from_addr, to_addr, instr_size);

        println!("DEBUG: create_jump_if_positive - from=0x{:X}, to=0x{:X}, instr_size={}, next_pc=0x{:X}, offset={}",
                 from_addr, to_addr, instr_size, next_pc, offset);

        println!("DEBUG: create_jump_if_positive - offset length={} bytes", offset.to_le_bytes().len());

        Self::new(
            Opcode::JmpIfPositive,
            InstructionFormat::jump_if_positive(),
            offset.to_le_bytes().to_vec(),
        )
    }

    pub fn create_jump_if_negative(from_addr: u32, to_addr: u32) -> Self {
        let temp_instr = Self::new(
            Opcode::JmpIfNegative,
            InstructionFormat::jump_if_negative(),
            vec![0, 0, 0, 0],
        );
        let instr_size = temp_instr.total_size() as u32;

        // Calculer l'offset relatif : target - (current + instruction_size)
        let next_pc = from_addr + instr_size;
        // let offset = (to_addr as i64 - next_pc as i64) as i32;
        // let offset = calculate_branch_offset(from_addr, to_addr, instr_size);
        let offset = calculate_branch_offset(from_addr, to_addr, instr_size);
        println!("DEBUG: create_jump_if_negative - from=0x{:X}, to=0x{:X}, instr_size={}, next_pc=0x{:X}, offset={}",
                 from_addr, to_addr, instr_size, next_pc, offset);

        println!("DEBUG: create_jump_if_negative - offset length={} bytes", offset.to_le_bytes().len());

        Self::new(
            Opcode::JmpIfNegative,
            InstructionFormat::jump_if_negative(),
            offset.to_le_bytes().to_vec(),
        )
    }



    // methode pour cree  un saut relative
    // Dans bytecode/instruction.rs
    // Ajouter une méthode utilitaire pour créer facilement des sauts relatifs
    pub fn create_relative_jump(opcode: Opcode, from_addr: u32, to_addr: u32) -> Self {
        let instr = Self::new(
            opcode,
            InstructionFormat::new(ArgType::None, ArgType::RelativeAddr, ArgType::None),
            vec![], // Temporaire
        );

        let instr_size = instr.total_size() as u32;
        // let offset = calculate_branch_offset(from_addr, to_addr, instr_size);
        let offset = calculate_branch_offset(from_addr, to_addr, instr_size);

        println!("DEBUG: create_relative_jump - from=0x{:X}, to=0x{:X}, instr_size={}, offset={}",
                 from_addr, to_addr, instr_size, offset);
        // Convertir l'offset en bytes
        // let offset_bytes = offset.to_le_bytes();
        // Créer l'instruction avec l'offset
        println!("DEBUG: create_relative_jump - offset length={} bytes", offset.to_le_bytes().len());
        Self::new(
            opcode,
            InstructionFormat::new(ArgType::None, ArgType::RelativeAddr, ArgType::None),
            offset.to_le_bytes()[0..4].to_vec(),
        )
    }

    /// Calcule l'offset de branchement correct entre deux adresses
    /// en tenant compte de la taille de l'instruction
    pub fn calculate_branch_offset(from_addr:u32,to_addr:u32,instr_size:u32) -> i32{
        // l'offset est calcule depuis l'adresse de l'instruction suivante
        let next_pc = from_addr + instr_size;
        (to_addr as i32) - (next_pc as i32)
    }


    /// Helper pour calculer l'adresse actuelle dans un programme en construction
    pub fn calculate_current_address(instructions: &[Instruction]) -> u32 {
        instructions.iter().map(|i| i.total_size() as u32).sum()
    }

    /// Version simplifiée : saut relatif en nombre d'instructions
    pub fn create_jump_skip_instructions(instructions_to_skip: usize) -> Self {
        // Estimer la taille : la plupart des instructions font 6-8 bytes
        // Cette estimation sera raffinée si nécessaire
        let estimated_offset = (instructions_to_skip * 6) as i32;

        println!("DEBUG: create_jump_skip_instructions - skipping {} instructions, estimated offset={}",
                 instructions_to_skip, estimated_offset);

        Self::new(
            Opcode::Jmp,
            InstructionFormat::jump(),
            estimated_offset.to_le_bytes().to_vec(),
        )
    }


    /// Helper pour la creation d'instruction CALL
    pub fn create_call(target_addr:u32) -> Self{
        let fmt = InstructionFormat::call();
        Self::new(Opcode::Call, fmt, target_addr.to_le_bytes().to_vec())

    }

    pub fn create_return() -> Self {
        let fmt = InstructionFormat::ret();
        Self::new(Opcode::Ret, fmt, Vec::new())
    }

    pub fn create_push_register(reg: u8) -> Self{
        let fmt = InstructionFormat::push_reg();
        Self::new(Opcode::Push,fmt, vec![reg & 0x0F])

    }

    pub fn create_push_immediate8(imm:u8) -> Self{
        let fmt = InstructionFormat::push_immediate8();
        Self::new(Opcode::Push, fmt, vec![imm])
    }

    pub fn create_pop_register(reg: u8) -> Self {
        let fmt = InstructionFormat::pop_reg();
        Self::new(Opcode::Pop, fmt, vec![reg & 0x0F])
    }

    pub fn create_pop_immediate8() -> Self {
        let fmt = InstructionFormat::pop_immediate8();
        Self::new(Opcode::Pop, fmt, Vec::new())
    }







}





pub fn calculate_branch_offset(from_addr:u32,to_addr:u32,instr_size:u32) -> i32{
    // l'offset est calcule depuis l'adresse de l'instruction suivante
    let next_pc = from_addr + instr_size;
    (to_addr as i32) - (next_pc as i32)
}



////////////////////////////////////////////////////////////////////////////////////////////////////


#[cfg(test)]
mod branch_instruction_tests {

    use crate::bytecode::instructions::{ArgValue, calculate_branch_offset, Instruction};
    use crate::bytecode::opcodes::Opcode;

    /// Test du calcul d'offset de branchement
    #[test]
    fn test_calculate_branch_offset() {
        // Test saut en avant
        let from = 0x10;
        let to = 0x20;
        let instr_size = 7;
        let offset = calculate_branch_offset(from, to, instr_size);
        assert_eq!(offset, 9); // 0x20 - (0x10 + 7) = 9

        // Test saut en arrière
        let from = 0x30;
        let to = 0x10;
        let instr_size = 7;
        let offset = calculate_branch_offset(from, to, instr_size);
        assert_eq!(offset, -39); // 0x10 - (0x30 + 7) = -39

        // Test saut sur place (boucle infinie)
        let from = 0x40;
        let to = 0x40;
        let instr_size = 7;
        let offset = calculate_branch_offset(from, to, instr_size);
        assert_eq!(offset, -7); // 0x40 - (0x40 + 7) = -7
    }

    /// Test JMP (saut inconditionnel)
    #[test]
    fn test_create_jump() {
        let from_addr = 0x1000;
        let to_addr = 0x1020;

        let instr = Instruction::create_jump(from_addr, to_addr);

        assert_eq!(instr.opcode, Opcode::Jmp);

        // Vérifier l'offset encodé
        if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
            let expected_offset = calculate_branch_offset(from_addr, to_addr, instr.total_size() as u32);
            assert_eq!(offset, expected_offset);
            println!("JMP offset: {} (from 0x{:X} to 0x{:X})", offset, from_addr, to_addr);
        } else {
            panic!("Failed to extract offset from JMP instruction");
        }
    }

    /// Test JmpIf
    #[test]
    fn test_create_jump_if() {
        let from_addr = 0x2000;
        let to_addr = 0x2010;

        let instr = Instruction::create_jump_if(from_addr, to_addr);

        assert_eq!(instr.opcode, Opcode::JmpIf);

        if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
            let expected_offset = calculate_branch_offset(from_addr, to_addr, instr.total_size() as u32);
            assert_eq!(offset, expected_offset);
        } else {
            panic!("Failed to extract offset from JmpIf instruction");
        }
    }

    /// Test JmpIfNot
    #[test]
    fn test_create_jump_if_not() {
        let from_addr = 0x3000;
        let to_addr = 0x2FF0; // Saut en arrière

        let instr = Instruction::create_jump_if_not(from_addr, to_addr);

        assert_eq!(instr.opcode, Opcode::JmpIfNot);

        if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
            assert!(offset < 0, "Offset should be negative for backward jump");
            let expected_offset = calculate_branch_offset(from_addr, to_addr, instr.total_size() as u32);
            assert_eq!(offset, expected_offset);
        } else {
            panic!("Failed to extract offset from JmpIfNot instruction");
        }
    }

    /// Test JmpIfEqual
    #[test]
    fn test_create_jump_if_equal() {
        let from_addr = 0x4000;
        let to_addr = 0x4100;

        let instr = Instruction::create_jump_if_equal(from_addr, to_addr);

        assert_eq!(instr.opcode, Opcode::JmpIfEqual);

        if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
            assert!(offset > 0, "Offset should be positive for forward jump");
            let expected_offset = calculate_branch_offset(from_addr, to_addr, instr.total_size() as u32);
            assert_eq!(offset, expected_offset);
        } else {
            panic!("Failed to extract offset from JmpIfEqual instruction");
        }
    }

    /// Test JmpIfNotEqual
    #[test]
    fn test_create_jump_if_not_equal() {
        let from_addr = 0x5000;
        let to_addr = 0x5050;

        let instr = Instruction::create_jump_if_not_equal(from_addr, to_addr);

        assert_eq!(instr.opcode, Opcode::JmpIfNotEqual);

        if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
            let expected_offset = calculate_branch_offset(from_addr, to_addr, instr.total_size() as u32);
            assert_eq!(offset, expected_offset);
        } else {
            panic!("Failed to extract offset from JmpIfNotEqual instruction");
        }
    }

    /// Test JmpIfGreater
    #[test]
    fn test_create_jump_if_greater() {
        let from_addr = 0x6000;
        let to_addr = 0x6040;

        let instr = Instruction::create_jump_if_greater(from_addr, to_addr);

        assert_eq!(instr.opcode, Opcode::JmpIfGreater);

        if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
            let expected_offset = calculate_branch_offset(from_addr, to_addr, instr.total_size() as u32);
            assert_eq!(offset, expected_offset);
        } else {
            panic!("Failed to extract offset from JmpIfGreater instruction");
        }
    }

    /// Test JmpIfGreaterEqual
    #[test]
    fn test_create_jump_if_greater_equal() {
        let from_addr = 0x7000;
        let to_addr = 0x7030;

        let instr = Instruction::create_jump_if_greater_equal(from_addr, to_addr);

        assert_eq!(instr.opcode, Opcode::JmpIfGreaterEqual);

        if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
            let expected_offset = calculate_branch_offset(from_addr, to_addr, instr.total_size() as u32);
            assert_eq!(offset, expected_offset);
        } else {
            panic!("Failed to extract offset from JmpIfGreaterEqual instruction");
        }
    }

    /// Test JmpIfLess
    #[test]
    fn test_create_jump_if_less() {
        let from_addr = 0x8000;
        let to_addr = 0x7FF0; // Saut en arrière

        let instr = Instruction::create_jump_if_less(from_addr, to_addr);

        assert_eq!(instr.opcode, Opcode::JmpIfLess);

        if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
            assert!(offset < 0, "Offset should be negative for backward jump");
            let expected_offset = calculate_branch_offset(from_addr, to_addr, instr.total_size() as u32);
            assert_eq!(offset, expected_offset);
        } else {
            panic!("Failed to extract offset from JmpIfLess instruction");
        }
    }

    /// Test JmpIfLessEqual
    #[test]
    fn test_create_jump_if_less_equal() {
        let from_addr = 0x9000;
        let to_addr = 0x9020;

        let instr = Instruction::create_jump_if_less_equal(from_addr, to_addr);

        assert_eq!(instr.opcode, Opcode::JmpIfLessEqual);

        if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
            let expected_offset = calculate_branch_offset(from_addr, to_addr, instr.total_size() as u32);
            assert_eq!(offset, expected_offset);
        } else {
            panic!("Failed to extract offset from JmpIfLessEqual instruction");
        }
    }

    /// Test JmpIfAbove (non signé)
    #[test]
    fn test_create_jump_if_above() {
        let from_addr = 0xA000;
        let to_addr = 0xA040;

        let instr = Instruction::create_jump_if_above(from_addr, to_addr);

        // Note: Il y a un bug dans votre code - create_jump_if_above crée JmpIfLessEqual
        // Ce test détectera ce bug
        assert_eq!(instr.opcode, Opcode::JmpIfAbove, "Bug: create_jump_if_above crée le mauvais opcode!");
    }

    /// Test JmpIfAboveEqual (non signé)
    #[test]
    fn test_create_jump_if_above_equal() {
        let from_addr = 0xB000;
        let to_addr = 0xB030;

        let instr = Instruction::create_jump_if_above_equal(from_addr, to_addr);

        assert_eq!(instr.opcode, Opcode::JmpIfAboveEqual);

        if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
            let expected_offset = calculate_branch_offset(from_addr, to_addr, instr.total_size() as u32);
            assert_eq!(offset, expected_offset);
        } else {
            panic!("Failed to extract offset from JmpIfAboveEqual instruction");
        }
    }

    /// Test JmpIfBelow (non signé)
    #[test]
    fn test_create_jump_below() {
        let from_addr = 0xC000;
        let to_addr = 0xC050;

        let instr = Instruction::create_jump_below(from_addr, to_addr);

        assert_eq!(instr.opcode, Opcode::JmpIfBelow);

        if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
            let expected_offset = calculate_branch_offset(from_addr, to_addr, instr.total_size() as u32);
            assert_eq!(offset, expected_offset);
        } else {
            panic!("Failed to extract offset from JmpIfBelow instruction");
        }
    }

    /// Test JmpIfBelowEqual (non signé)
    #[test]
    fn test_create_jump_if_below_equal() {
        let from_addr = 0xD000;
        let to_addr = 0xD020;

        let instr = Instruction::create_jump_if_below_equal(from_addr, to_addr);

        assert_eq!(instr.opcode, Opcode::JmpIfBelowEqual);

        if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
            let expected_offset = calculate_branch_offset(from_addr, to_addr, instr.total_size() as u32);
            assert_eq!(offset, expected_offset);
        } else {
            panic!("Failed to extract offset from JmpIfBelowEqual instruction");
        }
    }

    /// Test JmpIfZero
    #[test]
    fn test_create_jump_if_zero() {
        let from_addr = 0xE000;
        let to_addr = 0xE100;

        let instr = Instruction::create_jump_if_zero(from_addr, to_addr);

        assert_eq!(instr.opcode, Opcode::JmpIfZero);

        if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
            let expected_offset = calculate_branch_offset(from_addr, to_addr, instr.total_size() as u32);
            assert_eq!(offset, expected_offset);
        } else {
            panic!("Failed to extract offset from JmpIfZero instruction");
        }
    }

    /// Test JmpIfNotZero
    #[test]
    fn test_create_jump_if_not_zero() {
        let from_addr = 0xF000;
        let to_addr = 0xEFF0; // Saut en arrière

        let instr = Instruction::create_jump_if_not_zero(from_addr, to_addr);

        assert_eq!(instr.opcode, Opcode::JmpIfNotZero);

        if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
            assert!(offset < 0, "Offset should be negative for backward jump");
            let expected_offset = calculate_branch_offset(from_addr, to_addr, instr.total_size() as u32);
            assert_eq!(offset, expected_offset);
        } else {
            panic!("Failed to extract offset from JmpIfNotZero instruction");
        }
    }

    /// Test JmpIfOverflow
    #[test]
    fn test_create_jump_if_overflow() {
        let from_addr = 0x10000;
        let to_addr = 0x10020;

        let instr = Instruction::create_jump_if_overflow(from_addr, to_addr);

        assert_eq!(instr.opcode, Opcode::JmpIfOverflow);

        if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
            let expected_offset = calculate_branch_offset(from_addr, to_addr, instr.total_size() as u32);
            assert_eq!(offset, expected_offset);
        } else {
            panic!("Failed to extract offset from JmpIfOverflow instruction");
        }
    }

    /// Test JmpIfNotOverflow
    #[test]
    fn test_create_jump_if_not_overflow() {
        let from_addr = 0x11000;
        let to_addr = 0x11040;

        let instr = Instruction::create_jump_if_not_overflow(from_addr, to_addr); // Note: typo dans le nom de la fonction

        assert_eq!(instr.opcode, Opcode::JmpIfNotOverflow);

        if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
            let expected_offset = calculate_branch_offset(from_addr, to_addr, instr.total_size() as u32);
            assert_eq!(offset, expected_offset);
        } else {
            panic!("Failed to extract offset from JmpIfNotOverflow instruction");
        }
    }

    /// Test JmpIfPositive
    #[test]
    fn test_create_jump_if_positive() {
        let from_addr = 0x12000;
        let to_addr = 0x12050;

        let instr = Instruction::create_jump_if_positive(from_addr, to_addr);

        assert_eq!(instr.opcode, Opcode::JmpIfPositive);

        if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
            let expected_offset = calculate_branch_offset(from_addr, to_addr, instr.total_size() as u32);
            assert_eq!(offset, expected_offset);
        } else {
            panic!("Failed to extract offset from JmpIfPositive instruction");
        }
    }

    /// Test JmpIfNegative
    #[test]
    fn test_create_jump_if_negative() {
        let from_addr = 0x13000;
        let to_addr = 0x13030;

        let instr = Instruction::create_jump_if_negative(from_addr, to_addr);

        assert_eq!(instr.opcode, Opcode::JmpIfNegative);

        if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
            let expected_offset = calculate_branch_offset(from_addr, to_addr, instr.total_size() as u32);
            assert_eq!(offset, expected_offset);
        } else {
            panic!("Failed to extract offset from JmpIfNegative instruction");
        }
    }

    /// Test de boucle avec plusieurs types de sauts
    #[test]
    fn test_complex_branching_scenario() {
        // Simuler un programme avec différents types de branchements
        let mut current_addr = 0x20000;
        let mut instructions = Vec::new();

        // MOV R0, 0
        let instr = Instruction::create_reg_imm8(Opcode::Mov, 0, 0);
        instructions.push((current_addr, instr.clone()));
        current_addr += instr.total_size() as u32;

        // MOV R1, 10
        let instr = Instruction::create_reg_imm8(Opcode::Mov, 1, 10);
        instructions.push((current_addr, instr.clone()));
        current_addr += instr.total_size() as u32;

        // Début de boucle
        let loop_start = current_addr;

        // INC R0
        let instr = Instruction::create_single_reg(Opcode::Inc, 0);
        instructions.push((current_addr, instr.clone()));
        current_addr += instr.total_size() as u32;

        // CMP R0, R1
        let instr = Instruction::create_reg_reg(Opcode::Cmp, 0, 1);
        instructions.push((current_addr, instr.clone()));
        current_addr += instr.total_size() as u32;

        // JmpIfLess loop_start (saut en arrière)
        let jump_addr = current_addr;
        let instr = Instruction::create_jump_if_less(jump_addr, loop_start);
        instructions.push((current_addr, instr.clone()));

        // Vérifier que l'offset est négatif pour un saut en arrière
        if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
            assert!(offset < 0, "Offset should be negative for backward jump in loop");
            println!("Loop backward jump offset: {} (from 0x{:X} to 0x{:X})",
                     offset, jump_addr, loop_start);
        }

        // Vérifier que toutes les instructions ont été créées correctement
        assert_eq!(instructions.len(), 5);
        for (addr, instr) in &instructions {
            println!("0x{:X}: {:?} (size: {} bytes)", addr, instr.opcode, instr.total_size());
        }
    }

    /// Test d'encodage et décodage des instructions de branchement
    #[test]
    fn test_branch_encode_decode() {
        let from_addr = 0x30000;
        let to_addr = 0x30100;

        // Créer une instruction de branchement
        let original = Instruction::create_jump_if_equal(from_addr, to_addr);

        // Encoder
        let encoded = original.encode();
        println!("Encoded instruction: {:?}", encoded);

        // Decoder
        let (decoded, size) = Instruction::decode(&encoded).expect("Failed to decode instruction");

        // Vérifier que l'instruction décodée est identique
        assert_eq!(decoded.opcode, original.opcode);
        assert_eq!(decoded.format, original.format);
        assert_eq!(decoded.args, original.args);
        assert_eq!(size, encoded.len());

        // Vérifier que l'offset est préservé
        if let (Ok(ArgValue::RelativeAddr(orig_offset)), Ok(ArgValue::RelativeAddr(dec_offset))) =
            (original.get_arg2_value(), decoded.get_arg2_value()) {
            assert_eq!(orig_offset, dec_offset);
        } else {
            panic!("Failed to extract offsets for comparison");
        }
    }

    /// Test des cas limites
    #[test]


    fn test_branch_edge_cases() {
        // Test avec offset positif proche de la limite
        let from_addr = 0;
        let to_addr = 0x7FFFFF00; // Un peu moins que i32::MAX
        let instr = Instruction::create_jump(from_addr, to_addr);

        if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
            assert!(offset > 0, "Large positive offset should be positive");
            println!("Large positive offset: {}", offset);
        } else {
            panic!("Failed to extract offset from large positive jump");
        }

        // Test avec offset négatif sécurisé
        let from_addr = 0x7FFFFF00;
        let to_addr = 0x100; // Proche du début
        let instr = Instruction::create_jump(from_addr, to_addr);

        if let Ok(ArgValue::RelativeAddr(offset)) = instr.get_arg2_value() {
            assert!(offset < 0, "Large negative offset should be negative");
            println!("Large negative offset: {}", offset);
        } else {
            panic!("Failed to extract offset from large negative jump");
        }
    }
}


// Test unitaire pour les instructions
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction_new() {
        // Test création d'instruction simple
        let instr = Instruction::new(
            Opcode::Add,
            InstructionFormat::double_reg(),
            // vec![0x03, 0x05], // rd=3, rs1=5
            vec![1, 2, 3]
        );

        assert_eq!(instr.opcode, Opcode::Add);
        assert_eq!(instr.format.arg1_type, ArgType::Register);
        assert_eq!(instr.format.arg2_type, ArgType::Register);
        assert_eq!(instr.format.arg3_type, ArgType::None);
        assert_eq!(instr.size_type, SizeType::Compact);
        // assert_eq!(instr.args, vec![0x03, 0x05]);
        assert_eq!(instr.args, vec![1, 2, 3]);
    }

    #[test]
    fn test_instruction_total_size() {
        // Instruction sans arguments
        let instr1 = Instruction::create_no_args(Opcode::Nop);
        assert_eq!(instr1.total_size(), 4); // opcode (1) + format (2) + size (1)

        // Instruction avec 1 registre
        let instr2 = Instruction::create_single_reg(Opcode::Inc, 3);
        assert_eq!(instr2.total_size(), 5); // opcode (1) + format (2) + size (1) + reg (1)

        // Instruction avec 2 registres
        let instr3 = Instruction::create_reg_reg(Opcode::Add, 2, 3);
        assert_eq!(instr3.total_size(), 6); // opcode (1) + format (2) + size (1) + regs (2)

        // Instruction avec 3 registres
        let instr4 = Instruction::create_reg_reg_reg(Opcode::Add, 2, 3, 4);
        assert_eq!(instr4.total_size(), 7); // opcode (1) + format (2) + size (1) + regs (3)
    }


    #[test]
    fn test_instruction_helpers() {
        // Test création sans arguments
        let nop = Instruction::create_no_args(Opcode::Nop);
        assert_eq!(nop.opcode, Opcode::Nop);
        assert_eq!(nop.args.len(), 0);

        // Test création avec un registre
        let inc = Instruction::create_single_reg(Opcode::Inc, 3);
        assert_eq!(inc.opcode, Opcode::Inc);
        assert_eq!(inc.args, vec![3]);

        // Test création avec deux registres
        let mov = Instruction::create_reg_reg(Opcode::Mov, 1, 2);
        assert_eq!(mov.opcode, Opcode::Mov);
        assert_eq!(mov.args, vec![1, 2]);

        // Test création avec trois registres
        let add = Instruction::create_reg_reg_reg(Opcode::Add, 1, 2, 3);
        assert_eq!(add.opcode, Opcode::Add);
        assert_eq!(add.args, vec![1, 2, 3]);
    }

    // Tests pour le calcul de la taille des instructions
    mod size_calculation {
        use super::*;

        #[test]
        fn test_instruction_sizes() {
            let sizes = vec![
                (Instruction::create_no_args(Opcode::Nop), 4),
                (Instruction::create_single_reg(Opcode::Inc, 1), 5),
                (Instruction::create_reg_reg(Opcode::Mov, 1, 2), 6),
                (Instruction::create_reg_reg_reg(Opcode::Add, 1, 2, 3), 7),
                (Instruction::create_reg_imm8(Opcode::Load, 1, 42), 6),
                (Instruction::create_reg_imm16(Opcode::Load, 1, 1000), 7),
            ];

            for (instr, expected_size) in sizes {
                assert_eq!(instr.total_size(), expected_size);
            }
        }
    }

    #[test]
    fn test_instruction_encoding_decoding() {
        let instructions = vec![
            Instruction::create_no_args(Opcode::Nop),
            Instruction::create_single_reg(Opcode::Inc, 3),
            Instruction::create_reg_reg(Opcode::Mov, 1, 2),
            Instruction::create_reg_reg_reg(Opcode::Add, 1, 2, 3),
            Instruction::create_reg_imm8(Opcode::Load, 1, 42),
            Instruction::create_reg_imm16(Opcode::Load, 1, 1000),
        ];

        for original in instructions {
            let encoded = original.encode();
            let (decoded, size) = Instruction::decode(&encoded).unwrap();

            assert_eq!(decoded.opcode, original.opcode);
            assert_eq!(decoded.format, original.format);
            assert_eq!(decoded.args, original.args);
            assert_eq!(size, original.total_size());
        }
    }
    //
    #[test]
    fn test_instruction_encode_decode() {
        // Créer et encoder une instruction
        let original = Instruction::create_reg_imm8(Opcode::Load, 3, 42);
        let encoded = original.encode();

        // Décoder l'instruction encodée
        let (decoded, size) = Instruction::decode(&encoded).unwrap();

        // Vérifier que le décodage correspond à l'original
        assert_eq!(decoded.opcode, original.opcode);
        assert_eq!(decoded.format.arg1_type, original.format.arg1_type);
        assert_eq!(decoded.format.arg2_type, original.format.arg2_type);
        assert_eq!(decoded.format.arg3_type, original.format.arg3_type);
        assert_eq!(decoded.args, original.args);
        assert_eq!(size, original.total_size());
    }


    #[test]
    fn test_get_argument_values_1() {
        // Test pour les instructions à 3 registres
        let add = Instruction::create_reg_reg_reg(Opcode::Add, 1, 2, 3);
        assert_eq!(add.get_arg1_value().unwrap(), ArgValue::Register(1));
        assert_eq!(add.get_arg2_value().unwrap(), ArgValue::Register(2));
        assert_eq!(add.get_arg3_value().unwrap(), ArgValue::Register(3));

        // Test pour les instructions reg + imm8
        let load = Instruction::create_reg_imm8(Opcode::Load, 1, 42);
        assert_eq!(load.get_arg1_value().unwrap(), ArgValue::Register(1));
        assert_eq!(load.get_arg2_value().unwrap(), ArgValue::Immediate(42));
    }



    #[test]
    fn test_encode_decode_reg_reg_reg() {
        // Test spécifique pour l'encodage/décodage des instructions à 3 registres
        let original = Instruction::create_reg_reg_reg(Opcode::Add, 2, 3, 4);
        let encoded = original.encode();

        // Décoder l'instruction encodée
        let (decoded, size) = Instruction::decode(&encoded).unwrap();

        // Vérifier que le décodage correspond à l'original
        assert_eq!(decoded.opcode, original.opcode);
        assert_eq!(decoded.format.arg1_type, original.format.arg1_type);
        assert_eq!(decoded.format.arg2_type, original.format.arg2_type);
        assert_eq!(decoded.format.arg3_type, original.format.arg3_type);
        assert_eq!(decoded.args, original.args);
        assert_eq!(size, original.total_size());

        // Vérifier que les arguments sont correctement extraits
        if let Ok(ArgValue::Register(rd)) = decoded.get_arg1_value() {
            assert_eq!(rd, 2);
        } else {
            panic!("Failed to get destination register value");
        }

        if let Ok(ArgValue::Register(rs1)) = decoded.get_arg2_value() {
            assert_eq!(rs1, 3);
        } else {
            panic!("Failed to get first source register value");
        }

        if let Ok(ArgValue::Register(rs2)) = decoded.get_arg3_value() {
            assert_eq!(rs2, 4);
        } else {
            panic!("Failed to get second source register value");
        }
    }

    #[test]
    fn test_get_argument_values_2() {
        // Test pour les instructions à 3 registres
        let add = Instruction::create_reg_reg_reg(Opcode::Add, 1, 2, 3);
        assert_eq!(add.get_arg1_value().unwrap(), ArgValue::Register(1));
        assert_eq!(add.get_arg2_value().unwrap(), ArgValue::Register(2));
        assert_eq!(add.get_arg3_value().unwrap(), ArgValue::Register(3));

        // Test pour les instructions reg + imm8
        let load = Instruction::create_reg_imm8(Opcode::Load, 1, 42);
        assert_eq!(load.get_arg1_value().unwrap(), ArgValue::Register(1));
        assert_eq!(load.get_arg2_value().unwrap(), ArgValue::Immediate(42));
    }

    #[test]
    fn test_get_argument_values() {
        // Test avec un registre unique
        let instr1 = Instruction::create_single_reg(Opcode::Inc, 3);

        if let Ok(ArgValue::Register(r1)) = instr1.get_arg1_value() {
            assert_eq!(r1, 3);
        } else {
            panic!("Failed to get register value");
        }

        // Test avec deux registres
        let instr2 = Instruction::create_reg_reg(Opcode::Add, 3, 5);

        // Vérifier que les arguments sont correctement stockés
        assert_eq!(instr2.args.len(), 2);
        assert_eq!(instr2.args[0], 3);
        assert_eq!(instr2.args[1], 5);

        // Tester get_arg1_value
        if let Ok(ArgValue::Register(r1)) = instr2.get_arg1_value() {
            assert_eq!(r1, 3);
        } else {
            panic!("Failed to get first register value");
        }

        // Tester get_arg2_value
        if let Ok(ArgValue::Register(r2)) = instr2.get_arg2_value() {
            assert_eq!(r2, 5);
        } else {
            panic!("Failed to get second register value");
        }

        // Test avec valeur immédiate
        let instr3 = Instruction::create_reg_imm8(Opcode::Load, 2, 123);

        if let Ok(ArgValue::Register(r)) = instr3.get_arg1_value() {
            assert_eq!(r, 2);
        } else {
            panic!("Failed to get register value");
        }

        if let Ok(ArgValue::Immediate(imm)) = instr3.get_arg2_value() {
            assert_eq!(imm, 123);
        } else {
            panic!("Failed to get immediate value");
        }
    }
    //
    #[test]
    fn test_get_argument_values_reg_reg_reg() {
        // Test avec trois registres pour les opérations arithmétiques
        let instr = Instruction::create_reg_reg_reg(Opcode::Add, 2, 3, 4);

        // Vérifier que les arguments sont correctement stockés
        assert_eq!(instr.args.len(), 3);
        assert_eq!(instr.args[0], 2);
        assert_eq!(instr.args[1], 3);
        assert_eq!(instr.args[2], 4);

        // Tester get_arg1_value (registre destination)
        if let Ok(ArgValue::Register(rd)) = instr.get_arg1_value() {
            assert_eq!(rd, 2);
        } else {
            panic!("Failed to get destination register value");
        }

        // Tester get_arg2_value (premier registre source)
        if let Ok(ArgValue::Register(rs1)) = instr.get_arg2_value() {
            assert_eq!(rs1, 3);
        } else {
            panic!("Failed to get first source register value");
        }

        // Tester get_arg3_value (deuxième registre source)
        if let Ok(ArgValue::Register(rs2)) = instr.get_arg3_value() {
            assert_eq!(rs2, 4);
        } else {
            panic!("Failed to get second source register value");
        }
    }

    #[test]
    fn test_create_helper_functions() {
        // Test les fonctions helper pour créer différents types d'instructions

        // Instruction sans arguments
        let instr1 = Instruction::create_no_args(Opcode::Nop);
        assert_eq!(instr1.opcode, Opcode::Nop);
        assert_eq!(instr1.args.len(), 0);

        // Instruction avec un seul registre
        let instr2 = Instruction::create_single_reg(Opcode::Inc, 7);
        assert_eq!(instr2.opcode, Opcode::Inc);
        assert_eq!(instr2.args.len(), 1);
        assert_eq!(instr2.args[0], 7);

        // Instruction avec deux registres
        let instr3 = Instruction::create_reg_reg(Opcode::Add, 3, 4);
        assert_eq!(instr3.opcode, Opcode::Add);
        assert_eq!(instr3.args.len(), 2);
        assert_eq!(instr3.args[0], 3);
        assert_eq!(instr3.args[1], 4);

        // Instruction avec trois registres
        let instr4 = Instruction::create_reg_reg_reg(Opcode::Add, 2, 3, 4);
        assert_eq!(instr4.opcode, Opcode::Add);
        assert_eq!(instr4.args.len(), 3);
        assert_eq!(instr4.args[0], 2);
        assert_eq!(instr4.args[1], 3);
        assert_eq!(instr4.args[2], 4);

        // Instruction avec registre et immédiat 8-bit
        let instr5 = Instruction::create_reg_imm8(Opcode::Load, 2, 42);
        assert_eq!(instr5.opcode, Opcode::Load);
        assert_eq!(instr5.args.len(), 2);
        assert_eq!(instr5.args[0], 2);
        assert_eq!(instr5.args[1], 42);
    }

    #[test]
    fn test_arithmetic_instruction_creation() {
        // Test ADD R2, R0, R1
        let add_instr = Instruction::create_reg_reg_reg(Opcode::Add, 2, 0, 1);
        assert_eq!(add_instr.opcode, Opcode::Add);
        assert_eq!(add_instr.format.arg1_type, ArgType::Register);
        assert_eq!(add_instr.format.arg2_type, ArgType::Register);
        assert_eq!(add_instr.format.arg3_type, ArgType::Register);
        assert_eq!(add_instr.args, vec![2, 0, 1]);

        // Test SUB R3, R0, R1
        let sub_instr = Instruction::create_reg_reg_reg(Opcode::Sub, 3, 0, 1);
        assert_eq!(sub_instr.opcode, Opcode::Sub);
        assert_eq!(sub_instr.args, vec![3, 0, 1]);

        // Test MUL R4, R0, R1
        let mul_instr = Instruction::create_reg_reg_reg(Opcode::Mul, 4, 0, 1);
        assert_eq!(mul_instr.opcode, Opcode::Mul);
        assert_eq!(mul_instr.args, vec![4, 0, 1]);

        // Test DIV R5, R0, R1
        let div_instr = Instruction::create_reg_reg_reg(Opcode::Div, 5, 0, 1);
        assert_eq!(div_instr.opcode, Opcode::Div);
        assert_eq!(div_instr.args, vec![5, 0, 1]);
    }

    #[test]
    fn test_format_encoding_decoding() {
        // Tester l'encodage et le décodage du format à 2 octets
        let format = InstructionFormat::reg_reg_reg();
        let encoded = format.encode();
        let decoded = InstructionFormat::decode(encoded).unwrap();

        assert_eq!(decoded.arg1_type, ArgType::Register);
        assert_eq!(decoded.arg2_type, ArgType::Register);
        assert_eq!(decoded.arg3_type, ArgType::Register);
    }

    #[test]
    fn test_error_conditions() {
        // Test de décodage avec données insuffisantes
        let result = Instruction::decode(&[0x01]);
        assert!(result.is_err());

        if let Err(e) = result {
            assert_eq!(e, DecodeError::InsufficientData);
        }

        // Test de décodage avec opcode invalide
        let result = Instruction::decode(&[0xFF, 0x00, 0x00, 0x03]);
        assert!(result.is_err());

        if let Err(e) = result {
            match e {
                DecodeError::InvalidOpcode(_) => (), // Expected
                _ => panic!("Unexpected error type"),
            }
        }

        // Test de décodage avec format invalide
        let result = Instruction::decode(&[0x01, 0xFF, 0xFF, 0x03]);
        assert!(result.is_err());

        if let Err(e) = result {
            match e {
                DecodeError::InvalidFormat(_) => (), // Expected
                _ => panic!("Unexpected error type"),
            }
        }
    }

    #[test]
    fn test_extended_size_encoding() {
        // Créer une instruction avec suffisamment d'arguments pour forcer un encodage Extended
        let large_args = vec![0; 254]; // Suffisant pour dépasser la limite de 255 octets

        // Création de l'instruction avec un format simple
        let instr = Instruction::new(Opcode::Add, InstructionFormat::reg_reg(), large_args);

        // Vérifier qu'elle est bien en mode Extended
        assert_eq!(instr.size_type, SizeType::Extended);

        // Encoder l'instruction
        let encoded = instr.encode();

        // Vérifier que l'encodage est correct
        assert_eq!(encoded[3], 0xFF); // Marqueur du format Extended

        // Décoder l'instruction encodée
        let (decoded, size) = Instruction::decode(&encoded).unwrap();

        // Vérifier que le décodage a bien identifié le format Extended
        assert_eq!(decoded.size_type, SizeType::Extended);
        assert_eq!(size, instr.total_size());
    }

    #[test]
    fn test_args_size_calculation() {
        // Vérifier que le calcul de la taille des arguments est correct
        let format = InstructionFormat::reg_reg_reg();
        let args_size = format.args_size();
        assert_eq!(args_size, 3); // 1 octet par registre

        let format2 = InstructionFormat::reg_imm8();
        let args_size2 = format2.args_size();
        assert_eq!(args_size2, 2); // 1 octet pour registre + 1 octet pour immédiat
    }

    #[test]
    fn test_jump_instructions() {
        let from_addr = 0x1000;
        let to_addr = 0x1020;
        let instr_size = 8;
        let offset = Instruction::calculate_branch_offset(from_addr, to_addr, instr_size);

        // Créer un vecteur de tuples (instruction, opcode attendu)
        let jumps = vec![
            (Instruction::create_jump(from_addr, to_addr), Opcode::Jmp),
            (Instruction::create_jump_if(from_addr, to_addr), Opcode::JmpIf),
            (Instruction::create_jump_if_zero(from_addr, to_addr), Opcode::JmpIfZero),
            (Instruction::create_jump_if_not_zero(from_addr, to_addr), Opcode::JmpIfNotZero)
        ];

        for (instr, expected_opcode) in jumps {
            assert_eq!(instr.opcode, expected_opcode);

            // Vérifier que l'argument est bien un offset relatif
            if let Ok(ArgValue::Immediate(addr)) = instr.get_arg1_value() {
                assert_eq!(addr as i32, offset);
            } else {
                panic!("Expected Immediate argument for jump offset");
            }

            // Vérifier la taille de l'instruction
            assert_eq!(instr.total_size(), instr_size as usize);

            // Vérifier l'encodage/décodage
            let encoded = instr.encode();
            let (decoded, size) = Instruction::decode(&encoded).unwrap();

            assert_eq!(decoded.opcode, instr.opcode);
            assert_eq!(size, instr.total_size());

            if let Ok(ArgValue::Immediate(decoded_offset)) = decoded.get_arg1_value() {
                assert_eq!(decoded_offset as i32, offset);
            } else {
                panic!("Expected Immediate argument after decode");
            }
        }
    }
    // //
    // #[test]
    // fn test_jump_instructions() {
    //     // Test de création d'instruction de saut
    //     let offset = 42;
    //     let jump_instr = Instruction::create_jump(offset);
    //     assert_eq!(jump_instr.opcode, Opcode::Jmp);
    //     assert_eq!(jump_instr.args.len(), 4); // 4 octets pour l'offset
    //
    //     // Test de création d'instruction de saut conditionnel
    //     let jump_if_instr = Instruction::create_jump_if(offset);
    //     assert_eq!(jump_if_instr.opcode, Opcode::JmpIf);
    //     assert_eq!(jump_if_instr.args.len(), 4); // 4 octets pour l'offset
    // }
    // //
    #[test]
    fn test_jump_if() {
        // Test de création d'instruction de saut conditionnel
        // let offset = 42;
        let from_addr = 0x1000;
        let to_addr = 0x1020;
        // let jump_if_instr = Instruction::create_jump_if(offset);
        let jump_if_instr = Instruction::create_jump_if(from_addr, to_addr);
        assert_eq!(jump_if_instr.opcode, Opcode::JmpIf);
        assert_eq!(jump_if_instr.args.len(), 4); // 4 octets pour l'offset
    }

    #[test]
    fn test_jump_if_not() {
        // Test de création d'instruction de saut conditionnel
        // let offset = 42;
        let from_addr = 0x1000;
        let to_addr = 0x1020;
        // let jump_if_not_instr = Instruction::create_jump_if_not(offset);
        let jump_if_not_instr = Instruction::create_jump_if_not(from_addr, to_addr);
        assert_eq!(jump_if_not_instr.opcode, Opcode::JmpIfNot);
        assert_eq!(jump_if_not_instr.args.len(), 4); // 4 octets pour l'offset
    }

    #[test]
    fn test_jump_if_equal() {
        // Test de création d'instruction de saut conditionnel
        // let offset = 42;
        let from_addr = 0x1000;
        let to_addr = 0x1020;
        // let jump_if_equal_instr = Instruction::create_jump_if_equal(offset);
        let jump_if_equal_instr = Instruction::create_jump_if_equal(from_addr, to_addr);
        assert_eq!(jump_if_equal_instr.opcode, Opcode::JmpIfEqual);

        assert_eq!(jump_if_equal_instr.args.len(), 4); // 4 octets pour l'offset
    }

    #[test]
    fn test_jump_if_not_equal() {
        // Test de création d'instruction de saut conditionnel
        // let offset = 42;
        let from_addr = 0x1000;
        let to_addr = 0x1020;
        // let jump_if_not_equal_instr = Instruction::create_jump_if_not_equal(offset);
        let jump_if_not_equal_instr = Instruction::create_jump_if_not_equal(from_addr, to_addr);
        assert_eq!(jump_if_not_equal_instr.opcode, Opcode::JmpIfNotEqual);
        assert_eq!(jump_if_not_equal_instr.args.len(), 4); // 4 octets pour l'offset
    }

    #[test]
    fn test_jump_if_greater() {
        // Test de création d'instruction de saut conditionnel
        // let offset = 42;
        let from_addr = 0x1000;
        let to_addr = 0x1020;
        // let jump_if_greater_instr = Instruction::create_jump_if_greater(offset);
        let jump_if_greater_instr = Instruction::create_jump_if_greater(from_addr, to_addr);
        assert_eq!(jump_if_greater_instr.opcode, Opcode::JmpIfGreater);
        assert_eq!(jump_if_greater_instr.args.len(), 4); // 4 octets pour l'offset
    }

    #[test]
    fn test_jump_if_greater_equal() {
        // Test de création d'instruction de saut conditionnel
        // let offset = 42;
        let from_addr = 0x1000;
        let to_addr = 0x1020;
        // let jump_if_greater_equal_instr = Instruction::create_jump_if_greater_equal(offset);
        let jump_if_greater_equal_instr = Instruction::create_jump_if_greater_equal(from_addr, to_addr);
        assert_eq!(
            jump_if_greater_equal_instr.opcode,
            Opcode::JmpIfGreaterEqual
        );
        assert_eq!(jump_if_greater_equal_instr.args.len(), 4); // 4 octets pour l'offset
    }

    #[test]
    fn test_jump_if_less() {
        // Test de création d'instruction de saut conditionnel
        // let offset = 42;
        let from_addr = 0x1000;
        let to_addr = 0x1020;
        // let jump_if_less_instr = Instruction::create_jump_if_less(offset);
        let jump_if_less_instr = Instruction::create_jump_if_less(from_addr, to_addr);
        assert_eq!(jump_if_less_instr.opcode, Opcode::JmpIfLess);
        assert_eq!(jump_if_less_instr.args.len(), 4); // 4 octets pour l'offset
    }

    #[test]
    fn test_jump_if_less_equal() {
        // Test de création d'instruction de saut conditionnel
        // let offset = 42;
        let from_addr = 0x1000;
        let to_addr = 0x1020;
        // let jump_if_less_equal_instr = Instruction::create_jump_if_less_equal(offset);
        let jump_if_less_equal_instr = Instruction::create_jump_if_less_equal(from_addr, to_addr);
        assert_eq!(jump_if_less_equal_instr.opcode, Opcode::JmpIfLessEqual);
        assert_eq!(jump_if_less_equal_instr.args.len(), 4); // 4 octets pour l'offset
    }

    #[test]
    fn test_jump_above() {
        // Test de création d'instruction de saut conditionnel
        // let offset = 42;
        let from_addr = 0x1000;
        let to_addr = 0x1020;
        // let jump_above_instr = Instruction::create_jump_if_above(offset);
        // let jump_above_instr = Instruction::create_jump_if_above(from_addr, to_addr);
        let jump_above_inst = Instruction::create_jump_if_above(from_addr, to_addr);
        assert_eq!(jump_above_inst.opcode, Opcode::JmpIfAbove);
        assert_eq!(jump_above_inst.args.len(), 4); // 4 octets pour l'offset
    }
    #[test]
    fn test_jump_above_equal() {
        // Test de création d'instruction de saut conditionnel
        // let offset = 42;
        let from_addr = 0x1000;
        let to_addr = 0x1020;
        // let jump_above_equal_instr = Instruction::create_jump_if_above_equal(offset);
        let jump_above_equal_instr = Instruction::create_jump_if_above_equal(from_addr, to_addr);
        assert_eq!(jump_above_equal_instr.opcode, Opcode::JmpIfAboveEqual);
        assert_eq!(jump_above_equal_instr.args.len(), 4); // 4 octets pour l'offset
    }

    #[test]
    fn test_jump_below() {
        // Test de création d'instruction de saut conditionnel
        // let offset = 42;
        let from_addr = 0x1000;
        let to_addr = 0x1020;
        // let jump_below_instr = Instruction::create_jump_below(offset);
        let jump_below_instr = Instruction::create_jump_below(from_addr, to_addr);
        assert_eq!(jump_below_instr.opcode, Opcode::JmpIfBelow);
        assert_eq!(jump_below_instr.args.len(), 4); // 4 octets pour l'offset
    }

    #[test]
    fn test_jump_below_equal() {
        // Test de création d'instruction de saut conditionnel
        // let offset = 42;
        let from_addr = 0x1000;
        let to_addr = 0x1020;
        // let jump_below_equal_instr = Instruction::create_jump_if_below_equal(offset);
        let jump_below_equal_instr = Instruction::create_jump_if_below_equal(from_addr, to_addr);
        assert_eq!(jump_below_equal_instr.opcode, Opcode::JmpIfBelowEqual);
        assert_eq!(jump_below_equal_instr.args.len(), 4); // 4 octets pour l'offset
    }

    #[test]
    fn test_jump_zero() {
        // Test de création d'instruction de saut conditionnel
        // let offset = 42;
        let from_addr = 0x1000;
        let to_addr = 0x1020;
        // let jump_zero_instr = Instruction::create_jump_if_zero(offset);
        let jump_zero_instr = Instruction::create_jump_if_zero(from_addr, to_addr);
        assert_eq!(jump_zero_instr.opcode, Opcode::JmpIfZero);
        assert_eq!(jump_zero_instr.args.len(), 4); // 4 octets pour l'offset
    }

    #[test]
    fn test_jump_not_zero() {
        // Test de création d'instruction de saut conditionnel
        // let offset = 42;
        let from_addr = 0x1000;
        let to_addr = 0x1020;
        // let jump_not_zero_instr = Instruction::create_jump_if_not_zero(offset);
        let jump_not_zero_instr = Instruction::create_jump_if_not_zero(from_addr, to_addr);
        assert_eq!(jump_not_zero_instr.opcode, Opcode::JmpIfNotZero);
        assert_eq!(jump_not_zero_instr.args.len(), 4); // 4 octets pour l'offset
    }

    #[test]
    fn test_jump_if_overflow() {
        // Test de création d'instruction de saut conditionnel
        // let offset = 42;
        let from_addr = 0x1000;
        let to_addr = 0x1020;
        // let jump_if_overflow_instr = Instruction::create_jump_if_overflow(offset);
        let jump_if_overflow_instr = Instruction::create_jump_if_overflow(from_addr, to_addr);
        assert_eq!(jump_if_overflow_instr.opcode, Opcode::JmpIfOverflow);
        assert_eq!(jump_if_overflow_instr.args.len(), 4); // 4 octets pour l'offset
    }

    #[test]
    fn test_jump_if_not_overflow() {
        // Test de création d'instruction de saut conditionnel
        // let offset = 42;
        let from_addr = 0x1000;
        let to_addr = 0x1020;
        // let jump_if_not_overflow_instr = Instruction::create_jump_if_not_overflow(offset);
        let jump_if_not_overflow_instr = Instruction::create_jump_if_not_overflow(from_addr, to_addr);


        assert_eq!(jump_if_not_overflow_instr.opcode, Opcode::JmpIfNotOverflow);
        assert_eq!(jump_if_not_overflow_instr.args.len(), 4); // 4 octets pour l'offset
    }

    #[test]
    fn test_jump_if_positive() {
        // Test de création d'instruction de saut conditionnel
        // let offset = 42;
        let from_addr = 0x1000;
        let to_addr = 0x1020;
        // let jump_if_positive_instr = Instruction::create_jump_if_positive(offset);
        let jump_if_positive_instr = Instruction::create_jump_if_positive(from_addr, to_addr);
        assert_eq!(jump_if_positive_instr.opcode, Opcode::JmpIfPositive);
        assert_eq!(jump_if_positive_instr.args.len(), 4); // 4 octets pour l'offset
    }
}

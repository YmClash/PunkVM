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
        let fmt = InstructionFormat::reg_imm16(); // (Register, Immediate16, None)?
        // let mut args = vec![reg & 0x0FF];
        let args = vec![reg & 0x0FF,imm as u8, (imm >> 8) as u8];
        // args.push((imm & 0x00FF) as u8);
        // args.push(((imm >> 8  ) & 0xFF) as u8);
        Self::new(opcode, fmt, args)
    }

    /// Crée une instruction avec un registre et une valeur immédiate 32 bits
    pub fn create_reg_imm32(opcode: Opcode, reg: u8, imm: u32) -> Self {
        let fmt = InstructionFormat::reg_imm32(); // (Register, Immediate32, None)?
        let args = vec![
            reg & 0x0FF,
            (imm & 0x00FF) as u8,
            ((imm >> 8) & 0xFF) as u8,
            ((imm >> 16) & 0xFF) as u8,
            ((imm >> 24) & 0xFF) as u8,
        ];
        Self::new(opcode, fmt, args)
    }

    /// Crée une instruction avec un registre et une valeur immédiate 64 bits
    pub fn create_reg_imm64(opcode: Opcode, reg: u8, imm: u64) -> Self {
        let fmt = InstructionFormat::reg_imm64(); // (Register, Immediate64, None)?
        let args = vec![
            reg & 0x0FF,
            (imm & 0x00FF) as u8,
            ((imm >> 8) & 0xFF) as u8,
            ((imm >> 16) & 0xFF) as u8,
            ((imm >> 24) & 0xFF) as u8,
            ((imm >> 32) & 0xFF) as u8,
            ((imm >> 40) & 0xFF) as u8,
            ((imm >> 48) & 0xFF) as u8,
            ((imm >> 56) & 0xFF) as u8,
        ];
        Self::new(opcode, fmt, args)
    }

    /// Crée une instruction de chargement mémoire avec registre + offset
    pub fn create_load_reg_offset(reg_dest: u8, reg_base: u8, offset: i8) -> Self {
        let fmt = InstructionFormat::reg_regoff(); // (Register, RegisterOffset, None)?
        let args = vec![reg_dest & 0x0F, reg_base & 0x0F, offset as u8];
        Self::new(Opcode::Load, fmt, args)
    }

    /// Crée une instruction de stockage mémoire avec registre + offset
    pub fn create_store_reg_offset(opcode: Opcode, reg_src: u8, reg_base: u8, offset: i8) -> Self {
        // let fmt = InstructionFormat::reg_reg_imm8(); // (Register, RegisterOffset, None)?
        let fmt = InstructionFormat::reg_regoff(); // (Register, RegisterOffset, None)?

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


    /// Helper pour la creation d'instruction CALL avec adresse relative
    pub fn create_call_relative(from_addr: u32, target_addr: u32) -> Self {
        let temp_instr = Self::new(
            Opcode::Call,
            InstructionFormat::call(),
            vec![0, 0, 0, 0], // Placeholder pour 4 bytes d'offset
        );
        let instr_size = temp_instr.total_size() as u32;

        // Calculer l'offset relatif : target - (current + instruction_size)
        let offset = calculate_branch_offset(from_addr, target_addr, instr_size);

        println!("DEBUG: create_call_relative - from=0x{:X}, to=0x{:X}, instr_size={}, offset={}",
                 from_addr, target_addr, instr_size, offset);

        Self::new(
            Opcode::Call,
            InstructionFormat::call(),
            offset.to_le_bytes().to_vec(),
        )
    }

    /// Helper pour la creation d'instruction CALL (compatibilité)
    pub fn create_call(target_addr: u32) -> Self {
        // Pour compatibilité, utiliser adresse absolue convertie en offset depuis 0
        let offset = target_addr as i32;
        let fmt = InstructionFormat::call();
        Self::new(Opcode::Call, fmt, offset.to_le_bytes().to_vec())
    }

    pub fn create_return() -> Self {
        let fmt = InstructionFormat::ret();
        Self::new(Opcode::Ret, fmt, Vec::new())
    }

    pub fn create_push_register(reg: u8) -> Self{
        let fmt = InstructionFormat::push_reg();
        Self::new(Opcode::Push,fmt, vec![reg & 0x0F])

    }

    pub fn create_push_immediate8(reg:u8 ,imm8:u8) -> Self{
        let fmt = InstructionFormat::push_immediate8();
        Self::new(Opcode::Push, fmt, vec![reg,imm8])
    }

    pub fn create_pop_register(reg: u8) -> Self {
        let fmt = InstructionFormat::pop_reg();
        Self::new(Opcode::Pop, fmt, vec![reg & 0x0F])
    }

    pub fn create_pop_immediate8(reg:u8,imm8:u8) -> Self {
        let fmt = InstructionFormat::pop_immediate8();
        Self::new(Opcode::Pop, fmt, vec![reg, imm8])
    }
}

pub fn calculate_branch_offset(from_addr:u32,to_addr:u32,instr_size:u32) -> i32{
    // l'offset est calcule depuis l'adresse de l'instruction suivante
    let next_pc = from_addr + instr_size;
    (to_addr as i32) - (next_pc as i32)
}


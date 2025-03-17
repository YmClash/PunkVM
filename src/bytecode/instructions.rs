//src/bytecode/instructions.rs


use crate::bytecode::decode_errors::DecodeError;
use crate::bytecode::format::{ArgType, InstructionFormat};
use crate::bytecode::opcodes::Opcode;

/// Represente le type de taille d'instruction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeType{
    Compact,    // Taille sur 1 byte
    Extended,   // Taille sur 3 bytes      0xFF + 2 bytes
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

        println!("DEBUG: new() => overhead={}, args.len()={}, potential_size={}, needed_if_compact={}, size_type={:?}",
                 overhead, args.len(), potential_size, needed_if_compact, size_type
        );
        Self {
            opcode,
            format,
            size_type,
            args,
        }
    }

    /// Calcule la taille totale de l'instruction en bytes
    pub fn total_size(&self) -> usize{
       let overhead = 1 + 2; // opcode + format
        let size_field_size = match self.size_type{
            SizeType::Compact => 1,
            SizeType::Extended => 3,     // 3 octets: 0xFF (marqueur) + 2 octets de taille
        };

        println!("DEBUG: total_size => overhead=3, size_field_size={}, args.len()={}, => total={} ",
                 match self.size_type { SizeType::Compact=>1, SizeType::Extended=>3},
                 self.args.len(),
                 overhead + size_field_size + self.args.len()
        );

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
            },
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

        let opcode = Opcode::from_u8(bytes[0])
            .ok_or(DecodeError::InvalidOpcode(bytes[0]))?;

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
            if bytes.len() < 6 { // Minimum 5 octets: opcode, format, marker, size_lo, size_hi
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

        let total_header_size = 1 + 2 +  size_field_size; // opcode(1), format(2), champ taille (1 ou 3)
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
    pub fn get_arg3_value(&self) -> Result<ArgValue,DecodeError> {
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
                let reg = self.args[offset] ;
                Ok(ArgValue::Register(reg & 0x0F))
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
    pub fn create_reg_reg_offset(opcode: Opcode, reg_src: u8, reg_base: u8, offset: i8) -> Self{
        let fmt = InstructionFormat::reg_reg_imm8(); // (Register, RegisterOffset, None)?
        let mut args = vec![reg_src & 0x0F, reg_base & 0x0F, offset as u8];
        // let mut args = Vec::with_capacity(3);
        args.push(reg_src & 0x0F);
        // args.push(reg_base);
        args.push(reg_base & 0x0F);
        // args.push(reg_src);
        args.push(offset as u8);

        Self::new(opcode, fmt, args)

    }

    /// ici on a les Branch instructions  akaa Jump instructions ou bien saut conditionnel
    // pub fn create_branch(opcode: Opcode, reg: u8, offset: i32) -> Self {
    //     // Crée une instruction de saut conditionnel
    //     let fmt = InstructionFormat::branch(); // (Register, RelativeAddr, None)?
    //     let mut args = vec![reg & 0x0F];
    //     args.extend_from_slice(&offset.to_le_bytes()[..4]); // 4 octets pour l'adresse relative
    //     Self::new(opcode, fmt, args)
    // }

    pub fn create_jump(offset: i32) -> Self {
        // Crée une instruction de saut inconditionnel
        let fmt = InstructionFormat::jump();
        let mut args = Vec::new();
        // Encodage sur 4 octets de l’offset relatif (little-endian)
        args.extend_from_slice(&offset.to_le_bytes());
        Self::new(Opcode::Jmp, fmt, args)
    }

    pub fn create_jump_if(offset: i32) -> Self {
        // Crée une instruction de saut conditionnel
        let fmt = InstructionFormat::jumpif();
        let mut args = Vec::new();
        // Encodage sur 4 octets de l’offset relatif (little-endian)
        args.extend_from_slice(&offset.to_le_bytes());
        Self::new(Opcode::JmpIf, fmt, args)
    }

    pub fn create_jump_if_not(offset: i32) -> Self {
        // Crée une instruction de saut conditionnel
        let fmt = InstructionFormat::jump_if_not();
        let mut args = Vec::new();
        // Encodage sur 4 octets de l’offset relatif (little-endian)
        args.extend_from_slice(&offset.to_le_bytes());
        Self::new(Opcode::JmpIfNot, fmt, args)
    }

    pub fn create_jump_if_equal(offset: i32) -> Self {
        // Crée une instruction de saut conditionnel
        let fmt = InstructionFormat::jump_if_equal();
        let mut args = Vec::new();
        // Encodage sur 4 octets de l’offset relatif (little-endian)
        args.extend_from_slice(&offset.to_le_bytes());
        Self::new(Opcode::JmpIfEqual, fmt, args)
    }

    pub fn create_jump_if_not_equal(offset: i32) -> Self {
        // Crée une instruction de saut conditionnel
        let fmt = InstructionFormat::jump_if_notequal();
        let mut args = Vec::new();
        // Encodage sur 4 octets de l’offset relatif (little-endian)
        args.extend_from_slice(&offset.to_le_bytes());
        Self::new(Opcode::JmpIfNotEqual, fmt, args)
    }

    pub fn create_jump_if_greater(offset: i32) -> Self {
        // Crée une instruction de saut conditionnel
        let fmt = InstructionFormat::jump_if_greater();
        let mut args = Vec::new();
        // Encodage sur 4 octets de l’offset relatif (little-endian)
        args.extend_from_slice(&offset.to_le_bytes());
        Self::new(Opcode::JumpIfGreater, fmt, args)
    }

    pub fn create_jump_if_greater_equal(offset: i32) -> Self {
        // Crée une instruction de saut conditionnel
        let fmt = InstructionFormat::jump_if_greaterequal();
        let mut args = Vec::new();
        // Encodage sur 4 octets de l’offset relatif (little-endian)
        args.extend_from_slice(&offset.to_le_bytes());
        Self::new(Opcode::JumpIfGreaterEqual, fmt, args)
    }

    pub fn create_jump_if_less(offset: i32) -> Self {
        // Crée une instruction de saut conditionnel
        let fmt = InstructionFormat::jump_if_less();
        let mut args = Vec::new();
        // Encodage sur 4 octets de l’offset relatif (little-endian)
        args.extend_from_slice(&offset.to_le_bytes());
        Self::new(Opcode::JumpIfLess, fmt, args)
    }

    pub fn create_jump_if_less_equal(offset: i32) -> Self {
        // Crée une instruction de saut conditionnel
        let fmt = InstructionFormat::jump_if_lessequal();
        let mut args = Vec::new();
        // Encodage sur 4 octets de l’offset relatif (little-endian)
        args.extend_from_slice(&offset.to_le_bytes());
        Self::new(Opcode::JumpIfLessEqual, fmt, args)
    }

    pub fn create_jump_above(offset: i32) -> Self {
        // Crée une instruction de saut conditionnel
        let fmt = InstructionFormat::jump_if_above();
        let mut args = Vec::new();
        // Encodage sur 4 octets de l’offset relatif (little-endian)
        args.extend_from_slice(&offset.to_le_bytes());
        Self::new(Opcode::JumpIfAbove, fmt, args)
    }

    pub fn create_jump_above_equal(offset: i32) -> Self {
        // Crée une instruction de saut conditionnel
        let fmt = InstructionFormat::jump_if_aboveequal();
        let mut args = Vec::new();
        // Encodage sur 4 octets de l’offset relatif (little-endian)
        args.extend_from_slice(&offset.to_le_bytes());
        Self::new(Opcode::JumpIfAboveEqual, fmt, args)
    }

    pub fn create_jump_below(offset: i32) -> Self {
        // Crée une instruction de saut conditionnel
        let fmt = InstructionFormat::jump_if_below();
        let mut args = Vec::new();
        // Encodage sur 4 octets de l’offset relatif (little-endian)
        args.extend_from_slice(&offset.to_le_bytes());
        Self::new(Opcode::JumpIfBelow, fmt, args)
    }

    pub fn create_jump_below_equal(offset: i32) -> Self {
        // Crée une instruction de saut conditionnel
        let fmt = InstructionFormat::jump_if_belowequal();
        let mut args = Vec::new();
        // Encodage sur 4 octets de l’offset relatif (little-endian)
        args.extend_from_slice(&offset.to_le_bytes());
        Self::new(Opcode::JumpIfBelowEqual, fmt, args)
    }

    pub fn create_jump_not_zero(offset: i32) -> Self {
        // Crée une instruction de saut conditionnel
        let fmt = InstructionFormat::jump_if_not_zero();
        let mut args = Vec::new();
        // Encodage sur 4 octets de l’offset relatif (little-endian)
        args.extend_from_slice(&offset.to_le_bytes());
        Self::new(Opcode::JumpIfNotZero, fmt, args)
    }

    pub fn create_jump_zero(offset: i32) -> Self {
        // Crée une instruction de saut conditionnel
        let fmt = InstructionFormat::jump_if_zero();
        let mut args = Vec::new();
        // Encodage sur 4 octets de l’offset relatif (little-endian)
        args.extend_from_slice(&offset.to_le_bytes());
        Self::new(Opcode::JumpIfZero, fmt, args)
    }

    pub fn create_jump_if_overflow(offset: i32) -> Self {
        // Crée une instruction de saut conditionnel
        let fmt = InstructionFormat::jump_if_overflow();
        let mut args = Vec::new();
        // Encodage sur 4 octets de l’offset relatif (little-endian)
        args.extend_from_slice(&offset.to_le_bytes());
        Self::new(Opcode::JumpIfOverflow, fmt, args)
    }

    pub fn create_jump_if_not_overflow(offset: i32) -> Self {
        // Crée une instruction de saut conditionnel
        let fmt = InstructionFormat::jump_if_not_overflow();
        let mut args = Vec::new();
        // Encodage sur 4 octets de l’offset relatif (little-endian)
        args.extend_from_slice(&offset.to_le_bytes());
        Self::new(Opcode::JumpIfNotOverflow, fmt, args)
    }

    pub fn create_jump_if_positive(offset: i32) -> Self {
        // Crée une instruction de saut conditionnel
        let fmt = InstructionFormat::jump_if_positive();
        let mut args = Vec::new();
        // Encodage sur 4 octets de l’offset relatif (little-endian)
        args.extend_from_slice(&offset.to_le_bytes());
        Self::new(Opcode::JumpIfPositive, fmt, args)
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
            vec![0x03, 0x05]  // rd=3, rs1=5
        );

        assert_eq!(instr.opcode, Opcode::Add);
        assert_eq!(instr.format.arg1_type, ArgType::Register);
        assert_eq!(instr.format.arg2_type, ArgType::Register);
        assert_eq!(instr.format.arg3_type, ArgType::None);
        assert_eq!(instr.size_type, SizeType::Compact);
        assert_eq!(instr.args, vec![0x03, 0x05]);
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
        let large_args = vec![0; 254];  // Suffisant pour dépasser la limite de 255 octets

        // Création de l'instruction avec un format simple
        let instr = Instruction::new(Opcode::Add, InstructionFormat::reg_reg(), large_args);

        // Vérifier qu'elle est bien en mode Extended
        assert_eq!(instr.size_type, SizeType::Extended);

        // Encoder l'instruction
        let encoded = instr.encode();

        // Vérifier que l'encodage est correct
        assert_eq!(encoded[3], 0xFF);  // Marqueur du format Extended

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
        assert_eq!(args_size, 3);  // 1 octet par registre

        let format2 = InstructionFormat::reg_imm8();
        let args_size2 = format2.args_size();
        assert_eq!(args_size2, 2);  // 1 octet pour registre + 1 octet pour immédiat
    }


    #[test]
    fn test_jump_instructions() {
        // Test de création d'instruction de saut
        let offset = 42;
        let jump_instr = Instruction::create_jump(offset);
        assert_eq!(jump_instr.opcode, Opcode::Jmp);
        assert_eq!(jump_instr.args.len(), 4); // 4 octets pour l'offset

        // Test de création d'instruction de saut conditionnel
        let jump_if_instr = Instruction::create_jump_if(offset);
        assert_eq!(jump_if_instr.opcode, Opcode::JmpIf);
        assert_eq!(jump_if_instr.args.len(), 4); // 4 octets pour l'offset
    }

    #[test]
    fn test_jump_if() {
        // Test de création d'instruction de saut conditionnel
        let offset = 42;
        let jump_if_instr = Instruction::create_jump_if(offset);
        assert_eq!(jump_if_instr.opcode, Opcode::JmpIf);
        assert_eq!(jump_if_instr.args.len(), 4); // 4 octets pour l'offset
    }

    #[test]
    fn test_jump_if_not() {
        // Test de création d'instruction de saut conditionnel
        let offset = 42;
        let jump_if_not_instr = Instruction::create_jump_if_not(offset);
        assert_eq!(jump_if_not_instr.opcode, Opcode::JmpIfNot);
        assert_eq!(jump_if_not_instr.args.len(), 4); // 4 octets pour l'offset
    }

    #[test]
    fn test_jump_if_equal() {
        // Test de création d'instruction de saut conditionnel
        let offset = 42;
        let jump_if_equal_instr = Instruction::create_jump_if_equal(offset);
        assert_eq!(jump_if_equal_instr.opcode, Opcode::JmpIfEqual);
        assert_eq!(jump_if_equal_instr.args.len(), 4); // 4 octets pour l'offset
    }

    #[test]
    fn test_jump_if_not_equal() {
        // Test de création d'instruction de saut conditionnel
        let offset = 42;
        let jump_if_not_equal_instr = Instruction::create_jump_if_not_equal(offset);
        assert_eq!(jump_if_not_equal_instr.opcode, Opcode::JmpIfNotEqual);
        assert_eq!(jump_if_not_equal_instr.args.len(), 4); // 4 octets pour l'offset
    }

    #[test]
    fn test_jump_if_greater() {
        // Test de création d'instruction de saut conditionnel
        let offset = 42;
        let jump_if_greater_instr = Instruction::create_jump_if_greater(offset);
        assert_eq!(jump_if_greater_instr.opcode, Opcode::JumpIfGreater);
        assert_eq!(jump_if_greater_instr.args.len(), 4); // 4 octets pour l'offset
    }

    #[test]
    fn test_jump_if_greater_equal() {
        // Test de création d'instruction de saut conditionnel
        let offset = 42;
        let jump_if_greater_equal_instr = Instruction::create_jump_if_greater_equal(offset);
        assert_eq!(jump_if_greater_equal_instr.opcode, Opcode::JumpIfGreaterEqual);
        assert_eq!(jump_if_greater_equal_instr.args.len(), 4); // 4 octets pour l'offset
    }

    #[test]
    fn test_jump_if_less() {
        // Test de création d'instruction de saut conditionnel
        let offset = 42;
        let jump_if_less_instr = Instruction::create_jump_if_less(offset);
        assert_eq!(jump_if_less_instr.opcode, Opcode::JumpIfLess);
        assert_eq!(jump_if_less_instr.args.len(), 4); // 4 octets pour l'offset
    }

    #[test]
    fn test_jump_if_less_equal() {
        // Test de création d'instruction de saut conditionnel
        let offset = 42;
        let jump_if_less_equal_instr = Instruction::create_jump_if_less_equal(offset);
        assert_eq!(jump_if_less_equal_instr.opcode, Opcode::JumpIfLessEqual);
        assert_eq!(jump_if_less_equal_instr.args.len(), 4); // 4 octets pour l'offset
    }

    #[test]
    fn test_jump_above() {
        // Test de création d'instruction de saut conditionnel
        let offset = 42;
        let jump_above_instr = Instruction::create_jump_above(offset);
        assert_eq!(jump_above_instr.opcode, Opcode::JumpIfAbove);
        assert_eq!(jump_above_instr.args.len(), 4); // 4 octets pour l'offset
    }
    #[test]
    fn test_jump_above_equal() {
        // Test de création d'instruction de saut conditionnel
        let offset = 42;
        let jump_above_equal_instr = Instruction::create_jump_above_equal(offset);
        assert_eq!(jump_above_equal_instr.opcode, Opcode::JumpIfAboveEqual);
        assert_eq!(jump_above_equal_instr.args.len(), 4); // 4 octets pour l'offset
    }

    #[test]
    fn test_jump_below() {
        // Test de création d'instruction de saut conditionnel
        let offset = 42;
        let jump_below_instr = Instruction::create_jump_below(offset);
        assert_eq!(jump_below_instr.opcode, Opcode::JumpIfBelow);
        assert_eq!(jump_below_instr.args.len(), 4); // 4 octets pour l'offset
    }

    #[test]
    fn test_jump_below_equal() {
        // Test de création d'instruction de saut conditionnel
        let offset = 42;
        let jump_below_equal_instr = Instruction::create_jump_below_equal(offset);
        assert_eq!(jump_below_equal_instr.opcode, Opcode::JumpIfBelowEqual);
        assert_eq!(jump_below_equal_instr.args.len(), 4); // 4 octets pour l'offset
    }

    #[test]
    fn test_jump_zero() {
        // Test de création d'instruction de saut conditionnel
        let offset = 42;
        let jump_zero_instr = Instruction::create_jump_zero(offset);
        assert_eq!(jump_zero_instr.opcode, Opcode::JumpIfZero);
        assert_eq!(jump_zero_instr.args.len(), 4); // 4 octets pour l'offset
    }

    #[test]
    fn test_jump_not_zero() {
        // Test de création d'instruction de saut conditionnel
        let offset = 42;
        let jump_not_zero_instr = Instruction::create_jump_not_zero(offset);
        assert_eq!(jump_not_zero_instr.opcode, Opcode::JumpIfNotZero);
        assert_eq!(jump_not_zero_instr.args.len(), 4); // 4 octets pour l'offset
    }

    #[test]
    fn test_jump_if_overflow() {
        // Test de création d'instruction de saut conditionnel
        let offset = 42;
        let jump_if_overflow_instr = Instruction::create_jump_if_overflow(offset);
        assert_eq!(jump_if_overflow_instr.opcode, Opcode::JumpIfOverflow);
        assert_eq!(jump_if_overflow_instr.args.len(), 4); // 4 octets pour l'offset
    }

    #[test]
    fn test_jump_if_not_overflow() {
        // Test de création d'instruction de saut conditionnel
        let offset = 42;
        let jump_if_not_overflow_instr = Instruction::create_jump_if_not_overflow(offset);
        assert_eq!(jump_if_not_overflow_instr.opcode, Opcode::JumpIfNotOverflow);
        assert_eq!(jump_if_not_overflow_instr.args.len(), 4); // 4 octets pour l'offset
    }

    #[test]
    fn test_jump_if_positive() {
        // Test de création d'instruction de saut conditionnel
        let offset = 42;
        let jump_if_positive_instr = Instruction::create_jump_if_positive(offset);
        assert_eq!(jump_if_positive_instr.opcode, Opcode::JumpIfPositive);
        assert_eq!(jump_if_positive_instr.args.len(), 4); // 4 octets pour l'offset
    }


}
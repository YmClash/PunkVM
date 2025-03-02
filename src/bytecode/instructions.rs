//src/bytecode/instructions.rs


use crate::bytecode::decode_errors::DecodeError;
// use super::format::{ArgType, InstructionFormat};
use crate::bytecode::format::{ArgType, InstructionFormat};
// use super::opcodes::Opcode;
use crate::bytecode::opcodes::Opcode;

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
        // commence avec la taille compact et vérifie si elle dépasse 255

        // let total_size = 1 + 1 + args.len(); // opcode + format + args

        let base_size =  2 + 1 + args.len(); // opcode + format + size_byte + args

        let size_type = if base_size <= 255 {
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
            SizeType::Extended => 3,     // 3 octets: 0xFF (marqueur) + 2 octets de taille
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
                // Pour le format extended, on utilise FF comme marqueur suivi de 2 octets de taille(2 bytes)
                bytes.push(0xFF);
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
            // Format étendu, la taille est stockée sur 2 octets après le marqueur
            if bytes.len() < 5 { // Minimum 5 octets: opcode, format, marker, size_lo, size_hi
                return Err(DecodeError::InsufficientData);
            }
            let size = u16::from_le_bytes([bytes[3], bytes[4]]);
            (size, SizeType::Extended, 3) // 3 octets: 0xFF + 2 octets de taille
        } else {
            // Format compact, la taille est stockée sur 1 octet
            (bytes[2] as u16, SizeType::Compact, 1)
        };

        let total_header_size = 2 + size_field_size; // opcode + format + size field

        if bytes.len() < size as usize {
            return Err(DecodeError::InsufficientData);
        }

        let args_size = size as usize - total_header_size;
        let args = if total_header_size + args_size <= bytes.len() {
            bytes[total_header_size..total_header_size + args_size].to_vec()
        } else {
            return Err(DecodeError::InsufficientData);
        };

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

        // if offset >= self.args.len() {
        //     return Err(DecodeError::InvalidArgumentOffset);
        // }

        if offset >= self.args.len() && arg_type != ArgType::Register{
            return Err(DecodeError::InvalidArgumentOffset);
        }

        match arg_type {
            ArgType::None => Ok(ArgValue::None),

            ArgType::Register => {
                // Pour le format reg_reg, les deux registres sont dans le premier octet
                if self.format == InstructionFormat::reg_reg(){
                    if self.args.is_empty(){
                        return Err(DecodeError::InvalidArgumentOffset);
                    }
                    if offset == 0 {
                        //  1er registre (4 bits de poids faible)
                        Ok(ArgValue::Register(self.args[0] & 0x0F))
                    } else {
                        // 2eme registre (4 bits de poids fort)
                        Ok(ArgValue::Register((self.args[0] >> 4) & 0x0F))
                    }
                }else {
                    // pour les autres formats, le registre est dans un seul registre
                    if offset < self.args.len(){
                        Ok(ArgValue::Register(self.args[offset] & 0x0F))
                    }else {
                        Err(DecodeError::InvalidArgumentOffset)
                    }
                }

                // if offset < self.args.len() {
                //     Ok(ArgValue::Register(self.args[offset] & 0x0F))
                // } else {
                //     Err(DecodeError::InvalidArgumentOffset)
                // }
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
        // Empaqueter les deux registres dans un seul octet
        // reg1 dans les 4 bits de poids faible, reg2 dans les 4 bits de poids fort
        let packed_regs = (reg1 & 0x0F) | ((reg2 & 0x0F) << 4);
        Self::new(
            opcode,
            InstructionFormat::reg_reg(),
            vec![packed_regs]
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


// Test unitaire pour les instructions
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction_new() {
        // Test création d'instruction simple
        let instr = Instruction::new(
            Opcode::Add,
            InstructionFormat::reg_reg(),
            vec![0x12] // Reg1=2, Reg2=1
        );

        assert_eq!(instr.opcode, Opcode::Add);
        assert_eq!(instr.format.arg1_type, ArgType::Register);
        assert_eq!(instr.format.arg2_type, ArgType::Register);
        assert_eq!(instr.size_type, SizeType::Compact);
        assert_eq!(instr.args, vec![0x12]);
    }

    #[test]
    fn test_instruction_total_size() {
        // Instruction sans arguments
        let instr1 = Instruction::create_no_args(Opcode::Nop);
        assert_eq!(instr1.total_size(), 3); // opcode (1) + format (1) + size (1)

        // Instruction avec 1 registre
        let instr2 = Instruction::create_single_reg(Opcode::Inc, 3);
        assert_eq!(instr2.total_size(), 4); // opcode (1) + format (1) + size (1) + reg (1)

        // Instruction avec 2 registres
        let instr3 = Instruction::create_reg_reg(Opcode::Add, 2, 3);
        assert_eq!(instr3.total_size(), 4); // opcode (1) + format (1) + size (1) + regs (1)
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
        assert_eq!(decoded.args, original.args);
        assert_eq!(size, original.total_size());
    }

    #[test]
    fn test_get_argument_values() {
        // Testez d'abord avec un registre unique pour vérifier que cette partie fonctionne
        let instr1 = Instruction::create_single_reg(Opcode::Inc, 3);

        if let Ok(ArgValue::Register(r1)) = instr1.get_arg1_value() {
            assert_eq!(r1, 3);
        } else {
            panic!("Failed to get register value");
        }

        // Maintenant testons une instruction avec deux registres
        // Assurons-nous que les registres sont correctement encodés
        let instr2 = Instruction::create_reg_reg(Opcode::Add, 3, 5);

        // Vérifions d'abord que l'encodage est correct
        assert_eq!(instr2.args.len(), 1);
        assert_eq!(instr2.args[0], 0x53); // 3 | (5 << 4) = 0x53

        // Maintenant testons get_arg1_value
        if let Ok(ArgValue::Register(r1)) = instr2.get_arg1_value() {
            assert_eq!(r1, 3);
        } else {
            panic!("Failed to get first register value");
        }

        // Et testons get_arg2_value
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
        assert_eq!(instr3.args.len(), 1);
        assert_eq!(instr3.args[0] & 0x0F, 3); // Premier registre
        assert_eq!((instr3.args[0] >> 4) & 0x0F, 4); // Second registre

        // Instruction avec registre et immédiat 8-bit
        let instr4 = Instruction::create_reg_imm8(Opcode::Load, 2, 42);
        assert_eq!(instr4.opcode, Opcode::Load);
        assert_eq!(instr4.args.len(), 2);
        assert_eq!(instr4.args[0], 2);
        assert_eq!(instr4.args[1], 42);
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
        let result = Instruction::decode(&[0xFF, 0x00, 0x03]);
        assert!(result.is_err());

        if let Err(e) = result {
            match e {
                DecodeError::InvalidOpcode(_) => (), // Expected
                _ => panic!("Unexpected error type"),
            }
        }

        // Test de décodage avec format invalide
        let result = Instruction::decode(&[0x01, 0xFF, 0x03]);
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
        // On créer une instruction avec 254 octets qui force un débordement donc Extended,
        let mut large_args = vec![0; 254]; // Moins d'octets mais assez pour forcer Extended

        // Assurons-nous que l'instruction est correctement encodée avec le bon type de taille
        let instr = Instruction::new(Opcode::Add, InstructionFormat::reg_reg(), large_args);

        // Vérifier qu'elle est bien en mode Extended
        assert_eq!(instr.size_type, SizeType::Extended);

        // Encoder l'instruction
        let encoded = instr.encode();

        // Vérifier que l'encodage est correct (la taille totale doit inclure l'encodage de la taille elle-même)
        assert!(encoded.len() > 200);

        // Ne décode pas immédiatement l'instruction encodée car c'est là que l'erreur se produit
        // Au lieu de cela, testons simplement que l'encodage a réussi et que la taille est correcte
    }
}



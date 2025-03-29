// src/bytecode/format.rs

///Type d'argument pour les instructions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ArgType {
    None = 0x0,
    Register = 0x1,     //registre general 4 bits
    RegisterExt = 0x2,  //registre general 8 bits
    Immediate8 = 0x3,   //valeur immediate 8 bits
    Immediate16 = 0x4,  //valeur immediate 16 bits
    Immediate32 = 0x5,  //valeur immediate 32 bits
    Immediate64 = 0x6,  //valeur immediate 64 bits
    RelativeAddr = 0x7, // Adresse relative (offset par rapport au PC)
    AbsoluteAddr = 0x8, // Adresse absolue
    RegisterOffset = 0x9, // Registre + offset (pour accès mémoire indexé)
                        // Flag = 0xA, // 4 bits pour les flags (ex: ZF, SF, OF, CF)
                        // 0xA-0xF réservés pour extensions futures
}
impl ArgType {
    /// Convertit un u8 en ArgType 4 bits
    pub fn from_u8(value: u8) -> Option<Self> {
        match value & 0x0F {
            0x0 => Some(Self::None),
            0x1 => Some(Self::Register),
            0x2 => Some(Self::RegisterExt),
            0x3 => Some(Self::Immediate8),
            0x4 => Some(Self::Immediate16),
            0x5 => Some(Self::Immediate32),
            0x6 => Some(Self::Immediate64),
            0x7 => Some(Self::RelativeAddr),
            0x8 => Some(Self::AbsoluteAddr),
            0x9 => Some(Self::RegisterOffset),
            // 0xA => Some(Self::Flag),
            _ => None,
        }
    }

    /// Retourne la taille en bytes d'un type d'argument
    pub fn size(&self) -> usize {
        match self {
            Self::None => 0,
            Self::Register => 1, // 4 bits (moitié d'un byte), mais on aligne sur le byte
            Self::RegisterExt => 1,
            Self::Immediate8 => 1,
            Self::Immediate16 => 2,
            Self::Immediate32 => 4,
            Self::Immediate64 => 8,
            Self::RelativeAddr => 4, // Typiquement 32 bits pour un offset
            Self::AbsoluteAddr => 4, // Pourrait être 8 sur systèmes 64 bits
            Self::RegisterOffset => 2, // Registre (1B) + offset (1B)
                                      // Self::Flag => 1, // 4 bits pour les flags, mais on aligne sur le byte
        }
    }
}

/// Format d'une instruction - definit les types d'arguments
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstructionFormat {
    pub arg1_type: ArgType,
    pub arg2_type: ArgType,
    pub arg3_type: ArgType,
}

impl InstructionFormat {
    /// Crée un nouveau format d'instruction
    pub fn new(arg1_type: ArgType, arg2_type: ArgType, arg3_type: ArgType) -> Self {
        Self {
            arg1_type,
            arg2_type,
            arg3_type,
        }
    }

    /// Encode le format dans un byte
    pub fn encode(&self) -> [u8; 2] {
        // Sur 2 octets (16 bit ) : 4 bit par ArgType
        // arg1: bits [11:8], arg2: bits [7:4], arg3: bits [3:0]
        let bits = ((self.arg1_type as u16 & 0xF) << 8)
            | ((self.arg2_type as u16 & 0xF) << 4)
            | ((self.arg3_type as u16 & 0xF) << 0);
        bits.to_le_bytes()
    }

    /// Décode un byte en format d'instruction
    pub fn decode(bytes: [u8; 2]) -> Option<Self> {
        let bits = u16::from_le_bytes(bytes);
        let arg1 = (bits >> 8) & 0x0F; // bits [11:8] Arg1
        let arg2 = (bits >> 4) & 0x0F; // bits [7:4] Arg2
        let arg3 = (bits >> 0) & 0x0F; // bits [3:0] Arg3
        Some(Self::new(
            ArgType::from_u8(arg1 as u8)?,
            ArgType::from_u8(arg2 as u8)?,
            ArgType::from_u8(arg3 as u8)?,
        ))
    }

    /// Calcule la taille totale des arguments (en bytes)
    pub fn args_size(&self) -> usize {
        self.arg1_type.size() + self.arg2_type.size() + self.arg3_type.size()
    }

    /// Formats prédéfinis courants pour les instructions
    pub fn reg_reg_reg() -> Self {
        //reg, reg , reg
        Self::new(ArgType::Register, ArgType::Register, ArgType::Register)
    }

    pub fn reg_reg_imm8() -> Self {
        Self::new(ArgType::Register, ArgType::Register, ArgType::Immediate8)
    }

    pub fn reg_reg_imm16() -> Self {
        Self::new(ArgType::Register, ArgType::Register, ArgType::Immediate16)
    }

    pub fn reg_reg_imm32() -> Self {
        Self::new(ArgType::Register, ArgType::Register, ArgType::Immediate32)
    }

    pub fn reg_reg_addr() -> Self {
        Self::new(ArgType::Register, ArgType::Register, ArgType::AbsoluteAddr)
    }

    pub fn reg_reg_off() -> Self {
        Self::new(
            ArgType::Register,
            ArgType::Register,
            ArgType::RegisterOffset,
        )
    }

    pub fn addr_only() -> Self {
        Self::new(ArgType::None, ArgType::None, ArgType::AbsoluteAddr)
    }

    pub fn single_reg() -> Self {
        Self::new(ArgType::Register, ArgType::None, ArgType::None)
    }

    pub fn double_reg() -> Self {
        // Interprétation possible : (rd, rs1, rs2) => ou si tu veux 2 regs => 3ᵉ ArgType= None
        // => On fait 3 regs => c'est plus un triple reg, on adaptes si besoin
        Self::new(ArgType::Register, ArgType::Register, ArgType::None)
    }
    pub fn reg_reg() -> Self {
        // Interprétation possible : (rd, rs1, rs2) => ou si tu veux 2 regs => 3ᵉ ArgType= None
        // => On fait 3 regs => c'est plus un triple reg, on adaptes si besoin
        Self::new(ArgType::Register, ArgType::Register, ArgType::None)
    }

    pub fn reg_imm8() -> Self {
        Self::new(ArgType::Register, ArgType::Immediate8, ArgType::None)
    }
    pub fn reg_imm16() -> Self {
        Self::new(ArgType::Register, ArgType::Immediate16, ArgType::None)
    }
    pub fn reg_regoff() -> Self {
        Self::new(ArgType::Register, ArgType::RegisterOffset, ArgType::None)
    }

    pub fn no_args() -> Self {
        Self::new(ArgType::None, ArgType::None, ArgType::None)
    }

    // Format pout les instructions de saut
    pub fn jump() -> Self {
        Self::new(ArgType::None, ArgType::RelativeAddr, ArgType::None)
        // Self {
        //     arg1_type: ArgType::None,
        //     arg2_type: ArgType::Immediate32, // offset relatif sur 32 bits
        //     arg3_type: ArgType::None,
        // }
    }

    // pub fn jump() -> Self {
    //     Self::new(ArgType::None, ArgType::Flag, ArgType::None)
    //     // Self {
    //     //     arg1_type: ArgType::None,
    //     //     arg2_type: ArgType::Immediate32, // offset relatif sur 32 bits
    //     //     arg3_type: ArgType::None,
    //     // }
    // }

    pub fn jumpif() -> Self {
        Self::new(ArgType::None, ArgType::RelativeAddr, ArgType::None)
    }

    pub fn jump_if_not() -> Self {
        Self::new(ArgType::None, ArgType::RelativeAddr, ArgType::None)
    }

    pub fn jump_if_equal() -> Self {
        Self::new(ArgType::None, ArgType::RelativeAddr, ArgType::None)
    }
    pub fn jump_if_notequal() -> Self {
        Self::new(ArgType::None, ArgType::RelativeAddr, ArgType::None)
    }

    pub fn jump_if_greater() -> Self {
        Self::new(ArgType::None, ArgType::RelativeAddr, ArgType::None)
    }

    pub fn jump_if_greaterequal() -> Self {
        Self::new(ArgType::None, ArgType::RelativeAddr, ArgType::None)
    }

    pub fn jump_if_less() -> Self {
        Self::new(ArgType::None, ArgType::RelativeAddr, ArgType::None)
    }
    pub fn jump_if_lessequal() -> Self {
        Self::new(ArgType::None, ArgType::RelativeAddr, ArgType::None)
    }
    pub fn jump_if_above() -> Self {
        Self::new(ArgType::None, ArgType::RelativeAddr, ArgType::None)
    }

    pub fn jump_if_aboveequal() -> Self {
        Self::new(ArgType::None, ArgType::RelativeAddr, ArgType::None)
    }

    pub fn jump_if_below() -> Self {
        Self::new(ArgType::None, ArgType::RelativeAddr, ArgType::None)
    }

    pub fn jump_if_belowequal() -> Self {
        Self::new(ArgType::None, ArgType::RelativeAddr, ArgType::None)
    }

    pub fn jump_if_overflow() -> Self {
        Self::new(ArgType::None, ArgType::RelativeAddr, ArgType::None)
    }

    pub fn jump_if_not_overflow() -> Self {
        Self::new(ArgType::None, ArgType::RelativeAddr, ArgType::None)
    }

    pub fn jump_if_zero() -> Self {
        Self::new(ArgType::None, ArgType::RelativeAddr, ArgType::None)
    }
    pub fn jump_if_not_zero() -> Self {
        Self::new(ArgType::None, ArgType::RelativeAddr, ArgType::None)
    }

    pub fn jump_if_positive() -> Self {
        Self::new(ArgType::None, ArgType::RelativeAddr, ArgType::None)
    }

    pub fn jump_if_negative() -> Self {
        Self::new(ArgType::None, ArgType::RelativeAddr, ArgType::None)
    }

    //Format pour les instructions de type CALL
    pub fn call() -> Self {
        Self::new(ArgType::None, ArgType::RelativeAddr, ArgType::None)
    }

    //Format pour les instructions de type RET
    pub fn ret() -> Self {
        Self::no_args()
    }

    pub fn reg_reg_offset() -> Self {
        Self::new(ArgType::Register, ArgType::RegisterOffset, ArgType::None)
    }
}

// Test unitaire pour les formats d'instruction
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arg_type_from_u8() {
        // Test des conversions valides (4 bits)
        assert_eq!(ArgType::from_u8(0x0), Some(ArgType::None));
        assert_eq!(ArgType::from_u8(0x1), Some(ArgType::Register));
        assert_eq!(ArgType::from_u8(0x5), Some(ArgType::Immediate32));

        // Test avec bits supplémentaires (on ne garde que les 4 bits de poids faible)
        // Ex: 0xF0 => 0x0 => Some(ArgType::None)
        //     0xF1 => 0x1 => Some(ArgType::Register)
        assert_eq!(ArgType::from_u8(0xF0), Some(ArgType::None));
        assert_eq!(ArgType::from_u8(0xF1), Some(ArgType::Register));

        // Test avec une valeur invalide (ex: 0xA => 1010, n'est pas défini)
        // Ici, 0xA => 0xA (bits de poids faible), or c'est pas dans [0..9] => None
        assert_eq!(ArgType::from_u8(0xA), None);
        assert_eq!(ArgType::from_u8(0xAA), None); // par ex. 0xAA => 0xA => None
    }

    #[test]
    fn test_arg_type_size() {
        // Vérifie que chaque ArgType renvoie la bonne taille en octets
        assert_eq!(ArgType::None.size(), 0);
        assert_eq!(ArgType::Register.size(), 1);
        assert_eq!(ArgType::RegisterExt.size(), 1);
        assert_eq!(ArgType::Immediate8.size(), 1);
        assert_eq!(ArgType::Immediate16.size(), 2);
        assert_eq!(ArgType::Immediate32.size(), 4);
        assert_eq!(ArgType::Immediate64.size(), 8);
        assert_eq!(ArgType::RelativeAddr.size(), 4);
        assert_eq!(ArgType::AbsoluteAddr.size(), 4);
        assert_eq!(ArgType::RegisterOffset.size(), 2);
    }

    #[test]
    fn test_instruction_format_new() {
        // Test simple : Reg, Reg, Imm32
        let format =
            InstructionFormat::new(ArgType::Register, ArgType::Register, ArgType::Immediate32);
        assert_eq!(format.arg1_type, ArgType::Register);
        assert_eq!(format.arg2_type, ArgType::Register);
        assert_eq!(format.arg3_type, ArgType::Immediate32);
    }

    #[test]
    fn test_instruction_format_encode_decode_two_args() {
        // Exemple : (ArgType::Register, ArgType::Immediate8, ArgType::None)
        // => on s'attend à ce que arg1=Register (0x1), arg2=Immediate8 (0x3), arg3=None (0x0)

        let format = InstructionFormat::new(ArgType::Register, ArgType::Immediate8, ArgType::None);

        // Encodage : sur 16 bits
        // arg1=1 => bits [11..8], arg2=3 => bits [7..4], arg3=0 => bits [3..0]
        // => bits = 1<<8 + 3<<4 + 0= 0x100 + 0x30 = 0x130 => LE = [0x30, 0x01]
        let encoded = format.encode();
        assert_eq!(
            encoded,
            [0x30, 0x01],
            "Encoding incorrect pour (Reg, Imm8, None)"
        );

        // Décodage
        let decoded = InstructionFormat::decode(encoded).expect("Décodage should not fail");
        assert_eq!(decoded.arg1_type, ArgType::Register);
        assert_eq!(decoded.arg2_type, ArgType::Immediate8);
        assert_eq!(decoded.arg3_type, ArgType::None);
    }

    #[test]
    fn test_instruction_format_encode_decode_three_args() {
        // Ex: ArgType::Register, ArgType::Register, ArgType::Immediate8
        // => arg1=0x1, arg2=0x1, arg3=0x3
        // => bits = (1<<8) + (1<<4) + (3) = 0x100 + 0x10 + 0x3 = 0x113 => LE=[0x13,0x01]
        let format =
            InstructionFormat::new(ArgType::Register, ArgType::Register, ArgType::Immediate8);

        let encoded = format.encode();
        assert_eq!(
            encoded,
            [0x13, 0x01],
            "Encoding incorrect pour (Reg, Reg, Imm8)"
        );

        let decoded = InstructionFormat::decode(encoded).expect("Décodage should succeed");
        assert_eq!(decoded.arg1_type, ArgType::Register);
        assert_eq!(decoded.arg2_type, ArgType::Register);
        assert_eq!(decoded.arg3_type, ArgType::Immediate8);
    }

    #[test]
    fn test_instruction_format_args_size() {
        // (Register, Register, Immediate8) => 1 + 1 + 1 = 3
        let f1 = InstructionFormat::new(ArgType::Register, ArgType::Register, ArgType::Immediate8);
        assert_eq!(f1.args_size(), 3);

        // (None, None, None) => 0
        let f2 = InstructionFormat::new(ArgType::None, ArgType::None, ArgType::None);
        assert_eq!(f2.args_size(), 0);

        // (Register, Immediate16, RegisterOffset) => 1 + 2 + 2 = 5
        let f3 = InstructionFormat::new(
            ArgType::Register,
            ArgType::Immediate16,
            ArgType::RegisterOffset,
        );
        assert_eq!(f3.args_size(), 5);

        // (Immediate64, AbsoluteAddr, None) => 8 + 4 + 0 = 12
        let f4 = InstructionFormat::new(ArgType::Immediate64, ArgType::AbsoluteAddr, ArgType::None);
        assert_eq!(f4.args_size(), 12);
    }

    #[test]
    fn test_instruction_format_predefined() {
        // test de quelques formats prédéfinis (si tu les as adaptés en triple version)
        // Ex: "reg_reg_reg" => (Register, Register, Register)
        let triple_reg = InstructionFormat::reg_reg_reg();
        assert_eq!(triple_reg.arg1_type, ArgType::Register);
        assert_eq!(triple_reg.arg2_type, ArgType::Register);
        assert_eq!(triple_reg.arg3_type, ArgType::Register);

        let reg_reg_imm8 = InstructionFormat::reg_reg_imm8();
        assert_eq!(reg_reg_imm8.arg1_type, ArgType::Register);
        assert_eq!(reg_reg_imm8.arg2_type, ArgType::Register);
        assert_eq!(reg_reg_imm8.arg3_type, ArgType::Immediate8);

        let no_args = InstructionFormat::no_args();
        assert_eq!(no_args.arg1_type, ArgType::None);
        assert_eq!(no_args.arg2_type, ArgType::None);
        assert_eq!(no_args.arg3_type, ArgType::None);
    }

    #[test]
    fn test_instruction_format_invalid_decode() {
        // Suppose qu'on n'a que 9 ArgType valides => 0..9.
        // Testons si bits > 9 => None
        // ex: 0xA => ArgType::None => on out-of-range pour ArgType
        // On va injecter un short array => [0xFF] ?

        // On fait un code 16 bits:
        // par ex. ( arg1=0xA, arg2=0x1, arg3=0x0 ) => bits= (0xA<<8)+(0x1<<4)+(0x0) = 0xA10 => [0x10,0x0A]
        // ArgType::from_u8(0xA) => None => decode => None
        let encoded = [0x10, 0x0A];
        let result = InstructionFormat::decode(encoded);
        assert!(
            result.is_none(),
            "Devrait être None car arg1=0xA => ArgType invalide"
        );

        // Autre test invalid => arg2= 0xA
        // bits = (0x1<<8)+(0xA<<4)+(0x1)= 0x1xx ???
        // On peut juste forcer un code => [0xF0, 0xFF]
        let encoded2 = [0xF0, 0xFF];
        let result2 = InstructionFormat::decode(encoded2);
        assert!(
            result2.is_none(),
            "Décodage devrait échouer si on a un ArgType invalide"
        );
    }

    #[test]
    fn test_jump_format() {
        let fmt = InstructionFormat::jump();
        let encoded = fmt.encode();
        let decoded = InstructionFormat::decode(encoded).unwrap();
        assert_eq!(fmt, decoded);
    }

    #[test]
    fn test_jump_if_format() {
        let fmt = InstructionFormat::jumpif();
        let encoded = fmt.encode();
        let decoded = InstructionFormat::decode(encoded).unwrap();
        assert_eq!(fmt, decoded);
    }

    #[test]
    fn test_jump_if_not_format() {
        let fmt = InstructionFormat::jump_if_not();
        let encoded = fmt.encode();
        let decoded = InstructionFormat::decode(encoded).unwrap();
        assert_eq!(fmt, decoded);
    }

    #[test]
    fn test_jump_if_equal_format() {
        let fmt = InstructionFormat::jump_if_equal();
        let encoded = fmt.encode();
        let decoded = InstructionFormat::decode(encoded).unwrap();
        assert_eq!(fmt, decoded);
    }

    #[test]
    fn test_jump_if_not_equal_format() {
        let fmt = InstructionFormat::jump_if_notequal();
        let encoded = fmt.encode();
        let decoded = InstructionFormat::decode(encoded).unwrap();
        assert_eq!(fmt, decoded);
    }

    #[test]
    fn test_jump_if_greater_format() {
        let fmt = InstructionFormat::jump_if_greater();
        let encoded = fmt.encode();
        let decoded = InstructionFormat::decode(encoded).unwrap();
        assert_eq!(fmt, decoded);
    }

    #[test]
    fn test_jump_if_greater_equal_format() {
        let fmt = InstructionFormat::jump_if_greaterequal();
        let encoded = fmt.encode();
        let decoded = InstructionFormat::decode(encoded).unwrap();
        assert_eq!(fmt, decoded);
    }

    #[test]
    fn test_jump_if_less_format() {
        let fmt = InstructionFormat::jump_if_less();
        let encoded = fmt.encode();
        let decoded = InstructionFormat::decode(encoded).unwrap();
        assert_eq!(fmt, decoded);
    }

    #[test]
    fn test_jump_if_less_equal_format() {
        let fmt = InstructionFormat::jump_if_lessequal();
        let encoded = fmt.encode();
        let decoded = InstructionFormat::decode(encoded).unwrap();
        assert_eq!(fmt, decoded);
    }

    #[test]
    fn test_jump_if_above_format() {
        let fmt = InstructionFormat::jump_if_above();
        let encoded = fmt.encode();
        let decoded = InstructionFormat::decode(encoded).unwrap();
        assert_eq!(fmt, decoded);
    }

    #[test]
    fn test_jump_if_above_equal_format() {
        let fmt = InstructionFormat::jump_if_aboveequal();
        let encoded = fmt.encode();
        let decoded = InstructionFormat::decode(encoded).unwrap();
        assert_eq!(fmt, decoded);
    }

    #[test]
    fn test_jump_if_below_format() {
        let fmt = InstructionFormat::jump_if_below();
        let encoded = fmt.encode();
        let decoded = InstructionFormat::decode(encoded).unwrap();
        assert_eq!(fmt, decoded);
    }

    #[test]
    fn test_jump_if_below_equal_format() {
        let fmt = InstructionFormat::jump_if_belowequal();
        let encoded = fmt.encode();
        let decoded = InstructionFormat::decode(encoded).unwrap();
        assert_eq!(fmt, decoded);
    }

    #[test]
    fn test_jump_if_overflow_format() {
        let fmt = InstructionFormat::jump_if_overflow();
        let encoded = fmt.encode();
        let decoded = InstructionFormat::decode(encoded).unwrap();
        assert_eq!(fmt, decoded);
    }

    #[test]
    fn test_jump_if_not_overflow_format() {
        let fmt = InstructionFormat::jump_if_not_overflow();
        let encoded = fmt.encode();
        let decoded = InstructionFormat::decode(encoded).unwrap();
        assert_eq!(fmt, decoded);
    }

    #[test]
    fn test_jump_if_zero_format() {
        let fmt = InstructionFormat::jump_if_zero();
        let encoded = fmt.encode();
        let decoded = InstructionFormat::decode(encoded).unwrap();
        assert_eq!(fmt, decoded);
    }

    #[test]
    fn test_jump_if_not_zero_format() {
        let fmt = InstructionFormat::jump_if_not_zero();
        let encoded = fmt.encode();
        let decoded = InstructionFormat::decode(encoded).unwrap();
        assert_eq!(fmt, decoded);
    }

    #[test]
    fn test_jump_if_positive_format() {
        let fmt = InstructionFormat::jump_if_positive();
        let encoded = fmt.encode();
        let decoded = InstructionFormat::decode(encoded).unwrap();
        assert_eq!(fmt, decoded);
    }

    #[test]
    fn test_call_format() {
        let fmt = InstructionFormat::call();
        let encoded = fmt.encode();
        let decoded = InstructionFormat::decode(encoded).unwrap();
        assert_eq!(fmt, decoded);
    }

    #[test]
    fn test_ret_format() {
        let fmt = InstructionFormat::ret();
        let encoded = fmt.encode();
        let decoded = InstructionFormat::decode(encoded).unwrap();
        assert_eq!(fmt, decoded);
    }

    #[test]
    fn test_reg_reg_offset_format() {
        let fmt = InstructionFormat::reg_reg_offset();
        let encoded = fmt.encode();
        let decoded = InstructionFormat::decode(encoded).unwrap();
        assert_eq!(fmt, decoded);
    }
}

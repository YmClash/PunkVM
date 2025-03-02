// src/bytecode/format.rs

///Type d'argument pour les instructions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ArgType{
    None = 0x0,
    Register = 0x1,     //registre general 4 bits
    RegisterExt = 0x2,  //registre general 8 bits
    Immediate8 = 0x3,   //valeur immediate 8 bits
    Immediate16 = 0x4,  //valeur immediate 16 bits
    Immediate32 = 0x5,  //valeur immediate 32 bits
    Immediate64 = 0x6,  //valeur immediate 64 bits
    RelativeAddr = 0x7,        // Adresse relative (offset par rapport au PC)
    AbsoluteAddr = 0x8,        // Adresse absolue
    RegisterOffset = 0x9,      // Registre + offset (pour accès mémoire indexé)
    // 0xA-0xF réservés pour extensions futures
}
impl ArgType{
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
        }
    }

}

/// Format d'une instruction - definit les types d'arguments
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstructionFormat {
    pub arg1_type: ArgType,
    pub arg2_type: ArgType,
}


impl InstructionFormat {
    /// Crée un nouveau format d'instruction
    pub fn new(arg1_type: ArgType, arg2_type: ArgType) -> Self {
        Self { arg1_type, arg2_type }
    }

    /// Encode le format dans un byte
    pub fn encode(&self) -> u8 {
        ((self.arg1_type as u8) << 4) | (self.arg2_type as u8)
    }

    /// Décode un byte en format d'instruction
    pub fn decode(value: u8) -> Option<Self> {
        let arg1 = ArgType::from_u8(value >> 4)?;
        let arg2 = ArgType::from_u8(value & 0x0F)?;
        Some(Self::new(arg1, arg2))
    }

    /// Calcule la taille totale des arguments (en bytes)
    pub fn args_size(&self) -> usize {
        self.arg1_type.size() + self.arg2_type.size()
    }

    /// Formats prédéfinis courants pour les instructions
    pub fn reg_reg() -> Self {
        Self::new(ArgType::Register, ArgType::Register)
    }

    pub fn reg_imm8() -> Self {
        Self::new(ArgType::Register, ArgType::Immediate8)
    }

    pub fn reg_imm16() -> Self {
        Self::new(ArgType::Register, ArgType::Immediate16)
    }

    pub fn reg_imm32() -> Self {
        Self::new(ArgType::Register, ArgType::Immediate32)
    }

    pub fn reg_addr() -> Self {
        Self::new(ArgType::Register, ArgType::AbsoluteAddr)
    }

    pub fn reg_regoff() -> Self {
        Self::new(ArgType::Register, ArgType::RegisterOffset)
    }

    pub fn addr_only() -> Self {
        Self::new(ArgType::None, ArgType::AbsoluteAddr)
    }

    pub fn single_reg() -> Self {
        Self::new(ArgType::Register, ArgType::None)
    }

    pub fn no_args() -> Self {
        Self::new(ArgType::None, ArgType::None)
    }
}



// Test unitaire pour les formats d'instruction
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arg_type_from_u8() {
        // Test des conversions valides
        assert_eq!(ArgType::from_u8(0x0), Some(ArgType::None));
        assert_eq!(ArgType::from_u8(0x1), Some(ArgType::Register));
        assert_eq!(ArgType::from_u8(0x5), Some(ArgType::Immediate32));

        // Test des conversions avec bits supplémentaires (masquage)
        assert_eq!(ArgType::from_u8(0xF0), Some(ArgType::None));
        assert_eq!(ArgType::from_u8(0xF1), Some(ArgType::Register));

        // Test avec valeur invalide
        assert_eq!(ArgType::from_u8(0xA), None);
    }

    #[test]
    fn test_arg_type_size() {
        // Test des tailles d'arguments
        assert_eq!(ArgType::None.size(), 0);
        assert_eq!(ArgType::Register.size(), 1);
        assert_eq!(ArgType::Immediate8.size(), 1);
        assert_eq!(ArgType::Immediate16.size(), 2);
        assert_eq!(ArgType::Immediate32.size(), 4);
        assert_eq!(ArgType::Immediate64.size(), 8);
        assert_eq!(ArgType::AbsoluteAddr.size(), 4);
        assert_eq!(ArgType::RelativeAddr.size(), 4);
        assert_eq!(ArgType::RegisterOffset.size(), 2);
    }

    #[test]
    fn test_instruction_format_new() {
        let format = InstructionFormat::new(ArgType::Register, ArgType::Immediate8);

        assert_eq!(format.arg1_type, ArgType::Register);
        assert_eq!(format.arg2_type, ArgType::Immediate8);
    }

    #[test]
    fn test_instruction_format_encode_decode() {
        // Test d'encodage
        let format = InstructionFormat::new(ArgType::Register, ArgType::Immediate8);
        let encoded = format.encode();

        assert_eq!(encoded, 0x13); // (1 << 4) | 3

        // Test de décodage
        let decoded = InstructionFormat::decode(encoded).unwrap();

        assert_eq!(decoded.arg1_type, ArgType::Register);
        assert_eq!(decoded.arg2_type, ArgType::Immediate8);
    }

    #[test]
    fn test_instruction_format_args_size() {
        // Test avec différentes combinaisons
        let format1 = InstructionFormat::new(ArgType::None, ArgType::None);
        assert_eq!(format1.args_size(), 0);

        let format2 = InstructionFormat::new(ArgType::Register, ArgType::Immediate8);
        assert_eq!(format2.args_size(), 2); // 1 + 1

        let format3 = InstructionFormat::new(ArgType::Register, ArgType::Immediate32);
        assert_eq!(format3.args_size(), 5); // 1 + 4

        let format4 = InstructionFormat::new(ArgType::RegisterOffset, ArgType::AbsoluteAddr);
        assert_eq!(format4.args_size(), 6); // 2 + 4
    }

    #[test]
    fn test_instruction_format_predefined() {
        // Test des formats prédéfinis
        let reg_reg = InstructionFormat::reg_reg();
        assert_eq!(reg_reg.arg1_type, ArgType::Register);
        assert_eq!(reg_reg.arg2_type, ArgType::Register);

        let reg_imm8 = InstructionFormat::reg_imm8();
        assert_eq!(reg_imm8.arg1_type, ArgType::Register);
        assert_eq!(reg_imm8.arg2_type, ArgType::Immediate8);

        let no_args = InstructionFormat::no_args();
        assert_eq!(no_args.arg1_type, ArgType::None);
        assert_eq!(no_args.arg2_type, ArgType::None);

        let single_reg = InstructionFormat::single_reg();
        assert_eq!(single_reg.arg1_type, ArgType::Register);
        assert_eq!(single_reg.arg2_type, ArgType::None);

        let addr_only = InstructionFormat::addr_only();
        assert_eq!(addr_only.arg1_type, ArgType::None);
        assert_eq!(addr_only.arg2_type, ArgType::AbsoluteAddr);
    }

    #[test]
    fn test_instruction_format_invalid_decode() {
        // Test avec des bits invalides pour arg1_type
        let result = InstructionFormat::decode(0xA0);
        assert!(result.is_none());

        // Test avec des bits invalides pour arg2_type
        let result = InstructionFormat::decode(0x0A);
        assert!(result.is_none());

        // Test avec les deux types invalides
        let result = InstructionFormat::decode(0xAA);
        assert!(result.is_none());
    }
}
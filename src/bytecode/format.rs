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
//src/bytecode/decode_errors.rs

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    InsufficientData,
    InvalidOpcode(u8),
    InvalidFormat(u8),
    InvalidArgumentOffset,
    InvalidArgumentType,
}

impl fmt::Display for DecodeError{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InsufficientData => write!(f, "Données insuffisantes pour décoder l'instruction"),
            Self::InvalidOpcode(op) => write!(f, "Opcode invalide: {:#04x}", op),
            Self::InvalidFormat(fmt) => write!(f, "Format d'instruction invalide: {:#04x}", fmt),
            Self::InvalidArgumentOffset => write!(f, "Offset d'argument invalide"),
            Self::InvalidArgumentType => write!(f, "Type d'argument invalide"),
        }
    }
}

impl std::error::Error for DecodeError {}
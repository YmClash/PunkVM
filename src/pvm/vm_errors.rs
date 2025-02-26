// //src/pvm/vm_errors.rs
// use std::fmt;
// use std::error::Error;
// #[derive(Debug, Clone, PartialEq)]
// pub enum VMError {
//     MemoryError(String),
//     RegisterError(String),
//     InstructionError(String),
//     ConfigError(String),
//     ArithmeticError(String),
//     ExecutionError(String),
//     ALUError(String),
//     DecodeError(String),
// }
//
//
//
// // Ajouter type d'erreur pour les opérations arithmétiques
// impl VMError {
//     pub fn arithmetic_error(msg: &str) -> Self {
//         VMError::ArithmeticError(msg.to_string())
//     }
//
//     pub fn memory_error(msg: &str) -> Self {
//         VMError::MemoryError(msg.to_string())
//     }
//
//
// }
//
//
//
// impl fmt::Display for VMError{
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match self {
//             VMError::MemoryError(msg) => write!(f, "MemoryError: {}", msg),
//             VMError::RegisterError(msg) => write!(f, "RegisterError: {}", msg),
//             VMError::InstructionError(msg) => write!(f, "InstructionError: {}", msg),
//             VMError::ConfigError(msg) => write!(f, "ConfigError: {}", msg),
//             VMError::ArithmeticError(msg) => write!(f, "ArithmeticError: {}", msg),
//             VMError::ExecutionError(msg) => write!(f, "ExecutionError: {}", msg),
//             VMError::ALUError(msg) => write!(f, "ALUError: {}", msg),
//
//         }
//     }
// }
//
//
//
// impl Error for VMError{}
//
//
// /// Resultat type pour les operation de la VM
// pub type VMResult<T> = Result<T, VMError>;
//
//

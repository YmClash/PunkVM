// //src/pvm/registers.rs
//
// use crate::pvm::instructions::RegisterId;
// use crate::pvm::pipelines::StatusFlags;
// use crate::pvm::vm_errors::{VMError, VMResult};
//
//
// /// Banque de registres
// pub struct RegisterBank {
//     pub registers: Vec<Register>,
// }
//
// #[derive(Debug, Clone, PartialEq)]
// pub struct Register {
//     pub value: u64,
//     pub flags: RegisterFlags,
// }
//
// #[derive(Default,Copy, Clone, PartialEq,Debug)]
// pub struct RegisterFlags {
//     pub zero: bool,
//     pub negative: bool,
//     pub carry: bool
//
// }
//
//
// // Implémentation du trait Default pour Register
// impl Default for Register {
//     fn default() -> Self {
//         Self {
//             value: 0,
//             flags: RegisterFlags::default(),
//         }
//     }
// }
//
// /// Banque de registres
//
// impl RegisterBank {
//
//     /// Crée une nouvelle banque de registres avec le nombre de registres spécifié
//     pub fn new(count: usize) -> VMResult<Self> {
//         Ok(Self {
//             registers: vec![Register::default(); count],
//         })
//     }
//
//
//
//     pub fn reset(&mut self) -> VMResult<()> {
//         for reg in &mut self.registers {
//             *reg = Register::default();
//         }
//         Ok(())
//     }
//
//     // Méthodes supplémentaires pour la manipulation des registres
//     pub fn read(&self, index: usize) -> VMResult<&Register> {
//         self.registers.get(index)
//             .ok_or_else(|| VMError::RegisterError(format!("Index de registre invalide: {}", index)))
//     }
//
//     pub fn write(&mut self, index: usize, value: u64) -> VMResult<()> {
//         if let Some(reg) = self.registers.get_mut(index) {
//             reg.value = value;
//             // Mise à jour des flags
//             reg.flags.zero = value == 0;
//             reg.flags.negative = (value as i64) < 0;
//             Ok(())
//         } else {
//             Err(VMError::RegisterError(format!("Index de registre invalide: {}", index)))
//         }
//     }
//
//     pub fn get_flags(&self, index: usize) -> VMResult<&RegisterFlags> {
//         self.registers.get(index)
//             .map(|reg| &reg.flags)
//             .ok_or_else(|| VMError::RegisterError(format!("Index de registre invalide: {}", index)))
//     }
//
//     pub fn update_flags(&mut self, flags: StatusFlags) -> VMResult<()> {
//         // Maintenant flags est copiée, pas déplacée
//         if let Some(last_reg) = self.registers.last_mut() {
//             last_reg.flags = RegisterFlags {
//                 zero: flags.zero,
//                 negative: flags.negative,
//                 carry: flags.overflow,  // Mapping overflow à carry
//             };
//         }
//         Ok(())
//     }
//
//
//     pub fn write_register(&mut self, reg: RegisterId, value: i64) -> VMResult<()> {
//         self.write(reg.0 as usize, value as u64)
//     }
//
//     pub fn read_register(&self, reg: RegisterId) -> VMResult<i64> {
//         self.read(reg.0 as usize)
//             .map(|reg| reg.value as i64)
//     }
//
//     pub fn get_status_flags(&self) -> StatusFlags {
//         // Retourne les flags du dernier registre utilisé
//         if let Some(last_reg) = self.registers.last() {
//             StatusFlags {
//                 zero: last_reg.flags.zero,
//                 negative: last_reg.flags.negative,
//                 overflow: false,
//                 carry: last_reg.flags.carry,
//             }
//         } else {
//             StatusFlags::default()
//         }
//     }
//
//     pub fn get_status_flags_mut(&mut self) -> StatusFlags {
//         // Retourne-les flags du dernier registre utilisé
//         if let Some(last_reg) = self.registers.last() {
//             StatusFlags {
//                 zero: last_reg.flags.zero,
//                 negative: last_reg.flags.negative,
//                 overflow: false,
//                 carry: last_reg.flags.carry,
//             }
//         } else {
//             StatusFlags::default()
//         }
//     }
//     pub fn update_status_flags(&mut self, val1: i64, val2: i64) -> VMResult<()> {
//         let flags = StatusFlags {
//             zero: val1 == val2,
//             negative: val1 < val2,
//             overflow: false,
//             carry: false,
//         };
//         self.update_flags(flags)
//     }
//
// }
//
//
//
//
//
// // Tests unitaires
// #[cfg(test)]
// mod tests {
//
//     use super::*;
//
//     #[test]
//     fn test_register_bank_creation() {
//         let bank = RegisterBank::new(16);
//         assert!(bank.is_ok());
//         let bank = bank.unwrap();
//         assert_eq!(bank.registers.len(), 16);
//     }
//
//     #[test]
//     fn test_register_default_values() {
//         let reg = Register::default();
//         assert_eq!(reg.value, 0);
//         assert!(!reg.flags.zero);
//         assert!(!reg.flags.negative);
//         assert!(!reg.flags.carry);
//     }
//
//     #[test]
//     fn test_register_write_and_read() {
//         let mut bank = RegisterBank::new(16).unwrap();
//
//         // Test écriture
//         assert!(bank.write(0, 42).is_ok());
//
//         // Test lecture
//         let reg = bank.read(0).unwrap();
//         assert_eq!(reg.value, 42);
//         assert!(!reg.flags.zero);
//         assert!(!reg.flags.negative);
//     }
//
//     #[test]
//     fn test_register_flags() {
//         let mut bank = RegisterBank::new(16).unwrap();
//
//         // Test flag zero
//         assert!(bank.write(0, 0).is_ok());
//         assert!(bank.get_flags(0).unwrap().zero);
//
//         // Test flag negative (en utilisant le bit de poids fort)
//         assert!(bank.write(1, 0x8000_0000_0000_0000).is_ok());
//         assert!(bank.get_flags(1).unwrap().negative);
//     }
//
//     #[test]
//     fn test_invalid_register_access() {
//         let bank = RegisterBank::new(16).unwrap();
//         assert!(bank.read(16).is_err());
//         assert!(bank.get_flags(16).is_err());
//     }
//
//     use super::*;
//
//     #[test]
//     fn test_register_id_operations() {
//         let mut bank = RegisterBank::new(16).unwrap();
//         let reg_id = RegisterId(0);
//
//         // Test write_register
//         assert!(bank.write_register(reg_id, 42).is_ok());
//
//         // Test read_register
//         assert_eq!(bank.read_register(reg_id).unwrap(), 42);
//
//         // Test invalid register
//         let invalid_reg = RegisterId(16);
//         assert!(bank.write_register(invalid_reg, 42).is_err());
//         assert!(bank.read_register(invalid_reg).is_err());
//     }
//
//     #[test]
//     fn test_status_flags_copy() {
//         let flags = StatusFlags {
//             zero: true,
//             negative: false,
//             overflow: true,
//             carry: false,
//         };
//
//         // Test que Copy fonctionne
//         let flags2 = flags;  // Ceci devrait copier, pas déplacer
//         assert_eq!(flags.zero, flags2.zero);
//         assert_eq!(flags.negative, flags2.negative);
//         assert_eq!(flags.overflow, flags2.overflow);
//     }
//
//
//
// }
//

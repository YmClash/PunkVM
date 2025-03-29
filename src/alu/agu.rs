// use chrono::naive::NaiveDateDaysIterator;
//
// /// Address Generation Unit (AGU)
// /// Calcule les adresses mémoire pour les accès mémoire
// pub struct AGU {
//     /// Base registers (registres de base)
//     base_registers: [u64; 16],
//     /// Index registers (registres d'index)
//     index_registers: [u64; 16],
// }
//
//
// pub enum  AGUOperation {
//     Add,
//     Sub,
//
//
// }
//
// impl AGU {
//     /// Crée une nouvelle AGU
//     pub fn new() -> Self {
//         Self {
//             base_registers: [0; 16],
//             index_registers: [0; 16],
//         }
//     }
//
//
//
//     /// Calcule une adresse effective
//     /// base: registre de base
//     /// index: registre d'index
//     /// scale: facteur de multiplication pour l'index (1, 2, 4, 8)
//     /// offset: déplacement
//     pub fn calculate_address(&self, base: usize, index: usize, scale: u8, offset: i32) -> u64 {
//         let base_value = self.base_registers[base & 0xF];
//         let index_value = self.index_registers[index & 0xF];
//
//         // Calcul: base + (index * scale) + offset
//         base_value
//             .wrapping_add((index_value.wrapping_mul(scale as u64)))
//             .wrapping_add(offset as u64)
//     }
//
//     /// Met à jour un registre de base
//     pub fn set_base_register(&mut self, reg: usize, value: u64) {
//         self.base_registers[reg & 0xF] = value;
//     }
//
//     /// Met à jour un registre d'index
//     pub fn set_index_register(&mut self, reg: usize, value: u64) {
//         self.index_registers[reg & 0xF] = value;
//     }
//
//     /// Lit un registre de base
//     pub fn get_base_register(&self, reg: usize) -> u64 {
//         self.base_registers[reg & 0xF]
//     }
//
//     /// Lit un registre d'index
//     pub fn get_index_register(&self, reg: usize) -> u64 {
//         self.index_registers[reg & 0xF]
//     }
// }
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_address_calculation() {
//         let mut agu = AGU::new();
//
//         // Configure les registres
//         agu.set_base_register(0, 1000);
//         agu.set_index_register(1, 5);
//
//         // Test calcul simple
//         assert_eq!(agu.calculate_address(0, 1, 4, 20), 1040); // 1000 + (5 * 4) + 20
//
//         // Test avec offset négatif
//         assert_eq!(agu.calculate_address(0, 1, 2, -10), 1000); // 1000 + (5 * 2) - 10
//
//         // Test débordement
//         agu.set_base_register(2, u64::MAX);
//         agu.set_index_register(3, 1);
//         assert_eq!(agu.calculate_address(2, 3, 1, 1), 0); // Wraparound attendu
//     }
//
//     #[test]
//     fn test_register_operations() {
//         let mut agu = AGU::new();
//
//         // Test registres de base
//         agu.set_base_register(0, 42);
//         assert_eq!(agu.get_base_register(0), 42);
//
//         // Test registres d'index
//         agu.set_index_register(1, 24);
//         assert_eq!(agu.get_index_register(1), 24);
//
//         // Test masquage des indices de registres
//         agu.set_base_register(16, 100); // 16 & 0xF = 0
//         assert_eq!(agu.get_base_register(0), 100);
//     }
// }

// //src/pvm/hazards.rs
// use std::collections::{HashMap, VecDeque};
// use crate::pvm::instructions::{Address,  Instruction, RegisterId};
// use crate::pvm::pipelines::PipelineStage;
// use crate::pvm::registers::RegisterBank;
// use crate::pvm::WritePolicy;
//
// pub struct HazardUnit {
//     pub last_write_registers: Vec<RegisterId>,
//     pub load_target: Option<(RegisterId, Address)>,
//     pub store_address: Option<Address>,
//     pub pending_loads: Vec<HazardDetectionInfo>,
//     pub pending_writes: Vec<HazardDetectionInfo>,
//     pub store_buffer: VecDeque<StoreBufferEntry>,
//     pub load_queue: VecDeque<LoadQueueEntry>,
//     pub write_policy: WritePolicy,
//     pub commit_latency: usize,
//
//
//
// }
//
// pub struct StoreBufferEntry{
//     pub address: Address,
//     pub value: i64,
//     pub age: usize,
// }
//
// pub struct LoadQueueEntry{
//     pub address: Address,
//     pub destination: RegisterId,
// }
//
//
// #[derive(Debug)]
// pub struct HazardDetectionInfo {
//     pub reg: RegisterId,
//     pub hazard_type: HazardType,
//     pub stage: PipelineStage,
// }
//
// #[derive(Debug, PartialEq)]
// pub enum HazardType {
//     LoadUse,
//     StoreLoad,
//     DataDependency,
//
// }
//
// #[derive(Debug,Clone,Copy)]
// pub enum HazardResult{
//     None,
//     StoreLoad,
//     LoadUse,
//     DataDependency,
// }
//
//
// pub struct OptimizedHazardUnit {
//     // Utiliser des bitsets pour une détection plus rapide
//     pub hazard_bitmap: u64,
//     pub load_target: Option<(RegisterId, Address)>,
//     pub store_address: Option<Address>,
//     // Cache des derniers hazards pour éviter les recalculs
//     pub hazard_cache: HashMap<RegisterId, HazardType>,
// }
//
//
// impl HazardUnit {
//     pub fn new() -> Self {
//         Self {
//             last_write_registers: Vec::new(),
//             load_target: None,
//             store_address: None,
//             pending_loads: Vec::new(),
//             pending_writes: Vec::new(),
//             store_buffer: VecDeque::new(),
//             load_queue: VecDeque::new(),
//             write_policy: WritePolicy::WriteThrough,
//             commit_latency: 2,
//         }
//     }
//
//     // Vérifie les hazards pour une instruction décodée
//
//     // pub fn check_hazards(&mut self, instruction: &Instruction, registers: &RegisterBank) -> HazardResult {
//     //     match instruction {
//     //         Instruction::Load(reg, addr) => {
//     //             // Vérifier Store-Load hazard
//     //             if let Some(store_addr) = self.store_address {
//     //                 if store_addr == *addr {
//     //                     println!("Store-Load hazard détecté sur l'adresse {:?}", addr);
//     //                     self.store_address = None;
//     //                     return HazardResult::StoreLoad;
//     //                 }
//     //             }
//     //             self.load_target = Some((*reg, *addr));
//     //             HazardResult::None
//     //         },
//     //         Instruction::Store(reg, addr) => {
//     //             self.store_address = Some(*addr);
//     //             // Vérifier si le registre source est en attente d'un Load
//     //             if let Some((target_reg, _)) = self.load_target {
//     //                 if target_reg == *reg {
//     //                     println!("Load-Store hazard détecté sur le registre {:?}", reg);
//     //                     return HazardResult::LoadUse;
//     //                 }
//     //             }
//     //             HazardResult::None
//     //         },
//     //         Instruction::Add(_, src1, src2) |
//     //         Instruction::Sub(_, src1, src2) |
//     //         Instruction::Mul(_, src1, src2) |
//     //         Instruction::Div(_, src1, src2) => {
//     //             // Vérifier les dépendances avec les loads
//     //             if let Some((target_reg, _)) = self.load_target {
//     //                 if *src1 == target_reg || *src2 == target_reg {
//     //                     println!("Data dependency hazard détecté");
//     //                     return HazardResult::DataDependency;
//     //                 }
//     //             }
//     //             HazardResult::None
//     //         },
//     //         _ => HazardResult::None
//     //     }
//     // }
//     pub fn check_hazards(&mut self, instruction: &Instruction, registers: &RegisterBank) -> HazardResult {
//         match instruction {
//             Instruction::Load(reg, addr) => {
//                 // Vérifier les store buffer
//                 for entry in self.store_buffer.iter() {
//                     if entry.address == *addr {
//                         println!("Store-Load hazard détecté: addr={:?}, age={}", addr, entry.age);
//                         return HazardResult::StoreLoad;
//                     }
//                 }
//
//                 // Vérifier store_address immédiat
//                 if let Some(store_addr) = self.store_address {
//                     if store_addr == *addr {
//                         println!("Store-Load hazard immédiat détecté: addr={:?}", addr);
//                         return HazardResult::StoreLoad;
//                     }
//                 }
//
//                 self.load_target = Some((*reg, *addr));
//                 self.load_queue.push_back(LoadQueueEntry {
//                     address: *addr,
//                     destination: *reg,
//                 });
//
//                 HazardResult::None
//             }
//             // Instruction::Load(reg, addr) => {
//             //     // Vérifier le store buffer avec la politique d'écriture
//             //     let has_store_hazard = match self.write_policy {
//             //         WritePolicy::WriteThrough => {
//             //             // Pour write-through, vérifier seulement les stores très récents
//             //             self.store_buffer.iter().any(|entry|
//             //             entry.address == *addr && entry.age == 0
//             //             )
//             //         },
//             //         WritePolicy::WriteBack => {
//             //             // Pour write-back, vérifier tous les stores non commités
//             //             self.store_buffer.iter().any(|entry|
//             //             entry.address == *addr && entry.age < self.commit_latency
//             //             )
//             //         }
//             //     };
//             //
//             //     if has_store_hazard {
//             //         println!("Store-Load hazard détecté sur {:?}", addr);
//             //         return HazardResult::StoreLoad;
//             //     }
//             //
//             //     // Ajouter aux loads en attente
//             //     self.pending_loads.push(HazardDetectionInfo {
//             //         reg: *reg,
//             //         hazard_type: HazardType::LoadUse,
//             //         stage: PipelineStage::Memory
//             //     });
//             //
//             //     self.load_target = Some((*reg, *addr));
//             //     HazardResult::None
//             // },
//             Instruction::Store(reg, addr) => {
//                 // Vérifier dépendances load-use
//                 if let Some((target_reg, _)) = self.load_target {
//                     if *reg == target_reg {
//                         println!("Load-Use hazard détecté: reg={:?}", reg);
//                         return HazardResult::LoadUse;
//                     }
//                 }
//
//                 // Mettre à jour le store buffer
//                 self.store_buffer.push_back(StoreBufferEntry {
//                     address: *addr,
//                     value: registers.read_register(*reg).unwrap_or(0),
//                     age: 0
//                 });
//
//                 self.store_address = Some(*addr);
//                 HazardResult::None
//             },
//             // Instruction::Store(reg, addr) => {
//             //     // 1. Vérifier les dépendances avec les loads en attente
//             //     if self.pending_loads.iter().any(|load| load.reg == *reg) {
//             //         println!("Load-Use hazard détecté: reg={:?}", reg);
//             //         return HazardResult::LoadUse;
//             //     }
//             //
//             //     // 2. Ajouter au store buffer
//             //     let value = registers.read_register(*reg).unwrap_or(0);
//             //     self.store_buffer.push_back(StoreBufferEntry {
//             //         address: *addr,
//             //         value,
//             //         age: 0
//             //     });
//             //
//             //     self.store_address = Some(*addr);
//             //     HazardResult::None
//             // },
//             Instruction::Add(dest, src1, src2) |
//             Instruction::Sub(dest, src1, src2) |
//             Instruction::Mul(dest, src1, src2) |
//             Instruction::Div(dest, src1, src2) => {
//                 // 1. Vérifier les dépendances avec les registers sources
//                 if self.check_register_dependency(*src1) || self.check_register_dependency(*src2) {
//                     println!("Data dependency hazard détecté: src1={:?}, src2={:?}", src1, src2);
//                     return HazardResult::DataDependency;
//                 }
//
//                 // 2. Ajouter aux écritures en attente
//                 self.pending_writes.push(HazardDetectionInfo {
//                     reg: *dest,
//                     hazard_type: HazardType::DataDependency,
//                     stage: PipelineStage::Execute
//                 });
//
//                 HazardResult::None
//             },
//             _ => HazardResult::None
//         }
//     }
//
//
//     fn check_register_dependency(&self, reg: RegisterId) -> bool {
//         // 1. Vérifier les loads en attente
//         if self.pending_loads.iter().any(|load| load.reg == reg) {
//             return true;
//         }
//
//         // 2. Vérifier les écritures en attente
//         if self.pending_writes.iter().any(|write| write.reg == reg) {
//             return true;
//         }
//
//         // 3. Vérifier le load target actuel
//         if let Some((target_reg, _)) = self.load_target {
//             if target_reg == reg {
//                 return true;
//             }
//         }
//
//         false
//     }
//
//
//     pub fn clear_hazards(&mut self){
//         self.load_target = None;
//         self.store_address = None;
//
//     }
//
//     // Méthode améliorée pour nettoyer les hazards résolus
//     pub fn clean_resolved_hazards(&mut self, completed_stage: PipelineStage) {
//         // Nettoyer les loads terminés
//         self.pending_loads.retain(|load| load.stage != completed_stage);
//
//         // Nettoyer les écritures terminées
//         self.pending_writes.retain(|write| write.stage != completed_stage);
//
//         // Mettre à jour le store buffer
//         self.update_store_buffer();
//     }
//
//
//     //efface l'adresse de chargement
//     pub fn clear_load_target(&mut self){
//         self.load_target = None;
//     }
//
//     //efface l'adresse de stockage
//     pub fn clear_store_address(&mut self){
//         self.store_address = None;
//     }
//
//     // pub fn update_store_buffer(&mut self) {
//     //     // Vieillir les entrées du store buffer
//     //     for entry in self.store_buffer.iter_mut() {
//     //         entry.age += 1;
//     //     }
//     //
//     //     // Retirer les entrées trop vieilles
//     //     while let Some(entry) = self.store_buffer.front() {
//     //         if entry.age >= 4 { // 4 cycles maximum dans le buffer
//     //             self.store_buffer.pop_front();
//     //         } else {
//     //             break;
//     //         }
//     //     }
//     // }
//
//     pub fn update_store_buffer(&mut self) {
//         // Mise à jour des âges
//         for entry in self.store_buffer.iter_mut() {
//             entry.age += 1;
//         }
//
//         // Retirer les stores commités selon la politique
//         match self.write_policy {
//             WritePolicy::WriteThrough => {
//                 // Retirer rapidement les stores
//                 while let Some(entry) = self.store_buffer.front() {
//                     if entry.age >= 1 {
//                         self.store_buffer.pop_front();
//                     } else {
//                         break;
//                     }
//                 }
//             },
//             WritePolicy::WriteBack => {
//                 // Garder les stores plus longtemps
//                 while let Some(entry) = self.store_buffer.front() {
//                     if entry.age >= self.commit_latency {
//                         self.store_buffer.pop_front();
//                     } else {
//                         break;
//                     }
//                 }
//             }
//         }
//     }
//
//     // pub fn commit_store(&mut self, address: Address, value: i64) {
//     //     self.store_buffer.push_back(StoreBufferEntry {
//     //         address,
//     //         value,
//     //         age: 0
//     //     });
//     // }
//     pub fn commit_store(&mut self, address: Address, value: i64) {
//         // Important : mettre à jour la valeur dans le store buffer
//         if let Some(entry) = self.store_buffer.iter_mut()
//             .find(|entry| entry.address == address) {
//             entry.value = value;
//         }
//
//         // Mettre à jour le store_address
//         self.store_address = Some(address);
//     }
//
//     pub fn commit_memory_operation(&mut self, stage: PipelineStage) {
//         match stage {
//             PipelineStage::Memory => {
//                 // Clear load target après completion de l'étage mémoire
//                 self.load_target = None;
//             },
//             PipelineStage::Writeback => {
//                 // Clear store address après writeback
//                 self.store_address = None;
//             },
//             _ => {}
//         }
//     }
//
//     pub fn forward_store_value(&self, address: Address) -> Option<i64> {
//         // Chercher la valeur la plus récente dans le store buffer
//         self.store_buffer.iter()
//             .rev()
//             .find(|entry| entry.address == address)
//             .map(|entry| entry.value)
//     }
//
//     pub fn configure_latency(&mut self, latency: usize, policy: WritePolicy) {
//         self.commit_latency = latency;
//         self.write_policy = policy;
//     }
//
//
//
// }
//
//
//
// #[cfg(test)]
// mod tests {
//     use crate::pvm::memorys::MemoryController;
//     use crate::pvm::pipelines::Pipeline;
//     use super::*;
//
//     fn setup_test_env() -> (Pipeline, RegisterBank, MemoryController) {
//         let pipeline = Pipeline::new();
//         let register_bank = RegisterBank::new(8).unwrap();
//         let memory_controller = MemoryController::new(1024, 256).unwrap();
//         (pipeline, register_bank, memory_controller)
//     }
//
//     #[test]
//     fn test_load_use_hazard() {
//         let (mut pipeline, mut register_bank, mut memory_controller) = setup_test_env();
//
//         // Programme pour tester le Load-Use hazard:
//         // Store R0, @100 (stocker une valeur initiale)
//         // Load R1, @100  (charger la valeur)
//         // Add R2, R1, R1 (utiliser R1 immédiatement - devrait stall)
//         let program = vec![
//             Instruction::LoadImm(RegisterId(0), 42),
//             Instruction::Store(RegisterId(0), Address(100)),
//             Instruction::Load(RegisterId(1), Address(100)),
//             Instruction::Add(RegisterId(2), RegisterId(1), RegisterId(1)),
//         ];
//
//         pipeline.load_instructions(program).unwrap();
//
//         let mut cycles = 0;
//         while !pipeline.is_empty().unwrap() {
//             pipeline.cycle(&mut register_bank, &mut memory_controller).unwrap();
//             cycles += 1;
//             if cycles > 20 { // Sécurité anti-boucle infinie
//                 break;
//             }
//         }
//
//         // Vérifier que:
//         // 1. La valeur finale est correcte (R2 = 42 + 42 = 84)
//         // 2. Il y a eu au moins un stall (hazard)
//         assert_eq!(register_bank.read_register(RegisterId(2)).unwrap(), 84);
//         assert!(pipeline.stats.stalls > 0, "Le pipeline aurait dû avoir des stalls pour le Load-Use hazard");
//         assert!(pipeline.stats.hazards > 0, "Un Load-Use hazard aurait dû être détecté");
//     }
//
//     #[test]
//     fn test_store_load_hazard() {
//         let (mut pipeline, mut register_bank, mut memory_controller) = setup_test_env();
//
//         // Programme pour tester le Store-Load hazard:
//         // Store R0, @100
//         // Load R1, @100 (même adresse - devrait attendre que le store soit terminé)
//         let program = vec![
//             Instruction::LoadImm(RegisterId(0), 42),
//             Instruction::Store(RegisterId(0), Address(100)),
//             Instruction::Load(RegisterId(1), Address(100)),
//         ];
//
//         pipeline.load_instructions(program).unwrap();
//
//         let mut cycles = 0;
//         while !pipeline.is_empty().unwrap() {
//             pipeline.cycle(&mut register_bank, &mut memory_controller).unwrap();
//             cycles += 1;
//             if cycles > 20 {
//                 break;
//             }
//         }
//
//         // Vérifier que:
//         // 1. La valeur chargée est correcte
//         // 2. Il y a eu un stall pour le Store-Load hazard
//         assert_eq!(register_bank.read_register(RegisterId(1)).unwrap(), 42);
//         assert!(pipeline.stats.stalls > 0, "Le pipeline aurait dû avoir des stalls pour le Store-Load hazard");
//     }
//
//     #[test]
//     fn test_multiple_load_use_hazards() {
//         let (mut pipeline, mut register_bank, mut memory_controller) = setup_test_env();
//
//         let program = vec![
//             Instruction::LoadImm(RegisterId(0), 10),
//             Instruction::Store(RegisterId(0), Address(100)),
//             Instruction::Load(RegisterId(1), Address(100)),
//             Instruction::Add(RegisterId(2), RegisterId(1), RegisterId(1)),
//             Instruction::Load(RegisterId(3), Address(100)),
//             Instruction::Sub(RegisterId(4), RegisterId(3), RegisterId(2)),
//         ];
//
//         pipeline.load_instructions(program).unwrap();
//
//         while !pipeline.is_empty().unwrap() {
//             pipeline.cycle(&mut register_bank, &mut memory_controller).unwrap();
//         }
//
//         assert_eq!(register_bank.read_register(RegisterId(2)).unwrap() as i64, 20);
//         assert_eq!(register_bank.read_register(RegisterId(4)).unwrap() as i64, -10);
//     }
//
//     #[test]
//     fn test_interleaved_load_store() {
//         let (mut pipeline, mut register_bank, mut memory_controller) = setup_test_env();
//
//         // Test avec des loads et stores entrelacés
//         // R0 = 100
//         // Store R0, @200
//         // R1 = 50
//         // Load R2, @200  (doit attendre le store)
//         // Add R3, R2, R1
//         // Store R3, @300
//         // Load R4, @300  (doit attendre le second store)
//         let program = vec![
//             Instruction::LoadImm(RegisterId(0), 100),
//             Instruction::Store(RegisterId(0), Address(200)),
//             Instruction::LoadImm(RegisterId(1), 50),
//             Instruction::Load(RegisterId(2), Address(200)),
//             Instruction::Add(RegisterId(3), RegisterId(2), RegisterId(1)),
//             Instruction::Store(RegisterId(3), Address(300)),
//             Instruction::Load(RegisterId(4), Address(300)),
//         ];
//
//         pipeline.load_instructions(program).unwrap();
//
//         let mut cycles = 0;
//         while !pipeline.is_empty().unwrap() {
//             pipeline.cycle(&mut register_bank, &mut memory_controller).unwrap();
//             cycles += 1;
//             if cycles > 30 {
//                 break;
//             }
//         }
//
//         assert_eq!(register_bank.read_register(RegisterId(2)).unwrap(), 100);
//         assert_eq!(register_bank.read_register(RegisterId(3)).unwrap(), 150);
//         assert_eq!(register_bank.read_register(RegisterId(4)).unwrap(), 150);
//         assert!(pipeline.stats.stalls > 0, "Des stalls auraient dû être nécessaires");
//     }
//
//     #[test]
//     fn test_forward_chain() {
//         let (mut pipeline, mut register_bank, mut memory_controller) = setup_test_env();
//
//         // Test d'une chaîne de forwarding
//         // R0 = 1
//         // R1 = R0 + 1  (forward de R0)
//         // R2 = R1 + 1  (forward de R1)
//         // R3 = R2 + 1  (forward de R2)
//         let program = vec![
//             Instruction::LoadImm(RegisterId(0), 1),
//             Instruction::Add(RegisterId(1), RegisterId(0), RegisterId(0)),
//             Instruction::Add(RegisterId(2), RegisterId(1), RegisterId(0)),
//             Instruction::Add(RegisterId(3), RegisterId(2), RegisterId(0)),
//         ];
//
//         pipeline.load_instructions(program).unwrap();
//
//         while !pipeline.is_empty().unwrap() {
//             pipeline.cycle(&mut register_bank, &mut memory_controller).unwrap();
//         }
//
//         assert_eq!(register_bank.read_register(RegisterId(1)).unwrap(), 2);
//         assert_eq!(register_bank.read_register(RegisterId(2)).unwrap(), 3);
//         assert_eq!(register_bank.read_register(RegisterId(3)).unwrap(), 4);
//     }
//
//
//     #[test]
//     fn test_hazard_clearing() {
//         let (mut pipeline, mut register_bank, mut memory_controller) = setup_test_env();
//
//         // Vérifie que les hazards sont correctement nettoyés
//         let program = vec![
//             Instruction::LoadImm(RegisterId(0), 42),
//             Instruction::Store(RegisterId(0), Address(100)),
//             Instruction::Load(RegisterId(1), Address(100)),
//             Instruction::LoadImm(RegisterId(2), 24),
//             Instruction::Store(RegisterId(2), Address(100)),
//             Instruction::Load(RegisterId(3), Address(100)),
//         ];
//
//         pipeline.load_instructions(program).unwrap();
//
//         let mut cycles = 0;
//         while !pipeline.is_empty().unwrap() && cycles < 20 {
//             pipeline.cycle(&mut register_bank, &mut memory_controller).unwrap();
//             cycles += 1;
//         }
//
//         // Vérifier que les hazards ont été détectés mais aussi nettoyés
//         assert!(pipeline.stats.hazards > 0, "Des hazards auraient dû être détectés");
//         assert_eq!(register_bank.read_register(RegisterId(1)).unwrap(), 42);
//         assert_eq!(register_bank.read_register(RegisterId(3)).unwrap(), 24);
//     }
//
//     #[test]
//     fn test_basic_store_load_hazard() {
//         let (mut pipeline, mut register_bank, mut memory_controller) = setup_test_env();
//
//         // Programme simple avec Store suivi d'un Load à la même adresse
//         let program = vec![
//             Instruction::LoadImm(RegisterId(0), 42),
//             Instruction::Store(RegisterId(0), Address(100)),
//             Instruction::Load(RegisterId(1), Address(100)),
//         ];
//
//         pipeline.load_instructions(program).unwrap();
//
//         let mut cycles = 0;
//         let mut hazard_detected = false;
//
//         while !pipeline.is_empty().unwrap() && cycles < 10 {
//             pipeline.cycle(&mut register_bank, &mut memory_controller).unwrap();
//             cycles += 1;
//             if pipeline.stats.hazards > 0 {
//                 hazard_detected = true;
//             }
//         }
//
//         assert!(hazard_detected, "Store-Load hazard aurait dû être détecté");
//         assert_eq!(register_bank.read_register(RegisterId(1)).unwrap(), 42);
//     }
//
//     #[test]
//     fn test_multiple_store_load_sequence() {
//         let (mut pipeline, mut register_bank, mut memory_controller) = setup_test_env();
//
//         // Séquence de Store/Load multiples
//         let program = vec![
//             Instruction::LoadImm(RegisterId(0), 10),
//             Instruction::Store(RegisterId(0), Address(100)),
//             Instruction::Load(RegisterId(1), Address(100)),   // Hazard #1
//             Instruction::LoadImm(RegisterId(2), 20),
//             Instruction::Store(RegisterId(2), Address(100)),
//             Instruction::Load(RegisterId(3), Address(100)),   // Hazard #2
//         ];
//
//         pipeline.load_instructions(program).unwrap();
//
//         let mut cycles = 0;
//         while !pipeline.is_empty().unwrap() && cycles < 20 {
//             pipeline.cycle(&mut register_bank, &mut memory_controller).unwrap();
//             cycles += 1;
//         }
//
//         assert!(pipeline.stats.hazards >= 2, "Au moins deux hazards auraient dû être détectés");
//         assert_eq!(register_bank.read_register(RegisterId(1)).unwrap(), 10);
//         assert_eq!(register_bank.read_register(RegisterId(3)).unwrap(), 20);
//     }
//
//     #[test]
//     fn test_load_use_hazard_resolution() {
//         let (mut pipeline, mut register_bank, mut memory_controller) = setup_test_env();
//
//         // Test Load suivi immédiatement par une utilisation du registre
//         let program = vec![
//             Instruction::LoadImm(RegisterId(0), 5),
//             Instruction::Store(RegisterId(0), Address(100)),
//             Instruction::Load(RegisterId(1), Address(100)),
//             Instruction::Add(RegisterId(2), RegisterId(1), RegisterId(1)), // Utilise R1 juste après le Load
//         ];
//
//         pipeline.load_instructions(program).unwrap();
//
//         let mut cycles = 0;
//         while !pipeline.is_empty().unwrap() && cycles < 15 {
//             pipeline.cycle(&mut register_bank, &mut memory_controller).unwrap();
//             cycles += 1;
//         }
//
//         // Le résultat final devrait être 5 + 5 = 10 dans R2
//         assert_eq!(register_bank.read_register(RegisterId(2)).unwrap(), 10);
//         assert!(pipeline.stats.stalls > 0, "Des stalls auraient dû être nécessaires");
//     }
//
//     #[test]
//     fn test_store_load_different_addresses() {
//         let (mut pipeline, mut register_bank, mut memory_controller) = setup_test_env();
//
//         // Test Store/Load à des adresses différentes (pas de hazard attendu)
//         let program = vec![
//             Instruction::LoadImm(RegisterId(0), 42),
//             Instruction::Store(RegisterId(0), Address(100)),
//             Instruction::Load(RegisterId(1), Address(200)),  // Adresse différente
//         ];
//
//         pipeline.load_instructions(program).unwrap();
//
//         let mut cycles = 0;
//         while !pipeline.is_empty().unwrap() && cycles < 10 {
//             pipeline.cycle(&mut register_bank, &mut memory_controller).unwrap();
//             cycles += 1;
//         }
//
//         assert_eq!(pipeline.stats.hazards, 0, "Aucun hazard ne devrait être détecté");
//     }
//
//     #[test]
//     fn test_store_load_forwarding() {
//         let (mut pipeline, mut register_bank, mut memory_controller) = setup_test_env();
//
//         // Test si le forwarding fonctionne correctement après un Store
//         let program = vec![
//             Instruction::LoadImm(RegisterId(0), 42),
//             Instruction::Store(RegisterId(0), Address(100)),
//             Instruction::Load(RegisterId(1), Address(100)),
//             Instruction::Add(RegisterId(2), RegisterId(1), RegisterId(0)), // R2 = 42 + 42
//         ];
//
//         pipeline.load_instructions(program).unwrap();
//
//         let mut cycles = 0;
//         while !pipeline.is_empty().unwrap() && cycles < 15 {
//             pipeline.cycle(&mut register_bank, &mut memory_controller).unwrap();
//             cycles += 1;
//         }
//
//         assert_eq!(register_bank.read_register(RegisterId(2)).unwrap(), 84);
//         assert!(cycles > pipeline.stats.instructions_executed,
//                 "Le nombre de cycles devrait être supérieur au nombre d'instructions à cause des hazards");
//     }
//
// }

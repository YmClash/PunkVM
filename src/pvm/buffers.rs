// //src/pvm/buffers.rs


/// Store buffer pour les écritures mémoire
pub struct StoreBuffer {
    capacity: usize,        // Taille maximale du buffer
    entries: Vec<(u32, u8)>,    // Entrées du buffer (adresse -> valeur)
}



impl StoreBuffer{
    /// Crée un nouveau store buffer
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            entries: Vec::with_capacity(capacity),
        }
    }

    /// Ajoute une entrée au store buffer
    pub fn add(&mut self, addr: u32, value: u8) {
        // Vérifier si l'adresse est déjà dans le buffer
        if let Some(idx) = self.entries.iter().position(|&(a, _)| a == addr) {
            // Remplacer la valeur existante
            self.entries[idx] = (addr, value);
        } else {
            // Si le buffer est plein, vider la plus ancienne entrée
            if self.entries.len() >= self.capacity {
                self.entries.remove(0);
            }

            // Ajouter la nouvelle entrée
            self.entries.push((addr, value));
        }
    }

    pub fn lookup_byte(&self, addr: u32) -> Option<u8> {
        // Recherche de la dernière entrée correspondant à l'adresse
        self.entries
            .iter()
            .rev()
            .find(|&&(a, _)| a == addr)
            .map(|&(_, value)| value)
    }

    /// Vérifie si une adresse est dans le store buffer
    pub fn has_address(&self, addr: u32) -> bool {
        self.entries.iter().any(|&(a, _)| a == addr)
    }

    /// Vide le store buffer en écrivant toutes les données en mémoire
    pub fn flush(&mut self, memory: &mut [u8]) {
        for (addr, value) in &self.entries {
            if (*addr as usize) < memory.len() {
                memory[*addr as usize] = *value;
            }
        }

        self.entries.clear();
    }

    /// Nettoie le store buffer
    pub fn clear(&mut self) {
        self.entries.clear();
    }


}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_buffer_creation() {
        let buffer = StoreBuffer::new(8);
        assert_eq!(buffer.capacity, 8);
        assert_eq!(buffer.entries.len(), 0);
    }

    #[test]
    fn test_store_buffer_add() {
        let mut buffer = StoreBuffer::new(4);

        // Ajouter quelques entrées
        buffer.add(0x100, 42);
        buffer.add(0x101, 43);

        assert_eq!(buffer.entries.len(), 2);

        // Vérifier que les entrées sont correctes
        assert_eq!(buffer.lookup_byte(0x100), Some(42));
        assert_eq!(buffer.lookup_byte(0x101), Some(43));

        // Adresse non présente
        assert_eq!(buffer.lookup_byte(0x102), None);
    }

    #[test]
    fn test_store_buffer_update() {
        let mut buffer = StoreBuffer::new(4);

        // Ajouter une entrée
        buffer.add(0x100, 42);

        // Mettre à jour la même adresse
        buffer.add(0x100, 43);

        // Vérifier que l'entrée a été mise à jour
        assert_eq!(buffer.lookup_byte(0x100), Some(43));
        assert_eq!(buffer.entries.len(), 1);
    }

    #[test]
    fn test_store_buffer_capacity() {
        let mut buffer = StoreBuffer::new(3);

        // Remplir le buffer
        buffer.add(0x100, 42);
        buffer.add(0x101, 43);
        buffer.add(0x102, 44);

        // Le buffer est plein
        assert_eq!(buffer.entries.len(), 3);

        // Ajouter une entrée supplémentaire
        buffer.add(0x103, 45);

        // Le buffer est toujours plein, mais la plus ancienne entrée a été évincée
        assert_eq!(buffer.entries.len(), 3);
        assert_eq!(buffer.lookup_byte(0x100), None); // La plus ancienne entrée
        assert_eq!(buffer.lookup_byte(0x103), Some(45)); // La nouvelle entrée
    }

    #[test]
    fn test_store_buffer_has_address() {
        let mut buffer = StoreBuffer::new(4);

        // Ajouter quelques entrées
        buffer.add(0x100, 42);
        buffer.add(0x101, 43);

        // Vérifier si les adresses sont présentes
        assert!(buffer.has_address(0x100));
        assert!(buffer.has_address(0x101));
        assert!(!buffer.has_address(0x102));
    }

    #[test]
    fn test_store_buffer_flush() {
        let mut buffer = StoreBuffer::new(4);

        // Ajouter quelques entrées
        buffer.add(0x100, 42);
        buffer.add(0x101, 43);

        // Créer une mémoire simulée
        let mut memory = vec![0; 0x200];

        // Flush le buffer vers la mémoire
        buffer.flush(&mut memory);

        // Vérifier que le buffer est vide
        assert_eq!(buffer.entries.len(), 0);

        // Vérifier que la mémoire a été mise à jour
        assert_eq!(memory[0x100], 42);
        assert_eq!(memory[0x101], 43);
    }

    #[test]
    fn test_store_buffer_clear() {
        let mut buffer = StoreBuffer::new(4);

        // Ajouter quelques entrées
        buffer.add(0x100, 42);
        buffer.add(0x101, 43);

        // Vider le buffer
        buffer.clear();

        // Vérifier que le buffer est vide
        assert_eq!(buffer.entries.len(), 0);
        assert_eq!(buffer.lookup_byte(0x100), None);
        assert_eq!(buffer.lookup_byte(0x101), None);
    }

    #[test]
    fn test_store_buffer_latest_value() {
        let mut buffer = StoreBuffer::new(4);

        // Ajouter plusieurs entrées pour la même adresse
        buffer.add(0x100, 42);
        buffer.add(0x101, 43);
        buffer.add(0x100, 44);

        // Vérifier que la dernière valeur pour l'adresse est retournée
        assert_eq!(buffer.lookup_byte(0x100), Some(44));
    }
}
































// use crate::pvm::instructions::Address;
// use std::collections::VecDeque;
// use crate::pvm::instructions::Instruction;
//
// use std::collections::HashMap;
// use crate::pvm::branch_predictor::{BranchTargetBuffer, BTBEntry};
// use crate::pvm::pipeline_errors::PipelineError;
// use crate::pvm::pipelines::Pipeline;
//
//
// pub struct BypassBuffer {
//     pub entries: HashMap<u64, BypassEntry>,
//     pub capacity: usize,
// }
//
// #[derive(Debug)]
// pub struct BypassEntry {
//     pub value: u64,
//     pub valid: bool,
// }
//
// #[derive(Debug)]
// pub struct StoreOperation {
//     pub addr: u64,
//     pub value: u64,
// }
//
// impl BranchTargetBuffer {
//     pub fn new(size: usize) -> Self {
//         Self {
//             entries: vec![BTBEntry {
//                 tag: 0,
//                 target: 0,
//                 valid: false,
//             }; size],
//             size,
//         }
//     }
//
//     pub fn get_target(&self, pc: u64) -> Option<u64> {
//         let index = (pc as usize) % self.size;
//         let entry = &self.entries[index];
//
//         if entry.valid && entry.tag == pc {
//             Some(entry.target)
//         } else {
//             None
//         }
//     }
//
//     pub fn update(&mut self, pc: u64, target: u64) {
//         let index = (pc as usize) % self.size;
//         self.entries[index] = BTBEntry {
//             tag: pc,
//             target,
//             valid: true,
//         };
//     }
//
//     pub fn invalidate(&mut self, pc: u64) {
//         let index = (pc as usize) % self.size;
//         self.entries[index].valid = false;
//     }
// }
//
//
//
// #[derive(Default)]
// pub struct FetchBuffer {
//     pub instructions: VecDeque<Instruction>,
//     pub size: usize,
// }
//
// impl FetchBuffer {
//     pub fn new(size: usize) -> Self {
//         Self {
//             instructions: VecDeque::with_capacity(size),
//             size,
//         }
//     }
//
//     pub fn is_empty(&self) -> bool {
//         self.instructions.is_empty()
//     }
//
//     pub fn push_back(&mut self, instruction: Instruction) {
//         if self.instructions.len() < self.size {
//             self.instructions.push_back(instruction);
//         }
//     }
//
//     pub fn pop_front(&mut self) -> Option<Instruction> {
//         self.instructions.pop_front()
//     }
//
//     // peek() pour voir la prochaine instruction sans la retirer
//     pub fn peek(&self) -> Option<&Instruction> {
//         self.instructions.front()
//     }
//
//     // methode our voir plusieurs instructions sans les retirer
//     pub fn peek_multiple(&self, count: usize) -> Vec<&Instruction> {
//         self.instructions.iter().take(count).collect()
//     }
//
//     //est pleine
//     pub fn is_full(&self) -> bool {
//         self.instructions.len() >= self.size
//     }
//
//     //vider le buffer
//     pub fn clear(&mut self) {
//         self.instructions.clear();
//     }
//
//     // methode pour connaitre la taille du buffer
//     pub fn len(&self) -> usize {
//         self.instructions.len()
//     }
//
//     //methode pour regarder a un index specifique
//     pub fn peek_at(&self, index: usize) -> Option<&Instruction> {
//         self.instructions.get(index)
//     }
//
//     // Méthode pour retirer plusieurs instructions d'un coup
//     pub fn pop_multiple(&mut self, count: usize) -> Vec<Instruction> {
//         let mut result = Vec::new();
//         for _ in 0..count {
//             if let Some(inst) = self.instructions.pop_front() {
//                 result.push(inst);
//             } else {
//                 break;
//             }
//         }
//         result
//     }
//
//     // Méthode pour insérer une instruction en tête du buffer
//     pub fn push_front(&mut self, instruction: Instruction) {
//         if !self.is_full() {
//             self.instructions.push_front(instruction);
//         }
//     }
//
//
// }
//
//
//
// impl BypassBuffer {
//     pub fn new(capacity: usize) -> Self {
//         Self {
//             entries: HashMap::with_capacity(capacity),
//             capacity,
//         }
//     }
//
//     pub fn try_bypass(&self, addr: u64) -> Option<u64> {
//         self.entries.get(&addr).and_then(|entry| {
//             if entry.valid {
//                 Some(entry.value)
//             } else {
//                 None
//             }
//         })
//     }
//
//     pub fn push_bypass(&mut self, addr: u64, value: u64) {
//         if self.entries.len() >= self.capacity {
//             if let Some(oldest_addr) = self.entries.keys().next().copied() {
//                 self.entries.remove(&oldest_addr);
//             }
//         }
//
//         self.entries.insert(addr, BypassEntry { value, valid: true });
//     }
//
//     pub fn invalidate(&mut self, addr: u64) {
//         if let Some(entry) = self.entries.get_mut(&addr) {
//             entry.valid = false;
//         }
//     }
//
//     pub fn remove(&mut self, addr: u64) {
//         self.entries.remove(&addr);
//     }
// }
//
//
//
//
//
//
// impl Pipeline{
//     pub fn execute_load(&mut self, addr: u64) -> Result<u64, PipelineError> {
//         // Vérifier d'abord le bypass buffer
//         if let Some(value) = self.bypass_buffer.try_bypass(addr) {
//             return Ok(value);
//         }
//         // Sinon, essayer le cache
//         self.memory_access(addr)
//     }
//
//     pub fn execute_store(&mut self, addr: u64, value: u64) -> Result<(), PipelineError> {
//         // Mettre à jour le bypass buffer
//         self.bypass_buffer.push_bypass(addr, value);
//
//         // Ajouter aux stores en attente
//         self.pending_stores.push_back(StoreOperation { addr, value });
//         Ok(())
//     }
//
//     pub fn commit_stores(&mut self) {
//         while let Some(store) = self.pending_stores.pop_front() {
//             // Écrire dans le cache
//             if let Err(e) = self.cache_system.write(store.addr, store.value) {
//                 println!("Erreur d'écriture dans le cache: {:?}", e);
//             }
//             // Invalider l'entrée dans le bypass
//             self.bypass_buffer.invalidate(store.addr);
//         }
//     }
//
//     fn memory_access(&mut self, addr: u64) -> Result<u64, PipelineError> {
//         self.cache_system.read(addr)
//             .map_err(|e| PipelineError::MemoryError(e.to_string()))
//     }
//
//
// }
//
//
//
// // Tests unitaires
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_bypass_buffer_basic() {
//         let mut buffer = BypassBuffer::new(4);
//
//         buffer.push_bypass(0x1000, 42);
//         assert_eq!(buffer.try_bypass(0x1000), Some(42));
//
//         buffer.invalidate(0x1000);
//         assert_eq!(buffer.try_bypass(0x1000), None);
//
//         for i in 0..5 {
//             buffer.push_bypass(i, i as u64);
//         }
//         assert_eq!(buffer.entries.len(), 4);
//     }
//
//     #[test]
//     fn test_pipeline_integration() {
//         let mut pipeline = Pipeline::new();
//
//         // Test store suivi d'un load
//         pipeline.execute_store(0x2000, 123).unwrap();
//         assert_eq!(pipeline.execute_load(0x2000).unwrap(), 123);
//
//         // Test après commit
//         pipeline.commit_stores();
//         // Vérifier dans le cache
//         assert_eq!(pipeline.cache_system.read(0x2000).unwrap(), 123);
//     }
//     #[test]
//     fn test_fetch_buffer_peek() {
//         let mut buffer = FetchBuffer::new(4);
//         assert!(buffer.peek().is_none());
//
//         let inst = Instruction::Nop;
//         buffer.push_back(inst);
//         assert!(buffer.peek().is_some());
//         assert_eq!(buffer.len(), 1);
//     }
//
//     #[test]
//     fn test_fetch_buffer_operations() {
//         let mut buffer = FetchBuffer::new(2);
//
//         // Test push_back et is_full
//         buffer.push_back(Instruction::Nop);
//         buffer.push_back(Instruction::Halt);
//         assert!(buffer.is_full());
//
//         // Test peek et pop_front
//         assert!(matches!(buffer.peek(), Some(Instruction::Nop)));
//         assert!(matches!(buffer.pop_front(), Some(Instruction::Nop)));
//         assert!(!buffer.is_full());
//     }
//
//     #[test]
//     fn test_fetch_buffer_clear() {
//         let mut buffer = FetchBuffer::new(3);
//         buffer.push_back(Instruction::Nop);
//         buffer.push_back(Instruction::Halt);
//         buffer.clear();
//         assert!(buffer.is_empty());
//         assert_eq!(buffer.len(), 0);
//     }
//
//     #[test]
//     fn test_fetch_buffer_peek_multiple() {
//         let mut buffer = FetchBuffer::new(4);
//         buffer.push_back(Instruction::Nop);
//         buffer.push_back(Instruction::Halt);
//
//         let peeked = buffer.peek_multiple(2);
//         assert_eq!(peeked.len(), 2);
//     }
// }
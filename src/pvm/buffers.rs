//src/pvm/buffers.rs

use crate::pvm::instructions::Address;
use std::collections::VecDeque;
use crate::pvm::instructions::Instruction;

use std::collections::HashMap;
use crate::pvm::pipeline_errors::PipelineError;
use crate::pvm::pipelines::Pipeline;


pub struct BypassBuffer {
    pub entries: HashMap<u64, BypassEntry>,
    pub capacity: usize,
}

#[derive(Debug)]
pub struct BypassEntry {
    pub value: u64,
    pub valid: bool,
}

#[derive(Debug)]
pub struct StoreOperation {
    pub addr: u64,
    pub value: u64,
}



#[derive(Default)]
pub struct FetchBuffer {
    pub instructions: VecDeque<Instruction>,
    pub size: usize,
}

impl FetchBuffer {
    pub fn new(size: usize) -> Self {
        Self {
            instructions: VecDeque::with_capacity(size),
            size,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }

    pub fn push_back(&mut self, instruction: Instruction) {
        if self.instructions.len() < self.size {
            self.instructions.push_back(instruction);
        }
    }

    pub fn pop_front(&mut self) -> Option<Instruction> {
        self.instructions.pop_front()
    }
}



impl BypassBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: HashMap::with_capacity(capacity),
            capacity,
        }
    }

    pub fn try_bypass(&self, addr: u64) -> Option<u64> {
        self.entries.get(&addr).and_then(|entry| {
            if entry.valid {
                Some(entry.value)
            } else {
                None
            }
        })
    }

    pub fn push_bypass(&mut self, addr: u64, value: u64) {
        if self.entries.len() >= self.capacity {
            if let Some(oldest_addr) = self.entries.keys().next().copied() {
                self.entries.remove(&oldest_addr);
            }
        }

        self.entries.insert(addr, BypassEntry { value, valid: true });
    }

    pub fn invalidate(&mut self, addr: u64) {
        if let Some(entry) = self.entries.get_mut(&addr) {
            entry.valid = false;
        }
    }

    pub fn remove(&mut self, addr: u64) {
        self.entries.remove(&addr);
    }
}






impl Pipeline{
    pub fn execute_load(&mut self, addr: u64) -> Result<u64, PipelineError> {
        // Vérifier d'abord le bypass buffer
        if let Some(value) = self.bypass_buffer.try_bypass(addr) {
            return Ok(value);
        }
        // Sinon, essayer le cache
        self.memory_access(addr)
    }

    pub fn execute_store(&mut self, addr: u64, value: u64) -> Result<(), PipelineError> {
        // Mettre à jour le bypass buffer
        self.bypass_buffer.push_bypass(addr, value);

        // Ajouter aux stores en attente
        self.pending_stores.push_back(StoreOperation { addr, value });
        Ok(())
    }

    pub fn commit_stores(&mut self) {
        while let Some(store) = self.pending_stores.pop_front() {
            // Écrire dans le cache
            if let Err(e) = self.cache_system.write(store.addr, store.value) {
                println!("Erreur d'écriture dans le cache: {:?}", e);
            }
            // Invalider l'entrée dans le bypass
            self.bypass_buffer.invalidate(store.addr);
        }
    }

    fn memory_access(&mut self, addr: u64) -> Result<u64, PipelineError> {
        self.cache_system.read(addr)
            .map_err(|e| PipelineError::MemoryError(e.to_string()))
    }


}



// Tests unitaires
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bypass_buffer_basic() {
        let mut buffer = BypassBuffer::new(4);

        buffer.push_bypass(0x1000, 42);
        assert_eq!(buffer.try_bypass(0x1000), Some(42));

        buffer.invalidate(0x1000);
        assert_eq!(buffer.try_bypass(0x1000), None);

        for i in 0..5 {
            buffer.push_bypass(i, i as u64);
        }
        assert_eq!(buffer.entries.len(), 4);
    }

    #[test]
    fn test_pipeline_integration() {
        let mut pipeline = Pipeline::new();

        // Test store suivi d'un load
        pipeline.execute_store(0x2000, 123).unwrap();
        assert_eq!(pipeline.execute_load(0x2000).unwrap(), 123);

        // Test après commit
        pipeline.commit_stores();
        // Vérifier dans le cache
        assert_eq!(pipeline.cache_system.read(0x2000).unwrap(), 123);
    }
}
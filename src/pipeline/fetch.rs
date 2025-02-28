//src/pipeline/fetch.rs


use std::collections::VecDeque;
use crate::bytecode::instructions::Instruction;
use crate::pipeline::{FetchDecodeRegister, stage::PipelineStage};

/// implementation de l'étage Fetch du pipeline
pub struct  FetchStage{
    fetch_buffer: VecDeque<(u32, Instruction)>,
    buffer_size: usize,
}

impl FetchStage {
    /// Crée un nouvel étage Fetch
    pub fn new(buffer_size: usize) -> Self {
        Self {
            fetch_buffer: VecDeque::with_capacity(buffer_size),
            buffer_size,
        }
    }

    /// Précharge des instructions dans le buffer
    fn prefetch(&mut self, pc: u32, instructions: &[Instruction]) {
        // Si le buffer est déjà plein, ne rien faire
        if self.fetch_buffer.len() >= self.buffer_size {
            return;
        }

        // Trouver l'index de l'instruction à l'adresse PC
        let mut current_index = 0;
        let mut current_addr = 0;

        for (idx, instr) in instructions.iter().enumerate() {
            if current_addr == pc {
                current_index = idx;
                break;
            }
            current_addr += instr.total_size() as u32;
        }

        // Précharger les instructions suivantes
        let mut addr = pc;
        for idx in current_index..instructions.len() {
            if self.fetch_buffer.len() >= self.buffer_size {
                break;
            }

            // Ne pas ajouter l'instruction si elle est déjà dans le buffer
            if !self.fetch_buffer.iter().any(|(a, _)| *a == addr) {
                self.fetch_buffer.push_back((addr, instructions[idx].clone()));
            }

            addr += instructions[idx].total_size() as u32;
        }
    }

    // Traite l'étage Fetch
    pub fn process(&mut self, pc: u32, instructions: &[Instruction]) -> Result<FetchDecodeRegister, String> {
        // Si le buffer est vide ou ne contient pas l'instruction à PC, le remplir
        if self.fetch_buffer.is_empty() || !self.fetch_buffer.iter().any(|(addr, _)| *addr == pc) {
            self.fetch_buffer.clear();
            self.prefetch(pc, instructions);
        }

        // Récupérer l'instruction à l'adresse PC
        if let Some(idx) = self.fetch_buffer.iter().position(|(addr, _)| *addr == pc) {
            let (_, instruction) = self.fetch_buffer.remove(idx).unwrap();

            // Précharger davantage d'instructions si nécessaire
            self.prefetch(pc + instruction.total_size() as u32, instructions);

            Ok(FetchDecodeRegister {
                instruction,
                pc,
            })
        } else {
            Err(format!("Instruction non trouvée à l'adresse 0x{:08X}", pc))
        }
    }

    /// Réinitialise l'étage Fetch
    pub fn reset(&mut self) {
        self.fetch_buffer.clear();
    }

}

impl<'a> PipelineStage<'a> for FetchStage {
    type Input = (u32, &'a [Instruction]);
    type Output = FetchDecodeRegister;

    fn process(&mut self, input: &Self::Input) -> Result<Self::Output, String> {
        let (pc, instructions) = *input;
        self.process(pc, instructions)
    }

    fn reset(&mut self) {
        // Reset direct sans appel récursif
        self.fetch_buffer.clear();
    }
}
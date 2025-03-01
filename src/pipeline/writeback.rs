//src/pipeline/writeback.rs

use crate::pipeline::MemoryWritebackRegister;
// use crate::pipeline::stage::PipelineStage;



pub struct WritebackStage {
    // pub wb_buffer: Vec<usize>,
    //pas d'etat interne
}



impl WritebackStage{
    /// Crée un nouvel étage Writeback
    pub fn new() -> Self {
        Self {}
    }

    /// Traite l'étage Writeback directement
    pub fn process_direct(&mut self, wb_reg: &MemoryWritebackRegister, registers: &mut [u64]) -> Result<(), String> {
        // Si un registre destination est spécifié, y écrire le résultat
        if let Some(rd) = wb_reg.rd {
            if rd < registers.len() {
                registers[rd] = wb_reg.result;
            } else {
                return Err(format!("Registre destination invalide: R{}", rd));
            }
        }

        Ok(())
    }

    // /// Traite l'étage Writeback
    // pub fn process(&mut self, wb_reg: &MemoryWritebackRegister, registers: &mut [u64]) -> Result<(), String> {
    //     // Si un registre destination est spécifié, y écrire le résultat
    //     if let Some(rd) = wb_reg.rd {
    //         if rd < registers.len() {
    //             registers[rd] = wb_reg.result;
    //         } else {
    //             return Err(format!("Registre destination invalide: R{}", rd));
    //         }
    //     }
    //
    //     Ok(())
    // }

    /// Réinitialise l'étage Writeback
    pub fn reset(&mut self) {
        self.reset();
    }

}



// impl<'a> PipelineStage<'a> for WritebackStage {
//     type Input = (MemoryWritebackRegister, &'a mut [u64]);
//     type Output = ();
//
//     fn process(&mut self, input: &Self::Input) -> Result<Self::Output, String> {
//         let (wb_reg, registers) = input;
//         self.process(wb_reg, registers)
//     }
//
//     fn reset(&mut self) {
//         // Pas d'état à réinitialiser
//     }
// }
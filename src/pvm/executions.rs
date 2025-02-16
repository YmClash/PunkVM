use crate::pvm::instructions::{ArithmeticOp, ControlOp, MemoryOp};
use crate::pvm::pipelines::StatusFlags;
use crate::pvm::vm::PunkVM;
use crate::pvm::vm_errors::{VMError, VMResult};

impl PunkVM {

    /// Exécute une Operation Arithmetique
    pub fn execute_arithmetic(&mut self, op: &ArithmeticOp) -> VMResult<()> {
        match op {
            ArithmeticOp::Add { dest, src1, src2 } => {
                let val1 = self.register_bank.read_register(*src1)? as i64;
                let val2 = self.register_bank.read_register(*src2)? as i64;
                let result = val1.checked_add(val2)
                    .ok_or_else(|| VMError::ArithmeticError("Addition overflow".into()))?;

                let flags = StatusFlags {
                    zero: result == 0,
                    negative: result < 0,
                    overflow: false, // Mis à jour si overflow détecté
                };

                self.register_bank.write_register(*dest, result)?;
                self.register_bank.update_flags(flags)?;
            }
            ArithmeticOp::Sub { dest, src1, src2 } => {
                let val1 = self.register_bank.read_register(*src1)? as i64;
                let val2 = self.register_bank.read_register(*src2)? as i64;
                let result = val1.checked_sub(val2)
                    .ok_or_else(|| VMError::ArithmeticError("Subtraction overflow".into()))?;

                let flags = StatusFlags {
                    zero: result == 0,
                    negative: result < 0,
                    overflow: false,
                };

                self.register_bank.write_register(*dest, result)?;
                self.register_bank.update_flags(flags)?;
            }
            ArithmeticOp::Mul { dest, src1, src2 } => {
                let val1 = self.register_bank.read_register(*src1)? as i64;
                let val2 = self.register_bank.read_register(*src2)? as i64;
                let result = val1.checked_mul(val2)
                    .ok_or_else(|| VMError::ArithmeticError("Multiplication overflow".into()))?;

                let flags = StatusFlags {
                    zero: result == 0,
                    negative: result < 0,
                    overflow: false,
                };

                self.register_bank.write_register(*dest, result)?;
                self.register_bank.update_flags(flags)?;
            }
            ArithmeticOp::Div { dest, src1, src2 } => {
                let val1 = self.register_bank.read_register(*src1)? as i64;
                let val2 = self.register_bank.read_register(*src2)? as i64;

                if val2 == 0 {
                    return Err(VMError::ArithmeticError("Division by zero".into()));
                }

                let result = val1.checked_div(val2)
                    .ok_or_else(|| VMError::ArithmeticError("Division overflow".into()))?;

                let flags = StatusFlags {
                    zero: result == 0,
                    negative: result < 0,
                    overflow: false,
                };

                self.register_bank.write_register(*dest, result)?;
                self.register_bank.update_flags(flags)?;
            }
        }
        Ok(())
    }

    /// Exécute une opération mémoire
    pub fn execute_memory(&mut self, op: &MemoryOp) -> VMResult<()> {
        match op {
            MemoryOp::Load { reg, addr } => {
                let value = self.memory_controller.read(addr.0)?;
                self.register_bank.write_register(*reg, value as i64)?;
            }
            MemoryOp::Store { reg, addr } => {
                let value = self.register_bank.read_register(*reg)?;
                self.memory_controller.write(addr.0, value as u64)?;
            }
            MemoryOp::Move { dest, src } => {
                let value = self.register_bank.read_register(*src)?;
                self.register_bank.write_register(*dest, value as i64)?;
            }
            MemoryOp::LoadImm { reg, value } => {
                self.register_bank.write_register(*reg, *value)?;
            }
        }
        Ok(())
    }

    /// Exécute une opération de contrôle
    pub fn execute_control(&mut self, op: &ControlOp) -> VMResult<()> {
        match op {
            ControlOp::Jump { addr } => {
                // Mettre à jour le PC
                self.set_program_counter(addr.0)?;
            }
            ControlOp::JumpIf { condition, addr } => {
                let cond_value = self.register_bank.read_register(*condition)?;
                if cond_value != 0 {
                    self.set_program_counter(addr.0)?;
                }
            }
            ControlOp::Call { addr } => {
                // Sauvegarder le PC actuel sur la pile
                let current_pc = self.get_program_counter()?;
                self.push_stack(current_pc)?;
                self.set_program_counter(addr.0)?;
            }
            ControlOp::Return => {
                // Récupérer l'adresse de retour depuis la pile
                let return_addr = self.pop_stack()?;
                self.set_program_counter(return_addr)?;
            }
            ControlOp::Nop => {
                // Ne rien faire
            }
            ControlOp::Halt => {
                // Arrêter l'exécution
                self.halt()?;
            }
        }
        Ok(())
    }

    // Méthodes auxiliaires pour le contrôle de flux
    fn set_program_counter(&mut self, addr: u64) -> VMResult<()> {
        // À implémenter
        Ok(())
    }

    fn get_program_counter(&self) -> VMResult<u64> {
        // À implémenter
        Ok(0)
    }

    fn push_stack(&mut self, value: u64) -> VMResult<()> {
        // À implémenter
        Ok(())
    }

    fn pop_stack(&mut self) -> VMResult<u64> {
        // À implémenter
        Ok(0)
    }

    fn halt(&mut self) -> VMResult<()> {
        // À implémenter
        Ok(())
    }



}
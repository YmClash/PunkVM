//src/pvm/stacks.rs


use crate::PunkVM;
//src/pvm/stacks.rs
use crate::pvm::vm_errors::{VMError, VMResult};

#[derive(Debug, Clone, Copy)]
pub struct StackStats {
    pub pushes: u64,
    pub pops: u64,
    pub max_depth: usize,
    pub current_depth: usize,
    pub overflow_attempts: u64,
    pub underflow_attempts: u64,
}

impl StackStats {
    pub fn new() -> Self {
        Self {
            pushes: 0,
            pops: 0,
            max_depth: 0,
            current_depth: 0,
            overflow_attempts: 0,
            underflow_attempts: 0,
        }
    }
    
    pub fn reset(&mut self) {
        self.pushes = 0;
        self.pops = 0;
        self.max_depth = 0;
        self.current_depth = 0;
        self.overflow_attempts = 0;
        self.underflow_attempts = 0;
    }
}





/// Registres Speciaux pour le processus de la pile  Stack
#[derive(Debug,Clone,Copy)]
pub enum SpecialRegister {
    SP = 16, // Stack Pointer
    Bp = 17, // Base Pointer
    RA = 18, // Return Address
}

pub struct Stack{
    data: Vec<u64>,
    stack_pointer: usize,
    max_size: usize,
}


pub struct ReturnAddressStack {
    stack: Vec<u64>,
    top: usize,
    size: usize,
}
impl ReturnAddressStack {
    pub fn new(size: usize) -> Self {
        Self {
            stack: vec![0; size],
            top: 0,
            size,
        }
    }

    pub fn push(&mut self, addr: u64) {
        if self.top < self.size {
            self.stack[self.top] = addr;
            self.top += 1;
        }
    }

    pub fn pop(&mut self) -> Option<u64> {
        if self.top > 0 {
            self.top -= 1;
            Some(self.stack[self.top])
        } else {
            None
        }
    }

    pub fn peek(&self) -> Option<u64> {
        if self.top > 0 {
            Some(self.stack[self.top - 1])
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.top = 0;
    }

    pub fn is_empty(&self) -> bool {
        self.top == 0
    }

    pub fn is_full(&self) -> bool {
        self.top == self.size
    }
}


impl PunkVM {
    pub fn init_stack(&mut self) {
        let sp_index = SpecialRegister::SP as usize;
        self.registers[sp_index] = self.config.stack_base as u64 + self.config.stack_size as u64;

        // Réinitialiser les statistiques de pile
        self.stack_stats.reset();

        println!("Stack initialized: SP = 0x{:08X}", self.registers[sp_index]);
    }

    /// Acces au Stack
    pub fn get_sp(&self) -> u64 {
        self.registers[SpecialRegister::SP as usize]
    }

    /// Modification du Stack Pointer
    pub fn set_sp(&mut self,value:u64) {
        self.registers[SpecialRegister::SP as usize] = value
    }
    /// Push une valeur sur la pile
    pub fn push_stack(&mut self, value: u64) -> Result<(), String> {
        {
            let sp = self.get_sp();

            // Verifier la limite de la pile
            if sp < self.config.stack_base as u64 + 8 {
                self.stack_stats.overflow_attempts += 1;
                return Err("Stack overflow".to_string());
            }

            // Decrementer le Stack Pointer
            let new_sp = sp - 8;
            self.set_sp(new_sp);


            // Ecrire la  valeur  dans la memoire
            self.memory.write_qword(new_sp as u32, value)
                .map_err(|e| format!("Stack push error: {}", e))?;

            // Mettre à jour les statistiques
            self.stack_stats.pushes += 1;
            self.stack_stats.current_depth += 1;
            if self.stack_stats.current_depth > self.stack_stats.max_depth {
                self.stack_stats.max_depth = self.stack_stats.current_depth;
            }

            println!("Stack PUSH: value=0x{:016X}, SP=0x{:08X}", value, new_sp);
            Ok(())
        }
    }


    pub fn pop_stack(&mut self) -> Result<u64, String>{
        let sp = self.get_sp();

        // Verifier la limite de la pile
        if sp >= self.config.stack_base as u64 + self.config.stack_size as u64 {
            self.stack_stats.underflow_attempts += 1;
            return Err("Stack underflow".to_string());
        }

        // lire la valeur en memoire
        let value = self.memory.read_qword(sp as u32)
            .map_err(|e| format!("Stack pop error: {}", e))?;

        // Incrementer le Stack Pointer
        let new_sp = sp +8;
        self.set_sp(new_sp);

        // Mettre à jour les statistiques
        self.stack_stats.pops += 1;
        if self.stack_stats.current_depth > 0 {
            self.stack_stats.current_depth -= 1;
        }

        println!("Stack POP: value=0x{:016X}, SP=0x{:08X}", value, new_sp);
        Ok(value)
    }
    
    /// Retourne les statistiques de la pile
    pub fn get_stack_stats(&self) -> StackStats {
        self.stack_stats
    }
}






impl Stack {
    /// Crée une nouvelle instance de la pile avec la taille maximale spécifiée
    pub fn new(max_size: usize) -> VMResult<Self> {
        if max_size == 0 {
            return Err(VMError::ConfigError("Taille de pile invalide".into()));
        }

        Ok(Self {
            data: vec![0; max_size],
            stack_pointer: 0,
            max_size,
        })
    }

    pub fn push(&mut self, value: u64) -> VMResult<()> {
        if self.stack_pointer >= self.max_size {
            return Err(VMError::ExecutionError("Stack overflow".into()));
        }

        self.data[self.stack_pointer] = value;
        self.stack_pointer += 1;
        Ok(())

    }

    pub fn pop(&mut self) -> VMResult<u64> {
        if self.stack_pointer == 0 {
            return Err(VMError::ExecutionError("Stack underflow".into()));
        }

        self.stack_pointer -= 1;
        Ok(self.data[self.stack_pointer])
    }

    pub fn peek(&self) -> VMResult<u64> {
        if self.stack_pointer == 0 {
            return Err(VMError::ExecutionError("Stack est vide".into()));
        }

        Ok(self.data[self.stack_pointer - 1])
    }

    pub fn reset(&mut self) -> VMResult<()> {
        self.data.fill(0);
        self.stack_pointer = 0;
        Ok(())
    }

    pub fn depth(&self) -> usize {
        self.stack_pointer
    }

    pub fn is_empty(&self) -> bool {
        self.stack_pointer == 0
    }

    pub fn is_full(&self) -> bool {
        self.stack_pointer == self.max_size
    }


}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_creation() {
        let stack = Stack::new(64);
        assert!(stack.is_ok());
        let stack = stack.unwrap();
        assert_eq!(stack.depth(), 0);
        assert!(stack.is_empty());
    }

    #[test]
    fn test_push_pop() {
        let mut stack = Stack::new(64).unwrap();

        // Test push
        assert!(stack.push(42).is_ok());
        assert_eq!(stack.depth(), 1);

        // Test pop
        assert_eq!(stack.pop().unwrap(), 42);
        assert!(stack.is_empty());
    }

    #[test]
    fn test_stack_overflow() {
        let mut stack = Stack::new(2).unwrap();

        assert!(stack.push(1).is_ok());
        assert!(stack.push(2).is_ok());
        assert!(stack.push(3).is_err());
    }

    #[test]
    fn test_stack_underflow() {
        let mut stack = Stack::new(64).unwrap();
        assert!(stack.pop().is_err());
    }

    #[test]
    fn test_peek() {
        let mut stack = Stack::new(64).unwrap();

        stack.push(42).unwrap();
        assert_eq!(stack.peek().unwrap(), 42);
        assert_eq!(stack.depth(), 1); // Peek ne devrait pas modifier la pile
    }
}

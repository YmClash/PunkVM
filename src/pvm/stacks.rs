// //src/pvm/stacks.rs
// use crate::pvm::vm_errors::{VMError, VMResult};
//
// pub struct Stack{
//     data: Vec<u64>,
//     stack_pointer: usize,
//     max_size: usize,
// }
//
//
// pub struct ReturnAddressStack {
//     stack: Vec<u64>,
//     top: usize,
//     size: usize,
// }
// impl ReturnAddressStack {
//     pub fn new(size: usize) -> Self {
//         Self {
//             stack: vec![0; size],
//             top: 0,
//             size,
//         }
//     }
//
//     pub fn push(&mut self, addr: u64) {
//         if self.top < self.size {
//             self.stack[self.top] = addr;
//             self.top += 1;
//         }
//     }
//
//     pub fn pop(&mut self) -> Option<u64> {
//         if self.top > 0 {
//             self.top -= 1;
//             Some(self.stack[self.top])
//         } else {
//             None
//         }
//     }
//
//     pub fn peek(&self) -> Option<u64> {
//         if self.top > 0 {
//             Some(self.stack[self.top - 1])
//         } else {
//             None
//         }
//     }
//
//     pub fn clear(&mut self) {
//         self.top = 0;
//     }
//
//     pub fn is_empty(&self) -> bool {
//         self.top == 0
//     }
//
//     pub fn is_full(&self) -> bool {
//         self.top == self.size
//     }
// }
//
//
// impl Stack {
//     /// Crée une nouvelle instance de la pile avec la taille maximale spécifiée
//     pub fn new(max_size: usize) -> VMResult<Self> {
//         if max_size == 0 {
//             return Err(VMError::ConfigError("Taille de pile invalide".into()));
//         }
//
//         Ok(Self {
//             data: vec![0; max_size],
//             stack_pointer: 0,
//             max_size,
//         })
//     }
//
//     pub fn push(&mut self, value: u64) -> VMResult<()> {
//         if self.stack_pointer >= self.max_size {
//             return Err(VMError::ExecutionError("Stack overflow".into()));
//         }
//
//         self.data[self.stack_pointer] = value;
//         self.stack_pointer += 1;
//         Ok(())
//
//     }
//
//     pub fn pop(&mut self) -> VMResult<u64> {
//         if self.stack_pointer == 0 {
//             return Err(VMError::ExecutionError("Stack underflow".into()));
//         }
//
//         self.stack_pointer -= 1;
//         Ok(self.data[self.stack_pointer])
//     }
//
//     pub fn peek(&self) -> VMResult<u64> {
//         if self.stack_pointer == 0 {
//             return Err(VMError::ExecutionError("Stack est vide".into()));
//         }
//
//         Ok(self.data[self.stack_pointer - 1])
//     }
//
//     pub fn reset(&mut self) -> VMResult<()> {
//         self.data.fill(0);
//         self.stack_pointer = 0;
//         Ok(())
//     }
//
//     pub fn depth(&self) -> usize {
//         self.stack_pointer
//     }
//
//     pub fn is_empty(&self) -> bool {
//         self.stack_pointer == 0
//     }
//
//     pub fn is_full(&self) -> bool {
//         self.stack_pointer == self.max_size
//     }
//
//
// }
//
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_stack_creation() {
//         let stack = Stack::new(64);
//         assert!(stack.is_ok());
//         let stack = stack.unwrap();
//         assert_eq!(stack.depth(), 0);
//         assert!(stack.is_empty());
//     }
//
//     #[test]
//     fn test_push_pop() {
//         let mut stack = Stack::new(64).unwrap();
//
//         // Test push
//         assert!(stack.push(42).is_ok());
//         assert_eq!(stack.depth(), 1);
//
//         // Test pop
//         assert_eq!(stack.pop().unwrap(), 42);
//         assert!(stack.is_empty());
//     }
//
//     #[test]
//     fn test_stack_overflow() {
//         let mut stack = Stack::new(2).unwrap();
//
//         assert!(stack.push(1).is_ok());
//         assert!(stack.push(2).is_ok());
//         assert!(stack.push(3).is_err());
//     }
//
//     #[test]
//     fn test_stack_underflow() {
//         let mut stack = Stack::new(64).unwrap();
//         assert!(stack.pop().is_err());
//     }
//
//     #[test]
//     fn test_peek() {
//         let mut stack = Stack::new(64).unwrap();
//
//         stack.push(42).unwrap();
//         assert_eq!(stack.peek().unwrap(), 42);
//         assert_eq!(stack.depth(), 1); // Peek ne devrait pas modifier la pile
//     }
// }
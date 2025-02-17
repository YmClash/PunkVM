use std::collections::HashMap;
use crate::pvm::instructions::{RegisterId, DecodedInstruction, ArithmeticOp, MemoryOp};
use crate::pvm::vm_errors::VMResult;
use crate::pvm::pipelines::ExecutionResult;




/// Représente une source de forwarding
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ForwardingSource {
    Execute,
    Memory,
    Writeback,
}


/// Structure contenant les informations de forwarding pour un registre
#[derive(Debug, Clone)]
pub struct ForwardingInfo {
    pub source: ForwardingSource,
    pub value: i64,
}

/// unite de forwarding ameliorès
pub struct ForwardingUnit {
    // Table de correspondance entre registres et leurs valeurs forwardées
    pub forward_table: HashMap<RegisterId, ForwardingInfo>,
}



impl ForwardingUnit{
    /// crée une nouvelle table de forwarding
    pub fn new() -> Self {
        Self {
            forward_table: HashMap::new(),
        }
    }

    /// Enregistre une valeur pour le forwarding
    pub fn register_result(&mut self, dest: RegisterId, result: &ExecutionResult, source: ForwardingSource) {
        self.forward_table.insert(dest, ForwardingInfo {
            source,
            value: result.value,
        });
    }

    /// Nettoie les entrées du forwarding pour un étage spécifique
    pub fn clear_stage(&mut self, stage: ForwardingSource) {
        self.forward_table.retain(|_, info| info.source != stage);
    }

    /// Vérifie si une valeur est disponible pour le forwarding
    pub fn get_forwarded_value(&self, reg: RegisterId) -> Option<i64> {
        self.forward_table.get(&reg).map(|info| info.value)
    }

    /// Détermine les dépendances de données pour une instruction
    pub fn check_dependencies(&self, decoded: &DecodedInstruction) -> Vec<RegisterId> {
        let mut dependencies = Vec::new();

        match decoded {
            DecodedInstruction::Arithmetic(op) => {
                match op {
                    ArithmeticOp::Add { src1, src2, .. } |
                    ArithmeticOp::Sub { src1, src2, .. } |
                    ArithmeticOp::Mul { src1, src2, .. } |
                    ArithmeticOp::Div { src1, src2, .. } => {
                        dependencies.push(*src1);
                        dependencies.push(*src2);
                    }
                }
            }
            DecodedInstruction::Memory(op) => {
                match op {
                    MemoryOp::Store { reg, .. } |
                    MemoryOp::Move { src: reg, .. } => {
                        dependencies.push(*reg);
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        dependencies
    }

}


#[cfg(test)]
mod tests {
    use crate::pvm::pipelines::StatusFlags;
    use super::*;

    #[test]
    fn test_forwarding_basic() {
        let mut forwarding = ForwardingUnit::new();
        let reg = RegisterId(1);
        let result = ExecutionResult {
            value: 42,
            flags: StatusFlags::default(),
        };

        forwarding.register_result(reg, &result, ForwardingSource::Execute);
        assert_eq!(forwarding.get_forwarded_value(reg), Some(42));
    }

    #[test]
    fn test_forwarding_clear_stage() {
        let mut forwarding = ForwardingUnit::new();
        let reg1 = RegisterId(1);
        let reg2 = RegisterId(2);

        forwarding.register_result(reg1,
                                   &ExecutionResult { value: 42, flags: StatusFlags::default() },
                                   ForwardingSource::Execute);
        forwarding.register_result(reg2,
                                   &ExecutionResult { value: 24, flags: StatusFlags::default() },
                                   ForwardingSource::Memory);

        forwarding.clear_stage(ForwardingSource::Execute);
        assert_eq!(forwarding.get_forwarded_value(reg1), None);
        assert_eq!(forwarding.get_forwarded_value(reg2), Some(24));
    }

    #[test]
    fn test_dependency_detection() {
        let forwarding = ForwardingUnit::new();
        let instruction = DecodedInstruction::Arithmetic(ArithmeticOp::Add {
            dest: RegisterId(1),
            src1: RegisterId(2),
            src2: RegisterId(3),
        });

        let deps = forwarding.check_dependencies(&instruction);
        assert_eq!(deps.len(), 2);
        assert!(deps.contains(&RegisterId(2)));
        assert!(deps.contains(&RegisterId(3)));
    }


}
use crate::pvm::vm_errors::VMResult;

/// Décodeur d'instructions
pub struct InstructionDecoder;


/// Instruction décodée prête à être exécutée
#[derive(Debug,Copy,Clone)]
pub enum DecodedInstruction {
    Arithmetic(ArithmeticOp),
    Memory(MemoryOp),
    Control(ControlOp),
}

/// Adresse mémoire
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Address(pub u64);


/// Types d'instructions supportés
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction{

    // Instructions arithmétiques
    Add(RegisterId, RegisterId, RegisterId),    // add r1, r2, r3
    Sub(RegisterId, RegisterId, RegisterId),    // sub r1, r2, r3
    Mul(RegisterId, RegisterId, RegisterId),    // mul r1, r2, r3
    Div(RegisterId, RegisterId, RegisterId),    // div r1, r2, r3

    // Instructions de manipulation de registres
    Load(RegisterId, Address),                  // load r1, addr
    Store(RegisterId, Address),                 // store r1, addr
    Move(RegisterId, RegisterId),               // move r1, r2
    LoadImm(RegisterId, i64),                  // loadimm r1, value

    // Instructions de saut

    Jump(Address),                              // jump addr
    JumpIf(RegisterId, Address),                // jumpif r1, addr
    Call(Address),                              // call addr
    Return,                                    // return

    // Instructions spéciales

    Nop,                                       // nop
    Halt,                                      // halt
}


#[derive(Debug, Clone,Copy)]
pub enum ArithmeticOp {
    Add { dest: RegisterId, src1: RegisterId, src2: RegisterId },
    Sub { dest: RegisterId, src1: RegisterId, src2: RegisterId },
    Mul { dest: RegisterId, src1: RegisterId, src2: RegisterId },
    Div { dest: RegisterId, src1: RegisterId, src2: RegisterId },
}

#[derive(Debug, Clone,Copy)]
pub enum MemoryOp {
    Load { reg: RegisterId, addr: Address },
    Store { reg: RegisterId, addr: Address },
    Move { dest: RegisterId, src: RegisterId },
    LoadImm { reg: RegisterId, value: i64 },
}


#[derive(Debug, Clone,Copy)]
pub enum ControlOp {
    Jump { addr: Address },
    JumpIf { condition: RegisterId, addr: Address },
    Call { addr: Address },
    Return,
    Nop,
    Halt,
}


/// Identifiant de registre
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegisterId(pub u8);

#[derive(Debug)]
enum Stage {
    Fetch,
    Decode,
    Execute,
    Memory,
    Writeback,
}






impl InstructionDecoder {
    pub fn new() -> Self {
        Self
    }

    pub fn decode(&self, instruction: Instruction) -> VMResult<DecodedInstruction> {
        // À implémenter
        todo!()
    }
}
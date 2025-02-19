use crate::pvm::vm_errors::VMResult;

/// Décodeur d'instructions
pub struct InstructionDecoder;


/// Instruction décodée prête à être exécutée
#[derive(Debug,Copy,Clone, PartialEq)]
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


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArithmeticOp {
    Add { dest: RegisterId, src1: RegisterId, src2: RegisterId },
    Sub { dest: RegisterId, src1: RegisterId, src2: RegisterId },
    Mul { dest: RegisterId, src1: RegisterId, src2: RegisterId },
    Div { dest: RegisterId, src1: RegisterId, src2: RegisterId },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryOp {
    Load { reg: RegisterId, addr: Address },
    Store { reg: RegisterId, addr: Address },
    Move { dest: RegisterId, src: RegisterId },
    LoadImm { reg: RegisterId, value: i64 },
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ControlOp {
    Jump { addr: Address },
    JumpIf { condition: RegisterId, addr: Address },
    Call { addr: Address },
    Return,
    Nop,
    Halt,
}


/// Identifiant de registre
#[derive(Debug, Clone, Copy, PartialEq, Eq,Hash)]
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


impl From<DecodedInstruction> for Instruction {
    fn from(decoded: DecodedInstruction) -> Self {
        match decoded {
            DecodedInstruction::Arithmetic(op) => match op {
                ArithmeticOp::Add { dest, src1, src2 } => Instruction::Add(dest, src1, src2),
                ArithmeticOp::Sub { dest, src1, src2 } => Instruction::Sub(dest, src1, src2),
                ArithmeticOp::Mul { dest, src1, src2 } => Instruction::Mul(dest, src1, src2),
                ArithmeticOp::Div { dest, src1, src2 } => Instruction::Div(dest, src1, src2),
            },
            DecodedInstruction::Memory(op) => match op {
                MemoryOp::LoadImm { reg, value } => Instruction::LoadImm(reg, value),
                MemoryOp::Load { reg, addr } => Instruction::Load(reg, addr),
                MemoryOp::Store { reg, addr } => Instruction::Store(reg, addr),
                MemoryOp::Move { dest, src } => Instruction::Move(dest, src),
            },
            DecodedInstruction::Control(op) => match op {
                ControlOp::Jump { addr } => Instruction::Jump(addr),
                ControlOp::JumpIf { condition, addr } => Instruction::JumpIf(condition, addr),
                ControlOp::Call { addr } => Instruction::Call(addr),
                ControlOp::Return => Instruction::Return,
                ControlOp::Halt => Instruction::Halt,
                ControlOp::Nop => Instruction::Nop,
            },
        }
    }
}


// Optionnellement, on peut aussi implémenter From<Instruction> pour DecodedInstruction
impl From<Instruction> for DecodedInstruction {
    fn from(instruction: Instruction) -> Self {
        match instruction {
            Instruction::Add(dest, src1, src2) =>
                DecodedInstruction::Arithmetic(ArithmeticOp::Add { dest, src1, src2 }),
            Instruction::Sub(dest, src1, src2) =>
                DecodedInstruction::Arithmetic(ArithmeticOp::Sub { dest, src1, src2 }),
            Instruction::Mul(dest, src1, src2) =>
                DecodedInstruction::Arithmetic(ArithmeticOp::Mul { dest, src1, src2 }),
            Instruction::Div(dest, src1, src2) =>
                DecodedInstruction::Arithmetic(ArithmeticOp::Div { dest, src1, src2 }),
            Instruction::Load(reg, addr) =>
                DecodedInstruction::Memory(MemoryOp::Load { reg, addr }),
            Instruction::Store(reg, addr) =>
                DecodedInstruction::Memory(MemoryOp::Store { reg, addr }),
            Instruction::Move(dest, src) =>
                DecodedInstruction::Memory(MemoryOp::Move { dest, src }),
            Instruction::LoadImm(reg, value) =>
                DecodedInstruction::Memory(MemoryOp::LoadImm { reg, value }),
            Instruction::Jump(addr) =>
                DecodedInstruction::Control(ControlOp::Jump { addr }),
            Instruction::JumpIf(condition, addr) =>
                DecodedInstruction::Control(ControlOp::JumpIf { condition, addr }),
            Instruction::Call(addr) =>
                DecodedInstruction::Control(ControlOp::Call { addr }),
            Instruction::Return =>
                DecodedInstruction::Control(ControlOp::Return),
            Instruction::Halt =>
                DecodedInstruction::Control(ControlOp::Halt),
            Instruction::Nop =>
                DecodedInstruction::Control(ControlOp::Nop),
        }
    }
}


#[test]
fn test_instruction_conversion() {
    let original = Instruction::Add(RegisterId(0), RegisterId(1), RegisterId(2));
    let original_clone = original.clone(); // Clone l'instruction originale
    let decoded: DecodedInstruction = original.into();
    let back: Instruction = decoded.into();
    assert_eq!(original_clone, back);
}
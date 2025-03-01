// //src/pvm/instructions.rs
//
// use crate::pvm::vm_errors::VMResult;
//
//
// #[allow(dead_code)]
// #[derive(Debug,Clone,Copy,PartialEq)]
// pub enum Bytecode{
//     Header(Header),
//     Version(Version),
//     Metadata(Metadata),
//     Segment(Segment),
//     Instruction(Opcode),
//
// }
//
// #[allow(dead_code)]
// #[derive(Debug,Clone,Copy,PartialEq)]
// pub enum Opcode{
//
//
// }
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
// ///////////////////////////////////////////////////////////
// //
// //
// // //src/pvm/instructions.rs
// // use crate::pvm::vm_errors::VMResult;
// // use crate::pvm::registers::RegisterBank;
// //
// // /// Décodeur d'instructions
// // pub struct InstructionDecoder;
// //
// //
// // /// Instruction décodée prête à être exécutée
// // #[derive(Debug,Copy,Clone, PartialEq)]
// // pub enum DecodedInstruction {
// //     Arithmetic(ArithmeticOp),
// //     Memory(MemoryOp),
// //     Control(ControlOp),
// //     Branch(BranchOp),
// //     Compare{src1: RegisterId, src2: RegisterId},
// // }
// //
// // /// Adresse mémoire
// // #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// // pub struct Address(pub u64);
// //
// //
// // /// Types d'instructions supportés
// // #[derive(Debug, Clone, PartialEq)]
// // pub enum Instruction{
// //
// //     // Instructions arithmétiques
// //     Add(RegisterId, RegisterId, RegisterId),    // add r1, r2, r3
// //     Sub(RegisterId, RegisterId, RegisterId),    // sub r1, r2, r3
// //     Mul(RegisterId, RegisterId, RegisterId),    // mul r1, r2, r3
// //     Div(RegisterId, RegisterId, RegisterId),    // div r1, r2, r3
// //
// //     // Instructions de manipulation de registres
// //     Load(RegisterId, Address),                  // load r1, addr
// //     Store(RegisterId, Address),                 // store r1, addr
// //     Move(RegisterId, RegisterId),               // move r1, r2
// //     LoadImm(RegisterId, i64),                  // loadimm r1, value
// //
// //     // Instructions de saut
// //
// //     Jump(Address),                              // jump addr
// //     JumpIf(RegisterId, Address),                // jumpif r1, addr
// //     Call(Address),                              // call addr
// //     Return,                                    // return
// //
// //     // Instructions spéciales
// //
// //     Nop,                                       // nop
// //     Halt,                                      // halt
// //
// //     Cmp(RegisterId, RegisterId),                // cmp r1, r2
// // }
// //
// //
// //
// //
// //
// // #[derive(Debug, Clone, Copy, PartialEq)]
// // pub enum ArithmeticOp {
// //     Add { dest: RegisterId, src1: RegisterId, src2: RegisterId },
// //     Sub { dest: RegisterId, src1: RegisterId, src2: RegisterId },
// //     Mul { dest: RegisterId, src1: RegisterId, src2: RegisterId },
// //     Div { dest: RegisterId, src1: RegisterId, src2: RegisterId },
// // }
// //
// // #[derive(Debug, Clone, Copy, PartialEq)]
// // pub enum MemoryOp {
// //     Load { reg: RegisterId, addr: Address },
// //     Store { reg: RegisterId, addr: Address },
// //     Move { dest: RegisterId, src: RegisterId },
// //     LoadImm { reg: RegisterId, value: i64 },
// // }
// //
// //
// // #[derive(Debug, Clone, Copy, PartialEq)]
// // pub enum ControlOp {
// //     Jump { addr: Address },
// //     JumpIf { condition: RegisterId, addr: Address },
// //     Call { addr: Address },
// //     Return,
// //     Nop,
// //     Halt,
// // }
// // #[derive(Debug, Clone, Copy, PartialEq)]
// // pub enum BranchOp {
// //     Equal,
// //     NotEqual,
// //     LessThan,
// //     GreaterThan,
// //     LessOrEqual,
// //     GreaterOrEqual
// // }
// //
// //
// // /// Identifiant de registre
// // #[derive(Debug, Clone, Copy, PartialEq, Eq,Hash)]
// // pub struct RegisterId(pub u8);
// //
// // #[derive(Debug)]
// // enum Stage {
// //     Fetch,
// //     Decode,
// //     Execute,
// //     Memory,
// //     Writeback,
// // }
// //
// // impl Instruction {
// //     pub fn is_branch(&self) -> bool {
// //         matches!(self,
// //             Instruction::Jump(_) |
// //             Instruction::JumpIf(_, _) |
// //             Instruction::Call(_)
// //
// //         )
// //     }
// //
// //     pub fn get_target_address(&self) -> u64 {
// //         match self {
// //             Instruction::Jump(addr) => addr.0,
// //             Instruction::JumpIf(_, addr) => addr.0,
// //             Instruction::Call(addr) => addr.0,
// //             _ => 0
// //         }
// //     }
// //
// //     pub fn is_return(&self) -> bool {
// //         matches!(self, Instruction::Return)
// //     }
// // }
// //
// // impl DecodedInstruction {
// //     pub fn is_taken(&self, registers: &RegisterBank) -> bool {
// //         match self {
// //             DecodedInstruction::Control(ControlOp::JumpIf { condition, .. }) => {
// //                 // Vérifier la valeur du registre de condition
// //                 registers.read_register(*condition).unwrap_or(0) != 0
// //             },
// //             DecodedInstruction::Control(op) => match op {
// //                 ControlOp::Jump { .. } => true,
// //                 ControlOp::Call { .. } => true,
// //                 _ => false,
// //             },
// //             DecodedInstruction::Branch(op) => {
// //                 let flags = registers.get_status_flags();
// //                 match op {
// //                     BranchOp::Equal => flags.zero,
// //                     BranchOp::NotEqual => !flags.zero,
// //                     BranchOp::LessThan => flags.negative,
// //                     BranchOp::GreaterThan => !flags.negative && !flags.zero,
// //                     BranchOp::LessOrEqual => flags.negative || flags.zero,
// //                     BranchOp::GreaterOrEqual => !flags.negative,
// //                 }
// //             },
// //             _ => false,
// //         }
// //     }
// //
// //
// //     pub fn get_target_address(&self) -> u64 {
// //         match self {
// //             DecodedInstruction::Control(op) => match op {
// //                 ControlOp::Jump { addr } => addr.0,
// //                 ControlOp::JumpIf { addr, .. } => addr.0,
// //                 ControlOp::Call { addr } => addr.0,
// //                 _ => 0,
// //             },
// //             _ => 0,
// //         }
// //     }
// // }
// //
// //
// //
// // impl InstructionDecoder {
// //     pub fn new() -> Self {
// //         Self
// //     }
// //
// //     pub fn decode(&self, instruction: Instruction) -> VMResult<DecodedInstruction> {
// //         // À implémenter
// //         todo!()
// //     }
// // }
// //
// //
// // impl From<DecodedInstruction> for Instruction {
// //     fn from(decoded: DecodedInstruction) -> Self {
// //         match decoded {
// //             DecodedInstruction::Arithmetic(op) => match op {
// //                 ArithmeticOp::Add { dest, src1, src2 } => Instruction::Add(dest, src1, src2),
// //                 ArithmeticOp::Sub { dest, src1, src2 } => Instruction::Sub(dest, src1, src2),
// //                 ArithmeticOp::Mul { dest, src1, src2 } => Instruction::Mul(dest, src1, src2),
// //                 ArithmeticOp::Div { dest, src1, src2 } => Instruction::Div(dest, src1, src2),
// //             },
// //             DecodedInstruction::Memory(op) => match op {
// //                 MemoryOp::LoadImm { reg, value } => Instruction::LoadImm(reg, value),
// //                 MemoryOp::Load { reg, addr } => Instruction::Load(reg, addr),
// //                 MemoryOp::Store { reg, addr } => Instruction::Store(reg, addr),
// //                 MemoryOp::Move { dest, src } => Instruction::Move(dest, src),
// //             },
// //             DecodedInstruction::Control(op) => match op {
// //                 ControlOp::Jump { addr } => Instruction::Jump(addr),
// //                 ControlOp::JumpIf { condition, addr } => Instruction::JumpIf(condition, addr),
// //                 ControlOp::Call { addr } => Instruction::Call(addr),
// //                 ControlOp::Return => Instruction::Return,
// //                 ControlOp::Halt => Instruction::Halt,
// //                 ControlOp::Nop => Instruction::Nop,
// //             },
// //             DecodedInstruction::Branch(op) => match op {
// //                 BranchOp::Equal => Instruction::Cmp(RegisterId(0), RegisterId(0)),
// //                 BranchOp::NotEqual => Instruction::Cmp(RegisterId(0), RegisterId(0)),
// //                 BranchOp::LessThan => Instruction::Cmp(RegisterId(0), RegisterId(0)),
// //                 BranchOp::GreaterThan => Instruction::Cmp(RegisterId(0), RegisterId(0)),
// //                 BranchOp::LessOrEqual => Instruction::Cmp(RegisterId(0), RegisterId(0)),
// //                 BranchOp::GreaterOrEqual => Instruction::Cmp(RegisterId(0), RegisterId(0),
// //                 ),
// //             },
// //             DecodedInstruction::Compare { src1, src2 } => Instruction::Cmp(src1, src2),
// //
// //         }
// //     }
// // }
// //
// //
// // // Optionnellement, on peut aussi implémenter From<Instruction> pour DecodedInstruction
// // impl From<Instruction> for DecodedInstruction {
// //     fn from(instruction: Instruction) -> Self {
// //         match instruction {
// //             Instruction::Add(dest, src1, src2) =>
// //                 DecodedInstruction::Arithmetic(ArithmeticOp::Add { dest, src1, src2 }),
// //             Instruction::Sub(dest, src1, src2) =>
// //                 DecodedInstruction::Arithmetic(ArithmeticOp::Sub { dest, src1, src2 }),
// //             Instruction::Mul(dest, src1, src2) =>
// //                 DecodedInstruction::Arithmetic(ArithmeticOp::Mul { dest, src1, src2 }),
// //             Instruction::Div(dest, src1, src2) =>
// //                 DecodedInstruction::Arithmetic(ArithmeticOp::Div { dest, src1, src2 }),
// //             Instruction::Load(reg, addr) =>
// //                 DecodedInstruction::Memory(MemoryOp::Load { reg, addr }),
// //             Instruction::Store(reg, addr) =>
// //                 DecodedInstruction::Memory(MemoryOp::Store { reg, addr }),
// //             Instruction::Move(dest, src) =>
// //                 DecodedInstruction::Memory(MemoryOp::Move { dest, src }),
// //             Instruction::LoadImm(reg, value) =>
// //                 DecodedInstruction::Memory(MemoryOp::LoadImm { reg, value }),
// //             Instruction::Jump(addr) =>
// //                 DecodedInstruction::Control(ControlOp::Jump { addr }),
// //             Instruction::JumpIf(condition, addr) =>
// //                 DecodedInstruction::Control(ControlOp::JumpIf { condition, addr }),
// //             Instruction::Call(addr) =>
// //                 DecodedInstruction::Control(ControlOp::Call { addr }),
// //             Instruction::Return =>
// //                 DecodedInstruction::Control(ControlOp::Return),
// //             Instruction::Halt =>
// //                 DecodedInstruction::Control(ControlOp::Halt),
// //             Instruction::Nop =>
// //                 DecodedInstruction::Control(ControlOp::Nop),
// //             Instruction::Cmp(src1, src2) =>
// //                 DecodedInstruction::Compare { src1, src2 },
// //         }
// //     }
// // }
// //
// //
// //
// //
// // #[cfg(test)]
// // mod tests {
// //     use crate::pvm::pipelines::Pipeline;
// //     use super::*;
// //     use crate::pvm::registers::RegisterBank;
// //     use crate::pvm::memorys::MemoryController;
// //
// //
// //
// //     // Helper pour créer une banque de registres avec des valeurs prédéfinies
// //     fn setup_test_registers() -> RegisterBank {
// //         let mut registers = RegisterBank::new(8).unwrap();
// //         registers.write_register(RegisterId(0), 0).unwrap();  // R0 = 0
// //         registers.write_register(RegisterId(1), 42).unwrap(); // R1 = 42
// //         registers.write_register(RegisterId(2), -1).unwrap(); // R2 = -1
// //         registers
// //     }
// //
// //     fn test_branch_is_taken() {
// //         let mut registers = setup_test_registers();
// //
// //         // Test Jump inconditionnel
// //         let jump = DecodedInstruction::Control(
// //             ControlOp::Jump { addr: Address(0x1000) }
// //         );
// //         assert!(jump.is_taken(&registers), "Jump devrait toujours être pris");
// //
// //         // Test Call inconditionnel
// //         let call = DecodedInstruction::Control(
// //             ControlOp::Call { addr: Address(0x2000) }
// //         );
// //         assert!(call.is_taken(&registers), "Call devrait toujours être pris");
// //
// //         // Test JumpIf avec R0 = 0 (ne devrait pas être pris)
// //         registers.write_register(RegisterId(0), 0).unwrap();
// //         let jump_if_false = DecodedInstruction::Control(
// //             ControlOp::JumpIf { condition: RegisterId(0), addr: Address(0x3000) }
// //         );
// //         assert!(!jump_if_false.is_taken(&registers), "JumpIf avec R0=0 ne devrait pas être pris");
// //
// //         // Test JumpIf avec R1 = 42 (devrait être pris)
// //         let jump_if_true = DecodedInstruction::Control(
// //             ControlOp::JumpIf { condition: RegisterId(1), addr: Address(0x4000) }
// //         );
// //         assert!(jump_if_true.is_taken(&registers), "JumpIf avec R1=42 devrait être pris");
// //     }
// //
// //     #[test]
// //     fn test_get_target_address() {
// //         // Test Jump
// //         let jump = DecodedInstruction::Control(
// //             ControlOp::Jump { addr: Address(0x1000) }
// //         );
// //         assert_eq!(jump.get_target_address(), 0x1000, "Mauvaise adresse cible pour Jump");
// //
// //         // Test JumpIf
// //         let jump_if = DecodedInstruction::Control(
// //             ControlOp::JumpIf { condition: RegisterId(0), addr: Address(0x2000) }
// //         );
// //         assert_eq!(jump_if.get_target_address(), 0x2000, "Mauvaise adresse cible pour JumpIf");
// //
// //         // Test Call
// //         let call = DecodedInstruction::Control(
// //             ControlOp::Call { addr: Address(0x3000) }
// //         );
// //         assert_eq!(call.get_target_address(), 0x3000, "Mauvaise adresse cible pour Call");
// //
// //         // Test instruction non-branchement
// //         let add = DecodedInstruction::Arithmetic(
// //             ArithmeticOp::Add {
// //                 dest: RegisterId(0),
// //                 src1: RegisterId(1),
// //                 src2: RegisterId(2)
// //             }
// //         );
// //         assert_eq!(add.get_target_address(), 0, "Instruction non-branchement devrait retourner 0");
// //     }
// //
// //     #[test]
// //     fn test_instruction_branch_combinations() {
// //         let mut registers = setup_test_registers();
// //         registers.write_register(RegisterId(0), 0).unwrap();  // Assurer que R0 = 0
// //
// //         let test_cases = vec![
// //             (DecodedInstruction::Control(ControlOp::Jump { addr: Address(0x1000) }), true, 0x1000),
// //             (DecodedInstruction::Control(ControlOp::JumpIf { condition: RegisterId(0), addr: Address(0x2000) }), false, 0x2000),
// //             (DecodedInstruction::Control(ControlOp::JumpIf { condition: RegisterId(1), addr: Address(0x3000) }), true, 0x3000),
// //             (DecodedInstruction::Control(ControlOp::Call { addr: Address(0x4000) }), true, 0x4000),
// //             (DecodedInstruction::Control(ControlOp::Return), false, 0),
// //             (DecodedInstruction::Control(ControlOp::Nop), false, 0),
// //         ];
// //
// //         for (instruction, should_be_taken, expected_addr) in test_cases {
// //             assert_eq!(
// //                 instruction.is_taken(&registers),
// //                 should_be_taken,
// //                 "Mauvaise prédiction pour {:?}", instruction
// //             );
// //             assert_eq!(
// //                 instruction.get_target_address(),
// //                 expected_addr,
// //                 "Mauvaise adresse cible pour {:?}", instruction
// //             );
// //         }
// //     }
// //
// //     #[test]
// //     fn test_instruction_conversion() {
// //         let original = Instruction::Add(RegisterId(0), RegisterId(1), RegisterId(2));
// //         let original_clone = original.clone(); // Clone l'instruction originale
// //         let decoded: DecodedInstruction = original.into();
// //         let back: Instruction = decoded.into();
// //         assert_eq!(original_clone, back);
// //     }
// //
// //     // // Ajout des tests
// //     #[test]
// //     fn test_cmp_instruction_basic() {
// //         let mut register_bank = RegisterBank::new(8).unwrap();
// //
// //         // Préparer les registres avec write_register (au lieu de write)
// //         register_bank.write_register(RegisterId(0), 42).unwrap();
// //         register_bank.write_register(RegisterId(1), 42).unwrap();
// //         register_bank.write_register(RegisterId(2), 100).unwrap();
// //
// //         // Créer l'instruction Cmp
// //         let cmp_instruction = Instruction::Cmp(RegisterId(0), RegisterId(1));
// //         let decoded: DecodedInstruction = cmp_instruction.into();
// //
// //         // Vérifier la conversion
// //         match decoded {
// //             DecodedInstruction::Compare { src1, src2 } => {
// //                 assert_eq!(src1, RegisterId(0));
// //                 assert_eq!(src2, RegisterId(1));
// //             },
// //             _ => panic!("Mauvais décodage de l'instruction Cmp"),
// //         }
// //     }
// //     #[test]
// //     fn test_cmp_flags() {
// //         let mut register_bank = RegisterBank::new(8).unwrap();
// //         let mut pipeline = Pipeline::new();
// //
// //         // Test d'égalité (cas 1)
// //         register_bank.write_register(RegisterId(0), 42).unwrap();
// //         register_bank.write_register(RegisterId(1), 42).unwrap();
// //
// //         let cmp_equal = DecodedInstruction::Compare {
// //             src1: RegisterId(0),
// //             src2: RegisterId(1)
// //         };
// //
// //         pipeline.execute_instruction(&cmp_equal, &mut register_bank).unwrap();
// //         let flags = register_bank.get_status_flags_mut();
// //         assert!(flags.zero, "Les valeurs égales devraient donner un flag zero=true");
// //         assert!(!flags.negative, "Les valeurs égales devraient donner un flag negative=false");
// //
// //         // Test d'infériorité (cas 2)
// //         register_bank.write_register(RegisterId(0), 40).unwrap();
// //         register_bank.write_register(RegisterId(1), 42).unwrap();
// //
// //         let cmp_less = DecodedInstruction::Compare {
// //             src1: RegisterId(0),
// //             src2: RegisterId(1)
// //         };
// //
// //         pipeline.execute_instruction(&cmp_less, &mut register_bank).unwrap();
// //         let flags = register_bank.get_status_flags_mut();
// //         assert!(!flags.zero, "Les valeurs différentes devraient donner un flag zero=false");
// //         assert!(flags.negative, "La valeur inférieure devrait donner un flag negative=true");
// //
// //         // Test de supériorité (cas 3)
// //         register_bank.write_register(RegisterId(0), 44).unwrap();
// //         register_bank.write_register(RegisterId(1), 42).unwrap();
// //
// //         let cmp_greater = DecodedInstruction::Compare {
// //             src1: RegisterId(0),
// //             src2: RegisterId(1)
// //         };
// //
// //         pipeline.execute_instruction(&cmp_greater, &mut register_bank).unwrap();
// //         let flags = register_bank.get_status_flags_mut();
// //         assert!(!flags.zero, "Les valeurs différentes devraient donner un flag zero=false");
// //         assert!(!flags.negative, "La valeur supérieure devrait donner un flag negative=false");
// //     }
// //
// //
// //
// //     #[test]
// //     fn test_cmp_with_branch_execution() {
// //         let mut pipeline = Pipeline::new();
// //         let mut register_bank = RegisterBank::new(8).unwrap();
// //         let mut memory_controller = MemoryController::new(1024, 256).unwrap();
// //
// //         let program = vec![
// //             Instruction::LoadImm(RegisterId(0), 42),
// //             Instruction::LoadImm(RegisterId(1), 42),
// //             Instruction::Cmp(RegisterId(0), RegisterId(1)),
// //             // Si les valeurs sont égales (flag zero=true), sauter
// //             Instruction::JumpIf(RegisterId(0), Address(8)),
// //             Instruction::LoadImm(RegisterId(2), 1), // Cette instruction sera sautée
// //             Instruction::LoadImm(RegisterId(2), 2), // On devrait exécuter celle-ci
// //         ];
// //
// //         pipeline.load_instructions(program).unwrap();
// //
// //         while !pipeline.is_empty().unwrap() {
// //             pipeline.cycle(&mut register_bank, &mut memory_controller).unwrap();
// //         }
// //
// //         // Vérifier que R2 contient 2 car le branchement a été pris
// //         let result = register_bank.read_register(RegisterId(2)).unwrap();
// //         assert_eq!(result, 2, "Le branchement aurait dû être pris car les valeurs sont égales");
// //
// //         // Vérifier les flags
// //         let flags = register_bank.get_status_flags();
// //         assert!(flags.zero, "Le flag zero devrait être true pour des valeurs égales");
// //         assert!(!flags.negative, "Le flag negative devrait être false pour des valeurs égales");
// //     }
// //
// //     #[test]
// //     fn test_cmp_instruction() {
// //         let mut pipeline = Pipeline::new();
// //         let mut registers = RegisterBank::new(8).unwrap();
// //
// //         // Préparer les registres
// //         registers.write_register(RegisterId(0), 42).unwrap();
// //         registers.write_register(RegisterId(1), 42).unwrap();
// //         registers.write_register(RegisterId(2), 100).unwrap();
// //
// //         // Test d'égalité
// //         let cmp_equal = DecodedInstruction::Compare {
// //             src1: RegisterId(0),
// //             src2: RegisterId(1)
// //         };
// //
// //         let result = pipeline.execute_instruction(&cmp_equal, &mut registers).unwrap();
// //         let flags = result.flags;
// //         assert!(flags.zero);
// //         assert!(!flags.negative);
// //
// //         // Test inférieur
// //         let cmp_less = DecodedInstruction::Compare {
// //             src1: RegisterId(0),
// //             src2: RegisterId(2)
// //         };
// //
// //         let result = pipeline.execute_instruction(&cmp_less, &mut registers).unwrap();
// //         let flags = result.flags;
// //         assert!(!flags.zero);
// //         assert!(flags.negative);
// //     }
// // }
// //
// //
// //

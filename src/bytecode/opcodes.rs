//src/bytecode/opcodes.rs

/// Représente les opcodes supportés par PunkVM
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Opcode {
    // Instructions ALU (0x00 - 0x1F)
    Nop = 0x00,
    Add = 0x01,
    Sub = 0x02,
    Mul = 0x03,
    Div = 0x04,
    Mod = 0x05,
    Inc = 0x06,
    Dec = 0x07,
    Neg = 0x08,
    Mov = 0x09, //move
    //0x09 - 0x1F : Réservé pour les futures instructions ALU

    // Instructions Logiques et de bit (0x20 - 0x3F)
    And = 0x20,
    Or = 0x21,
    Xor = 0x22,
    Not = 0x23,
    Shl = 0x24, //shift left
    Shr = 0x25, //shift right
    Sar = 0x26, //shift arithmetic right
    Rol = 0x27, //rotate left
    Ror = 0x28, //rotate right
    //0x29 - 0x3F : Réservé pour les futures instructions Logiques et de bit

    // Instructions de controle de flux (0x40 - 0x5F)
    Jmp = 0x40,
    JmpIf = 0x41,
    JmpIfNot = 0x42,
    JmpIfEqual = 0x43,        //JmpIfEqual
    JmpIfNotEqual = 0x44,     //JmpIfNotEqual
    JmpIfGreater = 0x45,      //JumpIfGreater
    JmpIfGreaterEqual = 0x46, //JumpIfGreaterOrEqual
    JmpIfLess = 0x47,         //JumpIfLess
    JmpIfLessEqual = 0x48,    //JumpIfLessOrEqual
    JmpIfAbove = 0x49,        //JumpIfAbove
    JmpIfAboveEqual = 0x4A,   //JumpIfAboveOrEqual
    JmpIfBelow = 0x4B,        //JumpIfBelow
    JmpIfBelowEqual = 0x4C,   //JumpIfBelowOrEqual
    JmpIfNotZero = 0x4D,      //JumpIfNotZero
    JmpIfZero = 0x4E,         //JumpIfZero
    JmpIfOverflow = 0x4F,     //JumpIfOverflow
    JmpIfNotOverflow = 0x50,  //JumpIfNotOverflow
    JmpIfPositive = 0x51,     //JumpIfPositive
    JmpIfNegative = 0x52,     //JumpIfNegative

    Call = 0x53, //Call
    Ret = 0x54,  //Ret
    Cmp = 0x55,  //Cmp
    Test = 0x56, //Test
    //0x57 - 0x5F : Réservé pour les futures instructions de controle de flux



    // Instructions d'accès mémoire (0x60 - 0x7F)
    Load = 0x60,
    Store = 0x61,
    LoadB = 0x62,  //load byte
    StoreB = 0x63, //store byte
    LoadW = 0x64,  //load word (16 bits)
    StoreW = 0x65, //store word (16 bits)
    LoadD = 0x66,  //load double word (32 bits)
    StoreD = 0x67, //store double word (32 bits)
    Push = 0x68,    // Push pour empiler une valeur sur la pile
    Pop = 0x69,   // Pop pour depiler une valeur de la pile

    //0x6A - 0x7F : Réservé pour les futures instructions d'accès mémoire

    // Instructions speciales (0x80 - 0x9F)
    Syscall = 0x80,
    Break = 0x81,
    Halt = 0x82,
    //0x83 - 0x9F : Réservé pour les futures instructions speciales

    // Instructions SIMD 128-bit (0xA0 - 0xBF)
    Simd128Add = 0xA0,        // Addition vectorielle 128-bit
    Simd128Sub = 0xA1,        // Soustraction vectorielle 128-bit
    Simd128Mul = 0xA2,        // Multiplication vectorielle 128-bit
    Simd128Div = 0xA3,        // Division vectorielle 128-bit
    Simd128And = 0xA4,        // ET logique vectoriel 128-bit
    Simd128Or = 0xA5,         // OU logique vectoriel 128-bit
    Simd128Xor = 0xA6,        // XOR vectoriel 128-bit
    Simd128Not = 0xA7,        // NOT vectoriel 128-bit
    Simd128Load = 0xA8,       // Chargement vectoriel 128-bit
    Simd128Store = 0xA9,      // Stockage vectoriel 128-bit
    Simd128Mov = 0xAA,        // Mouvement vectoriel 128-bit
    Simd128Cmp = 0xAB,        // Comparaison vectorielle 128-bit
    Simd128Min = 0xAC,        // Minimum vectoriel 128-bit
    Simd128Max = 0xAD,        // Maximum vectoriel 128-bit
    Simd128Sqrt = 0xAE,       // Racine carrée vectorielle 128-bit
    Simd128Shuffle = 0xAF,    // Mélange vectoriel 128-bit
    //0xB0 - 0xBF : Réservé pour autres instructions SIMD 128-bit

    // Instructions SIMD 256-bit (0xC0 - 0xDF)  
    Simd256Add = 0xC0,        // Addition vectorielle 256-bit
    Simd256Sub = 0xC1,        // Soustraction vectorielle 256-bit
    Simd256Mul = 0xC2,        // Multiplication vectorielle 256-bit
    Simd256Div = 0xC3,        // Division vectorielle 256-bit
    Simd256And = 0xC4,        // ET logique vectoriel 256-bit
    Simd256Or = 0xC5,         // OU logique vectoriel 256-bit
    Simd256Xor = 0xC6,        // XOR vectoriel 256-bit
    Simd256Not = 0xC7,        // NOT vectoriel 256-bit
    Simd256Load = 0xC8,       // Chargement vectoriel 256-bit
    Simd256Store = 0xC9,      // Stockage vectoriel 256-bit
    Simd256Mov = 0xCA,        // Mouvement vectoriel 256-bit
    Simd256Cmp = 0xCB,        // Comparaison vectorielle 256-bit
    Simd256Min = 0xCC,        // Minimum vectoriel 256-bit
    Simd256Max = 0xCD,        // Maximum vectoriel 256-bit
    Simd256Sqrt = 0xCE,       // Racine carrée vectorielle 256-bit
    Simd256Shuffle = 0xCF,    // Mélange vectoriel 256-bit
    //0xD0 - 0xDF : Réservé pour autres instructions SIMD 256-bit

    // Instructions FPU (0xE0 - 0xEF)
    FpuAdd = 0xE0,           // Addition flottante
    FpuSub = 0xE1,           // Soustraction flottante
    FpuMul = 0xE2,           // Multiplication flottante
    FpuDiv = 0xE3,           // Division flottante
    FpuSqrt = 0xE4,          // Racine carrée flottante
    FpuCmp = 0xE5,           // Comparaison flottante
    FpuLoad = 0xE6,          // Chargement flottant
    FpuStore = 0xE7,         // Stockage flottant
    FpuMov = 0xE8,           // Mouvement flottant
    FpuConvert = 0xE9,       // Conversion flottante
    FpuRound = 0xEA,         // Arrondi flottant
    FpuMin = 0xEB,           // Minimum flottant
    FpuMax = 0xEC,           // Maximum flottant
    //0xED - 0xEF : Réservé pour autres instructions FPU

    // Instruction etendues (0xF0 - 0xFF)
    Extended = 0xF0,
    //0xF1 - 0xFF : Réservé pour les futures instructions etendues

}

/// Catégorie d'opcode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpcodeCategory {
    Alu,
    Logical,
    ControlFlow,
    Memory,
    Special,
    Simd128,
    Simd256,
    Fpu,
    Extended,
    Unknown,
}

impl Opcode {
    /// Convertit un u8 en Opcode
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(Self::Nop),
            0x01 => Some(Self::Add),
            0x02 => Some(Self::Sub),
            0x03 => Some(Self::Mul),
            0x04 => Some(Self::Div),
            0x05 => Some(Self::Mod),
            0x06 => Some(Self::Inc),
            0x07 => Some(Self::Dec),
            0x08 => Some(Self::Neg),

            0x20 => Some(Self::And),
            0x21 => Some(Self::Or),
            0x22 => Some(Self::Xor),
            0x23 => Some(Self::Not),
            0x24 => Some(Self::Shl),
            0x25 => Some(Self::Shr),
            0x26 => Some(Self::Sar),
            0x27 => Some(Self::Rol),
            0x28 => Some(Self::Ror),

            0x40 => Some(Self::Jmp),
            0x41 => Some(Self::JmpIf),
            0x42 => Some(Self::JmpIfNot),
            0x43 => Some(Self::JmpIfEqual),
            0x44 => Some(Self::JmpIfNotEqual),
            0x45 => Some(Self::JmpIfGreater),
            0x46 => Some(Self::JmpIfGreaterEqual),
            0x47 => Some(Self::JmpIfLess),
            0x48 => Some(Self::JmpIfLessEqual),
            0x49 => Some(Self::JmpIfAbove),
            0x4A => Some(Self::JmpIfAboveEqual),
            0x4B => Some(Self::JmpIfBelow),
            0x4C => Some(Self::JmpIfBelowEqual),
            0x4D => Some(Self::JmpIfZero),
            0x4E => Some(Self::JmpIfNotZero),
            0x4F => Some(Self::JmpIfOverflow),
            0x50 => Some(Self::JmpIfNotOverflow),
            0x51 => Some(Self::JmpIfPositive),
            0x52 => Some(Self::JmpIfNegative),

            0x53 => Some(Self::Call),
            0x54 => Some(Self::Ret),
            0x55 => Some(Self::Cmp),
            0x56 => Some(Self::Test),

            0x60 => Some(Self::Load),
            0x61 => Some(Self::Store),
            0x62 => Some(Self::LoadB),
            0x63 => Some(Self::StoreB),
            0x64 => Some(Self::LoadW),
            0x65 => Some(Self::StoreW),
            0x66 => Some(Self::LoadD),
            0x67 => Some(Self::StoreD),
            0x68 => Some(Self::Push),
            0x69 => Some(Self::Pop),

            0x80 => Some(Self::Syscall),
            0x81 => Some(Self::Break),
            0x82 => Some(Self::Halt),

            // SIMD 128-bit opcodes
            0xA0 => Some(Self::Simd128Add),
            0xA1 => Some(Self::Simd128Sub),
            0xA2 => Some(Self::Simd128Mul),
            0xA3 => Some(Self::Simd128Div),
            0xA4 => Some(Self::Simd128And),
            0xA5 => Some(Self::Simd128Or),
            0xA6 => Some(Self::Simd128Xor),
            0xA7 => Some(Self::Simd128Not),
            0xA8 => Some(Self::Simd128Load),
            0xA9 => Some(Self::Simd128Store),
            0xAA => Some(Self::Simd128Mov),
            0xAB => Some(Self::Simd128Cmp),
            0xAC => Some(Self::Simd128Min),
            0xAD => Some(Self::Simd128Max),
            0xAE => Some(Self::Simd128Sqrt),
            0xAF => Some(Self::Simd128Shuffle),

            // SIMD 256-bit opcodes
            0xC0 => Some(Self::Simd256Add),
            0xC1 => Some(Self::Simd256Sub),
            0xC2 => Some(Self::Simd256Mul),
            0xC3 => Some(Self::Simd256Div),
            0xC4 => Some(Self::Simd256And),
            0xC5 => Some(Self::Simd256Or),
            0xC6 => Some(Self::Simd256Xor),
            0xC7 => Some(Self::Simd256Not),
            0xC8 => Some(Self::Simd256Load),
            0xC9 => Some(Self::Simd256Store),
            0xCA => Some(Self::Simd256Mov),
            0xCB => Some(Self::Simd256Cmp),
            0xCC => Some(Self::Simd256Min),
            0xCD => Some(Self::Simd256Max),
            0xCE => Some(Self::Simd256Sqrt),
            0xCF => Some(Self::Simd256Shuffle),

            // FPU opcodes
            0xE0 => Some(Self::FpuAdd),
            0xE1 => Some(Self::FpuSub),
            0xE2 => Some(Self::FpuMul),
            0xE3 => Some(Self::FpuDiv),
            0xE4 => Some(Self::FpuSqrt),
            0xE5 => Some(Self::FpuCmp),
            0xE6 => Some(Self::FpuLoad),
            0xE7 => Some(Self::FpuStore),
            0xE8 => Some(Self::FpuMov),
            0xE9 => Some(Self::FpuConvert),
            0xEA => Some(Self::FpuRound),
            0xEB => Some(Self::FpuMin),
            0xEC => Some(Self::FpuMax),

            0xF0 => Some(Self::Extended),
            // 0xFF => Some(Self::Invalid),
            _ => None,
        }

    }

    /// Indique si l'opcode est valide
    pub fn is_valid(&self) -> bool {
        Self::from_u8(*self as u8).is_some()
    }

    /// Retourne la taille d'opcode en bytes
    pub fn size(&self) -> usize {
        1 //Tous les opcodes font 1 byte
    }

    /// Indique si l'opcode est une instruction de controle de flux
    pub fn is_branch(&self) -> bool {
        matches!(
            self,
            Self::Jmp
                | Self::JmpIf
                | Self::JmpIfNot
                | Self::JmpIfEqual
                | Self::JmpIfNotEqual
                | Self::JmpIfGreater
                | Self::JmpIfGreaterEqual
                | Self::JmpIfLess
                | Self::JmpIfLessEqual
                | Self::JmpIfAbove
                | Self::JmpIfAboveEqual
                | Self::JmpIfBelow
                | Self::JmpIfBelowEqual
                | Self::JmpIfNotZero
                | Self::JmpIfZero
                | Self::JmpIfOverflow
                | Self::JmpIfNotOverflow
                | Self::JmpIfPositive
                | Self::JmpIfNegative
                | Self::Call
                | Self::Ret
        )
    }

    /// Indique si l'opcode est une instruction call pour RAS
    pub fn is_call(&self) -> bool{
        matches!(self , Self::Call)
    }

    /// Indique si l'opcode est une instruction de retour pour RAS
    pub fn is_return(&self) -> bool {
        matches!(self, Self::Ret)
    }

    /// Retourne la categorie de l'opcode
    pub fn category(&self) -> OpcodeCategory {
        match *self as u8 {
            0x00..=0x1F => OpcodeCategory::Alu,
            0x20..=0x3F => OpcodeCategory::Logical,
            0x40..=0x5F => OpcodeCategory::ControlFlow,
            0x60..=0x7F => OpcodeCategory::Memory,
            0x80..=0x9F => OpcodeCategory::Special,
            0xA0..=0xBF => OpcodeCategory::Simd128,
            0xC0..=0xDF => OpcodeCategory::Simd256,
            0xE0..=0xEF => OpcodeCategory::Fpu,
            0xF0..=0xFF => OpcodeCategory::Extended,

            _ => OpcodeCategory::Unknown,
        }
    }
    pub fn name(&self) -> &'static str{
        match self {
            Self::Nop => "Nop",
            Self::Add => "Add",
            Self::Sub => "Sub",
            Self::Mul => "Mul",
            Self::Div => "Div",
            Self::Mod => "Mod",
            Self::Inc => "Inc",
            Self::Dec => "Dec",
            Self::Neg => "Neg",
            Self::Mov => "Mov",
            Self::And => "And",
            Self::Or => "Or",
            Self::Xor => "Xor",
            Self::Not => "Not",
            Self::Shl => "Shl",
            Self::Shr => "Shr",
            Self::Sar => "Sar",
            Self::Rol => "Rol",
            Self::Ror => "Ror",
            Self::Jmp => "Jmp",
            Self::JmpIf => "JmpIf",
            Self::JmpIfNot => "JmpIfNot",
            Self::JmpIfEqual => "JmpIfEqual",
            Self::JmpIfNotEqual => "JmpIfNotEqual",
            Self::JmpIfGreater => "JmpIfGreater",
            Self::JmpIfGreaterEqual => "JmpIfGreaterEqual",
            Self::JmpIfLess => "JmpIfLess",
            Self::JmpIfLessEqual => "JmpIfLessEqual",
            Self::JmpIfAbove => "JmpIfAbove",
            Self::JmpIfAboveEqual => "JmpIfAboveEqual",
            Self::JmpIfBelow => "JmpIfBelow",
            Self::JmpIfBelowEqual => "JmpIfBelowEqual",
            Self::JmpIfNotZero => "JmpIfNotZero",
            Self::JmpIfZero => "JmpIfZero",
            Self::JmpIfOverflow => "JmpIfOverflow",
            Self::JmpIfNotOverflow => "JmpIfNotOverflow",
            Self::JmpIfPositive => "JmpIfPositive",
            Self::JmpIfNegative => "JmpIfNegative",
            Self::Call => "Call",
            Self::Ret => "Ret",
            Self::Cmp => "Cmp",
            Self::Test => "Test",
            Self::Load => "Load",
            Self::Store => "Store",
            Self::LoadB => "LoadB",
            Self::StoreB => "StoreB",
            Self::LoadW => "LoadW",
            Self::StoreW => "StoreW",
            Self::LoadD => "LoadD",
            Self::StoreD => "StoreD",
            Self::Push => "Push",
            Self::Pop => "Pop",
            Self::Syscall => "Syscall",
            Self::Break => "Break",
            Self::Halt => "Halt",
            
            // SIMD 128-bit operations
            Self::Simd128Add => "Simd128Add",
            Self::Simd128Sub => "Simd128Sub",
            Self::Simd128Mul => "Simd128Mul",
            Self::Simd128Div => "Simd128Div",
            Self::Simd128And => "Simd128And",
            Self::Simd128Or => "Simd128Or",
            Self::Simd128Xor => "Simd128Xor",
            Self::Simd128Not => "Simd128Not",
            Self::Simd128Load => "Simd128Load",
            Self::Simd128Store => "Simd128Store",
            Self::Simd128Mov => "Simd128Mov",
            Self::Simd128Cmp => "Simd128Cmp",
            Self::Simd128Min => "Simd128Min",
            Self::Simd128Max => "Simd128Max",
            Self::Simd128Sqrt => "Simd128Sqrt",
            Self::Simd128Shuffle => "Simd128Shuffle",
            
            // SIMD 256-bit operations
            Self::Simd256Add => "Simd256Add",
            Self::Simd256Sub => "Simd256Sub",
            Self::Simd256Mul => "Simd256Mul",
            Self::Simd256Div => "Simd256Div",
            Self::Simd256And => "Simd256And",
            Self::Simd256Or => "Simd256Or",
            Self::Simd256Xor => "Simd256Xor",
            Self::Simd256Not => "Simd256Not",
            Self::Simd256Load => "Simd256Load",
            Self::Simd256Store => "Simd256Store",
            Self::Simd256Mov => "Simd256Mov",
            Self::Simd256Cmp => "Simd256Cmp",
            Self::Simd256Min => "Simd256Min",
            Self::Simd256Max => "Simd256Max",
            Self::Simd256Sqrt => "Simd256Sqrt",
            Self::Simd256Shuffle => "Simd256Shuffle",
            
            // FPU operations
            Self::FpuAdd => "FpuAdd",
            Self::FpuSub => "FpuSub",
            Self::FpuMul => "FpuMul",
            Self::FpuDiv => "FpuDiv",
            Self::FpuSqrt => "FpuSqrt",
            Self::FpuCmp => "FpuCmp",
            Self::FpuLoad => "FpuLoad",
            Self::FpuStore => "FpuStore",
            Self::FpuMov => "FpuMov",
            Self::FpuConvert => "FpuConvert",
            Self::FpuRound => "FpuRound",
            Self::FpuMin => "FpuMin",
            Self::FpuMax => "FpuMax",
            
            Self::Extended => "Extended",
            _ => "Unknown",


        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode_from_u8() {
        // Test des valeurs valides
        assert_eq!(Opcode::from_u8(0x00), Some(Opcode::Nop));
        assert_eq!(Opcode::from_u8(0x01), Some(Opcode::Add));
        assert_eq!(Opcode::from_u8(0x60), Some(Opcode::Load));

        // Test des valeurs invalides
        assert_eq!(Opcode::from_u8(0xFF), None);
        // assert_eq!(Opcode::from_u8(0xFF),Some(Opcode::Invalid));
        assert_eq!(Opcode::from_u8(0x09), None);
    }

    #[test]
    fn test_opcode_is_valid() {
        assert!(Opcode::Add.is_valid());
        assert!(Opcode::Load.is_valid());
        assert!(Opcode::Halt.is_valid());
    }

    #[test]
    fn test_opcode_size() {
        assert_eq!(Opcode::Add.size(), 1);
        assert_eq!(Opcode::Load.size(), 1);
        assert_eq!(Opcode::Halt.size(), 1);
    }

    #[test]
    fn test_opcode_is_branch() {
        // Instructions de branchement
        assert!(Opcode::Jmp.is_branch());
        assert!(Opcode::JmpIf.is_branch());
        assert!(Opcode::Call.is_branch());
        assert!(Opcode::Ret.is_branch());
        assert!(Opcode::JmpIfNot.is_branch());

        assert!(Opcode::JmpIfEqual.is_branch());
        assert!(Opcode::JmpIfNotEqual.is_branch());
        assert!(Opcode::JmpIfGreater.is_branch());
        assert!(Opcode::JmpIfGreaterEqual.is_branch());
        assert!(Opcode::JmpIfLess.is_branch());
        assert!(Opcode::JmpIfLessEqual.is_branch());
        assert!(Opcode::JmpIfAbove.is_branch());
        assert!(Opcode::JmpIfAboveEqual.is_branch());
        assert!(Opcode::JmpIfBelow.is_branch());
        assert!(Opcode::JmpIfBelowEqual.is_branch());
        assert!(Opcode::JmpIfNotZero.is_branch());
        assert!(Opcode::JmpIfZero.is_branch());
        assert!(Opcode::JmpIfOverflow.is_branch());
        assert!(Opcode::JmpIfNotOverflow.is_branch());
        assert!(Opcode::JmpIfPositive.is_branch());
        assert!(Opcode::JmpIfNegative.is_branch());

        // Instructions non-branchement
        assert!(!Opcode::Add.is_branch());
        assert!(!Opcode::Load.is_branch());
        assert!(!Opcode::Halt.is_branch());
    }

    #[test]
    fn test_opcode_is_call() {
        // Test des instructions de type call
        assert!(Opcode::Call.is_call());

    }

    #[test]
    fn test_opcode_is_return() {
        // Test des instructions de type return
        assert!(Opcode::Ret.is_return());
    }

    #[test]
    fn test_opcode_category() {
        // Test des différentes catégories
        assert_eq!(Opcode::Add.category(), OpcodeCategory::Alu);
        assert_eq!(Opcode::And.category(), OpcodeCategory::Logical);
        assert_eq!(Opcode::Jmp.category(), OpcodeCategory::ControlFlow);
        assert_eq!(Opcode::Load.category(), OpcodeCategory::Memory);
        assert_eq!(Opcode::Syscall.category(), OpcodeCategory::Special);
        assert_eq!(Opcode::Extended.category(), OpcodeCategory::Extended);
    }

    #[test]
    fn test_opcode_values() {
        // Test des valeurs spécifiques
        assert_eq!(Opcode::Add as u8, 0x01);
        assert_eq!(Opcode::Load as u8, 0x60);
        assert_eq!(Opcode::Halt as u8, 0x82);
    }

    #[test]
    fn test_memory_instructions() {
        let memory_ops = [
            Opcode::Load,
            Opcode::Store,
            Opcode::LoadB,
            Opcode::StoreB,
            Opcode::LoadW,
            Opcode::StoreW,
            Opcode::LoadD,
            Opcode::StoreD,
        ];

        for op in memory_ops.iter() {
            assert_eq!(op.category(), OpcodeCategory::Memory);
        }
    }
}

// Test unitaire pour les Opcodes
// #[cfg(test)]
// test  ici

//src/bytecode/simds.rs

/// Types de donnees vectorielles supportees par PunkVM
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VectorDataType {
    /// 8 elements de 16 bits (entier)
    I16x8,
    /// 4 elements de 32 bits (entier)
    I32x4,
    /// 2 elements de 64 bits (entier)
    I64x2,
    /// 4 elements de 32 bits (flottant)
    F32x4,
    /// 2 elements de 64 bits (flottant)
    F64x2,
}

/// Types de donnees vectorielles 256-bit
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Vector256DataType {
    /// 16 elements de 16 bits (entier)
    I16x16,
    /// 8 elements de 32 bits (entier)
    I32x8,
    /// 4 elements de 64 bits (entier)
    I64x4,
    /// 8 elements de 32 bits (flottant)
    F32x8,
    /// 4 elements de 64 bits (flottant)
    F64x4,
}

/// Vecteur 128-bit generique
#[derive(Clone, Copy)]
pub union Vector128 {
    /// 16 elements de 8 bits
    pub i8x16: [i8; 16],
    /// 8 elements de 16 bits
    pub i16x8: [i16; 8],
    /// 4 elements de 32 bits
    pub i32x4: [i32; 4],
    /// 2 elements de 64 bits
    pub i64x2: [i64; 2],
    /// 4 elements de 32 bits flottant
    pub f32x4: [f32; 4],
    /// 2 elements de 64 bits flottant
    pub f64x2: [f64; 2],
    /// Representation brute en bytes
    pub bytes: [u8; 16],
    /// Representation brute en u32
    pub u32s: [u32; 4],
    /// Representation brute en u64
    pub u64s: [u64; 2],
}

/// Vecteur 256-bit generique
#[derive(Clone, Copy)]
pub union Vector256 {
    /// 32 elements de 8 bits
    pub i8x32: [i8; 32],
    /// 16 elements de 16 bits
    pub i16x16: [i16; 16],
    /// 8 elements de 32 bits
    pub i32x8: [i32; 8],
    /// 4 elements de 64 bits
    pub i64x4: [i64; 4],
    /// 8 elements de 32 bits flottant
    pub f32x8: [f32; 8],
    /// 4 elements de 64 bits flottant
    pub f64x4: [f64; 4],
    /// Representation brute en bytes
    pub bytes: [u8; 32],
    /// Representation brute en u32
    pub u32s: [u32; 8],
    /// Representation brute en u64
    pub u64s: [u64; 4],
    /// Deux vecteurs 128-bit
    pub v128x2: [Vector128; 2],
}

impl Default for Vector128 {
    fn default() -> Self {
        Vector128 { bytes: [0; 16] }
    }
}

impl Default for Vector256 {
    fn default() -> Self {
        Vector256 { bytes: [0; 32] }
    }
}

impl Vector128 {
    /// Cree un nouveau vecteur 128-bit a zero
    pub fn zero() -> Self {
        Vector128 { bytes: [0; 16] }
    }

    /// Cree un vecteur a partir d'un array de bytes
    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        Vector128 { bytes }
    }

    /// Cree un vecteur a partir de 4 valeurs i32
    pub fn from_i32x4(values: [i32; 4]) -> Self {
        Vector128 { i32x4: values }
    }

    /// Cree un vecteur a partir de 8 valeurs i16
    pub fn from_i16x8(values: [i16; 8]) -> Self {
        Vector128 { i16x8: values }
    }

    /// Cree un vecteur a partir de 4 valeurs f32
    pub fn from_f32x4(values: [f32; 4]) -> Self {
        Vector128 { f32x4: values }
    }

    /// Cree un vecteur a partir de 2 valeurs i64
    pub fn from_i64x2(values: [i64; 2]) -> Self {
        Vector128 { i64x2: values }
    }

    /// Cree un vecteur a partir de 2 valeurs f64
    pub fn from_f64x2(values: [f64; 2]) -> Self {
        Vector128 { f64x2: values }
    }

    /// Cree un vecteur a partir de deux u64
    pub fn from_u64(values: [u64; 2]) -> Self {
        Vector128 { u64s: values }
    }

    /// Retourne la representation en bytes
    pub fn as_bytes(&self) -> &[u8; 16] {
        unsafe { &self.bytes }
    }

    /// Addition vectorielle i32x4
    pub fn add_i32x4(&self, other: &Vector128) -> Vector128 {
        unsafe {
            let a = self.i32x4;
            let b = other.i32x4;
            let result = [
                a[0].wrapping_add(b[0]),
                a[1].wrapping_add(b[1]),
                a[2].wrapping_add(b[2]),
                a[3].wrapping_add(b[3]),
            ];
            Vector128 { i32x4: result }
        }
    }

    /// Soustraction vectorielle i32x4
    pub fn sub_i32x4(&self, other: &Vector128) -> Vector128 {
        unsafe {
            let a = self.i32x4;
            let b = other.i32x4;
            let result = [
                a[0].wrapping_sub(b[0]),
                a[1].wrapping_sub(b[1]),
                a[2].wrapping_sub(b[2]),
                a[3].wrapping_sub(b[3]),
            ];
            Vector128 { i32x4: result }
        }
    }

    /// Multiplication vectorielle i32x4
    pub fn mul_i32x4(&self, other: &Vector128) -> Vector128 {
        unsafe {
            let a = self.i32x4;
            let b = other.i32x4;
            let result = [
                a[0].wrapping_mul(b[0]),
                a[1].wrapping_mul(b[1]),
                a[2].wrapping_mul(b[2]),
                a[3].wrapping_mul(b[3]),
            ];
            Vector128 { i32x4: result }
        }
    }

    /// Addition vectorielle f32x4
    pub fn add_f32x4(&self, other: &Vector128) -> Vector128 {
        unsafe {
            let a = self.f32x4;
            let b = other.f32x4;
            let result = [
                a[0] + b[0],
                a[1] + b[1],
                a[2] + b[2],
                a[3] + b[3],
            ];
            Vector128 { f32x4: result }
        }
    }

    /// ET logique vectoriel
    pub fn and(&self, other: &Vector128) -> Vector128 {
        unsafe {
            let a = self.u64s;
            let b = other.u64s;
            let result = [a[0] & b[0], a[1] & b[1]];
            Vector128 { u64s: result }
        }
    }

    /// OU logique vectoriel
    pub fn or(&self, other: &Vector128) -> Vector128 {
        unsafe {
            let a = self.u64s;
            let b = other.u64s;
            let result = [a[0] | b[0], a[1] | b[1]];
            Vector128 { u64s: result }
        }
    }

    /// XOR vectoriel
    pub fn xor(&self, other: &Vector128) -> Vector128 {
        unsafe {
            let a = self.u64s;
            let b = other.u64s;
            let result = [a[0] ^ b[0], a[1] ^ b[1]];
            Vector128 { u64s: result }
        }
    }

    /// NOT vectoriel
    pub fn not(&self) -> Vector128 {
        unsafe {
            let a = self.u64s;
            let result = [!a[0], !a[1]];
            Vector128 { u64s: result }
        }
    }
}

impl Vector256 {
    /// Cree un nouveau vecteur 256-bit a zero
    pub fn zero() -> Self {
        Vector256 { bytes: [0; 32] }
    }

    /// Cree un vecteur a partir d'un array de bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Vector256 { bytes }
    }

    /// Cree un vecteur a partir de 8 valeurs i32
    pub fn from_i32x8(values: [i32; 8]) -> Self {
        Vector256 { i32x8: values }
    }

    /// Cree un vecteur a partir de 8 valeurs f32
    pub fn from_f32x8(values: [f32; 8]) -> Self {
        Vector256 { f32x8: values }
    }

    /// Cree un vecteur a partir de 16 valeurs i16
    pub fn from_i16x16(values: [i16; 16]) -> Self {
        Vector256 { i16x16: values }
    }

    /// Cree un vecteur a partir de 4 valeurs i64
    pub fn from_i64x4(values: [i64; 4]) -> Self {
        Vector256 { i64x4: values }
    }

    /// Cree un vecteur a partir de 4 valeurs f64
    pub fn from_f64x4(values: [f64; 4]) -> Self {
        Vector256 { f64x4: values }
    }

    /// Retourne la representation en bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        unsafe { &self.bytes }
    }

    /// Addition vectorielle i32x8
    pub fn add_i32x8(&self, other: &Vector256) -> Vector256 {
        unsafe {
            let a = self.i32x8;
            let b = other.i32x8;
            let mut result = [0i32; 8];
            for i in 0..8 {
                result[i] = a[i].wrapping_add(b[i]);
            }
            Vector256 { i32x8: result }
        }
    }

    /// Soustraction vectorielle i32x8
    pub fn sub_i32x8(&self, other: &Vector256) -> Vector256 {
        unsafe {
            let a = self.i32x8;
            let b = other.i32x8;
            let mut result = [0i32; 8];
            for i in 0..8 {
                result[i] = a[i].wrapping_sub(b[i]);
            }
            Vector256 { i32x8: result }
        }
    }

    /// Addition vectorielle f32x8
    pub fn add_f32x8(&self, other: &Vector256) -> Vector256 {
        unsafe {
            let a = self.f32x8;
            let b = other.f32x8;
            let mut result = [0.0f32; 8];
            for i in 0..8 {
                result[i] = a[i] + b[i];
            }
            Vector256 { f32x8: result }
        }
    }

    /// ET logique vectoriel
    pub fn and(&self, other: &Vector256) -> Vector256 {
        unsafe {
            let a = self.u64s;
            let b = other.u64s;
            let mut result = [0u64; 4];
            for i in 0..4 {
                result[i] = a[i] & b[i];
            }
            Vector256 { u64s: result }
        }
    }

    /// OU logique vectoriel
    pub fn or(&self, other: &Vector256) -> Vector256 {
        unsafe {
            let a = self.u64s;
            let b = other.u64s;
            let mut result = [0u64; 4];
            for i in 0..4 {
                result[i] = a[i] | b[i];
            }
            Vector256 { u64s: result }
        }
    }

    /// XOR vectoriel
    pub fn xor(&self, other: &Vector256) -> Vector256 {
        unsafe {
            let a = self.u64s;
            let b = other.u64s;
            let mut result = [0u64; 4];
            for i in 0..4 {
                result[i] = a[i] ^ b[i];
            }
            Vector256 { u64s: result }
        }
    }

    /// Divise en deux vecteurs 128-bit
    pub fn split(&self) -> (Vector128, Vector128) {
        unsafe {
            (self.v128x2[0], self.v128x2[1])
        }
    }

    /// Combine deux vecteurs 128-bit
    pub fn combine(low: Vector128, high: Vector128) -> Vector256 {
        Vector256 {
            v128x2: [low, high],
        }
    }
}

// Manual implementations of Debug and PartialEq for unions
impl std::fmt::Debug for Vector128 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe {
            f.debug_struct("Vector128")
                .field("bytes", &self.bytes)
                .finish()
        }
    }
}

impl PartialEq for Vector128 {
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            self.bytes == other.bytes
        }
    }
}

impl std::fmt::Debug for Vector256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe {
            f.debug_struct("Vector256")
                .field("bytes", &self.bytes)
                .finish()
        }
    }
}

impl PartialEq for Vector256 {
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            self.bytes == other.bytes
        }
    }
}

/// Format d'instruction SIMD
#[derive(Debug, Clone, PartialEq)]
pub struct SimdInstruction {
    /// Type d'operation SIMD
    pub operation: SimdOperation,
    /// Registre destination
    pub dst_reg: u8,
    /// Premier registre source
    pub src1_reg: u8,
    /// Deuxieme registre source (optionnel)
    pub src2_reg: Option<u8>,
    /// Immediat (optionnel)
    pub immediate: Option<u32>,
    /// Type de donnees vectorielles
    pub data_type: SimdDataType,
}

/// Types d'operations SIMD
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SimdOperation {
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
    Xor,
    Not,
    Load,
    Store,
    Mov,
    Cmp,
    Min,
    Max,
    Sqrt,
    Shuffle,
}

/// Types de donnees SIMD unifies
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SimdDataType {
    Vector128(VectorDataType),
    Vector256(Vector256DataType),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector128_creation() {
        let v = Vector128::zero();
        assert_eq!(v.as_bytes(), &[0; 16]);

        let v = Vector128::from_i32x4([1, 2, 3, 4]);
        unsafe {
            assert_eq!(v.i32x4, [1, 2, 3, 4]);
        }
    }

    #[test]
    fn test_vector128_operations() {
        let a = Vector128::from_i32x4([1, 2, 3, 4]);
        let b = Vector128::from_i32x4([5, 6, 7, 8]);
        
        let result = a.add_i32x4(&b);
        unsafe {
            assert_eq!(result.i32x4, [6, 8, 10, 12]);
        }

        let result = a.sub_i32x4(&b);
        unsafe {
            assert_eq!(result.i32x4, [-4, -4, -4, -4]);
        }
    }

    #[test]
    fn test_vector128_logical() {
        let a = Vector128::from_u64([0xFF00FF00FF00FF00, 0x00FF00FF00FF00FF]);
        let b = Vector128::from_u64([0xF0F0F0F0F0F0F0F0, 0x0F0F0F0F0F0F0F0F]);
        
        let result = a.and(&b);
        unsafe {
            assert_eq!(result.u64s, [0xF000F000F000F000, 0x000F000F000F000F]);
        }
    }

    #[test]
    fn test_vector256_creation() {
        let v = Vector256::zero();
        assert_eq!(v.as_bytes(), &[0; 32]);

        let v = Vector256::from_i32x8([1, 2, 3, 4, 5, 6, 7, 8]);
        unsafe {
            assert_eq!(v.i32x8, [1, 2, 3, 4, 5, 6, 7, 8]);
        }
    }

    #[test]
    fn test_vector256_split_combine() {
        let low = Vector128::from_i32x4([1, 2, 3, 4]);
        let high = Vector128::from_i32x4([5, 6, 7, 8]);
        
        let combined = Vector256::combine(low, high);
        let (split_low, split_high) = combined.split();
        
        unsafe {
            assert_eq!(split_low.i32x4, [1, 2, 3, 4]);
            assert_eq!(split_high.i32x4, [5, 6, 7, 8]);
        }
    }

    // Tests d'intégration SIMD/FPU avec ALU et Pipeline
    #[test]
    fn test_vector_alu_integration() {
        use crate::alu::v_alu::{VectorALU, VectorOperation};

        let mut v_alu = VectorALU::new();

        // Test d'addition vectorielle 128-bit
        let vec1 = Vector128::from_i32x4([1, 2, 3, 4]);
        let vec2 = Vector128::from_i32x4([10, 20, 30, 40]);

        v_alu.write_v128(0, vec1).unwrap();
        v_alu.write_v128(1, vec2).unwrap();

        v_alu.execute_v128(
            VectorOperation::Add,
            2, // dst
            0, // src1
            Some(1), // src2
            VectorDataType::I32x4,
        ).unwrap();

        let result = v_alu.read_v128(2).unwrap();
        unsafe {
            assert_eq!(result.i32x4, [11, 22, 33, 44]);
        }
    }

    #[test]
    fn test_vector_alu_256_integration() {
        use crate::alu::v_alu::{VectorALU, VectorOperation};

        let mut v_alu = VectorALU::new();

        // Test d'addition vectorielle 256-bit
        let vec1 = Vector256::from_i32x8([1, 2, 3, 4, 5, 6, 7, 8]);
        let vec2 = Vector256::from_i32x8([10, 20, 30, 40, 50, 60, 70, 80]);

        v_alu.write_v256(0, vec1).unwrap();
        v_alu.write_v256(1, vec2).unwrap();

        v_alu.execute_v256(
            VectorOperation::Add,
            2, // dst
            0, // src1
            Some(1), // src2
            Vector256DataType::I32x8,
        ).unwrap();

        let result = v_alu.read_v256(2).unwrap();
        unsafe {
            assert_eq!(result.i32x8, [11, 22, 33, 44, 55, 66, 77, 88]);
        }
    }

    #[test]
    fn test_fpu_integration() {
        use crate::alu::fpu::{FPU, FPUOperation, FloatPrecision};

        let mut fpu = FPU::new();

        // Test d'opérations FPU
        fpu.write_fp_register(0, 10.5).unwrap();
        fpu.write_fp_register(1, 2.5).unwrap();

        // Addition
        fpu.execute(FPUOperation::Add, 2, 0, Some(1), FloatPrecision::Double).unwrap();
        let result = fpu.read_fp_register(2).unwrap();
        assert_eq!(result, 13.0);

        // Division
        fpu.execute(FPUOperation::Div, 3, 0, Some(1), FloatPrecision::Double).unwrap();
        let result = fpu.read_fp_register(3).unwrap();
        assert_eq!(result, 4.2);

        // Racine carrée
        fpu.write_fp_register(4, 16.0).unwrap();
        fpu.execute(FPUOperation::Sqrt, 5, 4, None, FloatPrecision::Double).unwrap();
        let result = fpu.read_fp_register(5).unwrap();
        assert_eq!(result, 4.0);
    }

    #[test]
    fn test_simd_logical_operations() {
        use crate::alu::v_alu::{VectorALU, VectorOperation};

        let mut v_alu = VectorALU::new();

        let vec1 = Vector128::from_u64([0xFF00FF00FF00FF00, 0x00FF00FF00FF00FF]);
        let vec2 = Vector128::from_u64([0xF0F0F0F0F0F0F0F0, 0x0F0F0F0F0F0F0F0F]);

        v_alu.write_v128(0, vec1).unwrap();
        v_alu.write_v128(1, vec2).unwrap();

        // Test AND
        v_alu.execute_v128(
            VectorOperation::And,
            2,
            0,
            Some(1),
            VectorDataType::I64x2,
        ).unwrap();

        let result = v_alu.read_v128(2).unwrap();
        unsafe {
            assert_eq!(result.u64s, [0xF000F000F000F000, 0x000F000F000F000F]);
        }
    }

    #[test]
    fn test_fpu_precision_modes() {
        use crate::alu::fpu::{FPU, FPUOperation, FloatPrecision};

        let mut fpu = FPU::new();

        fpu.write_fp_register(0, 3.14159265359).unwrap();

        // Test conversion simple → double
        fpu.execute(FPUOperation::Convert, 1, 0, None, FloatPrecision::Single).unwrap();
        let result = fpu.read_fp_register(1).unwrap();
        
        // En simple précision, on perd de la précision
        assert!((result - 3.1415927).abs() < 0.0000001);

        // Test conversion double (pas de changement)
        fpu.execute(FPUOperation::Convert, 2, 0, None, FloatPrecision::Double).unwrap();
        let result = fpu.read_fp_register(2).unwrap();
        assert_eq!(result, 3.14159265359);
    }

    #[test]
    fn test_execute_stage_with_simd() {
        use crate::alu::alu::ALU;
        use crate::bytecode::opcodes::Opcode;
        use crate::bytecode::instructions::Instruction;
        use crate::pipeline::execute::ExecuteStage;
        use crate::pipeline::DecodeExecuteRegister;

        let mut execute_stage = ExecuteStage::new();
        let mut alu = ALU::new();

        // Test d'une instruction SIMD 128-bit Add
        let simd_instruction = Instruction::create_reg_reg_reg(Opcode::Simd128Add, 2, 0, 1);

        let de_reg = DecodeExecuteRegister {
            instruction: simd_instruction,
            pc: 100,
            rs1: Some(0), // V0
            rs2: Some(1), // V1
            rd: Some(2),  // V2
            rs1_value: 0, // Pas utilisé pour SIMD
            rs2_value: 0, // Pas utilisé pour SIMD
            immediate: None,
            branch_addr: None,
            branch_prediction: None,
            stack_operation: None,
            mem_addr: None,
            stack_value: None,
        };

        // Initialiser les registres vectoriels avec des valeurs de test
        let vec1 = Vector128::from_i32x4([1, 2, 3, 4]);
        let vec2 = Vector128::from_i32x4([5, 6, 7, 8]);
        
        execute_stage.get_vector_alu_mut().write_v128(0, vec1).unwrap();
        execute_stage.get_vector_alu_mut().write_v128(1, vec2).unwrap();

        // Exécuter l'instruction
        let result = execute_stage.process_direct(&de_reg, &mut alu);
        assert!(result.is_ok());

        // Vérifier le résultat
        let result_vector = execute_stage.get_vector_alu().read_v128(2).unwrap();
        unsafe {
            assert_eq!(result_vector.i32x4, [6, 8, 10, 12]);
        }
    }

    #[test]
    fn test_execute_stage_with_fpu() {
        use crate::alu::alu::ALU;
        use crate::bytecode::opcodes::Opcode;
        use crate::bytecode::instructions::Instruction;
        use crate::pipeline::execute::ExecuteStage;
        use crate::pipeline::DecodeExecuteRegister;

        let mut execute_stage = ExecuteStage::new();
        let mut alu = ALU::new();

        // Test d'une instruction FPU Add
        let fpu_instruction = Instruction::create_reg_reg_reg(Opcode::FpuAdd, 2, 0, 1);

        let de_reg = DecodeExecuteRegister {
            instruction: fpu_instruction,
            pc: 100,
            rs1: Some(0), // F0
            rs2: Some(1), // F1
            rd: Some(2),  // F2
            rs1_value: 0, // Pas utilisé pour FPU
            rs2_value: 0, // Pas utilisé pour FPU
            immediate: None,
            branch_addr: None,
            branch_prediction: None,
            stack_operation: None,
            mem_addr: None,
            stack_value: None,
        };

        // Initialiser les registres FPU avec des valeurs de test
        execute_stage.get_fpu_mut().write_fp_register(0, 2.5).unwrap();
        execute_stage.get_fpu_mut().write_fp_register(1, 3.7).unwrap();

        // Exécuter l'instruction
        let result = execute_stage.process_direct(&de_reg, &mut alu);
        assert!(result.is_ok());

        // Vérifier le résultat
        let result_value = execute_stage.get_fpu().read_fp_register(2).unwrap();
        assert_eq!(result_value, 6.2);
    }
}
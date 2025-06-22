//src/alu/v_alu.rs

use crate::bytecode::simds::{Vector128, Vector256, VectorDataType, Vector256DataType};
use crate::pvm::vm_errors::{VMResult, VMError};

/// ALU Vectorielle pour operations SIMD
#[derive(Debug, Clone)]
pub struct VectorALU {
    /// Registres vectoriels 128-bit (V0-V15)
    pub v128_registers: [Vector128; 16],
    /// Registres vectoriels 256-bit (Y0-Y15)
    pub v256_registers: [Vector256; 16],
    /// Flags de status vectoriel
    pub flags: VectorFlags,
    /// Cache pour optimiser les opérations répétitives
    pub operation_cache: VectorOperationCache,
    /// Statistiques de performance SIMD
    pub simd_stats: SimdPerformanceStats,
}

/// Cache pour optimiser les opérations vectorielles répétitives
#[derive(Debug, Clone)]
pub struct VectorOperationCache {
    /// Dernière opération 128-bit effectuée
    pub last_128_op: Option<(VectorOperation, u8, u8, VectorDataType)>,
    /// Résultat de la dernière opération 128-bit
    pub last_128_result: Option<Vector128>,
    /// Dernière opération 256-bit effectuée
    pub last_256_op: Option<(VectorOperation, u8, u8, Vector256DataType)>,
    /// Résultat de la dernière opération 256-bit
    pub last_256_result: Option<Vector256>,
    /// Compteur de hits du cache
    pub cache_hits: u64,
    /// Compteur de misses du cache
    pub cache_misses: u64,
}

/// Statistiques de performance SIMD
#[derive(Debug, Clone, Default)]
pub struct SimdPerformanceStats {
    /// Nombre total d'opérations SIMD 128-bit
    pub simd128_ops: u64,
    /// Nombre total d'opérations SIMD 256-bit
    pub simd256_ops: u64,
    /// Temps total d'exécution SIMD (en cycles simulés)
    pub total_simd_cycles: u64,
    /// Opérations par cycle SIMD moyen
    pub simd_ops_per_cycle: f64,
    /// Nombre d'opérations parallélisées
    pub parallel_ops: u64,
}

impl VectorOperationCache {
    pub fn new() -> Self {
        Self {
            last_128_op: None,
            last_128_result: None,
            last_256_op: None,
            last_256_result: None,
            cache_hits: 0,
            cache_misses: 0,
        }
    }
    
    /// Vérifie si l'opération 128-bit est en cache
    pub fn check_128_cache(&self, op: VectorOperation, src1: u8, src2: u8, data_type: VectorDataType) -> Option<Vector128> {
        if let Some((cached_op, cached_src1, cached_src2, cached_type)) = &self.last_128_op {
            if *cached_op == op && *cached_src1 == src1 && *cached_src2 == src2 && *cached_type == data_type {
                return self.last_128_result;
            }
        }
        None
    }
    
    /// Met en cache le résultat d'une opération 128-bit
    pub fn cache_128_result(&mut self, op: VectorOperation, src1: u8, src2: u8, data_type: VectorDataType, result: Vector128) {
        self.last_128_op = Some((op, src1, src2, data_type));
        self.last_128_result = Some(result);
    }
    
    /// Statistiques du cache
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total > 0 {
            (self.cache_hits as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }
}

/// Flags de status pour operations vectorielles
#[derive(Debug, Clone, Copy, Default)]
pub struct VectorFlags {
    /// Flag zero (tous les elements sont zero)
    pub zero: bool,
    /// Flag signe (element le plus significatif est negatif)
    pub sign: bool,
    /// Flag overflow (au moins un element a overflow)
    pub overflow: bool,
    /// Flag underflow (au moins un element a underflow)
    pub underflow: bool,
    /// Flag denormal (au moins un element est denormalize)
    pub denormal: bool,
    /// Flag invalid (operation invalide detectee)
    pub invalid: bool,
}

/// Types d'operations vectorielles
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VectorOperation {
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
    Xor,
    Not,
    Min,
    Max,
    Sqrt,
    Cmp,
    Shuffle,

}

/// Resultats d'operations vectorielles
#[derive(Debug, Clone, PartialEq)]
pub enum VectorResult {
    Vector128(Vector128),
    Vector256(Vector256),
}

impl Default for VectorALU {
    fn default() -> Self {
        Self::new()
    }
}

impl VectorALU {
    /// Cree une nouvelle ALU vectorielle
    pub fn new() -> Self {
        Self {
            v128_registers: [Vector128::zero(); 16],
            v256_registers: [Vector256::zero(); 16],
            flags: VectorFlags::default(),
            operation_cache: VectorOperationCache::new(),
            simd_stats: SimdPerformanceStats::default(),
        }
    }

    /// Remet a zero tous les registres vectoriels
    pub fn reset(&mut self) {
        for i in 0..16 {
            self.v128_registers[i] = Vector128::zero();
            self.v256_registers[i] = Vector256::zero();
        }
        self.flags = VectorFlags::default();
    }

    /// Lit un registre vectoriel 128-bit
    pub fn read_v128(&self, reg: u8) -> VMResult<Vector128> {
        if reg >= 16 {
            return Err(VMError::register_error(&format!("Invalid V128 register: {}", reg)));
        }
        Ok(self.v128_registers[reg as usize])
    }

    /// Ecrit un registre vectoriel 128-bit
    pub fn write_v128(&mut self, reg: u8, value: Vector128) -> VMResult<()> {
        if reg >= 16 {
            return Err(VMError::register_error(&format!("Invalid V128 register: {}", reg)));
        }
        self.v128_registers[reg as usize] = value;
        self.update_flags_128(&value);
        Ok(())
    }

    /// Lit un registre vectoriel 256-bit
    pub fn read_v256(&self, reg: u8) -> VMResult<Vector256> {
        if reg >= 16 {
            return Err(VMError::register_error(&format!("Invalid V256 register: {}", reg)));
        }
        Ok(self.v256_registers[reg as usize])
    }

    /// Ecrit un registre vectoriel 256-bit
    pub fn write_v256(&mut self, reg: u8, value: Vector256) -> VMResult<()> {
        if reg >= 16 {
            return Err(VMError::register_error(&format!("Invalid V256 register: {}", reg)));
        }
        self.v256_registers[reg as usize] = value;
        self.update_flags_256(&value);
        Ok(())
    }

    /// Execute une operation vectorielle 128-bit
    pub fn execute_v128(
        &mut self,
        op: VectorOperation,
        dst: u8,
        src1: u8,
        src2: Option<u8>,
        data_type: VectorDataType,
    ) -> VMResult<()> {
        // Optimisation SIMD : Vérifier le cache d'abord
        let src2_reg = src2.unwrap_or(0);
        if let Some(cached_result) = self.operation_cache.check_128_cache(op, src1, src2_reg, data_type) {
            self.operation_cache.cache_hits += 1;
            self.write_v128(dst, cached_result)?;
            println!("SIMD Cache Hit: V{} = cached result", dst);
            return Ok(());
        }
        
        // Cache miss - exécuter l'opération normalement
        self.operation_cache.cache_misses += 1;
        self.simd_stats.simd128_ops += 1;
        
        let vec1 = self.read_v128(src1)?;
        
        let result = match op {
            VectorOperation::Add => {
                let vec2 = self.read_v128(src2.ok_or(VMError::instruction_error("Missing second operand for SIMD Add"))?)?;
                self.add_v128(&vec1, &vec2, data_type)?
            }
            VectorOperation::Sub => {
                let vec2 = self.read_v128(src2.ok_or(VMError::instruction_error("Invalid SIMD instruction"))?)?;
                self.sub_v128(&vec1, &vec2, data_type)?
            }
            VectorOperation::Mul => {
                let vec2 = self.read_v128(src2.ok_or(VMError::instruction_error("Invalid SIMD instruction"))?)?;
                self.mul_v128(&vec1, &vec2, data_type)?
            }
            VectorOperation::Div => {
                let vec2 = self.read_v128(src2.ok_or(VMError::instruction_error("Invalid SIMD instruction"))?)?;
                self.div_v128(&vec1, &vec2, data_type)?
            }
            VectorOperation::And => {
                let vec2 = self.read_v128(src2.ok_or(VMError::instruction_error("Invalid SIMD instruction"))?)?;
                vec1.and(&vec2)
            }
            VectorOperation::Or => {
                let vec2 = self.read_v128(src2.ok_or(VMError::instruction_error("Invalid SIMD instruction"))?)?;
                vec1.or(&vec2)
            }
            VectorOperation::Xor => {
                let vec2 = self.read_v128(src2.ok_or(VMError::instruction_error("Invalid SIMD instruction"))?)?;
                vec1.xor(&vec2)
            }
            VectorOperation::Not => vec1.not(),
            VectorOperation::Min => {
                let vec2 = self.read_v128(src2.ok_or(VMError::instruction_error("Invalid SIMD instruction"))?)?;
                self.min_v128(&vec1, &vec2, data_type)?
            }
            VectorOperation::Max => {
                let vec2 = self.read_v128(src2.ok_or(VMError::instruction_error("Invalid SIMD instruction"))?)?;
                self.max_v128(&vec1, &vec2, data_type)?
            }
            VectorOperation::Sqrt => self.sqrt_v128(&vec1, data_type)?,
            VectorOperation::Cmp => {
                let vec2 = self.read_v128(src2.ok_or(VMError::instruction_error("Invalid SIMD instruction"))?)?;
                self.cmp_v128(&vec1, &vec2, data_type)?
            }
            VectorOperation::Shuffle => {
                // Shuffle avec masque dans src2
                let mask = self.read_v128(src2.ok_or(VMError::instruction_error("Invalid SIMD instruction"))?)?;
                self.shuffle_v128(&vec1, &mask)?
            }
        };

        // Mettre en cache le résultat pour les futurs accès
        self.operation_cache.cache_128_result(op, src1, src2_reg, data_type, result);
        
        self.write_v128(dst, result)
    }

    /// Execute une operation vectorielle 256-bit
    pub fn execute_v256(
        &mut self,
        op: VectorOperation,
        dst: u8,
        src1: u8,
        src2: Option<u8>,
        data_type: Vector256DataType,
    ) -> VMResult<()> {
        let vec1 = self.read_v256(src1)?;
        
        let result = match op {
            VectorOperation::Add => {
                let vec2 = self.read_v256(src2.ok_or(VMError::instruction_error("Invalid SIMD instruction"))?)?;
                self.add_v256(&vec1, &vec2, data_type)?
            }
            VectorOperation::Sub => {
                let vec2 = self.read_v256(src2.ok_or(VMError::instruction_error("Invalid SIMD instruction"))?)?;
                self.sub_v256(&vec1, &vec2, data_type)?
            }
            VectorOperation::Mul => {
                let vec2 = self.read_v256(src2.ok_or(VMError::instruction_error("Invalid SIMD instruction"))?)?;
                self.mul_v256(&vec1, &vec2, data_type)?
            }
            VectorOperation::Div => {
                let vec2 = self.read_v256(src2.ok_or(VMError::instruction_error("Invalid SIMD instruction"))?)?;
                self.div_v256(&vec1, &vec2, data_type)?
            }
            VectorOperation::And => {
                let vec2 = self.read_v256(src2.ok_or(VMError::instruction_error("Invalid SIMD instruction"))?)?;
                vec1.and(&vec2)
            }
            VectorOperation::Or => {
                let vec2 = self.read_v256(src2.ok_or(VMError::instruction_error("Invalid SIMD instruction"))?)?;
                vec1.or(&vec2)
            }
            VectorOperation::Xor => {
                let vec2 = self.read_v256(src2.ok_or(VMError::instruction_error("Invalid SIMD instruction"))?)?;
                vec1.xor(&vec2)
            }
            VectorOperation::Not => {
                // NOT operation pour 256-bit
                self.not_v256(&vec1)
            }
            VectorOperation::Min => {
                let vec2 = self.read_v256(src2.ok_or(VMError::instruction_error("Invalid SIMD instruction"))?)?;
                self.min_v256(&vec1, &vec2, data_type)?
            }
            VectorOperation::Max => {
                let vec2 = self.read_v256(src2.ok_or(VMError::instruction_error("Invalid SIMD instruction"))?)?;
                self.max_v256(&vec1, &vec2, data_type)?
            }
            VectorOperation::Sqrt => self.sqrt_v256(&vec1, data_type)?,
            VectorOperation::Cmp => {
                let vec2 = self.read_v256(src2.ok_or(VMError::instruction_error("Invalid SIMD instruction"))?)?;
                self.cmp_v256(&vec1, &vec2, data_type)?
            }
            VectorOperation::Shuffle => {
                let mask = self.read_v256(src2.ok_or(VMError::instruction_error("Invalid SIMD instruction"))?)?;
                self.shuffle_v256(&vec1, &mask)?
            }
        };

        self.write_v256(dst, result)
    }

    // Operations arithmetiques 128-bit specifiques par type

    /// Addition vectorielle 128-bit selon le type de donnees
    fn add_v128(&self, a: &Vector128, b: &Vector128, data_type: VectorDataType) -> VMResult<Vector128> {
        match data_type {
            VectorDataType::I32x4 => Ok(a.add_i32x4(b)),
            VectorDataType::F32x4 => Ok(a.add_f32x4(b)),
            VectorDataType::I16x8 => {
                unsafe {
                    let a_vals = a.i16x8;
                    let b_vals = b.i16x8;
                    let mut result = [0i16; 8];
                    for i in 0..8 {
                        result[i] = a_vals[i].wrapping_add(b_vals[i]);
                    }
                    Ok(Vector128::from_i16x8(result))
                }
            }
            VectorDataType::I64x2 => {
                unsafe {
                    let a_vals = a.i64x2;
                    let b_vals = b.i64x2;
                    let result = [
                        a_vals[0].wrapping_add(b_vals[0]),
                        a_vals[1].wrapping_add(b_vals[1]),
                    ];
                    Ok(Vector128 { i64x2: result })
                }
            }
            VectorDataType::F64x2 => {
                unsafe {
                    let a_vals = a.f64x2;
                    let b_vals = b.f64x2;
                    let result = [a_vals[0] + b_vals[0], a_vals[1] + b_vals[1]];
                    Ok(Vector128::from_f64x2(result))
                }
            }
        }
    }

    /// Soustraction vectorielle 128-bit
    fn sub_v128(&self, a: &Vector128, b: &Vector128, data_type: VectorDataType) -> VMResult<Vector128> {
        match data_type {
            VectorDataType::I32x4 => Ok(a.sub_i32x4(b)),
            VectorDataType::F32x4 => {
                unsafe {
                    let a_vals = a.f32x4;
                    let b_vals = b.f32x4;
                    let result = [
                        a_vals[0] - b_vals[0],
                        a_vals[1] - b_vals[1],
                        a_vals[2] - b_vals[2],
                        a_vals[3] - b_vals[3],
                    ];
                    Ok(Vector128::from_f32x4(result))
                }
            }
            VectorDataType::I16x8 => {
                unsafe {
                    let a_vals = a.i16x8;
                    let b_vals = b.i16x8;
                    let mut result = [0i16; 8];
                    for i in 0..8 {
                        result[i] = a_vals[i].wrapping_sub(b_vals[i]);
                    }
                    Ok(Vector128::from_i16x8(result))
                }
            }
            VectorDataType::I64x2 => {
                unsafe {
                    let a_vals = a.i64x2;
                    let b_vals = b.i64x2;
                    let result = [
                        a_vals[0].wrapping_sub(b_vals[0]),
                        a_vals[1].wrapping_sub(b_vals[1]),
                    ];
                    Ok(Vector128 { i64x2: result })
                }
            }
            VectorDataType::F64x2 => {
                unsafe {
                    let a_vals = a.f64x2;
                    let b_vals = b.f64x2;
                    let result = [a_vals[0] - b_vals[0], a_vals[1] - b_vals[1]];
                    Ok(Vector128::from_f64x2(result))
                }
            }
        }
    }

    /// Multiplication vectorielle 128-bit
    fn mul_v128(&self, a: &Vector128, b: &Vector128, data_type: VectorDataType) -> VMResult<Vector128> {
        match data_type {
            VectorDataType::I32x4 => Ok(a.mul_i32x4(b)),
            VectorDataType::F32x4 => {
                unsafe {
                    let a_vals = a.f32x4;
                    let b_vals = b.f32x4;
                    let result = [
                        a_vals[0] * b_vals[0],
                        a_vals[1] * b_vals[1],
                        a_vals[2] * b_vals[2],
                        a_vals[3] * b_vals[3],
                    ];
                    Ok(Vector128::from_f32x4(result))
                }
            }
            VectorDataType::I16x8 => {
                unsafe {
                    let a_vals = a.i16x8;
                    let b_vals = b.i16x8;
                    let mut result = [0i16; 8];
                    for i in 0..8 {
                        result[i] = a_vals[i].wrapping_mul(b_vals[i]);
                    }
                    Ok(Vector128::from_i16x8(result))
                }
            }
            VectorDataType::I64x2 => {
                unsafe {
                    let a_vals = a.i64x2;
                    let b_vals = b.i64x2;
                    let result = [
                        a_vals[0].wrapping_mul(b_vals[0]),
                        a_vals[1].wrapping_mul(b_vals[1]),
                    ];
                    Ok(Vector128 { i64x2: result })
                }
            }
            VectorDataType::F64x2 => {
                unsafe {
                    let a_vals = a.f64x2;
                    let b_vals = b.f64x2;
                    let result = [a_vals[0] * b_vals[0], a_vals[1] * b_vals[1]];
                    Ok(Vector128::from_f64x2(result))
                }
            }
        }
    }

    /// Division vectorielle 128-bit
    fn div_v128(&self, a: &Vector128, b: &Vector128, data_type: VectorDataType) -> VMResult<Vector128> {
        match data_type {
            VectorDataType::I32x4 => {
                unsafe {
                    let a_vals = a.i32x4;
                    let b_vals = b.i32x4;
                    let mut result = [0i32; 4];
                    for i in 0..4 {
                        if b_vals[i] == 0 {
                            return Err(VMError::arithmetic_error("Division by zero"));
                        }
                        result[i] = a_vals[i].wrapping_div(b_vals[i]);
                    }
                    Ok(Vector128::from_i32x4(result))
                }
            }
            VectorDataType::F32x4 => {
                unsafe {
                    let a_vals = a.f32x4;
                    let b_vals = b.f32x4;
                    let result = [
                        a_vals[0] / b_vals[0],
                        a_vals[1] / b_vals[1],
                        a_vals[2] / b_vals[2],
                        a_vals[3] / b_vals[3],
                    ];
                    Ok(Vector128::from_f32x4(result))
                }
            }
            VectorDataType::I16x8 => {
                unsafe {
                    let a_vals = a.i16x8;
                    let b_vals = b.i16x8;
                    let mut result = [0i16; 8];
                    for i in 0..8 {
                        if b_vals[i] == 0 {
                            return Err(VMError::arithmetic_error("Division by zero"));
                        }
                        result[i] = a_vals[i].wrapping_div(b_vals[i]);
                    }
                    Ok(Vector128::from_i16x8(result))
                }
            }
            VectorDataType::I64x2 => {
                unsafe {
                    let a_vals = a.i64x2;
                    let b_vals = b.i64x2;
                    if b_vals[0] == 0 || b_vals[1] == 0 {
                        return Err(VMError::arithmetic_error("Division by zero"));
                    }
                    let result = [
                        a_vals[0].wrapping_div(b_vals[0]),
                        a_vals[1].wrapping_div(b_vals[1]),
                    ];
                    Ok(Vector128 { i64x2: result })
                }
            }
            VectorDataType::F64x2 => {
                unsafe {
                    let a_vals = a.f64x2;
                    let b_vals = b.f64x2;
                    let result = [a_vals[0] / b_vals[0], a_vals[1] / b_vals[1]];
                    Ok(Vector128::from_f64x2(result))
                }
            }
        }
    }

    /// Minimum vectoriel 128-bit
    fn min_v128(&self, a: &Vector128, b: &Vector128, data_type: VectorDataType) -> VMResult<Vector128> {
        match data_type {
            VectorDataType::I32x4 => {
                unsafe {
                    let a_vals = a.i32x4;
                    let b_vals = b.i32x4;
                    let result = [
                        a_vals[0].min(b_vals[0]),
                        a_vals[1].min(b_vals[1]),
                        a_vals[2].min(b_vals[2]),
                        a_vals[3].min(b_vals[3]),
                    ];
                    Ok(Vector128::from_i32x4(result))
                }
            }
            VectorDataType::F32x4 => {
                unsafe {
                    let a_vals = a.f32x4;
                    let b_vals = b.f32x4;
                    let result = [
                        a_vals[0].min(b_vals[0]),
                        a_vals[1].min(b_vals[1]),
                        a_vals[2].min(b_vals[2]),
                        a_vals[3].min(b_vals[3]),
                    ];
                    Ok(Vector128::from_f32x4(result))
                }
            }
            VectorDataType::I16x8 => {
                unsafe {
                    let a_vals = a.i16x8;
                    let b_vals = b.i16x8;
                    let mut result = [0i16; 8];
                    for i in 0..8 {
                        result[i] = a_vals[i].min(b_vals[i]);
                    }
                    Ok(Vector128::from_i16x8(result))
                }
            }
            VectorDataType::I64x2 => {
                unsafe {
                    let a_vals = a.i64x2;
                    let b_vals = b.i64x2;
                    let result = [a_vals[0].min(b_vals[0]), a_vals[1].min(b_vals[1])];
                    Ok(Vector128 { i64x2: result })
                }
            }
            VectorDataType::F64x2 => {
                unsafe {
                    let a_vals = a.f64x2;
                    let b_vals = b.f64x2;
                    let result = [a_vals[0].min(b_vals[0]), a_vals[1].min(b_vals[1])];
                    Ok(Vector128::from_f64x2(result))
                }
            }
        }
    }

    /// Maximum vectoriel 128-bit
    fn max_v128(&self, a: &Vector128, b: &Vector128, data_type: VectorDataType) -> VMResult<Vector128> {
        match data_type {
            VectorDataType::I32x4 => {
                unsafe {
                    let a_vals = a.i32x4;
                    let b_vals = b.i32x4;
                    let result = [
                        a_vals[0].max(b_vals[0]),
                        a_vals[1].max(b_vals[1]),
                        a_vals[2].max(b_vals[2]),
                        a_vals[3].max(b_vals[3]),
                    ];
                    Ok(Vector128::from_i32x4(result))
                }
            }
            VectorDataType::F32x4 => {
                unsafe {
                    let a_vals = a.f32x4;
                    let b_vals = b.f32x4;
                    let result = [
                        a_vals[0].max(b_vals[0]),
                        a_vals[1].max(b_vals[1]),
                        a_vals[2].max(b_vals[2]),
                        a_vals[3].max(b_vals[3]),
                    ];
                    Ok(Vector128::from_f32x4(result))
                }
            }
            VectorDataType::I16x8 => {
                unsafe {
                    let a_vals = a.i16x8;
                    let b_vals = b.i16x8;
                    let mut result = [0i16; 8];
                    for i in 0..8 {
                        result[i] = a_vals[i].max(b_vals[i]);
                    }
                    Ok(Vector128::from_i16x8(result))
                }
            }
            VectorDataType::I64x2 => {
                unsafe {
                    let a_vals = a.i64x2;
                    let b_vals = b.i64x2;
                    let result = [a_vals[0].max(b_vals[0]), a_vals[1].max(b_vals[1])];
                    Ok(Vector128 { i64x2: result })
                }
            }
            VectorDataType::F64x2 => {
                unsafe {
                    let a_vals = a.f64x2;
                    let b_vals = b.f64x2;
                    let result = [a_vals[0].max(b_vals[0]), a_vals[1].max(b_vals[1])];
                    Ok(Vector128::from_f64x2(result))
                }
            }
        }
    }

    /// Racine carree vectorielle 128-bit
    fn sqrt_v128(&self, a: &Vector128, data_type: VectorDataType) -> VMResult<Vector128> {
        match data_type {
            VectorDataType::F32x4 => {
                unsafe {
                    let a_vals = a.f32x4;
                    let result = [
                        a_vals[0].sqrt(),
                        a_vals[1].sqrt(),
                        a_vals[2].sqrt(),
                        a_vals[3].sqrt(),
                    ];
                    Ok(Vector128::from_f32x4(result))
                }
            }
            VectorDataType::F64x2 => {
                unsafe {
                    let a_vals = a.f64x2;
                    let result = [a_vals[0].sqrt(), a_vals[1].sqrt()];
                    Ok(Vector128::from_f64x2(result))
                }
            }
            _ => Err(VMError::instruction_error("Invalid SIMD instruction")), // Sqrt uniquement pour flottants
        }
    }

    /// Comparaison vectorielle 128-bit (retourne masque de bits)
    fn cmp_v128(&self, a: &Vector128, b: &Vector128, data_type: VectorDataType) -> VMResult<Vector128> {
        match data_type {
            VectorDataType::I32x4 => {
                unsafe {
                    let a_vals = a.i32x4;
                    let b_vals = b.i32x4;
                    let result = [
                        if a_vals[0] == b_vals[0] { -1i32 } else { 0 },
                        if a_vals[1] == b_vals[1] { -1i32 } else { 0 },
                        if a_vals[2] == b_vals[2] { -1i32 } else { 0 },
                        if a_vals[3] == b_vals[3] { -1i32 } else { 0 },
                    ];
                    Ok(Vector128::from_i32x4(result))
                }
            }
            VectorDataType::F32x4 => {
                unsafe {
                    let a_vals = a.f32x4;
                    let b_vals = b.f32x4;
                    let result = [
                        if a_vals[0] == b_vals[0] { f32::from_bits(0xFFFFFFFF) } else { 0.0 },
                        if a_vals[1] == b_vals[1] { f32::from_bits(0xFFFFFFFF) } else { 0.0 },
                        if a_vals[2] == b_vals[2] { f32::from_bits(0xFFFFFFFF) } else { 0.0 },
                        if a_vals[3] == b_vals[3] { f32::from_bits(0xFFFFFFFF) } else { 0.0 },
                    ];
                    Ok(Vector128::from_f32x4(result))
                }
            }
            VectorDataType::I16x8 => {
                unsafe {
                    let a_vals = a.i16x8;
                    let b_vals = b.i16x8;
                    let mut result = [0i16; 8];
                    for i in 0..8 {
                        result[i] = if a_vals[i] == b_vals[i] { -1i16 } else { 0 };
                    }
                    Ok(Vector128::from_i16x8(result))
                }
            }
            VectorDataType::I64x2 => {
                unsafe {
                    let a_vals = a.i64x2;
                    let b_vals = b.i64x2;
                    let result = [
                        if a_vals[0] == b_vals[0] { -1i64 } else { 0 },
                        if a_vals[1] == b_vals[1] { -1i64 } else { 0 },
                    ];
                    Ok(Vector128 { i64x2: result })
                }
            }
            VectorDataType::F64x2 => {
                unsafe {
                    let a_vals = a.f64x2;
                    let b_vals = b.f64x2;
                    let result = [
                        if a_vals[0] == b_vals[0] { f64::from_bits(0xFFFFFFFFFFFFFFFF) } else { 0.0 },
                        if a_vals[1] == b_vals[1] { f64::from_bits(0xFFFFFFFFFFFFFFFF) } else { 0.0 },
                    ];
                    Ok(Vector128::from_f64x2(result))
                }
            }
        }
    }

    /// Shuffle vectoriel 128-bit (reordonne les elements selon un masque)
    fn shuffle_v128(&self, a: &Vector128, mask: &Vector128) -> VMResult<Vector128> {
        unsafe {
            let a_bytes = a.bytes;
            let mask_bytes = mask.bytes;
            let mut result = [0u8; 16];
            
            for i in 0..16 {
                let idx = (mask_bytes[i] & 0x0F) as usize; // Masque sur 4 bits pour 16 elements
                if idx < 16 {
                    result[i] = a_bytes[idx];
                }
            }
            
            Ok(Vector128::from_bytes(result))
        }
    }

    // Operations 256-bit

    /// Addition vectorielle 256-bit
    fn add_v256(&self, a: &Vector256, b: &Vector256, data_type: Vector256DataType) -> VMResult<Vector256> {
        match data_type {
            Vector256DataType::I32x8 => Ok(a.add_i32x8(b)),
            Vector256DataType::F32x8 => Ok(a.add_f32x8(b)),
            Vector256DataType::I16x16 => {
                unsafe {
                    let a_vals = a.i16x16;
                    let b_vals = b.i16x16;
                    let mut result = [0i16; 16];
                    for i in 0..16 {
                        result[i] = a_vals[i].wrapping_add(b_vals[i]);
                    }
                    Ok(Vector256 { i16x16: result })
                }
            }
            Vector256DataType::I64x4 => {
                unsafe {
                    let a_vals = a.i64x4;
                    let b_vals = b.i64x4;
                    let mut result = [0i64; 4];
                    for i in 0..4 {
                        result[i] = a_vals[i].wrapping_add(b_vals[i]);
                    }
                    Ok(Vector256 { i64x4: result })
                }
            }
            Vector256DataType::F64x4 => {
                unsafe {
                    let a_vals = a.f64x4;
                    let b_vals = b.f64x4;
                    let mut result = [0.0f64; 4];
                    for i in 0..4 {
                        result[i] = a_vals[i] + b_vals[i];
                    }
                    Ok(Vector256 { f64x4: result })
                }
            }
        }
    }

    /// Soustraction vectorielle 256-bit
    fn sub_v256(&self, a: &Vector256, b: &Vector256, data_type: Vector256DataType) -> VMResult<Vector256> {
        match data_type {
            Vector256DataType::I32x8 => Ok(a.sub_i32x8(b)),
            Vector256DataType::F32x8 => {
                unsafe {
                    let a_vals = a.f32x8;
                    let b_vals = b.f32x8;
                    let mut result = [0.0f32; 8];
                    for i in 0..8 {
                        result[i] = a_vals[i] - b_vals[i];
                    }
                    Ok(Vector256::from_f32x8(result))
                }
            }
            Vector256DataType::I16x16 => {
                unsafe {
                    let a_vals = a.i16x16;
                    let b_vals = b.i16x16;
                    let mut result = [0i16; 16];
                    for i in 0..16 {
                        result[i] = a_vals[i].wrapping_sub(b_vals[i]);
                    }
                    Ok(Vector256 { i16x16: result })
                }
            }
            Vector256DataType::I64x4 => {
                unsafe {
                    let a_vals = a.i64x4;
                    let b_vals = b.i64x4;
                    let mut result = [0i64; 4];
                    for i in 0..4 {
                        result[i] = a_vals[i].wrapping_sub(b_vals[i]);
                    }
                    Ok(Vector256 { i64x4: result })
                }
            }
            Vector256DataType::F64x4 => {
                unsafe {
                    let a_vals = a.f64x4;
                    let b_vals = b.f64x4;
                    let mut result = [0.0f64; 4];
                    for i in 0..4 {
                        result[i] = a_vals[i] - b_vals[i];
                    }
                    Ok(Vector256 { f64x4: result })
                }
            }
        }
    }

    /// Multiplication vectorielle 256-bit
    fn mul_v256(&self, a: &Vector256, b: &Vector256, data_type: Vector256DataType) -> VMResult<Vector256> {
        match data_type {
            Vector256DataType::I32x8 => {
                unsafe {
                    let a_vals = a.i32x8;
                    let b_vals = b.i32x8;
                    let mut result = [0i32; 8];
                    for i in 0..8 {
                        result[i] = a_vals[i].wrapping_mul(b_vals[i]);
                    }
                    Ok(Vector256::from_i32x8(result))
                }
            }
            Vector256DataType::F32x8 => {
                unsafe {
                    let a_vals = a.f32x8;
                    let b_vals = b.f32x8;
                    let mut result = [0.0f32; 8];
                    for i in 0..8 {
                        result[i] = a_vals[i] * b_vals[i];
                    }
                    Ok(Vector256::from_f32x8(result))
                }
            }
            Vector256DataType::I16x16 => {
                unsafe {
                    let a_vals = a.i16x16;
                    let b_vals = b.i16x16;
                    let mut result = [0i16; 16];
                    for i in 0..16 {
                        result[i] = a_vals[i].wrapping_mul(b_vals[i]);
                    }
                    Ok(Vector256 { i16x16: result })
                }
            }
            Vector256DataType::I64x4 => {
                unsafe {
                    let a_vals = a.i64x4;
                    let b_vals = b.i64x4;
                    let mut result = [0i64; 4];
                    for i in 0..4 {
                        result[i] = a_vals[i].wrapping_mul(b_vals[i]);
                    }
                    Ok(Vector256 { i64x4: result })
                }
            }
            Vector256DataType::F64x4 => {
                unsafe {
                    let a_vals = a.f64x4;
                    let b_vals = b.f64x4;
                    let mut result = [0.0f64; 4];
                    for i in 0..4 {
                        result[i] = a_vals[i] * b_vals[i];
                    }
                    Ok(Vector256 { f64x4: result })
                }
            }
        }
    }

    /// Division vectorielle 256-bit
    fn div_v256(&self, a: &Vector256, b: &Vector256, data_type: Vector256DataType) -> VMResult<Vector256> {
        match data_type {
            Vector256DataType::I32x8 => {
                unsafe {
                    let a_vals = a.i32x8;
                    let b_vals = b.i32x8;
                    let mut result = [0i32; 8];
                    for i in 0..8 {
                        if b_vals[i] == 0 {
                            return Err(VMError::arithmetic_error("Division by zero"));
                        }
                        result[i] = a_vals[i].wrapping_div(b_vals[i]);
                    }
                    Ok(Vector256::from_i32x8(result))
                }
            }
            Vector256DataType::F32x8 => {
                unsafe {
                    let a_vals = a.f32x8;
                    let b_vals = b.f32x8;
                    let mut result = [0.0f32; 8];
                    for i in 0..8 {
                        result[i] = a_vals[i] / b_vals[i];
                    }
                    Ok(Vector256::from_f32x8(result))
                }
            }
            Vector256DataType::I16x16 => {
                unsafe {
                    let a_vals = a.i16x16;
                    let b_vals = b.i16x16;
                    let mut result = [0i16; 16];
                    for i in 0..16 {
                        if b_vals[i] == 0 {
                            return Err(VMError::arithmetic_error("Division by zero"));
                        }
                        result[i] = a_vals[i].wrapping_div(b_vals[i]);
                    }
                    Ok(Vector256 { i16x16: result })
                }
            }
            Vector256DataType::I64x4 => {
                unsafe {
                    let a_vals = a.i64x4;
                    let b_vals = b.i64x4;
                    let mut result = [0i64; 4];
                    for i in 0..4 {
                        if b_vals[i] == 0 {
                            return Err(VMError::arithmetic_error("Division by zero"));
                        }
                        result[i] = a_vals[i].wrapping_div(b_vals[i]);
                    }
                    Ok(Vector256 { i64x4: result })
                }
            }
            Vector256DataType::F64x4 => {
                unsafe {
                    let a_vals = a.f64x4;
                    let b_vals = b.f64x4;
                    let mut result = [0.0f64; 4];
                    for i in 0..4 {
                        result[i] = a_vals[i] / b_vals[i];
                    }
                    Ok(Vector256 { f64x4: result })
                }
            }
        }
    }

    /// NOT vectoriel 256-bit
    fn not_v256(&self, a: &Vector256) -> Vector256 {
        unsafe {
            let a_vals = a.u64s;
            let mut result = [0u64; 4];
            for i in 0..4 {
                result[i] = !a_vals[i];
            }
            Vector256 { u64s: result }
        }
    }

    /// Minimum vectoriel 256-bit
    fn min_v256(&self, a: &Vector256, b: &Vector256, data_type: Vector256DataType) -> VMResult<Vector256> {
        match data_type {
            Vector256DataType::I32x8 => {
                unsafe {
                    let a_vals = a.i32x8;
                    let b_vals = b.i32x8;
                    let mut result = [0i32; 8];
                    for i in 0..8 {
                        result[i] = a_vals[i].min(b_vals[i]);
                    }
                    Ok(Vector256::from_i32x8(result))
                }
            }
            Vector256DataType::F32x8 => {
                unsafe {
                    let a_vals = a.f32x8;
                    let b_vals = b.f32x8;
                    let mut result = [0.0f32; 8];
                    for i in 0..8 {
                        result[i] = a_vals[i].min(b_vals[i]);
                    }
                    Ok(Vector256::from_f32x8(result))
                }
            }
            Vector256DataType::I16x16 => {
                unsafe {
                    let a_vals = a.i16x16;
                    let b_vals = b.i16x16;
                    let mut result = [0i16; 16];
                    for i in 0..16 {
                        result[i] = a_vals[i].min(b_vals[i]);
                    }
                    Ok(Vector256 { i16x16: result })
                }
            }
            Vector256DataType::I64x4 => {
                unsafe {
                    let a_vals = a.i64x4;
                    let b_vals = b.i64x4;
                    let mut result = [0i64; 4];
                    for i in 0..4 {
                        result[i] = a_vals[i].min(b_vals[i]);
                    }
                    Ok(Vector256 { i64x4: result })
                }
            }
            Vector256DataType::F64x4 => {
                unsafe {
                    let a_vals = a.f64x4;
                    let b_vals = b.f64x4;
                    let mut result = [0.0f64; 4];
                    for i in 0..4 {
                        result[i] = a_vals[i].min(b_vals[i]);
                    }
                    Ok(Vector256 { f64x4: result })
                }
            }
        }
    }

    /// Maximum vectoriel 256-bit
    fn max_v256(&self, a: &Vector256, b: &Vector256, data_type: Vector256DataType) -> VMResult<Vector256> {
        match data_type {
            Vector256DataType::I32x8 => {
                unsafe {
                    let a_vals = a.i32x8;
                    let b_vals = b.i32x8;
                    let mut result = [0i32; 8];
                    for i in 0..8 {
                        result[i] = a_vals[i].max(b_vals[i]);
                    }
                    Ok(Vector256::from_i32x8(result))
                }
            }
            Vector256DataType::F32x8 => {
                unsafe {
                    let a_vals = a.f32x8;
                    let b_vals = b.f32x8;
                    let mut result = [0.0f32; 8];
                    for i in 0..8 {
                        result[i] = a_vals[i].max(b_vals[i]);
                    }
                    Ok(Vector256::from_f32x8(result))
                }
            }
            Vector256DataType::I16x16 => {
                unsafe {
                    let a_vals = a.i16x16;
                    let b_vals = b.i16x16;
                    let mut result = [0i16; 16];
                    for i in 0..16 {
                        result[i] = a_vals[i].max(b_vals[i]);
                    }
                    Ok(Vector256 { i16x16: result })
                }
            }
            Vector256DataType::I64x4 => {
                unsafe {
                    let a_vals = a.i64x4;
                    let b_vals = b.i64x4;
                    let mut result = [0i64; 4];
                    for i in 0..4 {
                        result[i] = a_vals[i].max(b_vals[i]);
                    }
                    Ok(Vector256 { i64x4: result })
                }
            }
            Vector256DataType::F64x4 => {
                unsafe {
                    let a_vals = a.f64x4;
                    let b_vals = b.f64x4;
                    let mut result = [0.0f64; 4];
                    for i in 0..4 {
                        result[i] = a_vals[i].max(b_vals[i]);
                    }
                    Ok(Vector256 { f64x4: result })
                }
            }
        }
    }

    /// Racine carree vectorielle 256-bit
    fn sqrt_v256(&self, a: &Vector256, data_type: Vector256DataType) -> VMResult<Vector256> {
        match data_type {
            Vector256DataType::F32x8 => {
                unsafe {
                    let a_vals = a.f32x8;
                    let mut result = [0.0f32; 8];
                    for i in 0..8 {
                        result[i] = a_vals[i].sqrt();
                    }
                    Ok(Vector256::from_f32x8(result))
                }
            }
            Vector256DataType::F64x4 => {
                unsafe {
                    let a_vals = a.f64x4;
                    let mut result = [0.0f64; 4];
                    for i in 0..4 {
                        result[i] = a_vals[i].sqrt();
                    }
                    Ok(Vector256 { f64x4: result })
                }
            }
            _ => Err(VMError::instruction_error("Invalid SIMD instruction")), // Sqrt uniquement pour flottants
        }
    }

    /// Comparaison vectorielle 256-bit
    fn cmp_v256(&self, a: &Vector256, b: &Vector256, data_type: Vector256DataType) -> VMResult<Vector256> {
        match data_type {
            Vector256DataType::I32x8 => {
                unsafe {
                    let a_vals = a.i32x8;
                    let b_vals = b.i32x8;
                    let mut result = [0i32; 8];
                    for i in 0..8 {
                        result[i] = if a_vals[i] == b_vals[i] { -1i32 } else { 0 };
                    }
                    Ok(Vector256::from_i32x8(result))
                }
            }
            Vector256DataType::F32x8 => {
                unsafe {
                    let a_vals = a.f32x8;
                    let b_vals = b.f32x8;
                    let mut result = [0.0f32; 8];
                    for i in 0..8 {
                        result[i] = if a_vals[i] == b_vals[i] { f32::from_bits(0xFFFFFFFF) } else { 0.0 };
                    }
                    Ok(Vector256::from_f32x8(result))
                }
            }
            Vector256DataType::I16x16 => {
                unsafe {
                    let a_vals = a.i16x16;
                    let b_vals = b.i16x16;
                    let mut result = [0i16; 16];
                    for i in 0..16 {
                        result[i] = if a_vals[i] == b_vals[i] { -1i16 } else { 0 };
                    }
                    Ok(Vector256 { i16x16: result })
                }
            }
            Vector256DataType::I64x4 => {
                unsafe {
                    let a_vals = a.i64x4;
                    let b_vals = b.i64x4;
                    let mut result = [0i64; 4];
                    for i in 0..4 {
                        result[i] = if a_vals[i] == b_vals[i] { -1i64 } else { 0 };
                    }
                    Ok(Vector256 { i64x4: result })
                }
            }
            Vector256DataType::F64x4 => {
                unsafe {
                    let a_vals = a.f64x4;
                    let b_vals = b.f64x4;
                    let mut result = [0.0f64; 4];
                    for i in 0..4 {
                        result[i] = if a_vals[i] == b_vals[i] { f64::from_bits(0xFFFFFFFFFFFFFFFF) } else { 0.0 };
                    }
                    Ok(Vector256 { f64x4: result })
                }
            }
        }
    }

    /// Shuffle vectoriel 256-bit
    fn shuffle_v256(&self, a: &Vector256, mask: &Vector256) -> VMResult<Vector256> {
        unsafe {
            let a_bytes = a.bytes;
            let mask_bytes = mask.bytes;
            let mut result = [0u8; 32];
            
            for i in 0..32 {
                let idx = (mask_bytes[i] & 0x1F) as usize; // Masque sur 5 bits pour 32 elements
                if idx < 32 {
                    result[i] = a_bytes[idx];
                }
            }
            
            Ok(Vector256::from_bytes(result))
        }
    }

    /// Met a jour les flags vectoriels pour un vecteur 128-bit
    fn update_flags_128(&mut self, vec: &Vector128) {
        unsafe {
            let bytes = vec.bytes;
            self.flags.zero = bytes.iter().all(|&b| b == 0);
            self.flags.sign = (bytes[15] & 0x80) != 0; // Bit de signe du dernier byte
        }
    }

    /// Met a jour les flags vectoriels pour un vecteur 256-bit
    fn update_flags_256(&mut self, vec: &Vector256) {
        unsafe {
            let bytes = vec.bytes;
            self.flags.zero = bytes.iter().all(|&b| b == 0);
            self.flags.sign = (bytes[31] & 0x80) != 0; // Bit de signe du dernier byte
        }
    }

    /// ALU vectorielle pour les operations SIMD



    /// Retourne les flags vectoriels actuels
    pub fn get_flags(&self) -> VectorFlags {
        self.flags
    }

    /// Remet a zero les flags vectoriels
    pub fn clear_flags(&mut self) {
        self.flags = VectorFlags::default();
    }

    /// Retourne les statistiques de performance SIMD
    pub fn get_simd_stats(&self) -> &SimdPerformanceStats {
        &self.simd_stats
    }
    
    /// Retourne les statistiques du cache d'opérations
    pub fn get_cache_stats(&self) -> (u64, u64, f64) {
        (
            self.operation_cache.cache_hits,
            self.operation_cache.cache_misses,
            self.operation_cache.cache_hit_rate()
        )
    }
    
    /// Met à jour les statistiques de performance SIMD
    pub fn update_simd_performance(&mut self, cycles: u64) {
        self.simd_stats.total_simd_cycles += cycles;
        let total_ops = self.simd_stats.simd128_ops + self.simd_stats.simd256_ops;
        if total_ops > 0 {
            self.simd_stats.simd_ops_per_cycle = total_ops as f64 / self.simd_stats.total_simd_cycles as f64;
        }
    }
    
    /// Affiche un rapport de performance SIMD détaillé
    pub fn print_simd_performance_report(&self) {
        println!("\n===== RAPPORT DE PERFORMANCE SIMD =====");
        println!("Opérations SIMD 128-bit: {}", self.simd_stats.simd128_ops);
        println!("Opérations SIMD 256-bit: {}", self.simd_stats.simd256_ops);
        println!("Total cycles SIMD: {}", self.simd_stats.total_simd_cycles);
        println!("Opérations/cycle SIMD: {:.2}", self.simd_stats.simd_ops_per_cycle);
        println!("Opérations parallélisées: {}", self.simd_stats.parallel_ops);
        
        let (hits, misses, hit_rate) = self.get_cache_stats();
        println!("\n--- Cache d'Opérations SIMD ---");
        println!("Cache hits: {}", hits);
        println!("Cache misses: {}", misses);
        println!("Taux de réussite du cache: {:.2}%", hit_rate);
        
        if hits + misses > 0 {
            println!("Efficacité du cache: {}%", ((hits as f64 / (hits + misses) as f64) * 100.0) as u32);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_alu_creation() {
        let alu = VectorALU::new();
        assert_eq!(alu.v128_registers.len(), 16);
        assert_eq!(alu.v256_registers.len(), 16);
    }

    #[test]
    fn test_v128_register_access() {
        let mut alu = VectorALU::new();
        let test_vec = Vector128::from_i32x4([1, 2, 3, 4]);
        
        alu.write_v128(5, test_vec).unwrap();
        let result = alu.read_v128(5).unwrap();
        
        unsafe {
            assert_eq!(result.i32x4, [1, 2, 3, 4]);
        }
    }

    #[test]
    fn test_v256_register_access() {
        let mut alu = VectorALU::new();
        let test_vec = Vector256::from_i32x8([1, 2, 3, 4, 5, 6, 7, 8]);
        
        alu.write_v256(10, test_vec).unwrap();
        let result = alu.read_v256(10).unwrap();
        
        unsafe {
            assert_eq!(result.i32x8, [1, 2, 3, 4, 5, 6, 7, 8]);
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v128_addition() {
        let mut alu = VectorALU::new();
        
        let vec1 = Vector128::from_i32x4([1, 2, 3, 4]);
        let vec2 = Vector128::from_i32x4([5, 6, 7, 8]);
        
        alu.write_v128(0, vec1).unwrap();
        alu.write_v128(1, vec2).unwrap();
        
        alu.execute_v128(VectorOperation::Add, 2, 0, Some(1), VectorDataType::I32x4).unwrap();
        
        let result = alu.read_v128(2).unwrap();
        unsafe {
            assert_eq!(result.i32x4, [6, 8, 10, 12]);
        }
    }

    #[test]
    fn test_v128_logical_operations() {
        let mut alu = VectorALU::new();
        
        let vec1 = Vector128::from_u64([0xFF00FF00FF00FF00, 0x00FF00FF00FF00FF]);
        let vec2 = Vector128::from_u64([0xF0F0F0F0F0F0F0F0, 0x0F0F0F0F0F0F0F0F]);
        
        alu.write_v128(0, vec1).unwrap();
        alu.write_v128(1, vec2).unwrap();
        
        alu.execute_v128(VectorOperation::And, 2, 0, Some(1), VectorDataType::I64x2).unwrap();
        
        let result = alu.read_v128(2).unwrap();
        unsafe {
            assert_eq!(result.u64s, [0xF000F000F000F000, 0x000F000F000F000F]);
        }
    }

    #[test]
    fn test_v256_addition() {
        let mut alu = VectorALU::new();
        
        let vec1 = Vector256::from_i32x8([1, 2, 3, 4, 5, 6, 7, 8]);
        let vec2 = Vector256::from_i32x8([10, 20, 30, 40, 50, 60, 70, 80]);
        
        alu.write_v256(0, vec1).unwrap();
        alu.write_v256(1, vec2).unwrap();
        
        alu.execute_v256(VectorOperation::Add, 2, 0, Some(1), Vector256DataType::I32x8).unwrap();
        
        let result = alu.read_v256(2).unwrap();
        unsafe {
            assert_eq!(result.i32x8, [11, 22, 33, 44, 55, 66, 77, 88]);
        }
    }

    #[test]
    fn test_invalid_register() {
        let mut alu = VectorALU::new();
        let test_vec = Vector128::from_i32x4([1, 2, 3, 4]);
        
        assert!(alu.write_v128(16, test_vec).is_err());
        assert!(alu.read_v128(16).is_err());
    }

    #[test]
    fn test_vector_flags() {
        let mut alu = VectorALU::new();
        let zero_vec = Vector128::zero();
        
        alu.write_v128(0, zero_vec).unwrap();
        assert!(alu.get_flags().zero);
    }

    #[test]
    fn test_division_by_zero() {
        let mut alu = VectorALU::new();
        
        let vec1 = Vector128::from_i32x4([1, 2, 3, 4]);
        let vec2 = Vector128::from_i32x4([1, 0, 3, 4]); // Zero en position 1
        
        alu.write_v128(0, vec1).unwrap();
        alu.write_v128(1, vec2).unwrap();
        
        let result = alu.execute_v128(VectorOperation::Div, 2, 0, Some(1), VectorDataType::I32x4);
        assert!(result.is_err());
    }
}
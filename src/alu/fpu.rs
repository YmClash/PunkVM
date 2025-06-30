//src/alu/fpu.rs

use crate::pvm::vm_errors::{VMResult, VMError};

/// Unite de calcul flottant IEEE-754
#[derive(Debug, Clone)]
pub struct FPU {
    /// Registres flottants F0-F31 (32 registres)
    pub fp_registers: [f64; 32],
    /// Status et controle FPU
    pub fpu_status: FPUStatus,
    /// Mode d'arrondi actuel
    pub rounding_mode: RoundingMode,
}

/// Status et flags du FPU
#[derive(Debug, Clone, Copy, Default)]
pub struct FPUStatus {
    /// Flag invalid operation (0x01)
    pub invalid: bool,
    /// Flag division par zero (0x02)
    pub divide_by_zero: bool,
    /// Flag overflow (0x04)
    pub overflow: bool,
    /// Flag underflow (0x08)
    pub underflow: bool,
    /// Flag inexact result (0x10)
    pub inexact: bool,
    /// Flag denormalized operand (0x20)
    pub denormal: bool,
}

/// Modes d'arrondi IEEE-754
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RoundingMode {
    /// Vers le plus proche (par defaut)
    ToNearest,
    /// Vers zero (troncature)
    TowardZero,
    /// Vers +infini
    TowardPositiveInfinity,
    /// Vers -infini
    TowardNegativeInfinity,
}

/// Types de precision flottante
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FloatPrecision {
    /// Simple precision (32-bit)
    Single,
    /// Double precision (64-bit)
    Double,
}

/// Operations FPU
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FPUOperation {
    Add,
    Sub,
    Mul,
    Div,
    Sqrt,
    Min,
    Max,
    Abs,
    Neg,
    Cmp,
    Convert,
    Round,
    Floor,
    Ceil,
    Trunc,
}

/// Resultats de comparaison FPU
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FPUCompareResult {
    Less,
    Equal,
    Greater,
    Unordered, // NaN implique
}

impl Default for RoundingMode {
    fn default() -> Self {
        RoundingMode::ToNearest
    }
}

impl Default for FPU {
    fn default() -> Self {
        Self::new()
    }
}

impl FPU {
    /// Cree une nouvelle unite FPU
    pub fn new() -> Self {
        Self {
            fp_registers: [0.0; 32],
            fpu_status: FPUStatus::default(),
            rounding_mode: RoundingMode::default(),
        }
    }

    /// Reset du FPU
    pub fn reset(&mut self) {
        self.fp_registers = [0.0; 32];
        self.fpu_status = FPUStatus::default();
        self.rounding_mode = RoundingMode::ToNearest;
    }

    /// Lit un registre flottant
    pub fn read_fp_register(&self, reg: u8) -> VMResult<f64> {
        if reg >= 32 {
            return Err(VMError::register_error(&format!("Invalid FP register: {}", reg)));
        }
        Ok(self.fp_registers[reg as usize])
    }

    /// Ecrit un registre flottant
    pub fn write_fp_register(&mut self, reg: u8, value: f64) -> VMResult<()> {
        if reg >= 32 {
            return Err(VMError::register_error(&format!("Invalid FP register: {}", reg)));
        }
        self.fp_registers[reg as usize] = value;
        self.check_value_flags(value);
        Ok(())
    }

    /// Execute une operation FPU
    pub fn execute(
        &mut self,
        op: FPUOperation,
        dst: u8,
        src1: u8,
        src2: Option<u8>,
        precision: FloatPrecision,
    ) -> VMResult<()> {
        let val1 = self.read_fp_register(src1)?;
        
        let result = match op {
            FPUOperation::Add => {
                let val2 = self.read_fp_register(src2.ok_or(VMError::instruction_error("Missing second operand for FPU Add"))?)?;
                self.add(val1, val2, precision)?
            }
            FPUOperation::Sub => {
                let val2 = self.read_fp_register(src2.ok_or(VMError::instruction_error("Missing second operand for FPU Sub"))?)?;
                self.sub(val1, val2, precision)?
            }
            FPUOperation::Mul => {
                let val2 = self.read_fp_register(src2.ok_or(VMError::instruction_error("Missing second operand for FPU Mul"))?)?;
                self.mul(val1, val2, precision)?
            }
            FPUOperation::Div => {
                let val2 = self.read_fp_register(src2.ok_or(VMError::instruction_error("Missing second operand for FPU Div"))?)?;
                self.div(val1, val2, precision)?
            }
            FPUOperation::Sqrt => self.sqrt(val1, precision)?,
            FPUOperation::Min => {
                let val2 = self.read_fp_register(src2.ok_or(VMError::instruction_error("Missing second operand for FPU Min"))?)?;
                self.min(val1, val2, precision)?
            }
            FPUOperation::Max => {
                let val2 = self.read_fp_register(src2.ok_or(VMError::instruction_error("Missing second operand for FPU Max"))?)?;
                self.max(val1, val2, precision)?
            }
            FPUOperation::Abs => self.abs(val1, precision)?,
            FPUOperation::Neg => self.neg(val1, precision)?,
            FPUOperation::Round => self.round(val1, precision)?,
            FPUOperation::Floor => self.floor(val1, precision)?,
            FPUOperation::Ceil => self.ceil(val1, precision)?,
            FPUOperation::Trunc => self.trunc(val1, precision)?,
            FPUOperation::Cmp => {
                let val2 = self.read_fp_register(src2.ok_or(VMError::instruction_error("Missing second operand for FPU Cmp"))?)?;
                let cmp_result = self.compare(val1, val2, precision)?;
                // Convertir le resultat en valeur numerique
                match cmp_result {
                    FPUCompareResult::Less => -1.0,
                    FPUCompareResult::Equal => 0.0,
                    FPUCompareResult::Greater => 1.0,
                    FPUCompareResult::Unordered => f64::NAN,
                }
            }
            FPUOperation::Convert => {
                // Conversion entre single et double precision
                match precision {
                    FloatPrecision::Single => self.to_single(val1)? as f64,
                    FloatPrecision::Double => val1, // Deja en double
                }
            }
        };

        self.write_fp_register(dst, result)
    }

    /// Addition flottante
    fn add(&mut self, a: f64, b: f64, precision: FloatPrecision) -> VMResult<f64> {
        self.clear_exception_flags();
        
        match precision {
            FloatPrecision::Single => {
                let a_single = self.to_single(a)?;
                let b_single = self.to_single(b)?;
                let result = a_single + b_single;
                self.check_single_result(result)?;
                Ok(result as f64)
            }
            FloatPrecision::Double => {
                let result = a + b;
                self.check_double_result(result)?;
                Ok(result)
            }
        }
    }

    /// Soustraction flottante
    fn sub(&mut self, a: f64, b: f64, precision: FloatPrecision) -> VMResult<f64> {
        self.clear_exception_flags();
        
        match precision {
            FloatPrecision::Single => {
                let a_single = self.to_single(a)?;
                let b_single = self.to_single(b)?;
                let result = a_single - b_single;
                self.check_single_result(result)?;
                Ok(result as f64)
            }
            FloatPrecision::Double => {
                let result = a - b;
                self.check_double_result(result)?;
                Ok(result)
            }
        }
    }

    /// Multiplication flottante
    fn mul(&mut self, a: f64, b: f64, precision: FloatPrecision) -> VMResult<f64> {
        self.clear_exception_flags();
        
        match precision {
            FloatPrecision::Single => {
                let a_single = self.to_single(a)?;
                let b_single = self.to_single(b)?;
                let result = a_single * b_single;
                self.check_single_result(result)?;
                Ok(result as f64)
            }
            FloatPrecision::Double => {
                let result = a * b;
                self.check_double_result(result)?;
                Ok(result)
            }
        }
    }

    /// Division flottante
    fn div(&mut self, a: f64, b: f64, precision: FloatPrecision) -> VMResult<f64> {
        self.clear_exception_flags();
        
        // Verification division par zero
        if b == 0.0 {
            self.fpu_status.divide_by_zero = true;
            return Ok(if a > 0.0 { f64::INFINITY } else if a < 0.0 { f64::NEG_INFINITY } else { f64::NAN });
        }
        
        match precision {
            FloatPrecision::Single => {
                let a_single = self.to_single(a)?;
                let b_single = self.to_single(b)?;
                let result = a_single / b_single;
                self.check_single_result(result)?;
                Ok(result as f64)
            }
            FloatPrecision::Double => {
                let result = a / b;
                self.check_double_result(result)?;
                Ok(result)
            }
        }
    }

    /// Racine carree flottante
    fn sqrt(&mut self, a: f64, precision: FloatPrecision) -> VMResult<f64> {
        self.clear_exception_flags();
        
        if a < 0.0 {
            self.fpu_status.invalid = true;
            return Ok(f64::NAN);
        }
        
        match precision {
            FloatPrecision::Single => {
                let a_single = self.to_single(a)?;
                let result = a_single.sqrt();
                self.check_single_result(result)?;
                Ok(result as f64)
            }
            FloatPrecision::Double => {
                let result = a.sqrt();
                self.check_double_result(result)?;
                Ok(result)
            }
        }
    }

    /// Minimum flottant
    fn min(&mut self, a: f64, b: f64, precision: FloatPrecision) -> VMResult<f64> {
        self.clear_exception_flags();
        
        if a.is_nan() || b.is_nan() {
            self.fpu_status.invalid = true;
            return Ok(f64::NAN);
        }
        
        match precision {
            FloatPrecision::Single => {
                let a_single = self.to_single(a)?;
                let b_single = self.to_single(b)?;
                Ok(a_single.min(b_single) as f64)
            }
            FloatPrecision::Double => Ok(a.min(b))
        }
    }

    /// Maximum flottant
    fn max(&mut self, a: f64, b: f64, precision: FloatPrecision) -> VMResult<f64> {
        self.clear_exception_flags();
        
        if a.is_nan() || b.is_nan() {
            self.fpu_status.invalid = true;
            return Ok(f64::NAN);
        }
        
        match precision {
            FloatPrecision::Single => {
                let a_single = self.to_single(a)?;
                let b_single = self.to_single(b)?;
                Ok(a_single.max(b_single) as f64)
            }
            FloatPrecision::Double => Ok(a.max(b))
        }
    }

    /// Valeur absolue flottante
    fn abs(&mut self, a: f64, precision: FloatPrecision) -> VMResult<f64> {
        match precision {
            FloatPrecision::Single => {
                let a_single = self.to_single(a)?;
                Ok(a_single.abs() as f64)
            }
            FloatPrecision::Double => Ok(a.abs())
        }
    }

    /// Negation flottante
    fn neg(&mut self, a: f64, precision: FloatPrecision) -> VMResult<f64> {
        match precision {
            FloatPrecision::Single => {
                let a_single = self.to_single(a)?;
                Ok((-a_single) as f64)
            }
            FloatPrecision::Double => Ok(-a)
        }
    }

    /// Arrondi selon le mode actuel
    fn round(&mut self, a: f64, precision: FloatPrecision) -> VMResult<f64> {
        self.clear_exception_flags();
        
        let result = match self.rounding_mode {
            RoundingMode::ToNearest => a.round(),
            RoundingMode::TowardZero => a.trunc(),
            RoundingMode::TowardPositiveInfinity => a.ceil(),
            RoundingMode::TowardNegativeInfinity => a.floor(),
        };
        
        match precision {
            FloatPrecision::Single => {
                let result_single = self.to_single(result)?;
                Ok(result_single as f64)
            }
            FloatPrecision::Double => Ok(result)
        }
    }

    /// Arrondi vers le bas (floor)
    fn floor(&mut self, a: f64, precision: FloatPrecision) -> VMResult<f64> {
        match precision {
            FloatPrecision::Single => {
                let a_single = self.to_single(a)?;
                Ok(a_single.floor() as f64)
            }
            FloatPrecision::Double => Ok(a.floor())
        }
    }

    /// Arrondi vers le haut (ceil)
    fn ceil(&mut self, a: f64, precision: FloatPrecision) -> VMResult<f64> {
        match precision {
            FloatPrecision::Single => {
                let a_single = self.to_single(a)?;
                Ok(a_single.ceil() as f64)
            }
            FloatPrecision::Double => Ok(a.ceil())
        }
    }

    /// Troncature (vers zero)
    fn trunc(&mut self, a: f64, precision: FloatPrecision) -> VMResult<f64> {
        match precision {
            FloatPrecision::Single => {
                let a_single = self.to_single(a)?;
                Ok(a_single.trunc() as f64)
            }
            FloatPrecision::Double => Ok(a.trunc())
        }
    }

    /// Comparaison flottante
    fn compare(&mut self, a: f64, b: f64, precision: FloatPrecision) -> VMResult<FPUCompareResult> {
        self.clear_exception_flags();
        
        let (val_a, val_b) = match precision {
            FloatPrecision::Single => {
                let a_single = self.to_single(a)?;
                let b_single = self.to_single(b)?;
                (a_single as f64, b_single as f64)
            }
            FloatPrecision::Double => (a, b)
        };
        
        if val_a.is_nan() || val_b.is_nan() {
            self.fpu_status.invalid = true;
            return Ok(FPUCompareResult::Unordered);
        }
        
        if val_a < val_b {
            Ok(FPUCompareResult::Less)
        } else if val_a > val_b {
            Ok(FPUCompareResult::Greater)
        } else {
            Ok(FPUCompareResult::Equal)
        }
    }

    /// Conversion vers simple precision
    fn to_single(&mut self, val: f64) -> VMResult<f32> {
        let result = val as f32;
        
        // Verification overflow/underflow lors de la conversion
        if val.is_finite() && result.is_infinite() {
            self.fpu_status.overflow = true;
        } else if val != 0.0 && result == 0.0 {
            self.fpu_status.underflow = true;
        } else if (val as f32) as f64 != val {
            self.fpu_status.inexact = true;
        }
        
        Ok(result)
    }

    /// Verification des flags pour une valeur single precision
    fn check_single_result(&mut self, val: f32) -> VMResult<()> {
        if val.is_nan() {
            self.fpu_status.invalid = true;
        } else if val.is_infinite() {
            self.fpu_status.overflow = true;
        } else if val == 0.0 {
            // Verifier si c'est un vrai zero ou un underflow
            if val.to_bits() != 0 {
                self.fpu_status.underflow = true;
            }
        }
        Ok(())
    }

    /// Verification des flags pour une valeur double precision
    fn check_double_result(&mut self, val: f64) -> VMResult<()> {
        if val.is_nan() {
            self.fpu_status.invalid = true;
        } else if val.is_infinite() {
            self.fpu_status.overflow = true;
        } else if val == 0.0 {
            // Verifier si c'est un vrai zero ou un underflow
            if val.to_bits() != 0 {
                self.fpu_status.underflow = true;
            }
        }
        Ok(())
    }

    /// Verification des flags pour une valeur quelconque
    fn check_value_flags(&mut self, val: f64) {
        if val.is_nan() {
            self.fpu_status.invalid = true;
        } else if val.is_infinite() {
            self.fpu_status.overflow = true;
        } else if val != 0.0 && val.abs() < f64::MIN_POSITIVE {
            self.fpu_status.denormal = true;
        }
    }

    /// Efface les flags d'exception
    fn clear_exception_flags(&mut self) {
        self.fpu_status.invalid = false;
        self.fpu_status.divide_by_zero = false;
        self.fpu_status.overflow = false;
        self.fpu_status.underflow = false;
        self.fpu_status.inexact = false;
        self.fpu_status.denormal = false;
    }

    /// Definit le mode d'arrondi
    pub fn set_rounding_mode(&mut self, mode: RoundingMode) {
        self.rounding_mode = mode;
    }

    /// Retourne le mode d'arrondi actuel
    pub fn get_rounding_mode(&self) -> RoundingMode {
        self.rounding_mode
    }

    /// Retourne le status FPU
    pub fn get_status(&self) -> FPUStatus {
        self.fpu_status
    }

    /// Efface tout le status FPU
    pub fn clear_status(&mut self) {
        self.fpu_status = FPUStatus::default();
    }

    /// Retourne le status sous forme de mot de status IEEE-754
    pub fn get_status_word(&self) -> u32 {
        let mut status = 0u32;
        if self.fpu_status.invalid { status |= 0x01; }
        if self.fpu_status.divide_by_zero { status |= 0x02; }
        if self.fpu_status.overflow { status |= 0x04; }
        if self.fpu_status.underflow { status |= 0x08; }
        if self.fpu_status.inexact { status |= 0x10; }
        if self.fpu_status.denormal { status |= 0x20; }
        status
    }

    /// Definit le status a partir d'un mot de status
    pub fn set_status_word(&mut self, status: u32) {
        self.fpu_status.invalid = (status & 0x01) != 0;
        self.fpu_status.divide_by_zero = (status & 0x02) != 0;
        self.fpu_status.overflow = (status & 0x04) != 0;
        self.fpu_status.underflow = (status & 0x08) != 0;
        self.fpu_status.inexact = (status & 0x10) != 0;
        self.fpu_status.denormal = (status & 0x20) != 0;
    }

    /// Convertit un f64 en f32 avec gestion des exceptions
    pub fn f64_to_f32(&mut self, val: f64) -> f32 {
        let result = val as f32;
        if val.is_finite() && result.is_infinite() {
            self.fpu_status.overflow = true;
        } else if val != 0.0 && result == 0.0 {
            self.fpu_status.underflow = true;
        } else if (result as f64) != val && val.is_finite() {
            self.fpu_status.inexact = true;
        }
        result
    }

    /// Convertit un entier en flottant
    pub fn int_to_float(&mut self, val: i64, precision: FloatPrecision) -> f64 {
        match precision {
            FloatPrecision::Single => {
                let result = val as f32;
                if (result as i64) != val {
                    self.fpu_status.inexact = true;
                }
                result as f64
            }
            FloatPrecision::Double => {
                let result = val as f64;
                if (result as i64) != val {
                    self.fpu_status.inexact = true;
                }
                result
            }
        }
    }

    /// Convertit un flottant en entier avec gestion des exceptions
    pub fn float_to_int(&mut self, val: f64, precision: FloatPrecision) -> VMResult<i64> {
        let source_val = match precision {
            FloatPrecision::Single => self.to_single(val)? as f64,
            FloatPrecision::Double => val,
        };
        
        if source_val.is_nan() {
            self.fpu_status.invalid = true;
            return Ok(0); // Valeur conventionnelle pour NaN
        }
        
        if source_val >= (i64::MAX as f64) || source_val <= (i64::MIN as f64) {
            self.fpu_status.invalid = true;
            return Ok(if source_val > 0.0 { i64::MAX } else { i64::MIN });
        }
        
        let result = source_val as i64;
        if (result as f64) != source_val {
            self.fpu_status.inexact = true;
        }
        
        Ok(result)
    }

    /// Test si une valeur est denormalisee
    pub fn is_denormal(&self, val: f64, precision: FloatPrecision) -> bool {
        match precision {
            FloatPrecision::Single => {
                let bits = (val as f32).to_bits();
                let exp = (bits >> 23) & 0xFF;
                let mantissa = bits & 0x7FFFFF;
                exp == 0 && mantissa != 0
            }
            FloatPrecision::Double => {
                let bits = val.to_bits();
                let exp = (bits >> 52) & 0x7FF;
                let mantissa = bits & 0xFFFFFFFFFFFFF;
                exp == 0 && mantissa != 0
            }
        }
    }
}

impl FPUStatus {
    /// Retourne true si des exceptions sont actives
    pub fn has_exceptions(&self) -> bool {
        self.invalid || self.divide_by_zero || self.overflow || self.underflow || self.inexact
    }

    /// Retourne true si des exceptions critiques sont actives
    pub fn has_critical_exceptions(&self) -> bool {
        self.invalid || self.divide_by_zero || self.overflow
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fpu_creation() {
        let fpu = FPU::new();
        assert_eq!(fpu.fp_registers.len(), 32);
        assert_eq!(fpu.rounding_mode, RoundingMode::ToNearest);
    }

    #[test]
    fn test_register_access() {
        let mut fpu = FPU::new();
        
        fpu.write_fp_register(5, 3.14159).unwrap();
        let result = fpu.read_fp_register(5).unwrap();
        assert_eq!(result, 3.14159);
        
        // Test registre invalide
        assert!(fpu.write_fp_register(32, 1.0).is_err());
        assert!(fpu.read_fp_register(32).is_err());
    }

    #[test]
    fn test_basic_arithmetic() {
        let mut fpu = FPU::new();
        
        fpu.write_fp_register(0, 2.5).unwrap();
        fpu.write_fp_register(1, 1.5).unwrap();
        
        // Addition
        fpu.execute(FPUOperation::Add, 2, 0, Some(1), FloatPrecision::Double).unwrap();
        let result = fpu.read_fp_register(2).unwrap();
        assert_eq!(result, 4.0);
        
        // Soustraction
        fpu.execute(FPUOperation::Sub, 3, 0, Some(1), FloatPrecision::Double).unwrap();
        let result = fpu.read_fp_register(3).unwrap();
        assert_eq!(result, 1.0);
        
        // Multiplication
        fpu.execute(FPUOperation::Mul, 4, 0, Some(1), FloatPrecision::Double).unwrap();
        let result = fpu.read_fp_register(4).unwrap();
        assert_eq!(result, 3.75);
    }

    #[test]
    fn test_division() {
        let mut fpu = FPU::new();
        
        fpu.write_fp_register(0, 6.0).unwrap();
        fpu.write_fp_register(1, 2.0).unwrap();
        fpu.write_fp_register(2, 0.0).unwrap();
        
        // Division normale
        fpu.execute(FPUOperation::Div, 3, 0, Some(1), FloatPrecision::Double).unwrap();
        let result = fpu.read_fp_register(3).unwrap();
        assert_eq!(result, 3.0);
        
        // Division par zero
        fpu.execute(FPUOperation::Div, 4, 0, Some(2), FloatPrecision::Double).unwrap();
        let result = fpu.read_fp_register(4).unwrap();
        assert!(result.is_infinite());
        assert!(fpu.get_status().divide_by_zero);
    }

    #[test]
    fn test_sqrt() {
        let mut fpu = FPU::new();
        
        fpu.write_fp_register(0, 9.0).unwrap();
        fpu.write_fp_register(1, -1.0).unwrap();
        
        // Racine carree normale
        fpu.execute(FPUOperation::Sqrt, 2, 0, None, FloatPrecision::Double).unwrap();
        let result = fpu.read_fp_register(2).unwrap();
        assert_eq!(result, 3.0);
        
        // Racine carree d'un nombre negatif
        fpu.execute(FPUOperation::Sqrt, 3, 1, None, FloatPrecision::Double).unwrap();
        let result = fpu.read_fp_register(3).unwrap();
        assert!(result.is_nan());
        assert!(fpu.get_status().invalid);
    }

    #[test]
    fn test_comparison() {
        let mut fpu = FPU::new();
        
        fpu.write_fp_register(0, 2.5).unwrap();
        fpu.write_fp_register(1, 1.5).unwrap();
        fpu.write_fp_register(2, 2.5).unwrap();
        
        // Comparaison plus grand
        fpu.execute(FPUOperation::Cmp, 3, 0, Some(1), FloatPrecision::Double).unwrap();
        let result = fpu.read_fp_register(3).unwrap();
        assert_eq!(result, 1.0); // Greater
        
        // Comparaison egal
        fpu.execute(FPUOperation::Cmp, 4, 0, Some(2), FloatPrecision::Double).unwrap();
        let result = fpu.read_fp_register(4).unwrap();
        assert_eq!(result, 0.0); // Equal
    }

    #[test]
    fn test_rounding_modes() {
        let mut fpu = FPU::new();
        
        fpu.write_fp_register(0, 2.7).unwrap();
        
        // Mode par defaut (ToNearest)
        fpu.execute(FPUOperation::Round, 1, 0, None, FloatPrecision::Double).unwrap();
        assert_eq!(fpu.read_fp_register(1).unwrap(), 3.0);
        
        // Mode vers zero
        fpu.set_rounding_mode(RoundingMode::TowardZero);
        fpu.execute(FPUOperation::Round, 2, 0, None, FloatPrecision::Double).unwrap();
        assert_eq!(fpu.read_fp_register(2).unwrap(), 2.0);
        
        // Mode vers +infini
        fpu.set_rounding_mode(RoundingMode::TowardPositiveInfinity);
        fpu.execute(FPUOperation::Round, 3, 0, None, FloatPrecision::Double).unwrap();
        assert_eq!(fpu.read_fp_register(3).unwrap(), 3.0);
    }

    #[test]
    fn test_single_precision() {
        let mut fpu = FPU::new();
        
        fpu.write_fp_register(0, 1.5).unwrap();
        fpu.write_fp_register(1, 2.5).unwrap();
        
        // Operation en simple precision
        fpu.execute(FPUOperation::Add, 2, 0, Some(1), FloatPrecision::Single).unwrap();
        let result = fpu.read_fp_register(2).unwrap();
        assert_eq!(result, 4.0);
    }

    #[test]
    fn test_conversion() {
        let mut fpu = FPU::new();
        
        // Test conversion double vers single
        fpu.write_fp_register(0, 3.14159265359).unwrap();
        fpu.execute(FPUOperation::Convert, 1, 0, None, FloatPrecision::Single).unwrap();
        let result = fpu.read_fp_register(1).unwrap();
        // Precision reduite en single
        assert!((result - 3.1415927).abs() < 0.0000001);
    }

    #[test]
    fn test_status_word() {
        let mut fpu = FPU::new();
        
        // Provoquer une division par zero
        fpu.write_fp_register(0, 1.0).unwrap();
        fpu.write_fp_register(1, 0.0).unwrap();
        fpu.execute(FPUOperation::Div, 2, 0, Some(1), FloatPrecision::Double).unwrap();
        
        let status_word = fpu.get_status_word();
        assert_eq!(status_word & 0x02, 0x02); // Bit division par zero
        
        // Test modification du status word
        fpu.set_status_word(0x05); // Invalid + overflow
        let status = fpu.get_status();
        assert!(status.invalid);
        assert!(status.overflow);
        assert!(!status.divide_by_zero);
    }

    #[test]
    fn test_min_max() {
        let mut fpu = FPU::new();
        
        fpu.write_fp_register(0, 2.5).unwrap();
        fpu.write_fp_register(1, 1.5).unwrap();
        
        // Test minimum
        fpu.execute(FPUOperation::Min, 2, 0, Some(1), FloatPrecision::Double).unwrap();
        assert_eq!(fpu.read_fp_register(2).unwrap(), 1.5);
        
        // Test maximum
        fpu.execute(FPUOperation::Max, 3, 0, Some(1), FloatPrecision::Double).unwrap();
        assert_eq!(fpu.read_fp_register(3).unwrap(), 2.5);
    }

    #[test]
    fn test_special_values() {
        let mut fpu = FPU::new();
        
        // Test avec NaN
        fpu.write_fp_register(0, f64::NAN).unwrap();
        fpu.write_fp_register(1, 1.0).unwrap();
        
        fpu.execute(FPUOperation::Add, 2, 0, Some(1), FloatPrecision::Double).unwrap();
        let result = fpu.read_fp_register(2).unwrap();
        assert!(result.is_nan());
        
        // Test avec infinity
        fpu.write_fp_register(3, f64::INFINITY).unwrap();
        fpu.execute(FPUOperation::Add, 4, 3, Some(1), FloatPrecision::Double).unwrap();
        let result = fpu.read_fp_register(4).unwrap();
        assert!(result.is_infinite());
    }
}
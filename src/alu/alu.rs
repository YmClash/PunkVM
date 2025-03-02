//src/alu/alu.rs


/// Structure des flags de l'ALU
#[derive(Debug, Clone, Copy, Default)]
pub struct ALUFlags{
    pub zero: bool,     // Flag zéro Resultat nul
    pub negative: bool,     // Flag négatif Resultat négatif, bit le plus significatif = 1
    pub overflow: bool,     // Flag de débordement signe Overflow, dépassement de capacité
    pub carry: bool,        // Flag de retenue Carry sur les opérations non signées
}

/// Type d'opération de l'ALU
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ALUOperation {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Xor,
    Not,
    Shl,
    Shr,
    Sar,  // Shift Arithmetic Right
    Rol,  // Rotate Left
    Ror,  // Rotate Right
    Inc,
    Dec,
    Neg,
    Cmp,  // Compare (comme Sub mais ne stocke pas le résultat)
    Test, // Test (comme And mais ne stocke pas le résultat)
    Mov,  // Copie la valeur
}

/// Unité ALU (Arithmetic Logic Unit)
pub struct ALU {
    /// Flags de l'ALU
    pub flags: ALUFlags,
}

impl Default for ALU {
    fn default() -> Self {
        Self::new()
    }
}

impl ALU {
    /// Crée une nouvelle instance ALU
    pub fn new() -> Self {
        Self {
            flags: ALUFlags::default(),
        }
    }

    /// Exécute une opération ALU
    /// Compute
    pub fn execute(&mut self, operation: ALUOperation, a: u64, b: u64) -> Result<u64, String> {
        let result = match operation {
            ALUOperation::Add => {
                let (result, carry) = a.overflowing_add(b);
                // Vérifier l'overflow pour les nombres signés
                let overflow = ((a as i64) + (b as i64)) != (result as i64);
                self.flags.carry = carry;
                self.flags.overflow = overflow;
                result
            },

            ALUOperation::Sub => {
                let (result, carry) = a.overflowing_sub(b);
                // Vérifier l'overflow pour les nombres signés
                let overflow = ((a as i64) - (b as i64)) != (result as i64);
                self.flags.carry = carry;
                self.flags.overflow = overflow;
                result
            },

            ALUOperation::Mul => {
                let (result, overflow) = a.overflowing_mul(b);
                self.flags.carry = overflow;
                self.flags.overflow = overflow;
                result
            },

            ALUOperation::Div => {
                if b == 0 {
                    return Err("Division par zéro".to_string());
                }
                let result = a / b;
                self.flags.carry = false;
                self.flags.overflow = false;
                result
            },

            ALUOperation::Mod => {
                if b == 0 {
                    return Err("Modulo par zéro".to_string());
                }
                let result = a % b;
                self.flags.carry = false;
                self.flags.overflow = false;
                result
            },

            ALUOperation::And => {
                let result = a & b;
                self.flags.carry = false;
                self.flags.overflow = false;
                result
            },

            ALUOperation::Or => {
                let result = a | b;
                self.flags.carry = false;
                self.flags.overflow = false;
                result
            },

            ALUOperation::Xor => {
                let result = a ^ b;
                self.flags.carry = false;
                self.flags.overflow = false;
                result
            },

            ALUOperation::Not => {
                let result = !a;
                self.flags.carry = false;
                self.flags.overflow = false;
                result
            },

            ALUOperation::Shl => {
                if b > 63 {
                    // Shifting by more than the width of the type
                    self.flags.carry = a & 1 != 0; // Last bit shifted out
                    self.flags.overflow = false;
                    0
                } else {
                    let shift_amount = b as u32;
                    let (result, carry) = if shift_amount == 0 {
                        (a, false)
                    } else {
                        let carry_bit = ((a >> (64 - shift_amount)) & 1) != 0;
                        (a << shift_amount, carry_bit)
                    };

                    self.flags.carry = carry;
                    self.flags.overflow = false;
                    result
                }
            },

            ALUOperation::Shr => {
                if b > 63 {
                    // Shifting by more than the width of the type
                    self.flags.carry = (a >> 63) & 1 != 0; // First bit shifted out
                    self.flags.overflow = false;
                    0
                } else {
                    let shift_amount = b as u32;
                    let (result, carry) = if shift_amount == 0 {
                        (a, false)
                    } else {
                        let carry_bit = ((a >> (shift_amount - 1)) & 1) != 0;
                        (a >> shift_amount, carry_bit)
                    };

                    self.flags.carry = carry;
                    self.flags.overflow = false;
                    result
                }
            },

            ALUOperation::Sar => {
                if b > 63 {
                    // For signed right shift, filling with sign bit
                    let sign = (a >> 63) & 1;
                    self.flags.carry = (a >> 62) & 1 != 0; // First bit shifted out
                    self.flags.overflow = false;
                    if sign == 1 {
                        !0u64 // All 1s
                    } else {
                        0u64
                    }
                } else {
                    let shift_amount = b as u32;
                    let (result, carry) = if shift_amount == 0 {
                        (a, false)
                    } else {
                        // Traiter comme un nombre signé pour le décalage arithmétique
                        let signed_a = a as i64;
                        let carry_bit = ((a >> (shift_amount - 1)) & 1) != 0;
                        let shifted = (signed_a >> shift_amount) as u64;
                        (shifted, carry_bit)
                    };

                    self.flags.carry = carry;
                    self.flags.overflow = false;
                    result
                }
            },

            ALUOperation::Rol => {
                let shift_amount = (b % 64) as u32;
                if shift_amount == 0 {
                    a
                } else {
                    let result = a.rotate_left(shift_amount);
                    self.flags.carry = (result & 1) != 0;
                    self.flags.overflow = false;
                    result
                }
            },

            ALUOperation::Ror => {
                let shift_amount = (b % 64) as u32;
                if shift_amount == 0 {
                    a
                } else {
                    let result = a.rotate_right(shift_amount);
                    self.flags.carry = ((result >> 63) & 1) != 0;
                    self.flags.overflow = false;
                    result
                }
            },

            ALUOperation::Inc => {
                let (result, carry) = a.overflowing_add(1);
                // Vérifier l'overflow pour les nombres signés
                let overflow = ((a as i64) + 1) != (result as i64);
                self.flags.carry = carry;
                self.flags.overflow = overflow;
                result
            },

            ALUOperation::Dec => {
                let (result, carry) = a.overflowing_sub(1);
                // Vérifier l'overflow pour les nombres signés
                let overflow = ((a as i64) - 1) != (result as i64);
                self.flags.carry = carry;
                self.flags.overflow = overflow;
                result
            },

            ALUOperation::Neg => {
                let (result, carry) = (!a).overflowing_add(1); // Two's complement negation
                // Overflow happens if the input is the minimum negative number
                let overflow = a == (1u64 << 63);
                self.flags.carry = carry;
                self.flags.overflow = overflow;
                result
            },

            ALUOperation::Cmp => {
                // Compare = Sub mais ne stocke pas le résultat
                let (result, carry) = a.overflowing_sub(b);
                // Vérifier l'overflow pour les nombres signés
                let overflow = ((a as i64) - (b as i64)) != (result as i64);
                self.flags.carry = carry;
                self.flags.overflow = overflow;
                result // Retourne le résultat mais il n'est normalement pas utilisé
            },

            ALUOperation::Test => {
                // Test = And mais ne stocke pas le résultat
                let result = a & b;
                self.flags.carry = false;
                self.flags.overflow = false;
                result // Retourne le résultat mais il n'est normalement pas utilisé
            },

            ALUOperation::Mov => {
                // Simplement retourne b (pas d'impact sur les flags)
                b
            },
        };

        // Mettre à jour les flags communs
        self.flags.zero = result == 0;
        self.flags.negative = (result >> 63) & 1 != 0;

        Ok(result)
    }

    /// Vérifie si une condition de branchement est satisfaite
    pub fn check_condition(&self, condition: BranchCondition) -> bool {
        match condition {
            BranchCondition::Always => true,
            BranchCondition::Equal => self.flags.zero,
            BranchCondition::NotEqual => !self.flags.zero,
            BranchCondition::Greater => !self.flags.zero && !self.flags.negative,
            BranchCondition::GreaterEqual => !self.flags.negative,
            BranchCondition::Less => self.flags.negative,
            BranchCondition::LessEqual => self.flags.zero || self.flags.negative,
            BranchCondition::Above => !self.flags.carry && !self.flags.zero,
            BranchCondition::AboveEqual => !self.flags.carry,
            BranchCondition::Below => self.flags.carry,
            BranchCondition::BelowEqual => self.flags.carry || self.flags.zero,
            BranchCondition::Overflow => self.flags.overflow,
            BranchCondition::NotOverflow => !self.flags.overflow,
            BranchCondition::Negative => self.flags.negative,
            BranchCondition::Positive => !self.flags.negative,
        }
    }

    /// Réinitialise les flags de l'ALU
    pub fn reset_flags(&mut self) {
        self.flags = ALUFlags::default();
    }

}

/// Condition de branchement basee sur les flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BranchCondition {
    Always,       // Toujours pris
    Equal,        // ZF = 1
    NotEqual,     // ZF = 0
    Greater,      // ZF = 0 et SF = 0
    GreaterEqual, // SF = 0
    Less,         // SF = 1
    LessEqual,    // ZF = 1 ou SF = 1
    Above,        // CF = 0 et ZF = 0 (non signé)
    AboveEqual,   // CF = 0 (non signé)
    Below,        // CF = 1 (non signé)
    BelowEqual,   // CF = 1 ou ZF = 1 (non signé)
    Overflow,     // OF = 1
    NotOverflow,  // OF = 0
    Negative,     // SF = 1
    Positive,     // SF = 0
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alu_flags_default() {
        let flags = ALUFlags::default();
        assert!(!flags.zero);
        assert!(!flags.negative);
        assert!(!flags.overflow);
        assert!(!flags.carry);
    }

    #[test]
    fn test_basic_arithmetic() {
        let mut alu = ALU::new();

        // Test addition
        let result = alu.execute(ALUOperation::Add, 5, 3).unwrap();
        assert_eq!(result, 8);
        assert!(!alu.flags.zero);
        assert!(!alu.flags.negative);
        assert!(!alu.flags.overflow);
        assert!(!alu.flags.carry);

        // Test soustraction
        let result = alu.execute(ALUOperation::Sub, 5, 3).unwrap();
        assert_eq!(result, 2);
        assert!(!alu.flags.zero);
        assert!(!alu.flags.negative);
        assert!(!alu.flags.overflow);
        assert!(!alu.flags.carry);

        // Test multiplication
        let result = alu.execute(ALUOperation::Mul, 5, 3).unwrap();
        assert_eq!(result, 15);
        assert!(!alu.flags.zero);
        assert!(!alu.flags.negative);
        assert!(!alu.flags.overflow);
        assert!(!alu.flags.carry);

        // Test division
        let result = alu.execute(ALUOperation::Div, 15, 3).unwrap();
        assert_eq!(result, 5);
        assert!(!alu.flags.zero);
        assert!(!alu.flags.negative);
        assert!(!alu.flags.overflow);
        assert!(!alu.flags.carry);
    }

    #[test]
    fn test_zero_flag() {
        let mut alu = ALU::new();

        // Zero flag with subtraction
        let result = alu.execute(ALUOperation::Sub, 5, 5).unwrap();
        assert_eq!(result, 0);
        assert!(alu.flags.zero);

        // Zero flag with AND
        let result = alu.execute(ALUOperation::And, 0x5, 0xA).unwrap();
        assert_eq!(result, 0);
        assert!(alu.flags.zero);
    }

    #[test]
    fn test_negative_flag() {
        let mut alu = ALU::new();

        // Negative flag with subtraction (underflow in unsigned)
        let result = alu.execute(ALUOperation::Sub, 3, 5).unwrap();
        assert_eq!(result, u64::MAX - 1); // Équivalent à -2 en complément à 2
        assert!(!alu.flags.zero);
        assert!(alu.flags.negative);
        assert!(!alu.flags.overflow);
        assert!(alu.flags.carry); // Carry indique un emprunt en soustraction

        // Negative flag avec Neg
        let result = alu.execute(ALUOperation::Neg, 1, 0).unwrap();
        assert_eq!(result, u64::MAX); // Équivalent à -1 en complément à 2
        assert!(!alu.flags.zero);
        assert!(alu.flags.negative);
    }

    #[test]
    fn test_overflow_flag() {
        let mut alu = ALU::new();

        // Overflow en additionnant deux grands nombres positifs
        let result = alu.execute(ALUOperation::Add, 0x7FFFFFFFFFFFFFFF, 1).unwrap();
        assert_eq!(result, 0x8000000000000000);
        assert!(!alu.flags.zero);
        assert!(alu.flags.negative);
        assert!(alu.flags.overflow); // Overflow car signe changé de façon inattendue
        assert!(!alu.flags.carry);
    }

    #[test]
    fn test_carry_flag() {
        let mut alu = ALU::new();

        // Carry en additionnant deux grands nombres
        let result = alu.execute(ALUOperation::Add, u64::MAX, 1).unwrap();
        assert_eq!(result, 0);
        assert!(alu.flags.zero);
        assert!(!alu.flags.negative);
        assert!(!alu.flags.overflow);
        assert!(alu.flags.carry); // Carry car bit 64 affecté
    }

    #[test]
    fn test_logical_operations() {
        let mut alu = ALU::new();

        // Test AND
        let result = alu.execute(ALUOperation::And, 0xF0, 0x0F).unwrap();
        assert_eq!(result, 0);
        assert!(alu.flags.zero);

        // Test OR
        let result = alu.execute(ALUOperation::Or, 0xF0, 0x0F).unwrap();
        assert_eq!(result, 0xFF);
        assert!(!alu.flags.zero);

        // Test XOR
        let result = alu.execute(ALUOperation::Xor, 0xFF, 0x0F).unwrap();
        assert_eq!(result, 0xF0);
        assert!(!alu.flags.zero);

        // Test NOT
        let result = alu.execute(ALUOperation::Not, 0xF0, 0).unwrap();
        assert_eq!(result, !0xF0);
        assert!(!alu.flags.zero);
    }

    #[test]
    fn test_shift_operations() {
        let mut alu = ALU::new();

        // Test SHL
        let result = alu.execute(ALUOperation::Shl, 0x1, 4).unwrap();
        assert_eq!(result, 0x10);

        // Test SHR
        let result = alu.execute(ALUOperation::Shr, 0x10, 4).unwrap();
        assert_eq!(result, 0x1);

        // Test SAR (avec nombre négatif)
        let val = 0x8000000000000010; // Nombre négatif en complément à 2
        let result = alu.execute(ALUOperation::Sar, val, 4).unwrap();
        assert_eq!(result, 0xF800000000000001); // Signe étendu

        // Test ROL
        let result = alu.execute(ALUOperation::Rol, 0x80000000, 1).unwrap();
        assert_eq!(result, 0x1);

        // Test ROR
        let result = alu.execute(ALUOperation::Ror, 0x1, 1).unwrap();
        assert_eq!(result, 0x8000000000000000);
    }

    #[test]
    fn test_division_by_zero() {
        let mut alu = ALU::new();

        // Division par zéro doit retourner une erreur
        let result = alu.execute(ALUOperation::Div, 5, 0);
        assert!(result.is_err());

        // Modulo par zéro doit retourner une erreur
        let result = alu.execute(ALUOperation::Mod, 5, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_compare_operations() {
        let mut alu = ALU::new();

        // Égal
        alu.execute(ALUOperation::Cmp, 5, 5).unwrap();
        assert!(alu.flags.zero);
        assert!(!alu.flags.negative);
        assert!(!alu.flags.carry);

        // Plus petit
        alu.execute(ALUOperation::Cmp, 3, 5).unwrap();
        assert!(!alu.flags.zero);
        assert!(alu.flags.negative);
        assert!(alu.flags.carry);

        // Plus grand
        alu.execute(ALUOperation::Cmp, 7, 5).unwrap();
        assert!(!alu.flags.zero);
        assert!(!alu.flags.negative);
        assert!(!alu.flags.carry);
    }

    #[test]
    fn test_branch_conditions() {
        let mut alu = ALU::new();

        // Equal
        alu.execute(ALUOperation::Cmp, 5, 5).unwrap();
        assert!(alu.check_condition(BranchCondition::Equal));
        assert!(!alu.check_condition(BranchCondition::NotEqual));

        // Greater/Less
        alu.execute(ALUOperation::Cmp, 7, 5).unwrap();
        assert!(alu.check_condition(BranchCondition::Greater));
        assert!(alu.check_condition(BranchCondition::GreaterEqual));
        assert!(!alu.check_condition(BranchCondition::Less));
        assert!(!alu.check_condition(BranchCondition::LessEqual));

        // Less than
        alu.execute(ALUOperation::Cmp, 3, 5).unwrap();
        assert!(!alu.check_condition(BranchCondition::Greater));
        assert!(!alu.check_condition(BranchCondition::GreaterEqual));
        assert!(alu.check_condition(BranchCondition::Less));
        assert!(alu.check_condition(BranchCondition::LessEqual));
    }

    #[test]
    fn test_reset_flags() {
        let mut alu = ALU::new();

        // Set some flags
        alu.execute(ALUOperation::Cmp, 5, 5).unwrap();
        assert!(alu.flags.zero);

        // Reset flags
        alu.reset_flags();
        assert!(!alu.flags.zero);
        assert!(!alu.flags.negative);
        assert!(!alu.flags.overflow);
        assert!(!alu.flags.carry);
    }
}
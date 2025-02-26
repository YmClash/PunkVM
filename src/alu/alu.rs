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
            flags: AlUFlags::default(),
        }
    }

    /// Exécute une opération ALU
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



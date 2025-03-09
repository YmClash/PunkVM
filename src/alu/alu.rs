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
                let a_sign = (a >>63) & 1;
                let b_sign = (b >>63) & 1;
                let result_sign = (result >> 63) & 1;

                // Overflow se produit si a et b ont le même signe mais le résultat a un signe différent
                let overflow = (a_sign == b_sign) && (result_sign != a_sign);

                // let overflow = ((a as i64) + (b as i64)) != (result as i64);

                self.flags.carry = carry;
                self.flags.overflow = overflow;
                result
            },

            ALUOperation::Sub => {
                let (result, carry) = a.overflowing_sub(b);
                // Vérifier l'overflow pour les nombres signés sans conversion
                let a_sign = (a >>63) & 1;
                let b_sign = (b >>63) & 1;
                let result_sign = (result >> 63) & 1;
                // Pour la soustraction, l'overflow se produit quand les signes sont différents
                // et que le signe du résultat ne correspond pas au signe du premier opérande

                let overflow = (a_sign != b_sign) && (a_sign != result_sign);

                // let overflow = ((a as i64) - (b as i64)) != (result as i64);

                self.flags.carry = carry;       // Indique un emprunt
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
                        // !0u64 // All 1s
                        u64::MAX
                    } else {
                        // 0u64
                        0
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
                let a_sign = (a >> 63) & 1;
                let result_sign = (result >> 63) & 1;
                // let overflow = (a_sign == 1) && (result_sign == 1);

                let overflow = (a_sign == 0) && (result_sign == 1) && (a == 0x7FFF_FFFF_FFFF_FFFF);

                // let overflow = ((a as i64) + 1) != (result as i64);

                self.flags.carry = carry;
                self.flags.overflow = overflow;
                result
            },

            ALUOperation::Dec => {
                let (result, carry) = a.overflowing_sub(1);
                // Vérifier l'overflow pour les nombres signés

                ///////////
                let a_sign = (a >> 63) & 1;
                let result_sign = (result >> 63) & 1;
                // let overflow =  (a_sign == 1) && (result_sign == 0);
                //////////

                let overflow = (a_sign == 0) && (result_sign == 1) && (a == 0x8000_0000_0000_0000);

                // let overflow = ((a as i64) - 1) != (result as i64);
                self.flags.carry = carry;
                self.flags.overflow = overflow;
                result
            },

            ALUOperation::Neg => {
                let (result, carry) = (!a).overflowing_add(1); // Two's complement negation
                // Overflow happens if the input is the minimum negative number

                let overflow = a == (1u64 << 63);
                // let overflow = (a == 0x8000_0000_0000_0000);


                self.flags.carry = carry;
                self.flags.overflow = overflow;
                result
            },

            ALUOperation::Cmp => {
                // Compare = Sub mais ne stocke pas le résultat
                let (result, carry) = a.overflowing_sub(b);

                // Extraction des signes et // Calcul de l'overflow
                let a_sign = (a >> 63) & 1;
                let b_sign = (b >> 63) & 1;
                let result_sign = (result >> 63) & 1;
                let overflow = (a_sign != b_sign) && (a_sign != result_sign);

                // Pour la comparaison signée:
                // 1. Si les signes sont différents:
                //    - Si a est négatif et b positif, alors a < b (negative flag = true)
                //    - Si a est positif et b négatif, alors a > b (negative flag = false)
                // 2. Si les signes sont identiques:
                //    - Le negative flag est déterminé par le carry flag (en non-signé)
                // let is_less_than;
                // if a_sign != b_sign {
                //     is_less_than = a_sign == 1;  // Si a est négatif et b positif, alors a < b
                // } else {
                //     is_less_than = carry;  // Sinon, utiliser la logique non signée
                // }

                // signe le negative si resultat < 0
                let negative =result_sign == 1;

                self.flags.carry = carry;
                self.flags.overflow = overflow;
                self.flags.zero = a == b;
                self.flags.negative = negative;


                result

            },

            ALUOperation::Test => {
                // test = And(a, b)  => flags => Zero, etc
                let result = a & b;
                self.flags.carry = false;
                self.flags.overflow = false;
                result // Retourne le résultat mais il n'est normalement pas utilisé
            },

            ALUOperation::Mov => {
                // Simplement retourne b (pas d'impact sur les flags)
                self.flags.carry = false;
                self.flags.overflow = false;
                b
            },
        };


        // Mettre à jour zero/negative sur le résultat (sauf si c'est un "Cmp" qui l'a déjà fait)
        if !matches!(operation, ALUOperation::Cmp) {
            self.flags.zero = result == 0;
            self.flags.negative = ((result >> 63) & 1) != 0;
        }

        // // Mettre à jour les flags communs
        // self.flags.zero = result == 0;
        // self.flags.negative = (result >> 63) & 1 != 0;

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
    use crate::bytecode::instructions::Instruction;
    use crate::bytecode::opcodes::Opcode;

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

        // Test modulo
        let result = alu.execute(ALUOperation::Mod, 17, 5).unwrap();
        assert_eq!(result, 2);
        assert!(!alu.flags.zero);
        assert!(!alu.flags.negative);
        assert!(!alu.flags.overflow);
        assert!(!alu.flags.carry);
    }

    #[test]
    fn test_sub_bigger() {
        let mut alu = ALU::new();
        // 2 - 3 => -1 => en u64 => 0xFFFF...FFFF
        let res = alu.execute(ALUOperation::Sub, 2, 3).unwrap();
        assert_eq!(res, u64::MAX);
        assert!(alu.flags.negative);
        assert!(!alu.flags.zero);
        assert!(!alu.flags.overflow);
        assert!(alu.flags.carry); // Indique un emprunt
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

        // Zero flag with XOR of the same value
        let result = alu.execute(ALUOperation::Xor, 0xAA, 0xAA).unwrap();
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

        // Negative flag with direct assignment of a negative value (MSB set)
        let result = alu.execute(ALUOperation::Mov, 0, 0x8000000000000000).unwrap();
        assert_eq!(result, 0x8000000000000000);
        assert!(!alu.flags.zero);
        assert!(alu.flags.negative);
    }

    #[test]
    fn test_overflow_flag() {
        let mut alu = ALU::new();

        // Provoquer un overflow avec des valeurs u64 qui activeront le bit de signe
        // 0x7FFFFFFFFFFFFFFF (max i64) + 1 = 0x8000000000000000 (qui est négatif en complément à 2)
        let result = alu.execute(ALUOperation::Add, 0x7FFFFFFFFFFFFFFF, 1).unwrap();

        assert_eq!(result, 0x8000000000000000);
        assert!(!alu.flags.zero);
        assert!(alu.flags.negative); // Le bit 63 est maintenant activé
        assert!(alu.flags.overflow); // Il y a un overflow car le signe change de manière inattendue
        assert!(!alu.flags.carry);   // Pas de carry car nous sommes bien en-dessous de u64::MAX

        // Test d'overflow en soustraction
        // 0x8000000000000000 - 1 = 0x7FFFFFFFFFFFFFFF (change de négatif à positif)
        alu.reset_flags();
        let result = alu.execute(ALUOperation::Sub, 0x8000000000000000, 1).unwrap();
        assert_eq!(result, 0x7FFFFFFFFFFFFFFF);
        assert!(!alu.flags.zero);
        assert!(!alu.flags.negative); // Le résultat est positif
        assert!(alu.flags.overflow);  // Overflow car le signe change
        assert!(!alu.flags.carry);    // Pas de carry
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

        // Carry en soustraction (borrow)
        alu.reset_flags();
        let result = alu.execute(ALUOperation::Sub, 0, 1).unwrap();
        assert_eq!(result, u64::MAX);
        assert!(!alu.flags.zero);
        assert!(alu.flags.negative);
        assert!(!alu.flags.overflow);
        assert!(alu.flags.carry); // Carry indique un emprunt
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
        // Utilisons une valeur qui convient mieux à un test en u64
        let val = 0x8000000000000010u64; // Nombre négatif en complément à 2
        let result = alu.execute(ALUOperation::Sar, val, 4).unwrap();
        // Pour un décalage arithmétique, les bits de signe sont préservés
        assert_eq!(result, 0xF800000000000001u64);

        // Test ROL
        // Utilisons une valeur plus petite pour éviter les problèmes
        let result = alu.execute(ALUOperation::Rol, 0x8000000000000000u64, 1).unwrap();
        assert_eq!(result, 0x1);

        // Test ROR
        let result = alu.execute(ALUOperation::Ror, 0x1, 1).unwrap();
        assert_eq!(result, 0x8000000000000000u64);
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

        let r = alu.execute(ALUOperation::Div, 10, 0);
        assert!(r.is_err());
    }

    #[test]
    fn test_inc_dec() {
        let mut alu = ALU::new();
        let res = alu.execute(ALUOperation::Inc, 41, 0).unwrap();
        assert_eq!(res, 42);

        let res = alu.execute(ALUOperation::Dec, 42, 0).unwrap();
        assert_eq!(res, 41);
    }

    #[test]
    fn test_cmp() {
        let mut alu = ALU::new();
        // 5 vs 5
        alu.execute(ALUOperation::Cmp, 5, 5).unwrap();
        assert!(alu.flags.zero);
        assert!(!alu.flags.negative);

        // 3 vs 5 => negative => carry
        alu.execute(ALUOperation::Cmp, 3, 5).unwrap();
        assert!(!alu.flags.zero);
        assert!(alu.flags.negative);
        assert!(alu.flags.carry);

        // 7 vs 5 => bigger => not negative
        alu.execute(ALUOperation::Cmp, 7, 5).unwrap();
        assert!(!alu.flags.zero);
        assert!(!alu.flags.negative);
        assert!(!alu.flags.carry);
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

        // // Test avec valeurs signées (utilisation de la MSB)
        // alu.execute(ALUOperation::Cmp, 0x6000000000000000, 0x0000000000000001).unwrap();
        // assert!(!alu.flags.zero);
        // assert!(alu.flags.negative);
        // assert!(alu.flags.carry);
    }

    #[test]
    fn test_compare_operations_1() {
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

        // Test des conditions spécifiques au carry
        alu.execute(ALUOperation::Add, u64::MAX, 1).unwrap();
        // assert!(alu.check_condition(BranchCondition::Carry));
        assert!(alu.flags.carry); // Vérifie directement le flag au lieu d'utiliser une condition inexistante
        assert!(alu.check_condition(BranchCondition::BelowEqual));
        assert!(!alu.check_condition(BranchCondition::AboveEqual));

        // Test des conditions spécifiques à l'overflow
        alu.execute(ALUOperation::Add, 0x7FFFFFFFFFFFFFFF, 1).unwrap();
        assert!(alu.check_condition(BranchCondition::Overflow));
        assert!(!alu.check_condition(BranchCondition::NotOverflow));
    }

    #[test]
    fn test_increment_decrement() {
        let mut alu = ALU::new();

        // Test INC
        let result = alu.execute(ALUOperation::Inc, 41, 0).unwrap();
        assert_eq!(result, 42);
        assert!(!alu.flags.zero);
        assert!(!alu.flags.negative);
        assert!(!alu.flags.overflow);
        assert!(!alu.flags.carry);

        // Test DEC
        let result = alu.execute(ALUOperation::Dec, 43, 0).unwrap();
        assert_eq!(result, 42);
        assert!(!alu.flags.zero);
        assert!(!alu.flags.negative);
        assert!(!alu.flags.overflow);
        assert!(!alu.flags.carry);

        // Test INC to overflow
        let result = alu.execute(ALUOperation::Inc, u64::MAX, 0).unwrap();
        assert_eq!(result, 0);
        assert!(alu.flags.zero);
        assert!(!alu.flags.negative);
        assert!(!alu.flags.overflow);
        assert!(alu.flags.carry);
    }

    #[test]
    fn test_alu_with_three_register_instructions() {
        let mut alu = ALU::new();

        // Simuler l'exécution d'une instruction ADD avec 3 registres
        // ADD R2, R0, R1 (R2 = R0 + R1)
        let r0 = 10; // Valeur dans R0
        let r1 = 5;  // Valeur dans R1

        // Simuler l'étape d'exécution avec l'ALU
        let result = alu.execute(ALUOperation::Add, r0, r1).unwrap();
        assert_eq!(result, 15); // R2 devrait contenir 15

        // Simuler l'exécution d'une instruction SUB avec 3 registres
        // SUB R3, R0, R1 (R3 = R0 - R1)
        let result = alu.execute(ALUOperation::Sub, r0, r1).unwrap();
        assert_eq!(result, 5); // R3 devrait contenir 5

        // Simuler l'exécution d'une instruction MUL avec 3 registres
        // MUL R4, R0, R1 (R4 = R0 * R1)
        let result = alu.execute(ALUOperation::Mul, r0, r1).unwrap();
        assert_eq!(result, 50); // R4 devrait contenir 50
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

    #[test]
    fn test_complex_instruction_sequence() {
        // Ce test simule l'exécution d'une séquence d'instructions
        // comme elles seraient exécutées dans votre VM
        let mut alu = ALU::new();
        let mut registers = vec![0u64; 16];

        // R0 = 10, R1 = 5
        registers[0] = 10;
        registers[1] = 5;

        // ADD R2, R0, R1 (R2 = R0 + R1)
        let result = alu.execute(ALUOperation::Add, registers[0], registers[1]).unwrap();
        registers[2] = result;
        assert_eq!(registers[2], 15);

        // SUB R3, R0, R1 (R3 = R0 - R1)
        let result = alu.execute(ALUOperation::Sub, registers[0], registers[1]).unwrap();
        registers[3] = result;
        assert_eq!(registers[3], 5);

        // MUL R4, R0, R1 (R4 = R0 * R1)
        let result = alu.execute(ALUOperation::Mul, registers[0], registers[1]).unwrap();
        registers[4] = result;
        assert_eq!(registers[4], 50);

        // CMP R2, R4 (Compare R2 with R4)
        alu.execute(ALUOperation::Cmp, registers[2], registers[4]).unwrap();
        assert!(!alu.flags.zero);      // Not equal
        assert!(alu.flags.negative);   // R2 < R4
        assert!(alu.flags.carry);      // Borrow happened

        // JMP_IF_LESS label (should take the branch as R2 < R4)
        assert!(alu.check_condition(BranchCondition::Less));
    }
}
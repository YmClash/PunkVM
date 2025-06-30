//src/pvm/branch_perceptor.rs


pub const HISTORY_LENGTH: usize = 16; // Longueur de l'historique global (N derniers branchements)

pub const WEIGHT_BITS: isize = 8; // Nombre de bits pour les poids (pour la quantification/clipping)
pub const THRESHOLD_FACTOR: isize = 10; // Facteur pour le seuil d'activation


// Paramètres configurables
pub const GLOBAL_HISTORY_LENGTH: usize = 16;
pub const LOCAL_HISTORY_LENGTH: usize = 8;
pub const TOTAL_HISTORY_LENGTH: usize = GLOBAL_HISTORY_LENGTH + LOCAL_HISTORY_LENGTH;
pub const NUM_PERCEPTRONS: usize = 1024; // Une table plus grande pour réduire l'aliasing



/// Perceptor Struct
#[derive(Debug, Clone)]
pub struct Perceptron {
    pub weight: Vec<isize>,
}

impl Perceptron{
    pub fn new(total_history_length:usize) -> Self {
        Perceptron {
            weight: vec![0; total_history_length +1], // +1 pourle Bias
        }
    }

    pub fn predict_sum(&self, input_history: &[isize]) -> isize {
        let mut sum = self.weight[0];   // Bias
        // Pour simuler la rapidité matérielle sans multiplications,
        // on ajoute ou soustrait les poids directement.
        // Chaque `history_bit` est 1 ou -1.
         for i in 0..input_history.len(){
             if input_history[i] == 1 {
                 sum+= self.weight[i + 1]; // +1 pour le Bias
             }else { sum -= self.weight[i + 1]; } // -1 pour le Bias
         }
        sum
    }


    pub fn train(&mut self, input_history: &[isize], actual_outcome: isize, predicted_sum: isize) {
        // Condition d'entraînement : si la prédiction était fausse, OU si la confiance était faible
        let prediction_correct = (actual_outcome > 0 && predicted_sum > 0) || (actual_outcome <= 0 && predicted_sum <= 0);

        // Seuil T (par exemple, 1.93 * total_history_length + 14, ou une constante simple)
        const THRESHOLD_T: isize = 20; // Exemple de seuil

        if !prediction_correct || predicted_sum.abs() <= THRESHOLD_T {
            // Mise à jour du biais
            self.weight[0] += actual_outcome;
            // Clamper les poids pour simuler les poids sur un nombre fixe de bits (e.g. 8 bits)
            self.weight[0] = self.weight[0].clamp(-(1 << (WEIGHT_BITS - 1)), (1 << (WEIGHT_BITS - 1)) - 1);

            // Mise à jour des poids pour chaque bit d'historique
            for i in 0..input_history.len() {
                // Si actual_outcome == input_history[i], on ajoute le poids. Si différents, on soustrait.
                // équivalent à self.weights[i+1] += actual_outcome * input_history[i]
                if actual_outcome == input_history[i] {
                    self.weight[i + 1] += 1;
                } else {
                    self.weight[i + 1] -= 1;
                }
                self.weight[i + 1] = self.weight[i + 1].clamp(-(1 << (WEIGHT_BITS - 1)), (1 << (WEIGHT_BITS - 1)) - 1);
            }
        }
    }
}


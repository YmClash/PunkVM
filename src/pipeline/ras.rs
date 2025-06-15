//src/pipeline/ras.rs

use std::collections::VecDeque;

pub struct ReturnAddressStack {
    stack: VecDeque<u32>,
    max_size: usize,
    ///Statistique

    pushes : u64,
    pops : u64,
    hits : u64,
    misses : u64,
}

#[derive(Debug, Clone, Copy)]
pub struct RASStats {
    pub pushes: u64,
    pub pops: u64,
    pub hits: u64,
    pub misses: u64,
    pub accuracy: f64,
    pub current_depth: usize,
    pub max_depth: usize,
}

impl ReturnAddressStack {
    /// crée un nouveau  ReturnAddressStack avec une taille maximale
    pub fn new(max_size: usize) -> Self {
        Self{
            stack: VecDeque::with_capacity(max_size),
            max_size,
            pushes: 0,
            pops: 0,
            hits: 0,
            misses: 0,
        }
    }

    /// Push une adresse de retour sur la pile lors de CALL
    pub fn push(&mut self, return_address: u32) {
        if self.stack.len() >= self.max_size{
            self.stack.pop_front();
        }
        self.stack.push_back(return_address);
        self.pushes += 1;
        println!("Return Address Stack PUSH: addr= 0x{:08X}, depth={}",return_address,self.stack.len())
    }
    /// Pop une adresse de retour de la pile lors de RET

    pub fn pop(&mut self) -> Option<u32> {
        let result = self.stack.pop_back();
        self.pops += 1;

        if let Some(addr) = result {
            println!("Return Address Stack POP: addr= 0x{:08X}, depth={}", addr, self.stack.len());
        }else {
            println!("Return Address Stack POP: stack is empty");
        }
        result
    }

    pub fn predict(&self) -> Option<u32> {
        self.stack.back().copied()
    }

    ///  Update les statistiques de la prédiction
    pub fn update_prediction(&mut self,predicted: Option<u32>, actual: u32) {
        if let  Some(pred) = predicted{
            if pred == actual {
                self.hits +=1;
            }else {
                self.misses +=1;
            }
        }else {
            self.misses +=1;
        }
    }

    /// Retourne  le taux de réussite de la prédiction

    pub fn accuracy(&self) -> f64 {
        let  total = self.hits + self.misses;
        if total == 0 {
            (self.hits as f64 / total as f64) * 100.0
        }else {
            0.0
        }
    }

    pub fn stats(&self) -> RASStats{
        RASStats{
            pushes:self.pushes,
            pops:self.pops,
            hits:self.hits,
            misses:self.misses,
            accuracy:self.accuracy(),
            current_depth: self.stack.len(),
            max_depth: self.max_size,
        }
    }
    /// Reset le RAS
    pub fn reset(&mut self) {
        self.stack.clear();
        self.pushes = 0;
        self.pops = 0;
        self.hits = 0;
        self.misses = 0;
        println!("Return Address Stack RESET");
    }

    // pub fn is_empty(&self) -> bool {
    //     self.stack.is_empty()
    // }

    pub fn debug_state(&self){
        println!("RAS State: depth={}/{}", self.stack.len(),self.max_size);
        for (i,addr) in self.stack.iter().enumerate(){
            println!("  [{}] 0x{:08X}", i, addr);
        }
    }


}
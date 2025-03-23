//src/

pub mod opcodes;
pub mod instructions;
pub mod format;
pub mod decode_errors;
pub mod files;


// Dans bytecode/mod.rs
pub fn calculate_branch_offset(from_addr: u32, to_addr: u32, instr_size: u32) -> i32 {
    // Pour un saut relatif, l'offset est calculé à partir de l'adresse
    // de l'instruction SUIVANTE (from_addr + instr_size)
    let next_addr = from_addr + instr_size;
    (to_addr as i64 - next_addr as i64) as i32
}
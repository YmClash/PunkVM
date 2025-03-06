// examples/disassemble.rs

use punk_vm::bytecode::{BytecodeFile, ArgValue};
use std::env;
use std::path::Path;
use std::collections::HashMap;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <fichier.punk>", args[0]);
        std::process::exit(1);
    }

    let path = Path::new(&args[1]);

    println!("Désassemblage de: {}", path.display());
    println!("====================================");

    // Charger le fichier bytecode
    let bytecode = BytecodeFile::read_from_file(path)?;

    // Afficher les informations d'en-tête
    println!("Version: {}", bytecode.version.to_string());

    // Afficher les métadonnées
    println!("\nMétadonnées:");
    for (key, value) in &bytecode.metadata {
        println!("  {}: {}", key, value);
    }

    // Créer une table d'adresses inversée pour les symboles
    let mut address_to_symbol: HashMap<u32, String> = HashMap::new();
    for (name, addr) in &bytecode.symbols {
        address_to_symbol.insert(*addr, name.clone());
    }

    // Afficher les segments
    println!("\nSegments:");
    for segment in &bytecode.segments {
        println!("  {:?}: offset={}, size={}, load_addr=0x{:08X}",
                 segment.segment_type, segment.offset, segment.size, segment.load_addr);
    }

    // Désassembler le code
    println!("\nCode ({} instructions):", bytecode.code.len());

    for (i, instruction) in bytecode.code.iter().enumerate() {
        // Vérifier si cette adresse a un symbole associé
        let addr = i as u32;
        let symbol = address_to_symbol.get(&addr);

        if let Some(symbol_name) = symbol {
            println!("\n{}:", symbol_name);
        }

        // Formatage de base de l'instruction
        let mut instr_str = format!("{:04X}: {:?}", i, instruction.opcode);

        // Formatage des arguments
        match (instruction.get_arg1_value(), instruction.get_arg2_value()) {
            (Ok(arg1), Ok(arg2)) => {
                instr_str = match (arg1, arg2) {
                    (ArgValue::None, ArgValue::None) => instr_str,

                    (ArgValue::Register(r), ArgValue::None) => {
                        format!("{} R{}", instr_str, r)
                    },

                    (ArgValue::Register(r1), ArgValue::Register(r2)) => {
                        format!("{} R{}, R{}", instr_str, r1, r2)
                    },

                    (ArgValue::Register(r), ArgValue::Immediate(imm)) => {
                        format!("{} R{}, #{}", instr_str, r, imm)
                    },

                    (ArgValue::Register(r), ArgValue::AbsoluteAddr(addr)) => {
                        if let Some(sym) = address_to_symbol.get(&(addr as u32)) {
                            format!("{} R{}, [{}]", instr_str, r, sym)
                        } else {
                            format!("{} R{}, [0x{:08X}]", instr_str, r, addr)
                        }
                    },

                    (ArgValue::Register(r1), ArgValue::RegisterOffset(r2, offset)) => {
                        format!("{} R{}, [R{}{}{}]",
                                instr_str,
                                r1,
                                r2,
                                if offset >= 0 { "+" } else { "" },
                                offset)
                    },

                    (ArgValue::None, ArgValue::AbsoluteAddr(addr)) => {
                        if let Some(sym) = address_to_symbol.get(&(addr as u32)) {
                            format!("{} {}", instr_str, sym)
                        } else {
                            format!("{} 0x{:08X}", instr_str, addr)
                        }
                    },

                    (ArgValue::None, ArgValue::RelativeAddr(offset)) => {
                        let target_addr = (i as i32 + offset) as u32;
                        if let Some(sym) = address_to_symbol.get(&target_addr) {
                            format!("{} {} (0x{:08X})", instr_str, sym, target_addr)
                        } else {
                            format!("{} 0x{:08X}", instr_str, target_addr)
                        }
                    },

                    _ => format!("{} <format non pris en charge>", instr_str),
                }
            },
            _ => {
                instr_str = format!("{} <erreur de décodage des arguments>", instr_str);
            }
        }

        println!("  {}", instr_str);
    }

    // Afficher les données en lecture seule
    if !bytecode.readonly_data.is_empty() {
        println!("\nDonnées en lecture seule:");
        dump_data(&bytecode.readonly_data, &address_to_symbol);
    }

    // Afficher les données
    if !bytecode.data.is_empty() {
        println!("\nDonnées:");
        dump_data(&bytecode.data, &address_to_symbol);
    }

    Ok(())
}



/// Affiche un dump hexadécimal des données avec des symboles
fn dump_data(data: &[u8], symbols: &HashMap<u32, String>) {
    const BYTES_PER_LINE: usize = 16;

    for chunk_offset in (0..data.len()).step_by(BYTES_PER_LINE) {
        // Vérifier si cette adresse a un symbole associé
        let addr = chunk_offset as u32;
        if let Some(symbol_name) = symbols.get(&addr) {
            println!("\n{}:", symbol_name);
        }

        // Adresse
        print!("  {:04X}:  ", chunk_offset);

        // Octets en hexadécimal
        for i in 0..BYTES_PER_LINE {
            if chunk_offset + i < data.len() {
                print!("{:02X} ", data[chunk_offset + i]);
            } else {
                print!("   ");
            }

            // Espace supplémentaire au milieu
            if i == BYTES_PER_LINE / 2 - 1 {
                print!(" ");
            }
        }

        // Caractères ASCII
        print!("  |");
        for i in 0..BYTES_PER_LINE {
            if chunk_offset + i < data.len() {
                let byte = data[chunk_offset + i];
                if byte >= 32 && byte <= 126 { // Caractères imprimables ASCII
                    print!("{}", byte as char);
                } else {
                    print!(".");
                }
            } else {
                print!(" ");
            }
        }
        println!("|");
    }
}
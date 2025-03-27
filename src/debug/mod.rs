//src/debug/mod.rs

use std::fmt;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Write;
use std::path::Path;

use std::time::{SystemTime, UNIX_EPOCH};

use crate::bytecode::instructions::Instruction;
use crate::pipeline::PipelineState;
use crate::pvm::vm_errors::VMResult;

use std::time::Instant;

//configuration du traceur
pub struct TracerConfig {
    pub enabled: bool,
    pub log_to_console: bool,
    pub log_to_file: bool,
    pub log_file_path: Option<String>,
    pub trace_fetch: bool,
    pub trace_decode: bool,
    pub trace_execute: bool,
    pub trace_memory: bool,
    pub trace_writeback: bool,
    pub trace_hazards: bool,
    pub trace_branches: bool,
    pub trace_registers: bool,
}

impl Default for TracerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            log_to_console: true,
            log_to_file: false,
            log_file_path: None,
            trace_fetch: true,
            trace_decode: true,
            trace_execute: true,
            trace_memory: true,
            trace_writeback: true,
            trace_hazards: true,
            trace_branches: true,
            trace_registers: true,
        }
    }
}

// Evenement de la pipeline a tracer

#[derive(Debug, Clone)]
pub enum TraceEvent {
    // Evenement par Etage
    Fetch {
        cycle: u64,
        pc: u32,
        instruction: Option<Instruction>,
    },

    Decode {
        cycle: u64,
        pc: u32,
        instruction: Option<Instruction>,
        rs1: Option<usize>,
        rs2: Option<usize>,
        rd: Option<usize>,
    },

    Execute {
        cycle: u64,
        pc: u32,
        target_pc: u32,
        branch_type: String,
        taken: bool,
        condition: String,
    },

    Memory {
        cycle: u64,
        pc: u32,
        instruction: Option<Instruction>,
        address: Option<u32>,
        value: Option<u32>,
        is_read: bool,
    },

    Writeback {
        cycle: u64,
        pc: u32,
        rd: Option<usize>,
        value: u64,
    },

    // Evenement special
    Hazard {
        cycle: u64,
        hazard_type: String,
        stall_cycles: u64,
        description: String,
    },

    Branch {
        cycle: u64,
        pc: u32,
        target_pc: u32,
        branch_type: String,
        taken: bool,
        condition: String,
    },

    RegisterUpdate {
        cycle: u64,
        pc: u32,
        register: usize,
        old_value: u64,
        new_value: u64,
        source: String, // "EXEC", "MEM", "WB"
    },

    PipelineStall {
        cycle: u64,
        reason: String,
    },

    PipelineFlush {
        cycle: u64,
        reason: String,
    },
}

impl Display for TraceEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TraceEvent::Fetch {
                cycle,
                pc,
                instruction,
            } => {
                write!(f, "[{:04}] FETCH: PC=0x{:08X}", cycle, pc)?;
                if let Some(instr) = instruction {
                    write!(f, " INSTR={:?}", instr.opcode)?;
                }
                Ok(())
            }
            TraceEvent::Decode {
                cycle,
                pc,
                instruction,
                rs1,
                rs2,
                rd,
            } => {
                write!(f, "[{:04}] DECODE: PC=0x{:08X}", cycle, pc)?;
                if let Some(instr) = instruction {
                    write!(f, " INSTR={:?}", instr.opcode)?;
                }
                let rs1_str = rs1.map_or("NONE".to_string(), |r| format!("R{}", r));
                let rs2_str = rs2.map_or("NONE".to_string(), |r| format!("R{}", r));
                let rd_str = rd.map_or("NONE".to_string(), |r| format!("R{}", r));
                write!(f, " RS1={} RS2={} RD={}", rs1_str, rs2_str, rd_str)
            }
            // TraceEvent::Execute { cycle,pc,target_pc,branch_type,taken,condition} => {
            //     write!(f, "[{:04}] EXECUTE: PC=0x{:08X}", cycle, pc)?;
            //     if let Some(instr) = instruction {
            //         write!(f, " INSTR={:?}", instr.opcode)?;
            //     }
            //     write!(f, " RESULT=0x{:016X} FLAGS={{Z:{} N:{} O:{} C:{}}}",
            //            alu_result, flags.zero as u8, flags.negative as u8,
            //            flags.overflow as u8, flags.carry as u8)
            // },
            TraceEvent::Execute {
                cycle,
                pc,
                target_pc,
                branch_type,
                taken,
                condition,
            } => {
                write!(
                    f,
                    "[{:04}] EXECUTE: PC=0x{:08X} TARGET=0x{:08X} TYPE={} TAKEN={} COND={}",
                    cycle, pc, target_pc, branch_type, taken, condition
                )
            }

            TraceEvent::Memory {
                cycle,
                pc,
                instruction,
                address,
                value,
                is_read,
            } => {
                write!(f, "[{:04}] MEMORY: PC=0x{:08X}", cycle, pc)?;
                if let Some(instr) = instruction {
                    write!(f, " INSTR={:?}", instr.opcode)?;
                }
                if let Some(addr) = address {
                    write!(f, " ADDR=0x{:08X}", addr)?;
                }
                if let Some(val) = value {
                    if *is_read {
                        write!(f, " READ=0x{:016X}", val)?;
                    } else {
                        write!(f, " WRITE=0x{:016X}", val)?;
                    }
                }
                Ok(())
            }
            TraceEvent::Writeback {
                cycle,
                pc,
                rd,
                value,
            } => {
                write!(f, "[{:04}] WRITEBACK: PC=0x{:08X}", cycle, pc)?;
                if let Some(reg) = rd {
                    write!(f, " RD=R{} VALUE=0x{:016X}", reg, value)?;
                }
                Ok(())
            }
            TraceEvent::Hazard {
                cycle,
                hazard_type,
                stall_cycles,
                description,
            } => {
                write!(
                    f,
                    "[{:04}] HAZARD: Type={} Stalls={} {}",
                    cycle, hazard_type, stall_cycles, description
                )
            }
            TraceEvent::Branch {
                cycle,
                pc,
                target_pc,
                branch_type,
                taken,
                condition,
            } => {
                write!(
                    f,
                    "[{:04}] BRANCH: PC=0x{:08X} TARGET=0x{:08X} TYPE={} TAKEN={} COND={}",
                    cycle, pc, target_pc, branch_type, taken, condition
                )
            }

            TraceEvent::RegisterUpdate {
                cycle,
                pc,
                register,
                old_value,
                new_value,
                source,
            } => {
                write!(
                    f,
                    "[{:04}] REG_UPDATE: R{}=0x{:016X} (was 0x{:016X}) SRC={}",
                    cycle, register, new_value, old_value, source
                )
            }
            TraceEvent::PipelineStall { cycle, reason } => {
                write!(f, "[{:04}] STALL: {}", cycle, reason)
            }
            TraceEvent::PipelineFlush { cycle, reason } => {
                write!(f, "[{:04}] FLUSH: {}", cycle, reason)
            }
        }
    }
}

//Gestion de tracage
pub struct PipelineTracer {
    config: TracerConfig,
    // tracer_events: Arc<Mutex<Vec<TraceEvent>>>,
    trace_events: Vec<TraceEvent>,
    current_cycle: u64,
    log_file: Option<File>,
}

impl PipelineTracer {
    // Crée un nouveau traceur
    pub fn new(config: TracerConfig) -> Self {
        let log_file = if config.log_to_file {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let path = config
                .log_file_path
                .clone()
                .unwrap_or_else(|| format!("punkvm_trace_{}.log", timestamp));

            match File::create(&path) {
                Ok(file) => {
                    println!("Traçage activé: écriture dans le fichier {}", path);
                    Some(file)
                }
                Err(e) => {
                    eprintln!("Erreur lors de la création du fichier de traçage: {}", e);
                    None
                }
            }
        } else {
            None
        };

        Self {
            config,
            trace_events: Vec::new(),
            current_cycle: 0,
            log_file,
        }
    }

    // Enregistre un événement de traçage
    pub fn trace(&mut self, event: TraceEvent) {
        if !self.config.enabled {
            return;
        }

        // Filtrer les événements selon la configuration
        let should_log = match &event {
            TraceEvent::Fetch { .. } => self.config.trace_fetch,
            TraceEvent::Decode { .. } => self.config.trace_decode,
            TraceEvent::Execute { .. } => self.config.trace_execute,
            TraceEvent::Memory { .. } => self.config.trace_memory,
            TraceEvent::Writeback { .. } => self.config.trace_writeback,
            TraceEvent::Hazard { .. } => self.config.trace_hazards,
            TraceEvent::Branch { .. } => self.config.trace_branches,
            TraceEvent::RegisterUpdate { .. } => self.config.trace_registers,
            _ => true, // Toujours tracer les stalls et flushes
        };

        if should_log {
            let event_str = format!("{}\n", event);

            // Écrire dans la console si configuré
            if self.config.log_to_console {
                print!("{}", event_str);
            }

            // Écrire dans le fichier si configuré
            if self.config.log_to_file {
                if let Some(file) = &mut self.log_file {
                    let _ = file.write_all(event_str.as_bytes());
                }
            }

            // Stocker l'événement
            self.trace_events.push(event);
        }
    }

    // Commence un nouveau cycle
    pub fn start_cycle(&mut self, cycle: u64) {
        self.current_cycle = cycle;
    }

    // Trace un état complet du pipeline
    pub fn trace_pipeline_state(&mut self, state: &PipelineState, registers: &[u64]) {
        if !self.config.enabled {
            return;
        }

        // Tracer l'état de stall
        if state.stalled {
            self.trace(TraceEvent::PipelineStall {
                cycle: self.current_cycle,
                reason: "Hazard détecté".to_string(),
            });
        }

        // Tracer les étages du pipeline
        if let Some(fd_reg) = &state.fetch_decode {
            self.trace(TraceEvent::Fetch {
                cycle: self.current_cycle,
                pc: fd_reg.pc,
                instruction: Some(fd_reg.instruction.clone()),
            });
        }

        if let Some(de_reg) = &state.decode_execute {
            self.trace(TraceEvent::Decode {
                cycle: self.current_cycle,
                pc: de_reg.pc,
                instruction: Some(de_reg.instruction.clone()),
                rs1: de_reg.rs1,
                rs2: de_reg.rs2,
                rd: de_reg.rd,
            });
        }

        if let Some(em_reg) = &state.execute_memory {
            // Tracer l'étage Execute
            self.trace(TraceEvent::Execute {
                cycle: self.current_cycle,
                pc: 0, // PC non disponible dans ExecuteMemoryRegister
                // target_pc: em_reg.target_pc,$
                target_pc: em_reg.branch_target.unwrap_or(0), // Utiliser la cible de branchement si disponible

                branch_type: format!("{:?}", em_reg.instruction.opcode),
                taken: em_reg.branch_taken,
                condition: "".to_string(), // Condition non disponible directement

                                           // instruction: Some(em_reg.instruction.clone()),
                                           // alu_result: em_reg.alu_result,
                                           // flags: ALUFlags::default(), // Flags non disponibles directement
            });

            // Tracer les branches
            if em_reg.branch_taken {
                if let Some(target) = em_reg.branch_target {
                    self.trace(TraceEvent::Branch {
                        cycle: self.current_cycle,
                        pc: 0, // PC non disponible directement
                        target_pc: target,
                        branch_type: format!("{:?}", em_reg.instruction.opcode),
                        taken: true,
                        condition: "".to_string(), // Condition non disponible directement
                    });
                }
            }
        }

        if let Some(mw_reg) = &state.memory_writeback {
            // Tracer l'étage Writeback
            if let Some(rd) = mw_reg.rd {
                if rd < registers.len() {
                    self.trace(TraceEvent::Writeback {
                        cycle: self.current_cycle,
                        pc: 0, // PC non disponible directement
                        rd: Some(rd),
                        value: mw_reg.result,
                    });
                }
            }
        }
    }

    // Trace une modification de registre
    pub fn trace_register_update(
        &mut self,
        register: usize,
        old_value: u64,
        new_value: u64,
        source: &str,
    ) {
        if !self.config.enabled || !self.config.trace_registers {
            return;
        }

        self.trace(TraceEvent::RegisterUpdate {
            cycle: self.current_cycle,
            pc: 0, // PC non disponible directement
            register,
            old_value,
            new_value,
            source: source.to_string(),
        });
    }

    // Exporte les événements de traçage dans un fichier CSV
    pub fn export_to_csv<P: AsRef<Path>>(&self, path: P) -> VMResult<()> {
        let mut file = File::create(path)?;

        // Écrire l'en-tête
        writeln!(file, "Cycle,Event,PC,Instruction,Details")?;

        // Écrire les événements
        for event in &self.trace_events {
            match event {
                TraceEvent::Fetch {
                    cycle,
                    pc,
                    instruction,
                } => {
                    let instr_str = instruction
                        .as_ref()
                        .map_or("None".to_string(), |i| format!("{:?}", i.opcode));
                    writeln!(file, "{},FETCH,0x{:08X},{},", cycle, pc, instr_str)?;
                }
                TraceEvent::Decode {
                    cycle,
                    pc,
                    instruction,
                    rs1,
                    rs2,
                    rd,
                } => {
                    let instr_str = instruction
                        .as_ref()
                        .map_or("None".to_string(), |i| format!("{:?}", i.opcode));
                    let rs1_str = rs1.map_or("None".to_string(), |r| format!("R{}", r));
                    let rs2_str = rs2.map_or("None".to_string(), |r| format!("R{}", r));
                    let rd_str = rd.map_or("None".to_string(), |r| format!("R{}", r));
                    writeln!(
                        file,
                        "{},DECODE,0x{:08X},{},\"RS1={} RS2={} RD={}\"",
                        cycle, pc, instr_str, rs1_str, rs2_str, rd_str
                    )?;
                }
                // Ajouter les autres types d'événements...
                _ => writeln!(file, "{},{}", event.to_string(), "")?,
            }
        }

        Ok(())
    }

    // Génère un rapport de synthèse des événements de traçage
    pub fn generate_summary(&self) -> String {
        println!("Génération du rapport de synthèse...");
        let mut summary = String::new();
        summary.push_str("=== Rapport de synthèse du traçage PunkVM ===\n\n");

        // Statistiques globales
        let total_cycles = self.current_cycle;
        let hazard_count = self
            .trace_events
            .iter()
            .filter(|e| matches!(e, TraceEvent::Hazard { .. }))
            .count();
        let branch_count = self
            .trace_events
            .iter()
            .filter(|e| matches!(e, TraceEvent::Branch { .. }))
            .count();
        let stall_count = self
            .trace_events
            .iter()
            .filter(|e| matches!(e, TraceEvent::PipelineStall { .. }))
            .count();
        let flush_count = self
            .trace_events
            .iter()
            .filter(|e| matches!(e, TraceEvent::PipelineFlush { .. }))
            .count();

        summary.push_str(&format!("Cycles totaux: {}\n", total_cycles));
        summary.push_str(&format!("Nombre de hazards: {}\n", hazard_count));
        summary.push_str(&format!("Nombre de branchements: {}\n", branch_count));
        summary.push_str(&format!("Nombre de stalls: {}\n", stall_count));
        summary.push_str(&format!("Nombre de flushes: {}\n", flush_count));

        // Statistiques des branchements
        if branch_count > 0 {
            let branches: Vec<_> = self
                .trace_events
                .iter()
                .filter_map(|e| {
                    if let TraceEvent::Branch {
                        branch_type, taken, ..
                    } = e
                    {
                        Some((branch_type.as_str(), *taken))
                    } else {
                        None
                    }
                })
                .collect();

            let taken_count = branches.iter().filter(|(_, taken)| *taken).count();
            let not_taken_count = branches.len() - taken_count;

            summary.push_str("\nStatistiques de branchement:\n");
            summary.push_str(&format!("  Total: {}\n", branches.len()));
            summary.push_str(&format!(
                "  Pris: {} ({:.1}%)\n",
                taken_count,
                (taken_count as f64 / branches.len() as f64) * 100.0
            ));
            summary.push_str(&format!(
                "  Non pris: {} ({:.1}%)\n",
                not_taken_count,
                (not_taken_count as f64 / branches.len() as f64) * 100.0
            ));
        }

        // Statistiques des hazards
        if hazard_count > 0 {
            let hazards: Vec<_> = self
                .trace_events
                .iter()
                .filter_map(|e| {
                    if let TraceEvent::Hazard {
                        hazard_type,
                        stall_cycles,
                        ..
                    } = e
                    {
                        Some((hazard_type.as_str(), *stall_cycles))
                    } else {
                        None
                    }
                })
                .collect();

            // Compter par type de hazard
            let mut hazard_types = std::collections::HashMap::new();
            let mut total_stall_cycles = 0;

            for (htype, stalls) in &hazards {
                *hazard_types.entry(*htype).or_insert(0) += 1;
                total_stall_cycles += *stalls;
            }

            summary.push_str("\nStatistiques des hazards:\n");
            for (htype, count) in hazard_types.iter() {
                summary.push_str(&format!(
                    "  {}: {} ({:.1}%)\n",
                    htype,
                    *count,
                    (*count as f64 / hazards.len() as f64) * 100.0
                ));
            }
            summary.push_str(&format!(
                "  Cycles de stall totaux: {}\n",
                total_stall_cycles
            ));
            summary.push_str(&format!(
                "  Moyenne stalls/hazard: {:.2}\n",
                total_stall_cycles as f64 / hazards.len() as f64
            ));
        }
        println!("Rapport de synthèse généré avec succès.");
        summary
    }
}

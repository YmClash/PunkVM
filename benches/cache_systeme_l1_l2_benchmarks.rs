// benches/cache_systeme_l1_l2_benchmarks.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use PunkVM::bytecode::files::{BytecodeFile, BytecodeVersion, SegmentMetadata, SegmentType};
use PunkVM::bytecode::instructions::Instruction;
use PunkVM::bytecode::opcodes::Opcode;
use PunkVM::pvm::vm::{PunkVM, VMConfig};
use std::time::Duration;

/// Crée un programme de test simple pour les benchmarks cache
fn create_simple_cache_test(data_size: usize) -> BytecodeFile {
    let mut program = BytecodeFile::new();
    program.version = BytecodeVersion::new(0, 1, 0, 0);
    program.add_metadata("name", "Simple Cache Test");
    program.add_metadata("description", &format!("Test cache avec {} bytes", data_size));
    program.add_metadata("author", "PunkVM Cache Benchmark");

    // Base d'adresse
    program.add_instruction(Instruction::create_reg_imm32(Opcode::Mov, 0, 0x1000));
    
    let num_accesses = data_size / 64; // Une adresse par ligne de cache
    
    // Phase 1: Écriture séquentielle 
    for i in 0..num_accesses {
        program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, (i % 256) as u8));
        program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 7, 64));
        program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Mul, 2, 1, 7)); // R2 = i * 64
        program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 3, 0, 2)); // R3 = base + offset
        program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 4, (i % 256) as u8));
        program.add_instruction(Instruction::create_store_reg_offset(Opcode::Store, 4, 3, 0));
    }

    // Phase 2: Lecture séquentielle
    for i in 0..num_accesses {
        program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 1, (i % 256) as u8));
        program.add_instruction(Instruction::create_reg_imm8(Opcode::Mov, 7, 64));
        program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Mul, 2, 1, 7)); 
        program.add_instruction(Instruction::create_reg_reg_reg(Opcode::Add, 3, 0, 2)); 
        program.add_instruction(Instruction::create_load_reg_offset(5, 3, 0));
    }

    program.add_instruction(Instruction::create_no_args(Opcode::Halt));

    // Configuration des segments
    let total_code_size: u32 = program.code.iter()
        .map(|instr| instr.total_size() as u32)
        .sum();
    program.segments = vec![SegmentMetadata::new(SegmentType::Code, 0, total_code_size, 0)];

    program
}

/// Exécute un benchmark simple et retourne le temps d'exécution
fn run_simple_cache_benchmark(l1_size: usize, l2_size: usize, data_size: usize) -> Duration {
    let vm_config = VMConfig {
        memory_size: 64 * 1024,
        num_registers: 19,
        l1_cache_size: l1_size,
        l2_cache_size: l2_size,
        store_buffer_size: 8,
        stack_size: 4 * 1024,
        stack_base: 0xC000,
        fetch_buffer_size: 8,
        btb_size: 16,
        ras_size: 4,
        enable_forwarding: true,
        enable_hazard_detection: true,
        enable_tracing: false,
    };

    let mut vm = PunkVM::with_config(vm_config);
    let program = create_simple_cache_test(data_size);
    
    vm.load_program_from_bytecode(program).unwrap();
    
    let start = std::time::Instant::now();
    vm.run().unwrap();
    start.elapsed()
}

/// Benchmark principal: comparaison des tailles de cache
fn bench_cache_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_hierarchy_sizes");
    group.measurement_time(Duration::from_secs(5));
    
    let configs = vec![
        ("Small_1KB_4KB", 1 * 1024, 4 * 1024),
        ("Medium_4KB_16KB", 4 * 1024, 16 * 1024),
        ("Large_8KB_32KB", 8 * 1024, 32 * 1024),
    ];
    
    let data_sizes = vec![2 * 1024, 8 * 1024, 16 * 1024]; // 2KB, 8KB, 16KB
    
    for (config_name, l1_size, l2_size) in &configs {
        for &data_size in &data_sizes {
            group.bench_with_input(
                BenchmarkId::new(*config_name, format!("{}KB", data_size / 1024)),
                &data_size,
                |b, &size| {
                    b.iter(|| {
                        let duration = run_simple_cache_benchmark(*l1_size, *l2_size, size);
                        black_box(duration)
                    })
                }
            );
        }
    }
    
    group.finish();
}

/// Benchmark: scalabilité avec la taille des données
fn bench_data_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_data_scaling");
    group.measurement_time(Duration::from_secs(5));
    
    let l1_size = 4 * 1024;  // 4KB L1
    let l2_size = 16 * 1024; // 16KB L2
    
    let data_sizes = vec![
        1 * 1024,   // 1KB - tout en L1
        8 * 1024,   // 8KB - déborde L1, entre en L2
        32 * 1024,  // 32KB - déborde L2, va en mémoire
    ];
    
    for &data_size in &data_sizes {
        group.bench_with_input(
            BenchmarkId::new("scaling", format!("{}KB", data_size / 1024)),
            &data_size,
            |b, &size| {
                b.iter(|| {
                    let duration = run_simple_cache_benchmark(l1_size, l2_size, size);
                    black_box(duration)
                })
            }
        );
    }
    
    group.finish();
}

criterion_group!(
    cache_benches,
    bench_cache_sizes,
    bench_data_scaling
);

criterion_main!(cache_benches);
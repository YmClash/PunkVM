// benches/simd_fpu_benchmarks.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use PunkVM::alu::v_alu::{VectorALU, VectorOperation};
use PunkVM::alu::fpu::{FPU, FPUOperation, FloatPrecision};
use PunkVM::bytecode::simds::{Vector128, Vector256, VectorDataType, Vector256DataType};

fn bench_vector_alu_128(c: &mut Criterion) {
    let mut group = c.benchmark_group("SIMD_128_Operations");
    
    let mut v_alu = VectorALU::new();
    let vec1 = Vector128::from_i32x4([1, 2, 3, 4]);
    let vec2 = Vector128::from_i32x4([5, 6, 7, 8]);
    
    v_alu.write_v128(0, vec1).unwrap();
    v_alu.write_v128(1, vec2).unwrap();

    // Benchmark des opérations arithmétiques
    group.bench_function("add_i32x4", |b| {
        b.iter(|| {
            v_alu.execute_v128(
                black_box(VectorOperation::Add),
                black_box(2),
                black_box(0),
                black_box(Some(1)),
                black_box(VectorDataType::I32x4),
            ).unwrap();
        })
    });

    group.bench_function("mul_i32x4", |b| {
        b.iter(|| {
            v_alu.execute_v128(
                black_box(VectorOperation::Mul),
                black_box(2),
                black_box(0),
                black_box(Some(1)),
                black_box(VectorDataType::I32x4),
            ).unwrap();
        })
    });

    group.bench_function("and_logical", |b| {
        b.iter(|| {
            v_alu.execute_v128(
                black_box(VectorOperation::And),
                black_box(2),
                black_box(0),
                black_box(Some(1)),
                black_box(VectorDataType::I64x2),
            ).unwrap();
        })
    });

    // Benchmark avec différents types de données
    let vec_f32 = Vector128::from_f32x4([1.5, 2.5, 3.5, 4.5]);
    v_alu.write_v128(3, vec_f32).unwrap();
    v_alu.write_v128(4, Vector128::from_f32x4([0.5, 1.5, 2.5, 3.5])).unwrap();

    group.bench_function("add_f32x4", |b| {
        b.iter(|| {
            v_alu.execute_v128(
                black_box(VectorOperation::Add),
                black_box(5),
                black_box(3),
                black_box(Some(4)),
                black_box(VectorDataType::F32x4),
            ).unwrap();
        })
    });

    group.finish();
}

fn bench_vector_alu_256(c: &mut Criterion) {
    let mut group = c.benchmark_group("SIMD_256_Operations");
    
    let mut v_alu = VectorALU::new();
    let vec1 = Vector256::from_i32x8([1, 2, 3, 4, 5, 6, 7, 8]);
    let vec2 = Vector256::from_i32x8([10, 20, 30, 40, 50, 60, 70, 80]);
    
    v_alu.write_v256(0, vec1).unwrap();
    v_alu.write_v256(1, vec2).unwrap();

    group.bench_function("add_i32x8", |b| {
        b.iter(|| {
            v_alu.execute_v256(
                black_box(VectorOperation::Add),
                black_box(2),
                black_box(0),
                black_box(Some(1)),
                black_box(Vector256DataType::I32x8),
            ).unwrap();
        })
    });

    group.bench_function("mul_i32x8", |b| {
        b.iter(|| {
            v_alu.execute_v256(
                black_box(VectorOperation::Mul),
                black_box(2),
                black_box(0),
                black_box(Some(1)),
                black_box(Vector256DataType::I32x8),
            ).unwrap();
        })
    });

    group.finish();
}

fn bench_fpu_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("FPU_Operations");
    
    let mut fpu = FPU::new();
    fpu.write_fp_register(0, 3.14159).unwrap();
    fpu.write_fp_register(1, 2.71828).unwrap();

    // Benchmark des opérations de base
    group.bench_function("add_double", |b| {
        b.iter(|| {
            fpu.execute(
                black_box(FPUOperation::Add),
                black_box(2),
                black_box(0),
                black_box(Some(1)),
                black_box(FloatPrecision::Double),
            ).unwrap();
        })
    });

    group.bench_function("mul_double", |b| {
        b.iter(|| {
            fpu.execute(
                black_box(FPUOperation::Mul),
                black_box(2),
                black_box(0),
                black_box(Some(1)),
                black_box(FloatPrecision::Double),
            ).unwrap();
        })
    });

    group.bench_function("div_double", |b| {
        b.iter(|| {
            fpu.execute(
                black_box(FPUOperation::Div),
                black_box(2),
                black_box(0),
                black_box(Some(1)),
                black_box(FloatPrecision::Double),
            ).unwrap();
        })
    });

    fpu.write_fp_register(3, 16.0).unwrap();
    group.bench_function("sqrt_double", |b| {
        b.iter(|| {
            fpu.execute(
                black_box(FPUOperation::Sqrt),
                black_box(4),
                black_box(3),
                black_box(None),
                black_box(FloatPrecision::Double),
            ).unwrap();
        })
    });

    // Benchmark simple vs double précision
    group.bench_function("add_single", |b| {
        b.iter(|| {
            fpu.execute(
                black_box(FPUOperation::Add),
                black_box(2),
                black_box(0),
                black_box(Some(1)),
                black_box(FloatPrecision::Single),
            ).unwrap();
        })
    });

    group.finish();
}

fn bench_cache_hierarchy(c: &mut Criterion) {
    use PunkVM::pvm::caches::CacheHierarchy;
    use PunkVM::pvm::cache_configs::CacheConfig;
    
    let mut group = c.benchmark_group("Cache_Hierarchy");
    
    use PunkVM::pvm::cache_configs::{WritePolicy, ReplacementPolicy};
    
    let l1_data_config = CacheConfig {
        size: 32 * 1024,        // 32KB
        lines_size: 64,         // 64 bytes par ligne
        associativity: 8,       // 8-way associative
        write_policy: WritePolicy::WriteThrough,
        replacement_policy: ReplacementPolicy::LRU,
    };
    
    let l1_inst_config = CacheConfig {
        size: 32 * 1024,        // 32KB
        lines_size: 64,         // 64 bytes par ligne
        associativity: 8,       // 8-way associative
        write_policy: WritePolicy::WriteThrough,
        replacement_policy: ReplacementPolicy::LRU,
    };
    
    let l2_config = CacheConfig {
        size: 256 * 1024,       // 256KB
        lines_size: 64,         // 64 bytes par ligne
        associativity: 8,       // 8-way associative
        write_policy: WritePolicy::WriteBack,
        replacement_policy: ReplacementPolicy::LRU,
    };
    
    let mut cache_hierarchy = CacheHierarchy::new(l1_data_config, l1_inst_config, l2_config);

    // Benchmark des accès cache
    group.bench_function("cache_read", |b| {
        b.iter(|| {
            cache_hierarchy.access_data(black_box(0x1000), black_box(false), black_box(None)).unwrap();
        })
    });

    group.bench_function("cache_write", |b| {
        b.iter(|| {
            cache_hierarchy.access_data(black_box(0x3000), black_box(true), black_box(Some(42))).unwrap();
        })
    });

    // Benchmark avec différents patterns d'accès
    group.bench_function("sequential_access", |b| {
        let mut addr = 0x10000;
        b.iter(|| {
            cache_hierarchy.access_data(black_box(addr), black_box(false), black_box(None)).unwrap();
            addr += 64; // Nouvelle ligne de cache
            if addr > 0x20000 { addr = 0x10000; }
        })
    });

    group.bench_function("random_access", |b| {
        use rand::Rng;
        let mut rng = rand::rng();
        b.iter(|| {
            let addr = rng.random_range(0x10000..0x50000) & !63; // Aligné sur 64 bytes
            cache_hierarchy.access_data(black_box(addr), black_box(false), black_box(None)).unwrap();
        })
    });

    group.finish();
}

fn bench_data_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("Data_Size_Scaling");
    
    // Benchmark de l'addition vectorielle avec différentes tailles
    for size in [4, 8, 16, 32].iter() {
        group.bench_with_input(BenchmarkId::new("vector_add", size), size, |b, &size| {
            let mut v_alu = VectorALU::new();
            
            match size {
                4 => {
                    let vec1 = Vector128::from_i32x4([1, 2, 3, 4]);
                    let vec2 = Vector128::from_i32x4([5, 6, 7, 8]);
                    v_alu.write_v128(0, vec1).unwrap();
                    v_alu.write_v128(1, vec2).unwrap();
                    
                    b.iter(|| {
                        v_alu.execute_v128(
                            VectorOperation::Add,
                            2, 0, Some(1),
                            VectorDataType::I32x4,
                        ).unwrap();
                    });
                }
                8 => {
                    let vec1 = Vector256::from_i32x8([1, 2, 3, 4, 5, 6, 7, 8]);
                    let vec2 = Vector256::from_i32x8([10, 20, 30, 40, 50, 60, 70, 80]);
                    v_alu.write_v256(0, vec1).unwrap();
                    v_alu.write_v256(1, vec2).unwrap();
                    
                    b.iter(|| {
                        v_alu.execute_v256(
                            VectorOperation::Add,
                            2, 0, Some(1),
                            Vector256DataType::I32x8,
                        ).unwrap();
                    });
                }
                _ => {}
            }
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_vector_alu_128,
    bench_vector_alu_256,
    bench_fpu_operations,
    bench_cache_hierarchy,
    bench_data_sizes
);

criterion_main!(benches);
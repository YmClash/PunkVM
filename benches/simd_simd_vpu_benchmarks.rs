// benches/simd_simd_vpu_benchmarks.rs
// Comprehensive SIMD benchmarks for PunkVM - All vector types and operations

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use PunkVM::alu::v_alu::{VectorALU, VectorOperation};
use PunkVM::alu::alu::{ALU, ALUOperation};
use PunkVM::bytecode::simds::{Vector128, Vector256, VectorDataType, Vector256DataType};
use PunkVM::bytecode::instructions::Instruction;
use PunkVM::bytecode::opcodes::Opcode;
use PunkVM::pipeline::execute::ExecuteStage;
use PunkVM::pipeline::DecodeExecuteRegister;

fn bench_extended_vector_types_128(c: &mut Criterion) {
    let mut group = c.benchmark_group("SIMD_128_Extended_Types");
    
    let mut v_alu = VectorALU::new();

    // Benchmark i16x8 operations
    group.bench_function("const_i16x8", |b| {
        b.iter(|| {
            let vec = Vector128::from_i16x8(black_box([1, 2, 3, 4, 5, 6, 7, 8]));
            v_alu.write_v128(black_box(0), black_box(vec)).unwrap();
        })
    });

    // Benchmark i64x2 operations  
    group.bench_function("const_i64x2", |b| {
        b.iter(|| {
            let vec = Vector128::from_i64x2(black_box([0x1234567890ABCDEF, 0x7EDCBA0987654321]));
            v_alu.write_v128(black_box(1), black_box(vec)).unwrap();
        })
    });

    // Benchmark f64x2 operations
    group.bench_function("const_f64x2", |b| {
        b.iter(|| {
            let vec = Vector128::from_f64x2(black_box([3.14159265359, 2.71828182846]));
            v_alu.write_v128(black_box(2), black_box(vec)).unwrap();
        })
    });

    // Arithmetic operations for extended types
    let vec1_i16 = Vector128::from_i16x8([1, 2, 3, 4, 5, 6, 7, 8]);
    let vec2_i16 = Vector128::from_i16x8([8, 7, 6, 5, 4, 3, 2, 1]);
    v_alu.write_v128(10, vec1_i16).unwrap();
    v_alu.write_v128(11, vec2_i16).unwrap();

    group.bench_function("add_i16x8", |b| {
        b.iter(|| {
            v_alu.execute_v128(
                black_box(VectorOperation::Add),
                black_box(12),
                black_box(10),
                black_box(Some(11)),
                black_box(VectorDataType::I16x8),
            ).unwrap();
        })
    });

    let vec1_i64 = Vector128::from_i64x2([100, 200]);
    let vec2_i64 = Vector128::from_i64x2([50, 75]);
    v_alu.write_v128(13, vec1_i64).unwrap();
    v_alu.write_v128(14, vec2_i64).unwrap();

    group.bench_function("add_i64x2", |b| {
        b.iter(|| {
            v_alu.execute_v128(
                black_box(VectorOperation::Add),
                black_box(15),
                black_box(13),
                black_box(Some(14)),
                black_box(VectorDataType::I64x2),
            ).unwrap();
        })
    });

    group.finish();
}

fn bench_extended_vector_types_256(c: &mut Criterion) {
    let mut group = c.benchmark_group("SIMD_256_Extended_Types");
    
    let mut v_alu = VectorALU::new();

    // Benchmark i16x16 operations
    group.bench_function("const_i16x16", |b| {
        b.iter(|| {
            let vec = Vector256::from_i16x16(black_box([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]));
            v_alu.write_v256(black_box(0), black_box(vec)).unwrap();
        })
    });

    // Benchmark i64x4 operations
    group.bench_function("const_i64x4", |b| {
        b.iter(|| {
            let vec = Vector256::from_i64x4(black_box([0x1111111111111111, 0x2222222222222222, 0x3333333333333333, 0x4444444444444444]));
            v_alu.write_v256(black_box(1), black_box(vec)).unwrap();
        })
    });

    // Benchmark f64x4 operations
    group.bench_function("const_f64x4", |b| {
        b.iter(|| {
            let vec = Vector256::from_f64x4(black_box([1.1, 2.2, 3.3, 4.4]));
            v_alu.write_v256(black_box(2), black_box(vec)).unwrap();
        })
    });

    // 256-bit arithmetic operations
    let vec1_i32x8 = Vector256::from_i32x8([1, 2, 3, 4, 5, 6, 7, 8]);
    let vec2_i32x8 = Vector256::from_i32x8([10, 20, 30, 40, 50, 60, 70, 80]);
    v_alu.write_v256(10, vec1_i32x8).unwrap();
    v_alu.write_v256(11, vec2_i32x8).unwrap();

    group.bench_function("add_i32x8", |b| {
        b.iter(|| {
            v_alu.execute_v256(
                black_box(VectorOperation::Add),
                black_box(12),
                black_box(10),
                black_box(Some(11)),
                black_box(Vector256DataType::I32x8),
            ).unwrap();
        })
    });

    group.finish();
}

fn bench_simd_const_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("SIMD_Const_Operations");
    
    let mut execute_stage = ExecuteStage::new();
    let mut alu = ALU::new();

    // Test all const opcodes
    let const_opcodes = [
        ("Simd128Const", Opcode::Simd128Const),
        ("Simd128ConstF32", Opcode::Simd128ConstF32),
        ("Simd128ConstI16x8", Opcode::Simd128ConstI16x8),
        ("Simd128ConstI64x2", Opcode::Simd128ConstI64x2),
        ("Simd128ConstF64x2", Opcode::Simd128ConstF64x2),
        ("Simd256Const", Opcode::Simd256Const),
        ("Simd256ConstF32", Opcode::Simd256ConstF32),
        ("Simd256ConstI16x16", Opcode::Simd256ConstI16x16),
        ("Simd256ConstI64x4", Opcode::Simd256ConstI64x4),
        ("Simd256ConstF64x4", Opcode::Simd256ConstF64x4),
    ];

    for (name, opcode) in const_opcodes.iter() {
        group.bench_function(*name, |b| {
            b.iter(|| {
                let instruction = match *opcode {
                    Opcode::Simd128Const => Instruction::create_simd128_const_i32x4(black_box(0), black_box([1, 2, 3, 4])),
                    Opcode::Simd128ConstF32 => Instruction::create_simd128_const_f32x4(black_box(0), black_box([1.0, 2.0, 3.0, 4.0])),
                    Opcode::Simd128ConstI16x8 => Instruction::create_simd128_const_i16x8(black_box(0), black_box([1, 2, 3, 4, 5, 6, 7, 8])),
                    Opcode::Simd128ConstI64x2 => Instruction::create_simd128_const_i64x2(black_box(0), black_box([100, 200])),
                    Opcode::Simd128ConstF64x2 => Instruction::create_simd128_const_f64x2(black_box(0), black_box([1.5, 2.5])),
                    Opcode::Simd256Const => Instruction::create_simd256_const_i32x8(black_box(0), black_box([1, 2, 3, 4, 5, 6, 7, 8])),
                    Opcode::Simd256ConstF32 => Instruction::create_simd256_const_f32x8(black_box(0), black_box([1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0])),
                    Opcode::Simd256ConstI16x16 => Instruction::create_simd256_const_i16x16(black_box(0), black_box([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16])),
                    Opcode::Simd256ConstI64x4 => Instruction::create_simd256_const_i64x4(black_box(0), black_box([100, 200, 300, 400])),
                    Opcode::Simd256ConstF64x4 => Instruction::create_simd256_const_f64x4(black_box(0), black_box([1.1, 2.2, 3.3, 4.4])),
                    _ => panic!("Unexpected opcode: {:?}", opcode),
                };
                
                let de_reg = DecodeExecuteRegister {
                    instruction,
                    pc: 100,
                    rs1: None,
                    rs2: None,
                    rd: Some(0),
                    rs1_value: 0,
                    rs2_value: 0,
                    immediate: Some(0x12345678),
                    branch_addr: None,
                    branch_prediction: None,
                    stack_operation: None,
                    mem_addr: None,
                    stack_value: None,
                };

                execute_stage.process_direct(&de_reg, &mut alu).unwrap();
            })
        });
    }

    group.finish();
}

fn bench_simd_vs_scalar(c: &mut Criterion) {
    let mut group = c.benchmark_group("SIMD_vs_Scalar_Comparison");
    
    let mut v_alu = VectorALU::new();
    let mut alu = ALU::new();

    // Setup SIMD data
    let vec1_i32x4 = Vector128::from_i32x4([1, 2, 3, 4]);
    let vec2_i32x4 = Vector128::from_i32x4([5, 6, 7, 8]);
    v_alu.write_v128(0, vec1_i32x4).unwrap();
    v_alu.write_v128(1, vec2_i32x4).unwrap();

    // SIMD addition (4 operations in parallel)
    group.bench_function("simd_add_4x_parallel", |b| {
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

    // Scalar addition (4 separate operations)
    group.bench_function("scalar_add_4x_sequential", |b| {
        b.iter(|| {
            for i in 0..4 {
                alu.execute(
                    black_box(ALUOperation::Add),
                    black_box(i + 1),
                    black_box(i + 5),
                ).unwrap();
            }
        })
    });

    // 256-bit SIMD (8 operations in parallel)
    let vec1_i32x8 = Vector256::from_i32x8([1, 2, 3, 4, 5, 6, 7, 8]);
    let vec2_i32x8 = Vector256::from_i32x8([10, 20, 30, 40, 50, 60, 70, 80]);
    v_alu.write_v256(0, vec1_i32x8).unwrap();
    v_alu.write_v256(1, vec2_i32x8).unwrap();

    group.bench_function("simd256_add_8x_parallel", |b| {
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

    group.bench_function("scalar_add_8x_sequential", |b| {
        b.iter(|| {
            for i in 0..8 {
                alu.execute(
                    black_box(ALUOperation::Add),
                    black_box(i + 1),
                    black_box(i + 10),
                ).unwrap();
            }
        })
    });

    group.finish();
}

fn bench_complete_pipeline_simd(c: &mut Criterion) {
    let mut group = c.benchmark_group("Complete_Pipeline_SIMD");
    
    let mut execute_stage = ExecuteStage::new();
    let mut alu = ALU::new();

    // Initialize vector registers
    let vec1 = Vector128::from_i32x4([1, 2, 3, 4]);
    let vec2 = Vector128::from_i32x4([5, 6, 7, 8]);
    execute_stage.get_vector_alu_mut().write_v128(0, vec1).unwrap();
    execute_stage.get_vector_alu_mut().write_v128(1, vec2).unwrap();

    // Benchmark different SIMD operations through pipeline
    let simd_ops = [
        ("Simd128Add", Opcode::Simd128Add),
        ("Simd128Sub", Opcode::Simd128Sub),
        ("Simd128Mul", Opcode::Simd128Mul),
        ("Simd128And", Opcode::Simd128And),
        ("Simd128Or", Opcode::Simd128Or),
        ("Simd128Xor", Opcode::Simd128Xor),
    ];

    for (name, opcode) in simd_ops.iter() {
        group.bench_function(*name, |b| {
            b.iter(|| {
                let instruction = Instruction::create_reg_reg_reg(*opcode, black_box(2), black_box(0), black_box(1));
                
                let de_reg = DecodeExecuteRegister {
                    instruction,
                    pc: 100,
                    rs1: Some(0),
                    rs2: Some(1),
                    rd: Some(2),
                    rs1_value: 0,
                    rs2_value: 0,
                    immediate: None,
                    branch_addr: None,
                    branch_prediction: None,
                    stack_operation: None,
                    mem_addr: None,
                    stack_value: None,
                };

                execute_stage.process_direct(&de_reg, &mut alu).unwrap();
            })
        });
    }

    // Benchmark 256-bit operations
    let vec1_256 = Vector256::from_i32x8([1, 2, 3, 4, 5, 6, 7, 8]);
    let vec2_256 = Vector256::from_i32x8([10, 20, 30, 40, 50, 60, 70, 80]);
    execute_stage.get_vector_alu_mut().write_v256(0, vec1_256).unwrap();
    execute_stage.get_vector_alu_mut().write_v256(1, vec2_256).unwrap();

    group.bench_function("Simd256Add", |b| {
        b.iter(|| {
            let instruction = Instruction::create_reg_reg_reg(Opcode::Simd256Add, black_box(2), black_box(0), black_box(1));
            
            let de_reg = DecodeExecuteRegister {
                instruction,
                pc: 100,
                rs1: Some(0),
                rs2: Some(1),
                rd: Some(2),
                rs1_value: 0,
                rs2_value: 0,
                immediate: None,
                branch_addr: None,
                branch_prediction: None,
                stack_operation: None,
                mem_addr: None,
                stack_value: None,
            };

            execute_stage.process_direct(&de_reg, &mut alu).unwrap();
        })
    });

    group.finish();
}

fn bench_memory_alignment(c: &mut Criterion) {
    let mut group = c.benchmark_group("SIMD_Memory_Alignment");
    
    let mut v_alu = VectorALU::new();

    // Test different memory access patterns for vectors
    group.bench_function("aligned_128bit_access", |b| {
        let vec = Vector128::from_i32x4([1, 2, 3, 4]);
        b.iter(|| {
            for i in 0..16 {
                v_alu.write_v128(black_box(i % 16), black_box(vec)).unwrap();
                let _result = v_alu.read_v128(black_box(i % 16)).unwrap();
            }
        })
    });

    group.bench_function("aligned_256bit_access", |b| {
        let vec = Vector256::from_i32x8([1, 2, 3, 4, 5, 6, 7, 8]);
        b.iter(|| {
            for i in 0..16 {
                v_alu.write_v256(black_box(i % 16), black_box(vec)).unwrap();
                let _result = v_alu.read_v256(black_box(i % 16)).unwrap();
            }
        })
    });

    // Test register bank switching performance
    group.bench_function("register_bank_switching", |b| {
        let vec128 = Vector128::from_i32x4([1, 2, 3, 4]);
        let vec256 = Vector256::from_i32x8([1, 2, 3, 4, 5, 6, 7, 8]);
        
        b.iter(|| {
            // Alternate between 128-bit and 256-bit operations
            v_alu.write_v128(black_box(0), black_box(vec128)).unwrap();
            v_alu.write_v256(black_box(0), black_box(vec256)).unwrap();
            let _r128 = v_alu.read_v128(black_box(0)).unwrap();
            let _r256 = v_alu.read_v256(black_box(0)).unwrap();
        })
    });

    group.finish();
}

fn bench_data_type_variations(c: &mut Criterion) {
    let mut group = c.benchmark_group("SIMD_Data_Type_Variations");
    
    let mut v_alu = VectorALU::new();

    // Benchmark different data type operations
    let data_types = [
        ("I32x4", VectorDataType::I32x4, Vector128::from_i32x4([1, 2, 3, 4])),
        ("F32x4", VectorDataType::F32x4, Vector128::from_f32x4([1.0, 2.0, 3.0, 4.0])),
        ("I16x8", VectorDataType::I16x8, Vector128::from_i16x8([1, 2, 3, 4, 5, 6, 7, 8])),
        ("I64x2", VectorDataType::I64x2, Vector128::from_i64x2([100, 200])),
        ("F64x2", VectorDataType::F64x2, Vector128::from_f64x2([1.5, 2.5])),
    ];

    for (name, data_type, vec1) in data_types.iter() {
        v_alu.write_v128(0, *vec1).unwrap();
        v_alu.write_v128(1, *vec1).unwrap();
        
        group.bench_function(&format!("add_{}", name), |b| {
            b.iter(|| {
                v_alu.execute_v128(
                    black_box(VectorOperation::Add),
                    black_box(2),
                    black_box(0),
                    black_box(Some(1)),
                    black_box(*data_type),
                ).unwrap();
            })
        });

        group.bench_function(&format!("mul_{}", name), |b| {
            b.iter(|| {
                v_alu.execute_v128(
                    black_box(VectorOperation::Mul),
                    black_box(2),
                    black_box(0),
                    black_box(Some(1)),
                    black_box(*data_type),
                ).unwrap();
            })
        });
    }

    group.finish();
}

fn bench_load_store_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("SIMD_Load_Store_Operations");
    
    let mut execute_stage = ExecuteStage::new();
    let mut alu = ALU::new();

    // Setup test vectors
    let vec128 = Vector128::from_i32x4([1, 2, 3, 4]);
    let vec256 = Vector256::from_i32x8([1, 2, 3, 4, 5, 6, 7, 8]);
    execute_stage.get_vector_alu_mut().write_v128(0, vec128).unwrap();
    execute_stage.get_vector_alu_mut().write_v256(0, vec256).unwrap();

    // Benchmark Store operations
    group.bench_function("Simd128Store", |b| {
        b.iter(|| {
            let instruction = Instruction::create_reg_imm32(Opcode::Simd128Store, black_box(0), black_box(0x1000));
            
            let de_reg = DecodeExecuteRegister {
                instruction,
                pc: 100,
                rs1: Some(0),
                rs2: None,
                rd: None,
                rs1_value: 0,
                rs2_value: 0,
                immediate: Some(0x1000),
                branch_addr: None,
                branch_prediction: None,
                stack_operation: None,
                mem_addr: Some(0x1000),
                stack_value: None,
            };

            execute_stage.process_direct(&de_reg, &mut alu).unwrap();
        })
    });

    group.bench_function("Simd256Store", |b| {
        b.iter(|| {
            let instruction = Instruction::create_reg_imm32(Opcode::Simd256Store, black_box(0), black_box(0x2000));
            
            let de_reg = DecodeExecuteRegister {
                instruction,
                pc: 100,
                rs1: Some(0),
                rs2: None,
                rd: None,
                rs1_value: 0,
                rs2_value: 0,
                immediate: Some(0x2000),
                branch_addr: None,
                branch_prediction: None,
                stack_operation: None,
                mem_addr: Some(0x2000),
                stack_value: None,
            };

            execute_stage.process_direct(&de_reg, &mut alu).unwrap();
        })
    });

    // Benchmark Load operations  
    group.bench_function("Simd128Load", |b| {
        b.iter(|| {
            let instruction = Instruction::create_reg_imm32(Opcode::Simd128Load, black_box(1), black_box(0x1000));
            
            let de_reg = DecodeExecuteRegister {
                instruction,
                pc: 100,
                rs1: None,
                rs2: None,
                rd: Some(1),
                rs1_value: 0,
                rs2_value: 0,
                immediate: Some(0x1000),
                branch_addr: None,
                branch_prediction: None,
                stack_operation: None,
                mem_addr: Some(0x1000),
                stack_value: None,
            };

            execute_stage.process_direct(&de_reg, &mut alu).unwrap();
        })
    });

    group.bench_function("Simd256Load", |b| {
        b.iter(|| {
            let instruction = Instruction::create_reg_imm32(Opcode::Simd256Load, black_box(1), black_box(0x2000));
            
            let de_reg = DecodeExecuteRegister {
                instruction,
                pc: 100,
                rs1: None,
                rs2: None,
                rd: Some(1),
                rs1_value: 0,
                rs2_value: 0,
                immediate: Some(0x2000),
                branch_addr: None,
                branch_prediction: None,
                stack_operation: None,
                mem_addr: Some(0x2000),
                stack_value: None,
            };

            execute_stage.process_direct(&de_reg, &mut alu).unwrap();
        })
    });

    group.finish();
}

criterion_group!(
    simd_benchmarks,
    bench_extended_vector_types_128,
    bench_extended_vector_types_256,
    bench_simd_const_operations,
    bench_simd_vs_scalar,
    bench_complete_pipeline_simd,
    bench_memory_alignment,
    bench_data_type_variations,
    bench_load_store_operations
);

criterion_main!(simd_benchmarks);
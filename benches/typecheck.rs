// Criterion-based benchmarks for the Fix typechecker.
//
// Three scenarios, each measuring full parse + elaboration + typecheck
// of a Fix program with a *cold* TypeCheckCache (a fresh in-memory
// cache per iteration, so every global value is a cache miss):
//
// - std_only      : minimal Main.fix on top of `std.fix` (~baseline)
// - small_project : Main.fix with ~30 user globals
// - medium_project: Main.fix with ~300 user globals
//
// `OutputFileType::DynamicLibrary` is selected so that the elaboration
// pipeline stops after typecheck (no entry-point or monomorphisation
// step), keeping the measurement focused on the typecheck phase.
//
// Run with `cargo bench`. Filter with e.g. `cargo bench std_only`.

use std::sync::Arc;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};

use fixlang::configuration::{Configuration, OutputFileType, SubCommand};
use fixlang::elaboration::elaborate_via_config;
use fixlang::elaboration::typecheckcache::MemoryCache;
use fixlang::misc::save_temporary_source;

fn build_config(source: &str, name_hint: &str) -> Configuration {
    let saved = match save_temporary_source(source, name_hint) {
        Ok(s) => s,
        Err(e) => panic!("save_temporary_source failed in bench setup: {}", e),
    };
    let mut config = match Configuration::release_mode(SubCommand::Build) {
        Ok(c) => c,
        Err(e) => panic!("Configuration::release_mode failed: {}", e),
    };
    config.output_file_type = OutputFileType::DynamicLibrary;
    config.type_check_cache = Arc::new(MemoryCache::new());
    config.add_user_source_file(saved.file_path);
    config
}

/// Generate a Fix source with `n` chained globals, a handful of trait
/// usages, and an `Array I64` constructed by repeated `push_back`. The
/// shape exercises unify (chained references), trait constraints
/// (`ToString`), and polymorphic instantiation (`Array::push_back`).
fn synthetic_source(n: usize) -> String {
    let mut s = String::new();
    s.push_str("module Main;\n\n");

    // Chain of I64 -> I64 functions, each referencing the previous one.
    // f0 : I64 -> I64 = |x| x + 1;
    // f1 : I64 -> I64 = |x| f0(x) + 2;
    // ...
    s.push_str("f0 : I64 -> I64;\nf0 = |x| x + 1;\n");
    for i in 1..n {
        s.push_str(&format!(
            "f{i} : I64 -> I64;\nf{i} = |x| f{prev}(x) + {step};\n",
            i = i,
            prev = i - 1,
            step = (i as i64 % 7) + 1
        ));
    }

    // A handful of polymorphic helpers that use ToString.
    let g_count = (n / 10).max(2);
    s.push_str(&format!(
        "g0 : [a : ToString] a -> String;\ng0 = |v| v.to_string;\n"
    ));
    for i in 1..g_count {
        s.push_str(&format!(
            "g{i} : [a : ToString] a -> String;\ng{i} = |v| g{prev}(v) + \"!\";\n",
            i = i,
            prev = i - 1
        ));
    }

    // Array construction chain. Forces `Array::push_back` (a method
    // with a polymorphic receiver) to be re-instantiated repeatedly.
    let arr_count = (n / 10).max(2);
    s.push_str("arr0 : Array I64;\narr0 = Array::empty(0);\n");
    for i in 1..arr_count {
        s.push_str(&format!(
            "arr{i} : Array I64;\narr{i} = arr{prev}.push_back({val});\n",
            i = i,
            prev = i - 1,
            val = i
        ));
    }

    // Main references the last element of each chain so they aren't
    // skipped by the compiler.
    s.push_str(&format!(
        "main : IO ();\nmain = (\n    eval f{flast}(0);\n    eval g{glast}(42);\n    eval arr{alast};\n    pure()\n);\n",
        flast = n - 1,
        glast = g_count - 1,
        alast = arr_count - 1
    ));

    s
}

fn bench_std_only(c: &mut Criterion) {
    let source = "module Main;\nmain : IO ();\nmain = pure();\n";
    let mut group = c.benchmark_group("typecheck");
    group.sample_size(20);
    group.bench_function("std_only", |b| {
        b.iter_with_setup(
            || build_config(source, "bench_std_only"),
            |cfg| {
                let _ = elaborate_via_config(&cfg);
            },
        );
    });
    group.finish();
}

fn bench_small_project(c: &mut Criterion) {
    let source = synthetic_source(30);
    let mut group = c.benchmark_group("typecheck");
    group.sample_size(20);
    group.throughput(Throughput::Elements(30));
    group.bench_function("small_project_30gv", |b| {
        b.iter_with_setup(
            || build_config(&source, "bench_small_project"),
            |cfg| {
                let _ = elaborate_via_config(&cfg);
            },
        );
    });
    group.finish();
}

fn bench_medium_project(c: &mut Criterion) {
    let source = synthetic_source(300);
    let mut group = c.benchmark_group("typecheck");
    group.sample_size(10);
    group.throughput(Throughput::Elements(300));
    group.bench_function("medium_project_300gv", |b| {
        b.iter_with_setup(
            || build_config(&source, "bench_medium_project"),
            |cfg| {
                let _ = elaborate_via_config(&cfg);
            },
        );
    });
    group.finish();
}

criterion_group!(benches, bench_std_only, bench_small_project, bench_medium_project);
criterion_main!(benches);

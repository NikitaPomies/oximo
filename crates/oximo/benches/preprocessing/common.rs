use std::hint::black_box;

use criterion::measurement::WallTime;
use criterion::{BenchmarkGroup, BenchmarkId, Throughput};

pub const LARGE: usize = 8192;
pub const SMALL: usize = 32;

/// Return the standard small, production-threshold, and large workloads.
pub fn sizes(threshold: usize) -> [(&'static str, usize); 3] {
    [("small", SMALL), ("threshold", threshold), ("large", LARGE)]
}

/// Report the size of the active Rayon pool in parallel benchmark identifiers.
pub fn threads() -> usize {
    rayon::current_num_threads()
}

/// Register forced-serial and forced-parallel variants over one shared input.
pub fn pair<I, O>(
    group: &mut BenchmarkGroup<'_, WallTime>,
    case: &str,
    rows: usize,
    input: &I,
    operation: impl Fn(&I, bool) -> O,
) {
    group.throughput(Throughput::Elements(rows as u64));
    group.bench_with_input(BenchmarkId::new(case, "serial"), input, |bencher, input| {
        bencher.iter(|| black_box(operation(black_box(input), false)));
    });
    let parallel = format!("parallel-{}t", threads());
    group.bench_with_input(BenchmarkId::new(case, parallel), input, |bencher, input| {
        bencher.iter(|| black_box(operation(black_box(input), true)));
    });
}

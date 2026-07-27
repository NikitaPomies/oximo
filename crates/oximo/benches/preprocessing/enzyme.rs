use criterion::Criterion;

#[cfg(feature = "_benchmark-enzyme")]
use super::common::{pair, sizes};

#[cfg(feature = "_benchmark-enzyme")]
/// Measure Enzyme evaluator classification and parameter-refresh preprocessing.
pub fn bench(criterion: &mut Criterion) {
    use std::hint::black_box;

    use criterion::{BenchmarkId, Throughput};
    use oximo_autodiff::benchmark_support;

    // Measures initial FunctionSlot classification for LP, QCP, and NLP rows.
    let mut classify = criterion.benchmark_group("preprocessing/enzyme_classification");
    for (kind, degree) in [("lp", 1), ("qcp", 2), ("nlp", 3)] {
        for (size, rows) in sizes(benchmark_support::THRESHOLD) {
            let model = benchmark_support::model(rows, degree);
            pair(
                &mut classify,
                &format!("{kind}/{size}/{rows}"),
                rows,
                &model,
                benchmark_support::classify,
            );
        }
    }
    classify.finish();

    // Measures reclassification of existing slots during evaluator refresh.
    let mut refresh = criterion.benchmark_group("preprocessing/enzyme_refresh");
    for (kind, degree) in [("lp", 1), ("qcp", 2), ("nlp", 3)] {
        for (size, rows) in sizes(benchmark_support::THRESHOLD) {
            let model = benchmark_support::model(rows, degree);
            let mut serial = benchmark_support::Refresh::new(&model);
            let mut parallel = benchmark_support::Refresh::new(&model);
            let case = format!("{kind}/{size}/{rows}");
            refresh.throughput(Throughput::Elements(rows as u64));
            refresh.bench_function(BenchmarkId::new(&case, "serial"), |bencher| {
                bencher.iter(|| black_box(serial.run(&model, false)));
            });
            refresh.bench_function(
                BenchmarkId::new(&case, format!("parallel-{}t", rayon::current_num_threads())),
                |bencher| {
                    bencher.iter(|| black_box(parallel.run(&model, true)));
                },
            );
        }
    }
    refresh.finish();
}

#[cfg(not(feature = "_benchmark-enzyme"))]
/// Leave the Enzyme groups absent unless the nightly-only feature is enabled.
pub fn bench(_criterion: &mut Criterion) {}

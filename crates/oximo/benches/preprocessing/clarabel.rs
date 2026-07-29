use criterion::Criterion;
use oximo_clarabel::benchmark_support;

use super::common::{LARGE, crossover_sizes, pair, sizes};

/// Measure Clarabel algebraic-row classification and explicit SOC extraction.
pub fn bench(criterion: &mut Criterion) {
    // Measures ordered linear/detected-SOC classification before matrix assembly.
    let mut rows_group = criterion.benchmark_group("preprocessing/clarabel_rows");
    for (size, rows) in sizes(benchmark_support::ROW_THRESHOLD) {
        let model = benchmark_support::row_model(rows, false);
        pair(&mut rows_group, &format!("lp/{size}/{rows}"), rows, &model, |model, parallel| {
            benchmark_support::classify(model, parallel).unwrap()
        });
    }
    for rows in crossover_sizes().into_iter().chain([LARGE]) {
        let model = benchmark_support::row_model(rows, true);
        pair(
            &mut rows_group,
            &format!("detected_soc/crossover/{rows}"),
            rows,
            &model,
            |model, parallel| benchmark_support::classify(model, parallel).unwrap(),
        );
    }
    rows_group.finish();

    // Measures extraction of already-declared explicit SOC constraints.
    let mut soc_group = criterion.benchmark_group("preprocessing/clarabel_explicit_soc");
    for (size, rows) in sizes(benchmark_support::SOC_THRESHOLD) {
        let model = benchmark_support::explicit_soc_model(rows);
        pair(&mut soc_group, &format!("{size}/{rows}"), rows, &model, |model, parallel| {
            benchmark_support::explicit_socs(model, parallel).unwrap()
        });
    }
    soc_group.finish();

    // Measure complete matrix/cone translation, with model kind cached during
    // Criterion warm-up as it is for persistent rebuilds.
    let mut translation = criterion.benchmark_group("preprocessing/clarabel_translation");
    for (size, rows) in sizes(benchmark_support::ROW_THRESHOLD) {
        let cases = [
            ("lp", benchmark_support::row_model(rows, false)),
            ("qp", benchmark_support::qp_model(rows)),
            ("detected_soc", benchmark_support::row_model(rows, true)),
            ("explicit_soc", benchmark_support::explicit_soc_model(rows)),
        ];
        for (kind, model) in &cases {
            translation.throughput(criterion::Throughput::Elements(rows as u64));
            translation.bench_with_input(
                criterion::BenchmarkId::new(format!("{kind}/{size}/{rows}"), "automatic"),
                model,
                |bencher, model| {
                    bencher.iter(|| {
                        std::hint::black_box(benchmark_support::translate(std::hint::black_box(
                            model,
                        )))
                        .unwrap()
                    });
                },
            );
        }
    }
    translation.finish();
}

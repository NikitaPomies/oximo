use criterion::Criterion;
use oximo_clarabel::benchmark_support;

use super::common::{pair, sizes};

/// Measure Clarabel algebraic-row classification and explicit SOC extraction.
pub fn bench(criterion: &mut Criterion) {
    // Measures ordered linear/detected-SOC classification before matrix assembly.
    let mut rows_group = criterion.benchmark_group("preprocessing/clarabel_rows");
    for (kind, soc) in [("lp", false), ("detected_soc", true)] {
        for (size, rows) in sizes(benchmark_support::ROW_THRESHOLD) {
            let model = benchmark_support::row_model(rows, soc);
            pair(
                &mut rows_group,
                &format!("{kind}/{size}/{rows}"),
                rows,
                &model,
                |model, parallel| benchmark_support::classify(model, parallel).unwrap(),
            );
        }
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
}

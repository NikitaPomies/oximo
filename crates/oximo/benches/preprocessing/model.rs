use criterion::Criterion;
use oximo_core::model::benchmark_support;

use super::common::{pair, sizes};

/// Measure model-kind inference across linear, quadratic, and nonlinear rows.
pub fn bench(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("preprocessing/model_kind");
    for (kind, degree) in [("lp", 1), ("qcp", 2), ("nlp", 3)] {
        for (size, rows) in sizes(benchmark_support::THRESHOLD) {
            let model = benchmark_support::model(rows, degree);
            pair(
                &mut group,
                &format!("{kind}/{size}/{rows}"),
                rows,
                &model,
                benchmark_support::infer,
            );
        }
    }
    group.finish();
}

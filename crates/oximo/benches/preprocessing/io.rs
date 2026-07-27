use criterion::Criterion;
use oximo_io::nl::benchmark_support;

use super::common::{pair, sizes};

/// Measure ordered NL row analysis and row-local variable-set collection.
pub fn bench(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("preprocessing/nl_analysis");
    for (kind, degree) in [("lp", 1), ("qcp", 2), ("nlp", 3)] {
        for (size, rows) in sizes(benchmark_support::THRESHOLD) {
            let model = benchmark_support::model(rows, degree);
            pair(&mut group, &format!("{kind}/{size}/{rows}"), rows, &model, |model, parallel| {
                benchmark_support::analyze(model, parallel).unwrap()
            });
        }
    }
    group.finish();
}

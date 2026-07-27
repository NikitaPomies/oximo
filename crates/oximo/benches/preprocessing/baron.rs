use criterion::Criterion;

use super::common::{pair, sizes};

/// Measure byte-producing equation rendering for BARON.
pub fn bench(criterion: &mut Criterion) {
    // Measures ordered BARON equation fragments, including fallible lowering.
    let mut group = criterion.benchmark_group("preprocessing/baron_rendering");
    for (kind, degree) in [("lp", 1), ("qcp", 2), ("nlp", 3)] {
        for (size, rows) in sizes(oximo_baron::benchmark_support::THRESHOLD) {
            let model = oximo_baron::benchmark_support::model(rows, degree);
            pair(&mut group, &format!("{kind}/{size}/{rows}"), rows, &model, |model, parallel| {
                oximo_baron::benchmark_support::render_equations(model, parallel).unwrap()
            });
        }
    }
    group.finish();
}

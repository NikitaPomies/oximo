use criterion::Criterion;

use super::common::{pair, sizes};

/// Measure byte-producing equation rendering for GAMS.
pub fn bench(criterion: &mut Criterion) {
    // Measures ordered GAMS equation fragments for LP, QCP, and NLP rows.
    let mut group = criterion.benchmark_group("preprocessing/gams_rendering");
    for (kind, degree) in [("lp", 1), ("qcp", 2), ("nlp", 3)] {
        for (size, rows) in sizes(oximo_gams::benchmark_support::THRESHOLD) {
            let model = oximo_gams::benchmark_support::model(rows, degree);
            pair(
                &mut group,
                &format!("{kind}/{size}/{rows}"),
                rows,
                &model,
                oximo_gams::benchmark_support::render_equations,
            );
        }
    }
    group.finish();
}

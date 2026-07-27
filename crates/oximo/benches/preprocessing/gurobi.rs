#[cfg(feature = "_benchmark-proprietary")]
use criterion::Criterion;

#[cfg(feature = "_benchmark-proprietary")]
use super::common::{pair, sizes};

#[cfg(feature = "_benchmark-proprietary")]
/// Measure Gurobi preprocessing before serial FFI uploads.
pub fn bench(criterion: &mut Criterion) {
    // Measures Gurobi's ordered linear-form probe before constraint lowering.
    let mut group = criterion.benchmark_group("preprocessing/gurobi_linear_extraction");
    for (kind, degree) in [("lp", 1), ("qcp", 2), ("nlp", 3)] {
        for (size, rows) in sizes(oximo_gurobi::benchmark_support::THRESHOLD) {
            let model = oximo_gurobi::benchmark_support::model(rows, degree);
            pair(
                &mut group,
                &format!("{kind}/{size}/{rows}"),
                rows,
                &model,
                oximo_gurobi::benchmark_support::extract,
            );
        }
    }
    group.finish();
}

#[cfg(not(feature = "_benchmark-proprietary"))]
/// Leave the Gurobi groups absent unless proprietary benchmarks are enabled.
pub fn bench(_criterion: &mut criterion::Criterion) {}

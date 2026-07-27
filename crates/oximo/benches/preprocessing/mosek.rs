#[cfg(feature = "_benchmark-proprietary")]
use criterion::Criterion;

#[cfg(feature = "_benchmark-proprietary")]
use super::common::{pair, sizes};

#[cfg(feature = "_benchmark-proprietary")]
/// Measure MOSEK preprocessing before serial task uploads.
pub fn bench(criterion: &mut Criterion) {
    // Measures MOSEK algebraic and detected-SOC row preparation.
    let mut rows_group = criterion.benchmark_group("preprocessing/mosek_rows");
    for (kind, degree) in [("lp", 1), ("qcp", 2), ("detected_soc", 3)] {
        for (size, rows) in sizes(oximo_mosek::benchmark_support::ROW_THRESHOLD) {
            let model = oximo_mosek::benchmark_support::row_model(rows, degree);
            pair(
                &mut rows_group,
                &format!("{kind}/{size}/{rows}"),
                rows,
                &model,
                |model, parallel| oximo_mosek::benchmark_support::rows(model, parallel).unwrap(),
            );
        }
    }
    rows_group.finish();

    // Measures MOSEK explicit SOC extraction before sequential task upload.
    let mut soc_group = criterion.benchmark_group("preprocessing/mosek_explicit_soc");
    for (size, rows) in sizes(oximo_mosek::benchmark_support::SOC_THRESHOLD) {
        let model = oximo_mosek::benchmark_support::explicit_soc_model(rows);
        pair(&mut soc_group, &format!("{size}/{rows}"), rows, &model, |model, parallel| {
            oximo_mosek::benchmark_support::explicit_socs(model, parallel).unwrap()
        });
    }
    soc_group.finish();
}

#[cfg(not(feature = "_benchmark-proprietary"))]
/// Leave the MOSEK groups absent unless proprietary benchmarks are enabled.
pub fn bench(_criterion: &mut criterion::Criterion) {}

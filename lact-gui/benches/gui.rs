use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use gtk::glib::{subclass::types::ObjectSubclassIsExt, Object};
use lact_gui::app::{Plot, PlotData};
use plotters::backend::SVGBackend;

pub fn criterion_benchmark(c: &mut Criterion) {
    gtk::init().unwrap();

    let mut plot_data = PlotData::default();
    let mut time = chrono::NaiveDateTime::new(
        chrono::NaiveDate::from_yo_opt(2024, 1).unwrap(),
        chrono::NaiveTime::default(),
    );
    for value in (0..100).step_by(5) {
        plot_data.push_line_series_with_time("value", value as f64, time);
        time += chrono::TimeDelta::seconds(2);
    }

    let plot: Plot = Object::builder().build();
    *plot.data_mut() = plot_data.clone();

    let imp = plot.imp();

    c.bench_function("plot_pdf", |b| {
        b.iter(|| {
            let mut buf = String::new();
            let plotters_backend = SVGBackend::with_string(&mut buf, (1000, 1000));
            imp.plot_pdf(plotters_backend).unwrap();
        })
    });

    c.bench_function("trim_plot_data", |b| {
        b.iter_batched(
            || plot_data.clone(),
            |mut data| {
                data.trim_data(60);
            },
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

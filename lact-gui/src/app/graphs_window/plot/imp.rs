use super::cubic_spline::cubic_spline_interpolation;
use anyhow::Context;
use chrono::TimeDelta;
use glib::Properties;
use gtk::{glib, prelude::*, subclass::prelude::*};
use itertools::Itertools;
use plotters::prelude::*;
use plotters::style::colors::full_palette::DEEPORANGE_100;
use plotters_cairo::CairoBackend;
use serde::Deserialize;
use serde::Serialize;
use std::cell::RefCell;
use std::cmp::max;
use std::collections::BTreeMap;
use tracing::error;

#[derive(Properties, Default)]
#[properties(wrapper_type = super::Plot)]
pub struct Plot {
    #[property(get, set)]
    title: RefCell<String>,
    #[property(get, set)]
    values_json: RefCell<String>,
    #[property(get, set)]
    value_suffix: RefCell<String>,
}

#[glib::object_subclass]
impl ObjectSubclass for Plot {
    const NAME: &'static str = "Plot";
    type Type = super::Plot;
    type ParentType = gtk::Widget;
}

impl ObjectImpl for Plot {
    fn properties() -> &'static [glib::ParamSpec] {
        Self::derived_properties()
    }

    fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
        Self::derived_set_property(self, id, value, pspec);
        self.obj().queue_draw();
    }

    fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        Self::derived_property(self, id, pspec)
    }

    fn constructed(&self) {
        self.parent_constructed();

        let obj = self.obj();
        obj.set_height_request(250);
    }
}

impl WidgetImpl for Plot {
    fn snapshot(&self, snapshot: &gtk::Snapshot) {
        let width = self.obj().width() as u32;
        let height = self.obj().height() as u32;

        if width == 0 || height == 0 {
            return;
        }

        let bounds = gtk::graphene::Rect::new(0.0, 0.0, width as f32, height as f32);
        let cr = snapshot.append_cairo(&bounds);
        // Supersample the plot area
        let backend = CairoBackend::new(&cr, (width * 2, height * 2)).unwrap();
        if let Err(err) = self.plot_pdf(backend) {
            error!("Failed to plot PDF chart: {err:?}")
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct PlotData {
    line_series: BTreeMap<String, BTreeMap<chrono::DateTime<chrono::Local>, f64>>,
    throttling: BTreeMap<chrono::DateTime<chrono::Local>, (String, bool)>,
}

impl PlotData {
    pub fn push_line_series(&mut self, name: &str, point: f64) {
        self.line_series
            .entry(name.to_owned())
            .or_default()
            .insert(chrono::Local::now(), point);
    }

    pub fn push_throttling(&mut self, name: &str, point: bool) {
        self.throttling
            .insert(chrono::Local::now(), (name.to_owned(), point));
    }

    pub fn line_series_iter(
        &self,
    ) -> impl Iterator<Item = (&String, &BTreeMap<chrono::DateTime<chrono::Local>, f64>)> {
        self.line_series.iter()
    }

    pub fn throttling_iter(
        &self,
    ) -> impl Iterator<Item = (chrono::DateTime<chrono::Local>, &str, bool)> {
        self.throttling
            .iter()
            .map(|(time, (name, point))| (*time, name.as_str(), *point))
    }

    pub fn trim_data(&mut self, last_seconds: u64) {
        // Limit data to N seconds
        for data in self.line_series.values_mut() {
            let maximum_point = data
                .last_key_value()
                .map(|(date_time, _)| *date_time)
                .unwrap_or_default();

            data.retain(|time_point, _| {
                ((maximum_point - time_point).num_seconds() as u64) < last_seconds
            });
        }

        self.line_series.retain(|_, data| !data.is_empty());

        // Limit data to N seconds
        let maximum_point = self
            .throttling
            .last_key_value()
            .map(|(date_time, _)| *date_time)
            .unwrap_or_default();

        self.throttling.retain(|time_point, _| {
            ((maximum_point - time_point).num_seconds() as u64) < last_seconds
        });
    }
}

impl Plot {
    fn plot_pdf<'a, DB: DrawingBackend + 'a>(&self, backend: DB) -> anyhow::Result<()>
    where
        <DB as plotters::prelude::DrawingBackend>::ErrorType: 'static,
    {
        let root = backend.into_drawing_area();

        let data: PlotData =
            serde_json::from_str(&self.values_json.borrow()).expect("Failed to parse JSON");

        let start_date = data
            .line_series_iter()
            .filter_map(|(_, data)| Some(data.first_key_value()?.0))
            .min()
            .cloned()
            .unwrap_or_default();
        let end_date = data
            .line_series_iter()
            .map(|(_, value)| value)
            .filter_map(|data| Some(data.last_key_value()?.0))
            .max()
            .cloned()
            .unwrap_or_default();

        let mut maximum_value = data
            .line_series_iter()
            .flat_map(|(_, data)| data.values())
            .max_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal))
            .cloned()
            .unwrap_or_default();

        if maximum_value < 100.0f64 {
            maximum_value = 100.0f64;
        }

        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .x_label_area_size(40)
            .y_label_area_size(80)
            .margin(20)
            .caption(self.title.borrow().as_str(), ("sans-serif", 30))
            .build_cartesian_2d(
                start_date..max(end_date, start_date + TimeDelta::seconds(60)),
                0f64..maximum_value,
            )?;

        chart
            .configure_mesh()
            .x_label_formatter(&|date_time| date_time.format("%H:%M:%S").to_string())
            .y_label_formatter(&|x| format!("{x}{}", self.value_suffix.borrow()))
            .x_labels(5)
            .y_labels(10)
            .label_style(("sans-serif", 30))
            .draw()
            .context("Failed to draw mesh")?;

        // Draw the throttling histogram
        chart
            .draw_series(
                data.throttling_iter()
                    // Group segments of consecutive enabled/disabled throttlings
                    .group_by(|(_, _, point)| *point)
                    .into_iter()
                    // Filter only when throttling is enabled
                    .filter_map(|(point, group_iter)| point.then_some(group_iter))
                    // Get last and first times
                    .filter_map(|mut group_iter| {
                        let first = group_iter.next()?;
                        Some((first, group_iter.last().unwrap_or(first)))
                    })
                    // Filter out redundant data
                    .map(|((start, name, _), (end, _, _))| ((start, end), name))
                    .map(|((start_time, end_time), _)| {
                        let mut bar = Rectangle::new(
                            [(start_time, 0f64), (end_time, maximum_value)],
                            DEEPORANGE_100.filled(),
                        );
                        bar.set_margin(0, 0, 5, 5);
                        bar
                    }),
            )
            .context("Failed to draw throttling histogram")?;

        for (idx, (caption, data)) in (0..).zip(data.line_series_iter()) {
            chart
                .draw_series(LineSeries::new(
                    cubic_spline_interpolation(data.iter())
                        .into_iter()
                        .flat_map(|((first_time, second_time), segment)| {
                            let mut current_date = first_time;

                            let mut result = vec![];
                            while current_date < second_time {
                                result.push((current_date, segment.evaluate(&current_date)));

                                // Interpolate in intervals of one millisecond
                                current_date += TimeDelta::milliseconds(1);
                            }

                            result
                        }),
                    Palette99::pick(idx).stroke_width(1),
                ))
                .context("Failed to draw series")?
                .label(caption)
                .legend(move |(x, y)| {
                    Rectangle::new([(x - 10, y - 10), (x + 10, y + 10)], Palette99::pick(idx))
                });
        }

        chart
            .configure_series_labels()
            .margin(30)
            .label_font(("sans-serif", 30))
            .position(SeriesLabelPosition::LowerRight)
            .background_style(WHITE.mix(0.8))
            .border_style(BLACK)
            .draw()
            .context("Failed to draw series labels")?;

        root.present()?;
        Ok(())
    }
}

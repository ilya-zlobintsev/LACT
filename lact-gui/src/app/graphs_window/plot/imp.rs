use super::cubic_spline::cubic_spline_interpolation;
use anyhow::Context;
use chrono::NaiveDateTime;
use glib::Properties;
use gtk::{glib, prelude::*, subclass::prelude::*};
use itertools::Itertools;
use plotters::prelude::*;
use plotters::style::colors::full_palette::DEEPORANGE_100;
use plotters_cairo::CairoBackend;
use std::cell::Cell;
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
    value_suffix: RefCell<String>,
    #[property(get, set)]
    secondary_value_suffix: RefCell<String>,
    #[property(get, set)]
    y_label_area_size: Cell<u32>,
    #[property(get, set)]
    secondary_y_label_area_size: Cell<u32>,
    pub(super) data: RefCell<PlotData>,
}

#[glib::object_subclass]
impl ObjectSubclass for Plot {
    const NAME: &'static str = "Plot";
    type Type = super::Plot;
    type ParentType = gtk::Widget;
}

#[glib::derived_properties]
impl ObjectImpl for Plot {
    fn constructed(&self) {
        self.parent_constructed();

        let obj = self.obj();
        obj.set_height_request(250);
        obj.set_hexpand(true);
        obj.set_vexpand(true);
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

#[derive(Default)]
#[cfg_attr(feature = "bench", derive(Clone))]
pub struct PlotData {
    line_series: BTreeMap<String, Vec<(i64, f64)>>,
    secondary_line_series: BTreeMap<String, Vec<(i64, f64)>>,
    throttling: Vec<(i64, (String, bool))>,
}

impl PlotData {
    pub fn push_line_series(&mut self, name: &str, point: f64) {
        self.push_line_series_with_time(name, point, chrono::Local::now().naive_local());
    }

    pub fn push_secondary_line_series(&mut self, name: &str, point: f64) {
        self.push_secondary_line_series_with_time(name, point, chrono::Local::now().naive_local());
    }

    pub fn push_line_series_with_time(&mut self, name: &str, point: f64, time: NaiveDateTime) {
        self.line_series
            .entry(name.to_owned())
            .or_default()
            .push((time.and_utc().timestamp_millis(), point));
    }

    pub fn push_secondary_line_series_with_time(
        &mut self,
        name: &str,
        point: f64,
        time: NaiveDateTime,
    ) {
        self.secondary_line_series
            .entry(name.to_owned())
            .or_default()
            .push((time.and_utc().timestamp_millis(), point));
    }

    pub fn push_throttling(&mut self, name: &str, point: bool) {
        self.throttling.push((
            chrono::Local::now()
                .naive_local()
                .and_utc()
                .timestamp_millis(),
            (name.to_owned(), point),
        ));
    }

    pub fn line_series_iter(&self) -> impl Iterator<Item = (&String, &Vec<(i64, f64)>)> {
        self.line_series.iter()
    }

    pub fn secondary_line_series_iter(&self) -> impl Iterator<Item = (&String, &Vec<(i64, f64)>)> {
        self.secondary_line_series.iter()
    }

    pub fn throttling_iter(&self) -> impl Iterator<Item = (i64, &str, bool)> {
        self.throttling
            .iter()
            .map(|(time, (name, point))| (*time, name.as_str(), *point))
    }

    pub fn trim_data(&mut self, last_seconds: i64) {
        // Limit data to N seconds
        for data in self.line_series.values_mut() {
            let maximum_point = data
                .last()
                .map(|(date_time, _)| *date_time)
                .unwrap_or_default();

            data.retain(|(time_point, _)| ((maximum_point - *time_point) / 1000) < last_seconds);
        }

        self.line_series.retain(|_, data| !data.is_empty());

        for data in self.secondary_line_series.values_mut() {
            let maximum_point = data
                .last()
                .map(|(date_time, _)| *date_time)
                .unwrap_or_default();

            data.retain(|(time_point, _)| ((maximum_point - *time_point) / 1000) < last_seconds);
        }

        self.secondary_line_series
            .retain(|_, data| !data.is_empty());

        // Limit data to N seconds
        let maximum_point = self
            .throttling
            .last()
            .map(|(date_time, _)| *date_time)
            .unwrap_or_default();

        self.throttling
            .retain(|(time_point, _)| ((maximum_point - *time_point) / 1000) < last_seconds);
    }
}

impl Plot {
    pub fn plot_pdf<'a, DB>(&self, backend: DB) -> anyhow::Result<()>
    where
        DB: DrawingBackend + 'a,
        <DB as plotters::prelude::DrawingBackend>::ErrorType: 'static,
    {
        let root = backend.into_drawing_area();

        let data = self.data.borrow();

        let start_date = data
            .line_series_iter()
            .filter_map(|(_, data)| Some(data.first()?.0))
            .min()
            .unwrap_or_default();
        let end_date = data
            .line_series_iter()
            .map(|(_, value)| value)
            .filter_map(|data| Some(data.first()?.0))
            .max()
            .unwrap_or_default();

        let mut maximum_value = data
            .line_series_iter()
            .flat_map(|(_, data)| data.iter().map(|(_, value)| value))
            .max_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal))
            .cloned()
            .unwrap_or_default();

        if maximum_value < 100.0f64 {
            maximum_value = 100.0f64;
        }

        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .x_label_area_size(40)
            .y_label_area_size(self.y_label_area_size.get())
            .right_y_label_area_size(self.secondary_y_label_area_size.get())
            .margin(20)
            .caption(self.title.borrow().as_str(), ("sans-serif", 30))
            .build_cartesian_2d(
                start_date..max(end_date, start_date + 60 * 1000),
                0f64..maximum_value,
            )?
            .set_secondary_coord(
                start_date..max(end_date, start_date + 60 * 1000),
                0.0..100.0,
            );

        chart
            .configure_mesh()
            .x_label_formatter(&|date_time| {
                let date_time = chrono::DateTime::from_timestamp_millis(*date_time).unwrap();
                date_time.format("%H:%M:%S").to_string()
            })
            .y_label_formatter(&|x| format!("{x}{}", self.value_suffix.borrow()))
            .x_labels(5)
            .y_labels(10)
            .label_style(("sans-serif", 30))
            .draw()
            .context("Failed to draw mesh")?;

        chart
            .configure_secondary_axes()
            .y_label_formatter(&|x| format!("{x}{}", self.secondary_value_suffix.borrow()))
            .y_labels(10)
            .label_style(("sans-serif", 30))
            .draw()
            .context("Failed to draw mesh")?;

        // Draw the throttling histogram
        chart
            .draw_series(
                data.throttling_iter()
                    // Group segments of consecutive enabled/disabled throttlings
                    .chunk_by(|(_, _, point)| *point)
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
                            // Interpolate in intervals of one millisecond
                            (first_time..second_time).map(move |current_date| {
                                (current_date, segment.evaluate(current_date))
                            })
                        }),
                    Palette99::pick(idx).stroke_width(1),
                ))
                .context("Failed to draw series")?
                .label(caption)
                .legend(move |(x, y)| {
                    Rectangle::new([(x - 10, y - 10), (x + 10, y + 10)], Palette99::pick(idx))
                });
        }

        for (idx, (caption, data)) in (0..).zip(data.secondary_line_series_iter()) {
            chart
                .draw_secondary_series(LineSeries::new(
                    cubic_spline_interpolation(data.iter())
                        .into_iter()
                        .flat_map(|((first_time, second_time), segment)| {
                            // Interpolate in intervals of one millisecond
                            (first_time..second_time).map(move |current_date| {
                                (current_date, segment.evaluate(current_date))
                            })
                        }),
                    Palette99::pick(idx + 10).stroke_width(1), // Offset the pallete pick compared to the main graph
                ))
                .context("Failed to draw series")?
                .label(caption)
                .legend(move |(x, y)| {
                    Rectangle::new(
                        [(x - 10, y - 10), (x + 10, y + 10)],
                        Palette99::pick(idx + 10),
                    )
                });
        }

        chart
            .configure_series_labels()
            .margin(40)
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

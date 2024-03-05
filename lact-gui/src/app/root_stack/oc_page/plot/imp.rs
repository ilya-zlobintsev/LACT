use anyhow::Context;
use glib::Properties;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use std::cell::RefCell;
// use std::collections::btree_map::{Iter as BTreeMapIter, IterMut as BTreeMapIterMut};

use std::collections::BTreeMap;

use plotters::prelude::*;
use plotters_cairo::CairoBackend;
use tracing::error;

use chrono::TimeDelta;
use std::cmp::max;

#[derive(Default, Properties)]
#[properties(wrapper_type = super::Plot)]
pub struct Plot {
    #[property(get, set)]
    values_json: RefCell<String>,
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
        let backend = CairoBackend::new(&cr, (width, height)).unwrap();
        if let Err(err) = self.plot_pdf(backend) {
            error!("Failed to plot PDF chart: {err:?}")
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct PlotData {
    line_series: BTreeMap<String, BTreeMap<chrono::DateTime<chrono::Local>, f64>>,
}

impl PlotData {
    pub fn push_line_series(&mut self, name: &str, point: f64) {
        self.line_series
            .entry(name.to_owned())
            .or_default()
            .insert(chrono::Local::now(), point);
    }

    pub fn linear_iter(
        &self,
    ) -> impl Iterator<Item = (&String, &BTreeMap<chrono::DateTime<chrono::Local>, f64>)> {
        self.line_series.iter()
    }

    pub fn linear_iter_mut(
        &mut self,
    ) -> impl Iterator<Item = (&String, &mut BTreeMap<chrono::DateTime<chrono::Local>, f64>)> {
        self.line_series.iter_mut()
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
            .linear_iter()
            .filter_map(|(_, data)| Some(data.first_key_value()?.0))
            .min()
            .cloned()
            .unwrap_or_default();
        let end_date = data
            .linear_iter()
            .map(|(_, value)| value)
            .filter_map(|data| Some(data.last_key_value()?.0))
            .max()
            .cloned()
            .unwrap_or_default();

        let maximum_value = data
            .linear_iter()
            .flat_map(|(_, data)| data.values())
            .max_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal))
            .cloned()
            .unwrap_or_default();

        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .x_label_area_size(20)
            .y_label_area_size(30)
            .margin(10)
            .build_cartesian_2d(
                start_date..max(end_date, start_date + TimeDelta::seconds(60)),
                if maximum_value > 100.0f64 {
                    0f64..maximum_value
                } else {
                    0f64..100.0f64
                },
            )?;

        chart
            .configure_mesh()
            .x_label_formatter(&|date_time| date_time.format("%H:%M:%S").to_string())
            .x_labels(5)
            .y_labels(5)
            .draw()
            .context("Failed to draw mesh")?;

        for (idx, (caption, data)) in (0..).zip(data.linear_iter()) {
            chart
                .draw_series(LineSeries::new(
                    data.iter().map(|(a, b)| (*a, *b)),
                    &Palette99::pick(idx),
                ))
                .context("Failed to draw series")?
                .label(caption)
                .legend(move |(x, y)| {
                    Rectangle::new([(x - 2, y - 2), (x, y + 2)], Palette99::pick(idx))
                });
        }

        chart
            .configure_series_labels()
            .margin(20)
            .legend_area_size(30)
            .position(SeriesLabelPosition::LowerLeft)
            .background_style(WHITE.mix(0.8))
            .border_style(BLACK)
            .draw()
            .context("Failed to draw series labels")?;

        root.present()?;
        Ok(())
    }
}

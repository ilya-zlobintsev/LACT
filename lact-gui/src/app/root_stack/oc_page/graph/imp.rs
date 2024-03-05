use anyhow::Context;
use glib::Properties;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell::RefCell;
use std::collections::BTreeMap;

use plotters::prelude::*;
use plotters_cairo::CairoBackend;
use tracing::error;

#[derive(Default, Properties)]
#[properties(wrapper_type = super::Graph)]
pub struct Graph {
    #[property(get, set)]
    values_json: RefCell<String>,
}

#[glib::object_subclass]
impl ObjectSubclass for Graph {
    const NAME: &'static str = "Graph";
    type Type = super::Graph;
    type ParentType = gtk::Widget;
}

impl ObjectImpl for Graph {
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

impl WidgetImpl for Graph {
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

pub type GraphData = BTreeMap<String, BTreeMap<chrono::DateTime<chrono::Local>, f64>>;

impl Graph {
    fn plot_pdf<'a, DB: DrawingBackend + 'a>(&self, backend: DB) -> anyhow::Result<()>
    where
        <DB as plotters::prelude::DrawingBackend>::ErrorType: 'static,
    {
        let root = backend.into_drawing_area();

        let data: GraphData =
            serde_json::from_str(&self.values_json.borrow()).expect("Failed to parse JSON");

        let start_date = data
            .iter()
            .filter_map(|(_, data)| Some(data.first_key_value()?.0))
            .min()
            .cloned()
            .unwrap_or_default();
        let end_date = data
            .iter()
            .filter_map(|(_, data)| Some(data.last_key_value()?.0))
            .max()
            .cloned()
            .unwrap_or_default();

        let maximum_value = data
            .values()
            .flat_map(|data| data.values())
            .max_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal))
            .cloned()
            .unwrap_or_default();

        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .x_label_area_size(20)
            .y_label_area_size(30)
            .margin(10)
            .build_cartesian_2d(start_date..end_date, 0f64..maximum_value)?;

        chart
            .configure_mesh()
            .x_label_formatter(&|date_time| date_time.format("%H:%M:%S").to_string())
            .x_labels(5)
            .y_labels(5)
            .draw()
            .context("Failed to draw mesh")?;

        for (idx, (caption, data)) in (0..).zip(data.iter()) {
            chart
                .draw_series(LineSeries::new(
                    data.iter().map(|(a, b)| (*a, *b)),
                    &Palette99::pick(idx),
                ))
                .context("Failed to draw series")?
                .label(caption)
                .legend(move |(x, y)| {
                    Rectangle::new([(x - 5, y - 5), (x + 5, y + 5)], Palette99::pick(idx))
                });
        }

        chart
            .configure_series_labels()
            .margin(20)
            .legend_area_size(30)
            .background_style(WHITE.mix(0.8))
            .border_style(BLACK)
            .draw()
            .context("Failed to draw series labels")?;

        root.present()?;
        Ok(())
    }
}

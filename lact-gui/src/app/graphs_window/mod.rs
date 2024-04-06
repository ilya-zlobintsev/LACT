mod plot;

use self::plot::PlotData;
use glib::Object;
use gtk::glib;
use lact_client::schema::DeviceStats;

const GRAPH_WIDTH_SECONDS: u64 = 60;

glib::wrapper! {
    pub struct GraphsWindow(ObjectSubclass<imp::GraphsWindow>)
        @extends gtk::Box, gtk::Widget, gtk::Window,
        @implements gtk::Orientable, gtk::Accessible, gtk::Buildable;
}

impl GraphsWindow {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn set_stats(&self, stats: &DeviceStats) {
        let mut temperature_plot = self.temperature_plot_values();
        let mut clockspeed_plot = self.clockspeed_plot_values();

        let throttling_plots = [&mut temperature_plot, &mut clockspeed_plot];
        match &stats.throttle_info {
            Some(throttle_info) => {
                if throttle_info.is_empty() {
                    for plot in throttling_plots {
                        plot.push_throttling("No", false);
                    }
                } else {
                    let type_text: Vec<String> = throttle_info
                        .iter()
                        .map(|(throttle_type, details)| {
                            format!("{throttle_type} ({})", details.join(", "))
                        })
                        .collect();

                    let text = type_text.join(", ");

                    for plot in throttling_plots {
                        plot.push_throttling(&text, true);
                    }
                }
            }
            None => {
                for plot in throttling_plots {
                    plot.push_throttling("Unknown", false);
                }
            }
        }

        for (name, value) in &stats.temps {
            temperature_plot.push_line_series(name, value.current.unwrap_or(0.0) as f64);
        }

        temperature_plot.trim_data(GRAPH_WIDTH_SECONDS);

        if let Some(point) = stats.clockspeed.gpu_clockspeed {
            clockspeed_plot.push_line_series("GPU (Avg)", point as f64);
        }
        if let Some(point) = stats.clockspeed.current_gfxclk {
            clockspeed_plot.push_line_series("GPU (Trgt)", point as f64);
        }
        if let Some(point) = stats.clockspeed.vram_clockspeed {
            clockspeed_plot.push_line_series("VRAM", point as f64);
        }

        self.set_temperature_plot_values(&temperature_plot);
        self.set_clockspeed_plot_values(&clockspeed_plot);
    }

    pub fn clear(&self) {
        self.set_temperature_plot_values(&PlotData::default());
        self.set_clockspeed_plot_values(&PlotData::default());
    }

    // TODO: Figure out better way to send data to plot widget
    fn set_temperature_plot_values(&self, value: &PlotData) {
        self.set_temperature_plot_values_json(serde_json::to_string(value).unwrap());
    }

    fn set_clockspeed_plot_values(&self, value: &PlotData) {
        self.set_clockspeed_plot_values_json(serde_json::to_string(value).unwrap());
    }

    fn temperature_plot_values(&self) -> PlotData {
        serde_json::from_str(&self.temperature_plot_values_json()).unwrap_or_default()
    }

    fn clockspeed_plot_values(&self) -> PlotData {
        serde_json::from_str(&self.clockspeed_plot_values_json()).unwrap_or_default()
    }
}

impl Default for GraphsWindow {
    fn default() -> Self {
        Self::new()
    }
}

mod imp {
    use super::plot::Plot;
    use glib::Properties;
    use gtk::{
        glib::{self, subclass::InitializingObject},
        prelude::*,
        subclass::{
            prelude::*,
            widget::{CompositeTemplateClass, WidgetImpl},
        },
        CompositeTemplate,
    };
    use std::cell::RefCell;

    #[derive(CompositeTemplate, Default, Properties)]
    #[properties(wrapper_type = super::GraphsWindow)]
    #[template(file = "ui/graphs_window.blp")]
    pub struct GraphsWindow {
        #[template_child]
        temperature_plot: TemplateChild<Plot>,
        #[template_child]
        clockspeed_plot: TemplateChild<Plot>,

        #[property(get, set)]
        temperature_plot_values_json: RefCell<String>,
        #[property(get, set)]
        clockspeed_plot_values_json: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GraphsWindow {
        const NAME: &'static str = "GraphsWindow";
        type Type = super::GraphsWindow;
        type ParentType = gtk::Window;

        fn class_init(class: &mut Self::Class) {
            Plot::ensure_type();

            class.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for GraphsWindow {}

    impl WidgetImpl for GraphsWindow {}
    impl WindowImpl for GraphsWindow {}
    impl ApplicationWindowImpl for GraphsWindow {}
}

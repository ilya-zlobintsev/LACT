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
        let mut plot = self.plot_values();

        match &stats.throttle_info {
            Some(throttle_info) => {
                if throttle_info.is_empty() {
                    plot.push_throttling("No", false);
                } else {
                    let type_text: Vec<String> = throttle_info
                        .iter()
                        .map(|(throttle_type, details)| {
                            format!("{throttle_type} ({})", details.join(", "))
                        })
                        .collect();

                    let text = type_text.join(", ");
                    plot.push_throttling(&text, true);
                }
            }
            None => {
                plot.push_throttling("Unknown", false);
            }
        }

        for (name, value) in &stats.temps {
            plot.push_line_series(name, value.current.unwrap_or(0.0) as f64);
        }

        // plot.push_line_series("GPU Usage", stats.busy_percent.unwrap_or_default() as f64);
        plot.trim_data(GRAPH_WIDTH_SECONDS);

        self.set_plot_values(&plot);
    }

    pub fn clear(&self) {
        self.set_plot_values(&PlotData::default());
    }

    // TODO: Figure out better way to send data to plot widget
    fn set_plot_values(&self, value: &PlotData) {
        self.set_plot_values_json(serde_json::to_string(value).unwrap());
    }

    fn plot_values(&self) -> PlotData {
        serde_json::from_str(&self.plot_values_json()).unwrap_or_default()
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
        plot: TemplateChild<Plot>,

        #[property(get, set)]
        plot_values_json: RefCell<String>,
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

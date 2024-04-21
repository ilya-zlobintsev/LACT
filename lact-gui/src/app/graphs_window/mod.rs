mod plot;

use self::plot::PlotData;
use glib::Object;
use gtk::{
    glib::{self, subclass::types::ObjectSubclassIsExt},
    prelude::WidgetExt,
};
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
        let imp = self.imp();

        let mut temperature_plot = imp.temperature_plot.data_mut();
        let mut clockspeed_plot = imp.clockspeed_plot.data_mut();
        let mut power_plot = imp.power_plot.data_mut();

        let throttling_plots = [&mut temperature_plot, &mut clockspeed_plot, &mut power_plot];
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

        if let Some(average) = stats.power.average {
            power_plot.push_line_series("Average", average);
        }
        if let Some(current) = stats.power.current {
            power_plot.push_line_series("Current", current);
        }
        if let Some(limit) = stats.power.cap_current {
            power_plot.push_line_series("Limit", limit);
        }

        if let Some(point) = stats.clockspeed.gpu_clockspeed {
            clockspeed_plot.push_line_series("GPU (Avg)", point as f64);
        }
        if let Some(point) = stats.clockspeed.current_gfxclk {
            clockspeed_plot.push_line_series("GPU (Trgt)", point as f64);
        }
        if let Some(point) = stats.clockspeed.vram_clockspeed {
            clockspeed_plot.push_line_series("VRAM", point as f64);
        }

        temperature_plot.trim_data(GRAPH_WIDTH_SECONDS);
        clockspeed_plot.trim_data(GRAPH_WIDTH_SECONDS);
        power_plot.trim_data(GRAPH_WIDTH_SECONDS);

        imp.temperature_plot.queue_draw();
        imp.clockspeed_plot.queue_draw();
        imp.power_plot.queue_draw();
    }

    pub fn clear(&self) {
        let imp = self.imp();
        *imp.temperature_plot.data_mut() = PlotData::default();
        *imp.clockspeed_plot.data_mut() = PlotData::default();
        imp.temperature_plot.queue_draw();
        imp.clockspeed_plot.queue_draw();
    }
}

impl Default for GraphsWindow {
    fn default() -> Self {
        Self::new()
    }
}

mod imp {
    use super::plot::Plot;
    use gtk::{
        glib::{self, subclass::InitializingObject},
        prelude::*,
        subclass::{
            prelude::*,
            widget::{CompositeTemplateClass, WidgetImpl},
        },
        CompositeTemplate,
    };

    #[derive(CompositeTemplate, Default)]
    #[template(file = "ui/graphs_window.blp")]
    pub struct GraphsWindow {
        #[template_child]
        pub(super) temperature_plot: TemplateChild<Plot>,
        #[template_child]
        pub(super) clockspeed_plot: TemplateChild<Plot>,
        #[template_child]
        pub(super) power_plot: TemplateChild<Plot>,
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

    impl ObjectImpl for GraphsWindow {}

    impl WidgetImpl for GraphsWindow {}
    impl WindowImpl for GraphsWindow {}
    impl ApplicationWindowImpl for GraphsWindow {}
}

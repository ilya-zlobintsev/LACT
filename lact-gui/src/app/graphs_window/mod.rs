mod plot;
mod plot_component;
mod stat;

use gtk::{glib, prelude::*};
use lact_schema::DeviceStats;
use plot_component::{PlotComponent, PlotComponentMsg};
use relm4::{
    binding::{BoolBinding, ConnectBinding, F64Binding},
    prelude::{DynamicIndex, FactoryVecDeque},
    ComponentParts, ComponentSender, RelmWidgetExt,
};
use stat::{StatType, StatsData};
use std::sync::{Arc, RwLock};

pub struct GraphsWindow {
    time_period_seconds_adj: gtk::Adjustment,
    edit_mode: BoolBinding,
    plots_per_row: F64Binding,
    vram_clock_ratio: f64,
    plots: FactoryVecDeque<PlotComponent>,
    stats_data: Arc<RwLock<StatsData>>,
}

#[derive(Debug)]
pub enum GraphsWindowMsg {
    Stats(Arc<DeviceStats>),
    VramClockRatio(f64),
    NotifyEditing,
    NotifyPlotsPerRow,
    SwapPlots(DynamicIndex, DynamicIndex),
    Show,
    Clear,
}

#[relm4::component(pub)]
impl relm4::Component for GraphsWindow {
    type Init = ();
    type Input = GraphsWindowMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk::Window {
            set_default_height: 700,
            set_default_width: 1200,
            set_title: Some("Historical data"),
            set_hide_on_close: true,

            gtk::ScrolledWindow {
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 5,
                    set_margin_all: 10,

                    append = model.plots.widget() {
                        set_margin_all: 10,
                        set_row_spacing: 20,
                        set_column_spacing: 20,
                    },

                    append = &gtk::Box {
                        set_halign: gtk::Align::End,
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 5,


                        append = &gtk::Box {
                            set_halign: gtk::Align::End,
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 5,
                            #[watch]
                            set_visible: model.edit_mode.value(),

                            append = &gtk::Label {
                                set_label: "Graphs per row:",
                            },

                            append = &gtk::SpinButton {
                                set_numeric: true,
                                set_snap_to_ticks: true,
                                set_range: (1.0, 5.0),
                                set_increments: (1.0, 1.0),
                                bind: &model.plots_per_row,

                                connect_value_notify => GraphsWindowMsg::NotifyPlotsPerRow,
                            },
                        },

                        append = &gtk::ToggleButton {
                            set_label: "Edit",
                            bind: &model.edit_mode,

                            connect_active_notify => GraphsWindowMsg::NotifyEditing,
                        },
                    }
                },
            },

            /*gtk::Grid {
                set_margin_all: 10,
                set_row_spacing: 20,
                set_column_spacing: 20,
                set_column_homogeneous: true,

                attach[0, 0, 1, 1]: temperature_plot = &Plot {
                    set_title: "Temperature",
                    set_hexpand: true,
                    set_value_suffix: "°C",
                    set_y_label_area_size: 60,
                    #[watch]
                    set_time_period_seconds: model.time_period_seconds_adj.value() as i64,

                    set_data: model.stats_data.clone(),
                },

                attach[0, 1, 1, 1]: fan_plot = &Plot {
                    set_title: "Fan speed",
                    set_hexpand: true,
                    set_value_suffix: "RPM",
                    set_secondary_value_suffix: "%",
                    set_y_label_area_size: 90,
                    set_secondary_y_label_area_size: 60,
                    #[watch]
                    set_time_period_seconds: model.time_period_seconds_adj.value() as i64,

                    set_config: PlotConfig {
                        left_stats: vec![StatType::FanRpm],
                        right_stats: vec![StatType::FanPwm],
                    },

                    set_data: model.stats_data.clone(),
                },

                attach[1, 0, 1, 1]: clockspeed_plot = &Plot {
                    set_title: "Clockspeed",
                    set_hexpand: true,
                    set_value_suffix: "MHz",
                    set_y_label_area_size: 95,
                    #[watch]
                    set_time_period_seconds: model.time_period_seconds_adj.value() as i64,

                    set_data: model.stats_data.clone(),
                },

                attach[1, 1, 1, 1]: power_plot = &Plot {
                    set_title: "Power usage",
                    set_hexpand: true,
                    set_value_suffix: "W",
                    set_y_label_area_size: 65,
                    #[watch]
                    set_time_period_seconds: model.time_period_seconds_adj.value() as i64,

                    set_data: model.stats_data.clone(),
                },

                attach[1, 2, 1, 1] = &gtk::Box {
                    set_halign: gtk::Align::End,
                    set_spacing: 5,

                    gtk::Label {
                        set_label: "Time period (seconds):"
                    },

                    gtk::SpinButton {
                        set_adjustment: &model.time_period_seconds_adj,
                    },
                },
            },*/

        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let time_period_seconds_adj = gtk::Adjustment::new(60.0, 15.0, 3601.0, 1.0, 1.0, 1.0);

        let stats_data = Arc::new(RwLock::new(StatsData::default()));
        let plots_per_row = F64Binding::new(2.0);
        let mut plots = FactoryVecDeque::builder()
            .launch_default()
            .forward(sender.input_sender(), |x| x);

        let edit_mode = BoolBinding::new(false);

        plots.guard().push_back(PlotComponent {
            plots_per_row: plots_per_row.clone(),
            stats: vec![StatType::GpuClock, StatType::VramClock],
            data: stats_data.clone(),
            edit_mode: edit_mode.clone(),
        });

        plots.guard().push_back(PlotComponent {
            plots_per_row: plots_per_row.clone(),
            stats: vec![StatType::PowerCurrent, StatType::PowerCap],
            data: stats_data.clone(),
            edit_mode: edit_mode.clone(),
        });

        plots.guard().push_back(PlotComponent {
            plots_per_row: plots_per_row.clone(),
            stats: vec![StatType::FanRpm, StatType::FanPwm],
            data: stats_data.clone(),
            edit_mode: edit_mode.clone(),
        });

        let model = Self {
            time_period_seconds_adj,
            plots_per_row,
            edit_mode,
            plots,
            vram_clock_ratio: 1.0,
            stats_data,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::Input,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match msg {
            GraphsWindowMsg::Show => {
                root.show();
            }
            GraphsWindowMsg::VramClockRatio(ratio) => {
                self.vram_clock_ratio = ratio;
            }
            GraphsWindowMsg::Stats(stats) => {
                /*let mut temperature_plot = widgets.temperature_plot.data_mut();
                let mut clockspeed_plot = widgets.clockspeed_plot.data_mut();
                let mut power_plot = widgets.power_plot.data_mut();
                let mut fan_plot = widgets.fan_plot.data_mut();

                let throttling_plots =
                    [&mut temperature_plot, &mut clockspeed_plot, &mut power_plot];
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
                    clockspeed_plot.push_line_series("VRAM", point as f64 * self.vram_clock_ratio);
                }

                if let Some(max_speed) = stats.fan.speed_max {
                    fan_plot.push_line_series("Maximum", max_speed as f64);
                }
                if let Some(min_speed) = stats.fan.speed_min {
                    fan_plot.push_line_series("Minimum", min_speed as f64);
                }

                if let Some(current_speed) = stats.fan.speed_current {
                    fan_plot.push_line_series("Current", current_speed as f64);
                }

                if let Some(pwm) = stats.fan.pwm_current {
                    fan_plot.push_secondary_line_series(
                        "Percentage",
                        (pwm as f64 / u8::MAX as f64) * 100.0,
                    );
                }*/

                let mut data = self.stats_data.write().unwrap();
                data.update(&stats);

                let time_period_seconds = self.time_period_seconds_adj.value() as i64;
                data.trim(time_period_seconds);
            }
            GraphsWindowMsg::Clear => {
                self.stats_data.write().unwrap().clear();
            }
            GraphsWindowMsg::NotifyEditing => {
                self.plots.broadcast(PlotComponentMsg::Redraw);
            }
            GraphsWindowMsg::NotifyPlotsPerRow => {
                self.update_plots_layout();
            }
            GraphsWindowMsg::SwapPlots(left, right) => {
                let mut guard = self.plots.guard();
                guard.swap(left.current_index(), right.current_index());
            }
        }

        self.update_view(widgets, sender);
        self.queue_plots_draw();
    }
}

impl GraphsWindow {
    fn update_plots_layout(&mut self) {
        let mut guard = self.plots.guard();

        // This is an ugly workaround because it's not possible to manually trigger a re-layout of the grid factory

        let mut plots = Vec::with_capacity(guard.len());

        while let Some(element) = guard.pop_front() {
            plots.push(element);
        }

        for plot in plots {
            guard.push_back(plot);
        }
    }

    fn queue_plots_draw(&self) {
        self.plots.broadcast(PlotComponentMsg::Redraw);
    }
}

#[derive(Clone, glib::Boxed)]
#[boxed_type(name = "DynamicIndexValue")]
pub struct DynamicIndexValue(DynamicIndex);

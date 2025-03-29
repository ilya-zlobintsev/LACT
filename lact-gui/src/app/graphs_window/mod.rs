mod plot;
mod plot_component;
mod stat;

use gtk::{glib, prelude::*};
use lact_schema::DeviceStats;
use plot_component::{PlotComponent, PlotComponentConfig, PlotComponentMsg};
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
    Stats {
        stats: Arc<DeviceStats>,
        initial: bool,
    },
    VramClockRatio(f64),
    NotifyEditing,
    NotifyPlotsPerRow,
    SwapPlots(DynamicIndex, DynamicIndex),
    RemovePlot(DynamicIndex),
    AddPlot,
    SetConfig(Vec<Vec<StatType>>),
    Show,
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
                        set_row_spacing: 10,
                        set_column_spacing: 10,
                    },

                    append = &gtk::Box {
                        set_halign: gtk::Align::End,
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 10,

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

                            append = &gtk::Label {
                                set_label: "Time period (seconds):"
                            },

                            append = &gtk::SpinButton {
                                set_adjustment: &model.time_period_seconds_adj,
                            },

                            append = &gtk::Button {
                                set_icon_name: "list-add-symbolic",
                                connect_clicked => GraphsWindowMsg::AddPlot,
                                set_tooltip: "Add graph",
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
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let time_period_seconds_adj = gtk::Adjustment::new(60.0, 15.0, 3600.0, 1.0, 0.0, 1.0);

        let stats_data = Arc::new(RwLock::new(StatsData::default()));
        let plots_per_row = F64Binding::new(2.0);
        let plots = FactoryVecDeque::builder()
            .launch_default()
            .forward(sender.input_sender(), |x| x);

        let edit_mode = BoolBinding::new(false);

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
                self.update_plots_layout();
            }
            GraphsWindowMsg::VramClockRatio(ratio) => {
                self.vram_clock_ratio = ratio;
            }
            GraphsWindowMsg::Stats { stats, initial } => {
                if initial {
                    self.stats_data.write().unwrap().clear();
                    sender.input(GraphsWindowMsg::SetConfig(default_plots()));
                }

                let mut data = self.stats_data.write().unwrap();
                data.update(&stats);

                let time_period_seconds = self.time_period_seconds_adj.value() as i64;
                data.trim(time_period_seconds);
            }
            GraphsWindowMsg::SetConfig(configured_plots) => {
                let mut plots = self.plots.guard();
                plots.clear();

                for stats in configured_plots {
                    plots.push_back(PlotComponentConfig {
                        selected_stats: stats,
                        data: self.stats_data.clone(),
                        edit_mode: self.edit_mode.clone(),
                        plots_per_row: self.plots_per_row.clone(),
                        time_period: self.time_period_seconds_adj.clone(),
                    });
                }
            }
            GraphsWindowMsg::NotifyEditing => {
                self.plots.broadcast(PlotComponentMsg::Redraw);
            }
            GraphsWindowMsg::NotifyPlotsPerRow => {
                self.update_plots_layout();
            }
            GraphsWindowMsg::RemovePlot(index) => {
                self.plots.guard().remove(index.current_index());
            }
            GraphsWindowMsg::AddPlot => {
                self.plots.guard().push_back(PlotComponentConfig {
                    selected_stats: vec![],
                    data: self.stats_data.clone(),
                    edit_mode: self.edit_mode.clone(),
                    plots_per_row: self.plots_per_row.clone(),
                    time_period: self.time_period_seconds_adj.clone(),
                });
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
            guard.push_back(plot.into_config());
        }
    }

    fn queue_plots_draw(&self) {
        self.plots.broadcast(PlotComponentMsg::Redraw);
    }
}

#[derive(Clone, glib::Boxed)]
#[boxed_type(name = "DynamicIndexValue")]
pub struct DynamicIndexValue(DynamicIndex);

fn default_plots() -> Vec<Vec<StatType>> {
    vec![
        vec![
            StatType::Temperature("GPU".into()),
            StatType::Temperature("edge".into()),
            StatType::Temperature("junction".into()),
            StatType::Temperature("mem".into()),
        ],
        vec![
            StatType::GpuClock,
            StatType::GpuTargetClock,
            StatType::VramClock,
        ],
        vec![StatType::FanPwm, StatType::FanRpm],
        vec![
            StatType::PowerAverage,
            StatType::PowerCurrent,
            StatType::PowerCap,
        ],
    ]
}

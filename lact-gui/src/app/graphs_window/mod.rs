pub mod plot;
mod plot_component;
pub mod stat;

use super::{msg::AppMsg, APP_BROKER};
use crate::{CONFIG, I18N};
use anyhow::Context;
use chrono::Local;
use gtk::{glib, prelude::*};
use i18n_embed_fl::fl;
use lact_schema::DeviceStats;
use plot_component::{PlotComponent, PlotComponentConfig, PlotComponentMsg};
use relm4::{
    binding::{BoolBinding, ConnectBinding, F64Binding},
    prelude::{DynamicIndex, FactoryVecDeque},
    ComponentController, ComponentParts, ComponentSender, RelmWidgetExt,
};
use relm4_components::save_dialog::{
    SaveDialog, SaveDialogMsg, SaveDialogResponse, SaveDialogSettings,
};
use stat::{StatType, StatsData};
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
    sync::{Arc, RwLock},
};

pub struct GraphsWindow {
    time_period_seconds_adj: gtk::Adjustment,
    gpu_id: Option<String>,
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
        /// Fill for initial message
        selected_gpu_id: Option<String>,
    },
    VramClockRatio(f64),
    NotifyEditing,
    NotifyPlotsPerRow,
    SwapPlots(DynamicIndex, DynamicIndex),
    RemovePlot(DynamicIndex),
    AddPlot,
    SetConfig(Vec<Vec<StatType>>),
    SaveConfig,
    Show,
    ExportData,
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
            set_title: Some(&fl!(I18N, "historical-data-title")),
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
                                set_label: &fl!(I18N, "graphs-per-row"),
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
                                set_label: &fl!(I18N, "time-period-seconds"),
                            },

                            append = &gtk::SpinButton {
                                set_adjustment: &model.time_period_seconds_adj,
                                connect_value_notify => GraphsWindowMsg::NotifyPlotsPerRow,
                            },

                            append = &gtk::Button {
                                set_label: &fl!(I18N, "reset-button"),
                                set_tooltip: &fl!(I18N, "reset-all-graphs-tooltip"),
                                set_css_classes: &["destructive-action"],
                                connect_clicked => GraphsWindowMsg::SetConfig(default_plots()),
                            },

                            append = &gtk::Button {
                                set_icon_name: "list-add-symbolic",
                                connect_clicked => GraphsWindowMsg::AddPlot,
                                set_tooltip: &fl!(I18N, "add-graph"),
                            },
                        },

                        append = &gtk::ToggleButton {
                            set_label: &fl!(I18N, "edit-graphs"),
                            bind: &model.edit_mode,
                            connect_active_notify => GraphsWindowMsg::NotifyEditing,
                        },

                        append = &gtk::Button {
                            set_label: &fl!(I18N, "export-csv"),
                            connect_clicked => GraphsWindowMsg::ExportData,
                        }
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
        let time_period_seconds_adj = gtk::Adjustment::new(60.0, 10.0, 3600.0, 1.0, 1.0, 0.0);

        if let Some(time_period) = CONFIG.read().plots_time_period {
            time_period_seconds_adj.set_value(time_period as f64);
        }

        let stats_data = Arc::new(RwLock::new(StatsData::default()));
        let plots_per_row = F64Binding::new(2.0);

        if let Some(plot_count) = CONFIG.read().plots_per_row {
            plots_per_row.set_value(plot_count as f64);
        }

        let plots = FactoryVecDeque::builder()
            .launch_default()
            .forward(sender.input_sender(), |x| x);

        let edit_mode = BoolBinding::new(false);

        let model = Self {
            time_period_seconds_adj,
            plots_per_row,
            edit_mode,
            plots,
            gpu_id: None,
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
            GraphsWindowMsg::Stats {
                stats,
                selected_gpu_id,
            } => {
                if let Some(selected_gpu_id) = selected_gpu_id {
                    self.stats_data.write().unwrap().clear();

                    let config = CONFIG.read();
                    let plots_config = config
                        .gpus
                        .get(&selected_gpu_id)
                        .map(|config| config.plots.clone())
                        .filter(|plots| !plots.is_empty())
                        .unwrap_or_else(default_plots);

                    self.gpu_id = Some(selected_gpu_id);
                    sender.input(GraphsWindowMsg::SetConfig(plots_config));
                }

                let mut data = self.stats_data.write().unwrap();
                data.update(&stats, self.vram_clock_ratio);

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
                sender.input(GraphsWindowMsg::SaveConfig);
            }
            GraphsWindowMsg::RemovePlot(index) => {
                self.plots.guard().remove(index.current_index());
                sender.input(GraphsWindowMsg::SaveConfig);
            }
            GraphsWindowMsg::AddPlot => {
                self.plots.guard().push_back(PlotComponentConfig {
                    selected_stats: vec![],
                    data: self.stats_data.clone(),
                    edit_mode: self.edit_mode.clone(),
                    plots_per_row: self.plots_per_row.clone(),
                    time_period: self.time_period_seconds_adj.clone(),
                });
                sender.input(GraphsWindowMsg::SaveConfig);
            }
            GraphsWindowMsg::SwapPlots(left, right) => {
                self.plots
                    .guard()
                    .swap(left.current_index(), right.current_index());
                sender.input(GraphsWindowMsg::SaveConfig);
            }
            GraphsWindowMsg::SaveConfig => {
                if let Some(gpu_id) = self.gpu_id.clone() {
                    CONFIG.write().edit(|config| {
                        config.gpus.entry(gpu_id).or_default().plots = self
                            .plots
                            .iter()
                            .map(|plot| plot.selected_stats())
                            .collect();

                        config.plots_time_period =
                            Some(self.time_period_seconds_adj.value() as u64);
                        config.plots_per_row = Some(self.plots_per_row.value() as u64);
                    });
                }
            }
            GraphsWindowMsg::ExportData => {
                let settings = SaveDialogSettings {
                    cancel_label: "Cancel".to_owned(),
                    accept_label: "Save".to_owned(),
                    create_folders: true,
                    is_modal: true,
                    filters: vec![],
                };

                let save_dialog = SaveDialog::builder().launch(settings);
                save_dialog.emit(SaveDialogMsg::SaveAs(format!(
                    "LACT-stats-{}.csv",
                    Local::now().format("%Y%m%d-%H%M%S")
                )));
                let save_dialog_stream = save_dialog.into_stream();

                let data = self.stats_data.clone();
                relm4::spawn(async move {
                    if let Some(SaveDialogResponse::Accept(path)) =
                        save_dialog_stream.recv_one().await
                    {
                        let data = data.read().unwrap();
                        if let Err(err) = export_to_file(&data, &path) {
                            APP_BROKER.send(AppMsg::Error(Arc::new(err)));
                        }
                    }
                });
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
            StatType::Temperature("GPU Hotspot".into()),
            StatType::Temperature("VRAM".into()),
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

fn export_to_file(data: &StatsData, path: &Path) -> anyhow::Result<()> {
    let file = File::create(path).context("Could not create file")?;
    let mut output = BufWriter::new(file);

    let header = data
        .list_stats()
        .map(|stat| stat.display())
        .collect::<Vec<_>>()
        .join(",");
    writeln!(output, "timestamp,{header}")?;

    let all_stats = data.all_stats().values().collect::<Vec<_>>();

    if let Some(first_stat_type) = all_stats.first() {
        for i in 0..first_stat_type.len() {
            let timestamp = first_stat_type[i].0;
            write!(output, "{timestamp}")?;

            for stat in &all_stats {
                let value = stat[i].1;
                write!(output, ",{value}")?;
            }

            writeln!(output)?;
        }
    }

    output.flush()?;

    Ok(())
}

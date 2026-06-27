use crate::app::{
    components::{
        info_row::{InfoRow, InfoRowExt},
        info_row_level::InfoRowLevel,
        page_section::PageSection,
    },
    utils::ext::FlowBoxExt,
};
use crate::{StatContext, StatType};
use gtk::pango::AttrList;
use gtk::prelude::{BoxExt, OrientableExt, PopoverExt as _, WidgetExt};
use lact_schema::{DeviceInfo, DeviceStats, PowerStates};
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt as _};
use std::str::FromStr as _;
use std::sync::Arc;

pub struct GpuStatsSection {
    stats: Arc<DeviceStats>,
    vram_clock_ratio: f64,
    gpu_model: String,
    max_gpu_clock: Option<u64>,
    max_vram_clock: Option<u64>,
    min_gpu_clock: Option<u64>,
    min_vram_clock: Option<u64>,
}

#[derive(Debug)]
pub enum GpuStatsSectionMsg {
    Info(Arc<DeviceInfo>),
    Stats(Arc<DeviceStats>),
    PowerStates(Arc<PowerStates>),
}

#[relm4::component(pub)]
impl relm4::SimpleComponent for GpuStatsSection {
    type Input = GpuStatsSectionMsg;
    type Output = ();
    type Init = ();

    view! {
        gtk::Box {
            add_css_class: "gpu-stats-section",
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 10,

            PageSection::new("") {
                append_child = &gtk::FlowBox {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_column_spacing: 10,
                    set_row_spacing: 10,
                    set_homogeneous: true,
                    set_selection_mode: gtk::SelectionMode::None,

                    append = &InfoRow {
                        #[watch]
                        set_name: StatType::DeviceName.gui_label(&context),
                        #[watch]
                        set_value: StatType::DeviceName.gui_value(&context),
                    },

                    append = &InfoRow {
                        #[watch]
                        set_name: StatType::Throttling.gui_label(&context),
                        #[watch]
                        set_value: StatType::Throttling.gui_value(&context),
                    },

                    append_child = &InfoRow {
                        #[watch]
                        set_name: StatType::GpuTargetClock.gui_label(&context),
                        #[watch]
                        set_value: StatType::GpuTargetClock.gui_value(&context),
                    } -> clockspeed_target_item: gtk::FlowBoxChild {
                        #[watch]
                        set_visible: StatType::GpuTargetClock.gui_visible(&context),
                    },

                    append_child = &InfoRow {
                        #[watch]
                        set_name: StatType::GpuVoltage.gui_label(&context),
                        #[watch]
                        set_value: StatType::GpuVoltage.gui_value(&context),
                    } -> gpu_voltage_item: gtk::FlowBoxChild {
                        #[watch]
                        set_visible: StatType::GpuVoltage.gui_visible(&context),
                    },


                    append_child = &InfoRow {
                        #[watch]
                        set_name: StatType::Temperatures.gui_label(&context),
                        #[watch]
                        set_value: StatType::Temperatures.gui_value(&context),
                    } -> basic_temps_item: gtk::FlowBoxChild {
                        #[watch]
                        set_visible: StatType::Temperatures.gui_visible(&context)
                            && secondary_temperatures.is_empty(),
                    },

                    append_child = &InfoRow {
                        #[watch]
                        set_name: StatType::Temperatures.gui_label(&context),
                        #[watch]
                        set_value: StatType::Temperatures.gui_value(&context),

                        set_icon: "go-down-symbolic".to_string(),

                        #[name = "secondary_temps_popover"]
                        set_popover = &gtk::Popover {
                            gtk::Label {
                                set_margin_all: 10,
                                set_selectable: false,
                                set_use_markup: true,
                                set_attributes: Some(&AttrList::from_str("0 -1 weight bold").unwrap()),

                                #[watch]
                                set_label: &secondary_temperatures.join("\n"),
                            },
                        },

                        connect_clicked[secondary_temps_popover] => move |_| {
                            secondary_temps_popover.popup();
                        },
                    } -> full_temps_item: gtk::FlowBoxChild {
                        #[watch]
                        set_visible: StatType::Temperatures.gui_visible(&context)
                            && !secondary_temperatures.is_empty(),
                    },
                },
            },

            PageSection::new("") {
                append_child = &gtk::FlowBox {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_column_spacing: 10,
                    set_homogeneous: true,
                    set_selection_mode: gtk::SelectionMode::None,

                    append_child = &InfoRowLevel {
                        #[watch]
                        set_name: StatType::GpuClock.gui_label(&context),
                        #[watch]
                        set_value: StatType::GpuClock.gui_value(&context),
                        #[watch]
                        set_level_value: StatType::GpuClock.gui_level(&context).unwrap_or(0.0),
                    } -> gpu_clock_item: gtk::FlowBoxChild {
                        #[watch]
                        set_visible: StatType::GpuClock.gui_visible(&context),
                    },

                    append_child = &InfoRowLevel {
                        #[watch]
                        set_name: StatType::VramClock.gui_label(&context),
                        #[watch]
                        set_value: StatType::VramClock.gui_value(&context),
                        #[watch]
                        set_level_value: StatType::VramClock.gui_level(&context).unwrap_or(0.0),
                    } -> vram_clock_item: gtk::FlowBoxChild {
                        #[watch]
                        set_visible: StatType::VramClock.gui_visible(&context),
                    },

                    append_child = &InfoRowLevel {
                        #[watch]
                        set_name: StatType::GpuUsage.gui_label(&context),
                        #[watch]
                        set_value: StatType::GpuUsage.gui_value(&context),
                        #[watch]
                        set_level_value: StatType::GpuUsage.gui_level(&context).unwrap_or(0.0),
                    } -> gpu_usage_item: gtk::FlowBoxChild {
                        #[watch]
                        set_visible: StatType::GpuUsage.gui_visible(&context),
                    },

                    append_child = &InfoRowLevel {
                        #[watch]
                        set_name: StatType::VramUsage.gui_label(&context),
                        #[watch]
                        set_value: StatType::VramUsage.gui_value(&context),
                        #[watch]
                        set_level_value: StatType::VramUsage.gui_level(&context).unwrap_or(0.0),
                    } -> vram_usage_item: gtk::FlowBoxChild {},

                    append_child = &InfoRowLevel {
                        #[watch]
                        set_name: StatType::GttUsage.gui_label(&context),
                        #[watch]
                        set_value: StatType::GttUsage.gui_value(&context),
                        #[watch]
                        set_level_value: StatType::GttUsage.gui_level(&context).unwrap_or(0.0),
                    } -> gtt_usage_item: gtk::FlowBoxChild {
                        #[watch]
                        set_visible: StatType::GttUsage.gui_visible(&context),
                    },

                    append_child = &InfoRowLevel {
                        #[watch]
                        set_name: StatType::PowerUsage.gui_label(&context),
                        #[watch]
                        set_value: StatType::PowerUsage.gui_value(&context),
                        #[watch]
                        set_level_value: StatType::PowerUsage.gui_level(&context).unwrap_or(0.0),
                    } -> power_usage_item: gtk::FlowBoxChild {
                        #[watch]
                        set_visible: StatType::PowerUsage.gui_visible(&context),
                    },

                    append_child = &InfoRowLevel {
                        #[watch]
                        set_name: StatType::FanSpeed.gui_label(&context),
                        #[watch]
                        set_value: StatType::FanSpeed.gui_value(&context),
                        #[watch]
                        set_level_value: StatType::FanSpeed.gui_level(&context).unwrap_or(0.0),
                    } -> fan_speed_item: gtk::FlowBoxChild {
                        #[watch]
                        set_visible: StatType::FanSpeed.gui_visible(&context),
                    },
                },
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            stats: Arc::new(DeviceStats::default()),
            vram_clock_ratio: 1.0,
            gpu_model: String::new(),
            max_gpu_clock: None,
            max_vram_clock: None,
            min_gpu_clock: None,
            min_vram_clock: None,
        };
        let context = model.stat_context();
        let (_, secondary_temperatures) = StatType::Temperatures
            .temperature_values(model.stats.as_ref())
            .unwrap_or_default();

        let widgets = view_output!();

        ComponentParts { widgets, model }
    }

    fn update(&mut self, msg: Self::Input, _sender: relm4::ComponentSender<Self>) {
        match msg {
            GpuStatsSectionMsg::Info(info) => {
                self.vram_clock_ratio = info.vram_clock_ratio();
                if let Some(pci_info) = &info.pci_info {
                    self.gpu_model = info
                        .drm_info
                        .as_ref()
                        .and_then(|drm| drm.device_name.as_deref())
                        .or(pci_info.device_pci_info.model.as_deref())
                        .unwrap_or("Unknown")
                        .to_owned();
                }
            }
            GpuStatsSectionMsg::Stats(stats) => {
                self.stats = stats;
            }
            GpuStatsSectionMsg::PowerStates(pstates) => {
                self.max_gpu_clock = pstates.max_gpu_clock();
                self.max_vram_clock = pstates.max_vram_clock();
                self.min_gpu_clock = pstates.min_gpu_clock();
                self.min_vram_clock = pstates.min_vram_clock();
            }
        }
    }

    fn pre_view(&self) {
        let context = model.stat_context();
        let (_, secondary_temperatures) = StatType::Temperatures
            .temperature_values(model.stats.as_ref())
            .unwrap_or_default();
    }
}

impl GpuStatsSection {
    fn stat_context(&self) -> StatContext<'_> {
        StatContext {
            stats: self.stats.as_ref(),
            gpu_model: &self.gpu_model,
            vram_clock_ratio: self.vram_clock_ratio,
            max_gpu_clock: self.max_gpu_clock,
            max_vram_clock: self.max_vram_clock,
            min_gpu_clock: self.min_gpu_clock,
            min_vram_clock: self.min_vram_clock,
        }
    }
}

use crate::app::{
    components::{
        info_row::{InfoRow, InfoRowExt},
        info_row_level::InfoRowLevel,
        page_section::PageSection,
    },
    utils::{
        ext::FlowBoxExt,
        stat_view::{
            StatConfig, StatConfigMap, StatContext, StatType, StatViewConfig, fixed_stat_configs,
        },
    },
};
use gtk::pango::AttrList;
use gtk::prelude::{BoxExt, Cast, FlowBoxChildExt, OrientableExt, PopoverExt as _, WidgetExt};
use lact_schema::{DeviceInfo, DeviceStats, PowerStates};
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt as _};
use std::str::FromStr as _;
use std::sync::Arc;

pub struct GpuStatsSection {
    stats: Arc<DeviceStats>,
    vram_clock_ratio: f64,
    gpu_model: String,
    value_size_group: gtk::SizeGroup,
    max_gpu_clock: Option<u64>,
    max_vram_clock: Option<u64>,
    min_gpu_clock: Option<u64>,
    min_vram_clock: Option<u64>,
    stat_configs: StatConfigMap,
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
                        set_name: model.stat_label(&StatType::DeviceName).to_owned(),
                        #[watch]
                        set_value: model.stat_view(&StatType::DeviceName, &context).display_value().to_owned(),
                    },

                    append = &InfoRow {
                        #[watch]
                        set_name: model.stat_label(&StatType::Throttling).to_owned(),
                        #[watch]
                        set_value: model.stat_view(&StatType::Throttling, &context).display_value().to_owned(),
                    },

                    append_child = &InfoRow {
                        #[watch]
                        set_name: model.stat_label(&StatType::GpuTargetClock).to_owned(),
                        #[watch]
                        set_value: model.stat_view(&StatType::GpuTargetClock, &context).display_value().to_owned(),
                    } -> clockspeed_target_item: gtk::FlowBoxChild {
                        #[watch]
                        set_visible: model.stat_view(&StatType::GpuTargetClock, &context).visible(),
                    },

                    append_child = &InfoRow {
                        #[watch]
                        set_name: model.stat_label(&StatType::GpuVoltage).to_owned(),
                        #[watch]
                        set_value: model.stat_view(&StatType::GpuVoltage, &context).display_value().to_owned(),
                    } -> gpu_voltage_item: gtk::FlowBoxChild {
                        #[watch]
                        set_visible: model.stat_view(&StatType::GpuVoltage, &context).visible(),
                    },


                    append_child = &InfoRow {
                        #[watch]
                        set_name: model.stat_label(&StatType::Temperatures).to_owned(),
                        #[watch]
                        set_value: model.stat_view(&StatType::Temperatures, &context).display_value().to_owned(),
                    } -> basic_temps_item: gtk::FlowBoxChild {
                        #[watch]
                        set_visible: model.stat_view(&StatType::Temperatures, &context).visible()
                            && secondary_temperatures.is_empty(),
                    },

                    append_child = &InfoRow {
                        #[watch]
                        set_name: model.stat_label(&StatType::Temperatures).to_owned(),
                        #[watch]
                        set_value: model.stat_view(&StatType::Temperatures, &context).display_value().to_owned(),

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
                        set_visible: model.stat_view(&StatType::Temperatures, &context).visible()
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
                        set_name: model.stat_label(&StatType::GpuClock).to_owned(),
                        #[watch]
                        set_value: model.stat_view(&StatType::GpuClock, &context).display_value().to_owned(),
                        #[watch]
                        set_level_value: model.stat_view(&StatType::GpuClock, &context).level_value(),
                    } -> gpu_clock_item: gtk::FlowBoxChild {
                        #[watch]
                        set_visible: model.stat_view(&StatType::GpuClock, &context).visible(),
                    },

                    append_child = &InfoRowLevel {
                        #[watch]
                        set_name: model.stat_label(&StatType::VramClock).to_owned(),
                        #[watch]
                        set_value: model.stat_view(&StatType::VramClock, &context).display_value().to_owned(),
                        #[watch]
                        set_level_value: model.stat_view(&StatType::VramClock, &context).level_value(),
                    } -> vram_clock_item: gtk::FlowBoxChild {
                        #[watch]
                        set_visible: model.stat_view(&StatType::VramClock, &context).visible(),
                    },

                    append_child = &InfoRowLevel {
                        #[watch]
                        set_name: model.stat_label(&StatType::GpuUsage).to_owned(),
                        #[watch]
                        set_value: model.stat_view(&StatType::GpuUsage, &context).display_value().to_owned(),
                        #[watch]
                        set_level_value: model.stat_view(&StatType::GpuUsage, &context).level_value(),
                    } -> gpu_usage_item: gtk::FlowBoxChild {
                        #[watch]
                        set_visible: model.stat_view(&StatType::GpuUsage, &context).visible(),
                    },

                    append_child = &InfoRowLevel {
                        #[watch]
                        set_name: model.stat_label(&StatType::VramUsage).to_owned(),
                        #[watch]
                        set_value: model.stat_view(&StatType::VramUsage, &context).display_value().to_owned(),
                        #[watch]
                        set_level_value: model.stat_view(&StatType::VramUsage, &context).level_value(),
                    } -> vram_usage_item: gtk::FlowBoxChild {},

                    append_child = &InfoRowLevel {
                        #[watch]
                        set_name: model.stat_label(&StatType::GttUsage).to_owned(),
                        #[watch]
                        set_value: model.stat_view(&StatType::GttUsage, &context).display_value().to_owned(),
                        #[watch]
                        set_level_value: model.stat_view(&StatType::GttUsage, &context).level_value(),
                    } -> gtt_usage_item: gtk::FlowBoxChild {
                        #[watch]
                        set_visible: model.stat_view(&StatType::GttUsage, &context).visible(),
                    },

                    append_child = &InfoRowLevel {
                        #[watch]
                        set_name: model.stat_label(&StatType::PowerUsage).to_owned(),
                        #[watch]
                        set_value: model.stat_view(&StatType::PowerUsage, &context).display_value().to_owned(),
                        #[watch]
                        set_level_value: model.stat_view(&StatType::PowerUsage, &context).level_value(),
                    } -> power_usage_item: gtk::FlowBoxChild {
                        #[watch]
                        set_visible: model.stat_view(&StatType::PowerUsage, &context).visible(),
                    },

                    append_child = &InfoRowLevel {
                        #[watch]
                        set_name: model.stat_label(&StatType::FanSpeed).to_owned(),
                        #[watch]
                        set_value: model.stat_view(&StatType::FanSpeed, &context).display_value().to_owned(),
                        #[watch]
                        set_level_value: model.stat_view(&StatType::FanSpeed, &context).level_value(),
                    } -> fan_speed_item: gtk::FlowBoxChild {
                        #[watch]
                        set_visible: model.stat_view(&StatType::FanSpeed, &context).visible(),
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
        let value_size_group = gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal);
        let model = Self {
            stats: Arc::new(DeviceStats::default()),
            vram_clock_ratio: 1.0,
            gpu_model: String::new(),
            value_size_group,
            max_gpu_clock: None,
            max_vram_clock: None,
            min_gpu_clock: None,
            min_vram_clock: None,
            stat_configs: fixed_stat_configs().clone(),
        };
        let context = model.stat_context();
        let secondary_temperatures = model
            .stat_view(&StatType::Temperatures, &context)
            .secondary_temperatures()
            .to_owned();

        let widgets = view_output!();

        widgets
            .power_usage_item
            .child()
            .unwrap()
            .downcast::<InfoRowLevel>()
            .unwrap()
            .set_value_size_group(&model.value_size_group);
        widgets
            .gpu_usage_item
            .child()
            .unwrap()
            .downcast::<InfoRowLevel>()
            .unwrap()
            .set_value_size_group(&model.value_size_group);
        widgets
            .vram_usage_item
            .child()
            .unwrap()
            .downcast::<InfoRowLevel>()
            .unwrap()
            .set_value_size_group(&model.value_size_group);
        widgets
            .gpu_clock_item
            .child()
            .unwrap()
            .downcast::<InfoRowLevel>()
            .unwrap()
            .set_value_size_group(&model.value_size_group);
        widgets
            .vram_clock_item
            .child()
            .unwrap()
            .downcast::<InfoRowLevel>()
            .unwrap()
            .set_value_size_group(&model.value_size_group);
        widgets
            .fan_speed_item
            .child()
            .unwrap()
            .downcast::<InfoRowLevel>()
            .unwrap()
            .set_value_size_group(&model.value_size_group);

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
        let secondary_temperatures = model
            .stat_view(&StatType::Temperatures, &context)
            .secondary_temperatures()
            .to_owned();
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

    fn stat_config(&self, stat_type: &StatType) -> &StatConfig {
        self.stat_configs
            .get(stat_type)
            .expect("fixed stat config missing")
    }

    fn stat_label(&self, stat_type: &StatType) -> &str {
        &self.stat_config(stat_type).label
    }

    fn stat_view(&self, stat_type: &StatType, context: &StatContext<'_>) -> StatViewConfig {
        self.stat_config(stat_type)
            .view(stat_type, context)
            .expect("fixed stat view missing")
    }
}

use crate::{
    I18N,
    app::{
        ext::FlowBoxExt,
        formatting::{self, Mono},
        info_row::{InfoRow, InfoRowExt},
        info_row_level::InfoRowLevel,
        page_section::PageSection,
    },
};
use gtk::prelude::{BoxExt, Cast, FlowBoxChildExt, OrientableExt, WidgetExt};
use i18n_embed_fl::fl;
use lact_schema::{DeviceStats, PowerStats};
use relm4::{ComponentParts, ComponentSender};
use std::sync::Arc;

pub struct PowerStatsSection {
    stats: Arc<DeviceStats>,
    value_size_group: gtk::SizeGroup,
}

#[derive(Debug)]
pub enum PowerStatsSectionMsg {
    Stats(Arc<DeviceStats>),
}

#[relm4::component(pub)]
impl relm4::SimpleComponent for PowerStatsSection {
    type Input = PowerStatsSectionMsg;
    type Output = ();
    type Init = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 10,

            PageSection::new("") {
                append_child = &gtk::FlowBox {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_column_spacing: 10,
                    set_row_spacing: 10,
                    set_homogeneous: true,
                    set_selection_mode: gtk::SelectionMode::None,

                    append_child = &InfoRow {
                        set_name: fl!(I18N, "throttling"),
                        #[watch]
                        set_value: formatting::fmt_throttling_text(&model.stats),
                    },

                    append_child = &InfoRow {
                        set_name: fl!(I18N, "gpu-temp"),
                        #[watch]
                        set_value: temperatures_text(&model.stats),
                    },

                    append_child = &InfoRow {
                        set_name: fl!(I18N, "gpu-voltage"),
                        #[watch]
                        set_value: model
                            .stats
                            .voltage
                            .gpu
                            .map(|voltage| format!("{} V", Mono::float(voltage as f64 / 1000.0, 3)))
                            .unwrap_or_else(|| fl!(I18N, "missing-stat")),
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
                        set_name: fl!(I18N, "gpu-usage"),
                        #[watch]
                        set_value: format!("{}%", Mono::uint(model.stats.busy_percent.unwrap_or(0))),
                        #[watch]
                        set_level_value: model.stats.busy_percent.unwrap_or(0) as f64 / 100.0,
                    } -> gpu_usage_item: gtk::FlowBoxChild {
                        #[watch]
                        set_visible: model.stats.busy_percent.is_some(),
                    },

                    append_child = &InfoRowLevel {
                        set_name: fl!(I18N, "power-usage"),
                        #[watch]
                        set_value: power_usage_text(&model.stats.power),
                        #[watch]
                        set_level_value: power_usage_level(&model.stats.power),
                    } -> power_usage_item: gtk::FlowBoxChild {
                        #[watch]
                        set_visible: model.stats.power.average.is_some() || model.stats.power.current.is_some(),
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
            value_size_group: gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal),
        };

        let widgets = view_output!();

        widgets
            .gpu_usage_item
            .child()
            .unwrap()
            .downcast::<InfoRowLevel>()
            .unwrap()
            .set_value_size_group(&model.value_size_group);
        widgets
            .power_usage_item
            .child()
            .unwrap()
            .downcast::<InfoRowLevel>()
            .unwrap()
            .set_value_size_group(&model.value_size_group);

        ComponentParts { widgets, model }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            PowerStatsSectionMsg::Stats(stats) => {
                self.stats = stats;
            }
        }
    }
}

fn temperatures_text(stats: &DeviceStats) -> String {
    let (primary_temperatures, secondary_temperatures) = formatting::fmt_temperature_text(stats);
    let temperatures = primary_temperatures
        .into_iter()
        .chain(secondary_temperatures)
        .collect::<Vec<_>>();

    if temperatures.is_empty() {
        fl!(I18N, "missing-stat")
    } else {
        temperatures.join(", ")
    }
}

fn power_usage_text(power: &PowerStats) -> String {
    let power_current = power
        .current
        .filter(|value| *value != 0.0)
        .or(power.average);

    format!(
        "{} {}",
        Mono::float(power_current.unwrap_or(0.0), 1),
        fl!(I18N, "watt")
    )
}

fn power_usage_level(power: &PowerStats) -> f64 {
    let power_current = power
        .current
        .filter(|value| *value != 0.0)
        .or(power.average);

    power_current
        .zip(power.cap_current)
        .map(|(current, cap)| current / cap)
        .unwrap_or(0.0)
}

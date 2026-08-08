mod stat;
mod stat_item;

pub use stat::{GpuStat, GpuStatDisplay};

use crate::{app::components::page_section::PageSection, config::StatsLayout};
use gtk::prelude::{BoxExt, OrientableExt, WidgetExt};
use lact_schema::{DeviceInfo, DeviceStats, PowerStates};
use relm4::{ComponentParts, ComponentSender, prelude::FactoryVecDeque};
use stat::StatsContext;
use stat_item::{StatItem, StatItemMsg};
use std::sync::Arc;

pub struct GpuStatsSection {
    layout: StatsLayout,
    ctx: StatsContext,
    text_items: FactoryVecDeque<StatItem>,
    level_items: FactoryVecDeque<StatItem>,
}

#[derive(Debug)]
pub enum GpuStatsSectionMsg {
    Info(Arc<DeviceInfo>),
    Stats(Arc<DeviceStats>),
    PowerStates(Arc<PowerStates>),
    SetLayout(StatsLayout),
}

#[relm4::component(pub)]
impl relm4::SimpleComponent for GpuStatsSection {
    type Input = GpuStatsSectionMsg;
    type Output = ();
    type Init = StatsLayout;

    view! {
        gtk::Box {
            add_css_class: "gpu-stats-section",
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 10,
            #[watch]
            set_visible: model.visible_count(GpuStatDisplay::Text) > 0
                || model.visible_count(GpuStatDisplay::LevelBar) > 0,

            PageSection::new("") {
                #[watch]
                set_visible: model.visible_count(GpuStatDisplay::Text) > 0,

                append_child = &model.text_items.widget().clone() -> gtk::FlowBox {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_column_spacing: 10,
                    set_row_spacing: 10,
                    set_homogeneous: true,
                    #[watch]
                    set_max_children_per_line: model.columns(GpuStatDisplay::Text),
                    set_selection_mode: gtk::SelectionMode::None,
                },
            },

            PageSection::new("") {
                #[watch]
                set_visible: model.visible_count(GpuStatDisplay::LevelBar) > 0,

                append_child = &model.level_items.widget().clone() -> gtk::FlowBox {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_column_spacing: 10,
                    set_homogeneous: true,
                    #[watch]
                    set_max_children_per_line: model.columns(GpuStatDisplay::LevelBar),
                    set_selection_mode: gtk::SelectionMode::None,
                },
            },
        }
    }

    fn init(
        layout: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut model = Self {
            layout,
            ctx: StatsContext::default(),
            text_items: FactoryVecDeque::builder().launch_default().detach(),
            level_items: FactoryVecDeque::builder().launch_default().detach(),
        };
        model.rebuild_factories();

        let widgets = view_output!();
        ComponentParts { widgets, model }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            GpuStatsSectionMsg::Info(info) => {
                self.ctx.vram_clock_ratio = info.vram_clock_ratio();
                self.ctx.gpu_model = info
                    .pci_info
                    .as_ref()
                    .map(|pci_info| {
                        info.drm_info
                            .as_ref()
                            .and_then(|drm| drm.device_name.as_deref())
                            .or(pci_info.device_pci_info.model.as_deref())
                            .unwrap_or("Unknown")
                            .to_owned()
                    })
                    .unwrap_or_default();
                self.broadcast_context();
            }
            GpuStatsSectionMsg::Stats(stats) => {
                let old_items = self.visible_items();
                self.ctx.stats = stats;
                if old_items == self.visible_items() {
                    self.broadcast_context();
                } else {
                    self.rebuild_factories();
                }
            }
            GpuStatsSectionMsg::PowerStates(pstates) => {
                self.ctx.max_gpu_clock = pstates.max_gpu_clock();
                self.ctx.max_vram_clock = pstates.max_vram_clock();
                self.ctx.min_gpu_clock = pstates.min_gpu_clock();
                self.ctx.min_vram_clock = pstates.min_vram_clock();
                self.broadcast_context();
            }
            GpuStatsSectionMsg::SetLayout(layout) => {
                self.layout = layout;
                self.rebuild_factories();
            }
        }
    }
}

impl GpuStatsSection {
    fn visible_items(&self) -> Vec<(GpuStat, GpuStatDisplay)> {
        self.layout
            .0
            .iter()
            .filter(|entry| entry.enabled && entry.stat.has_data_for(&self.ctx.stats))
            .map(|entry| (entry.stat, entry.display))
            .collect()
    }

    fn rebuild_factories(&mut self) {
        let visible_items = self.visible_items();
        let level_size_group = gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal);
        let mut text_items = self.text_items.guard();
        let mut level_items = self.level_items.guard();
        text_items.clear();
        level_items.clear();

        for (stat, display) in visible_items {
            let value_size_group = if display == GpuStatDisplay::LevelBar {
                level_size_group.clone()
            } else {
                gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal)
            };
            let item = StatItem {
                stat,
                display,
                ctx: self.ctx.clone(),
                value_size_group,
            };

            match display {
                GpuStatDisplay::Text => {
                    text_items.push_back(item);
                }
                GpuStatDisplay::LevelBar => {
                    level_items.push_back(item);
                }
            }
        }
    }

    fn broadcast_context(&self) {
        self.text_items
            .broadcast(StatItemMsg::Context(self.ctx.clone()));
        self.level_items
            .broadcast(StatItemMsg::Context(self.ctx.clone()));
    }

    fn visible_count(&self, display: GpuStatDisplay) -> u32 {
        match display {
            GpuStatDisplay::Text => self.text_items.len() as u32,
            GpuStatDisplay::LevelBar => self.level_items.len() as u32,
        }
    }

    fn columns(&self, display: GpuStatDisplay) -> u32 {
        self.visible_count(display).clamp(1, 3)
    }
}

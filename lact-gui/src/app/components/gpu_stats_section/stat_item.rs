use super::stat::{GpuStat, GpuStatDisplay, StatsContext, level, name, text};
use crate::app::{
    components::{
        info_row::{InfoRow, InfoRowExt},
        info_row_level::InfoRowLevel,
    },
    utils::formatting,
};
use gtk::{
    pango::AttrList,
    prelude::{OrientableExt, PopoverExt as _, WidgetExt},
};
use relm4::RelmWidgetExt as _;
use std::str::FromStr as _;

pub struct StatItem {
    pub stat: GpuStat,
    pub display: GpuStatDisplay,
    pub ctx: StatsContext,
    pub value_size_group: gtk::SizeGroup,
}

#[derive(Debug, Clone)]
pub enum StatItemMsg {
    Context(StatsContext),
}

#[relm4::factory(pub)]
impl relm4::factory::FactoryComponent for StatItem {
    type ParentWidget = gtk::FlowBox;
    type CommandOutput = ();
    type Input = StatItemMsg;
    type Output = ();
    type Init = Self;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            InfoRow {
                set_hexpand: true,
                #[watch]
                set_visible: self.display == GpuStatDisplay::Text
                    && (self.stat != GpuStat::Temperature || !self.has_secondary_temperatures()),
                #[watch]
                set_name: name(self.stat, &self.ctx),
                #[watch]
                set_value: text(self.stat, &self.ctx),
            },

            InfoRow {
                set_hexpand: true,
                #[watch]
                set_visible: self.display == GpuStatDisplay::Text
                    && self.stat == GpuStat::Temperature
                    && self.has_secondary_temperatures(),
                #[watch]
                set_name: name(self.stat, &self.ctx),
                #[watch]
                set_value: text(self.stat, &self.ctx),
                set_icon: "go-down-symbolic".to_owned(),

                #[name = "secondary_temps_popover"]
                set_popover = &gtk::Popover {
                    gtk::Label {
                        set_margin_all: 10,
                        set_selectable: false,
                        set_use_markup: true,
                        set_attributes: Some(&AttrList::from_str("0 -1 weight bold").unwrap()),
                        #[watch]
                        set_label: &self.secondary_temperatures().join("\n"),
                    },
                },

                connect_clicked[secondary_temps_popover] => move |_| {
                    secondary_temps_popover.popup();
                },
            },

            InfoRowLevel {
                set_hexpand: true,
                #[watch]
                set_visible: self.display == GpuStatDisplay::LevelBar,
                #[watch]
                set_name: name(self.stat, &self.ctx),
                #[watch]
                set_value: text(self.stat, &self.ctx),
                #[watch]
                set_level_value: level(self.stat, &self.ctx),
                set_value_size_group: &self.value_size_group,
            },
        }
    }

    fn init_model(
        init: Self::Init,
        _index: &Self::Index,
        _sender: relm4::FactorySender<Self>,
    ) -> Self {
        init
    }

    fn update(&mut self, msg: Self::Input, _sender: relm4::FactorySender<Self>) {
        let StatItemMsg::Context(ctx) = msg;
        self.ctx = ctx;
    }
}

impl StatItem {
    fn secondary_temperatures(&self) -> Vec<String> {
        formatting::fmt_temperature_text(&self.ctx.stats).1
    }

    fn has_secondary_temperatures(&self) -> bool {
        !self.secondary_temperatures().is_empty()
    }
}

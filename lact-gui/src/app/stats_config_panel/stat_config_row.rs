use crate::{
    I18N,
    app::{components::gpu_stats_section::GpuStatDisplay, stats_config_panel::StatsConfigPanelMsg},
    config::StatEntry,
};
use adw::prelude::*;
use i18n_embed_fl::fl;
use relm4::FactorySender;

pub struct StatConfigRow {
    pub entry: StatEntry,
    pub available: bool,
}

#[relm4::factory(pub)]
impl relm4::factory::FactoryComponent for StatConfigRow {
    type Init = Self;
    type Input = ();
    type Output = StatsConfigPanelMsg;
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        adw::ActionRow {
            set_title: &self.entry.stat.title(),
            set_subtitle: &if self.available {
                String::new()
            } else {
                fl!(I18N, "stat-not-available")
            },

            add_suffix = &gtk::Switch {
                set_valign: gtk::Align::Center,
                set_active: self.entry.enabled,
                connect_active_notify[sender, stat = self.entry.stat] => move |switch| {
                    let _ = sender.output(StatsConfigPanelMsg::SetEnabled(stat, switch.is_active()));
                },
            },

            add_suffix = &gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_valign: gtk::Align::Center,
                set_visible: self.entry.stat.supported_displays().len() > 1,
                add_css_class: "linked",

                #[name = "text_button"]
                gtk::ToggleButton {
                    set_label: &fl!(I18N, "stats-display-text"),
                    set_active: self.entry.display == GpuStatDisplay::Text,
                    connect_toggled[sender, stat = self.entry.stat] => move |button| {
                        if button.is_active() {
                            let _ = sender.output(StatsConfigPanelMsg::SetDisplay(stat, GpuStatDisplay::Text));
                        }
                    },
                },

                gtk::ToggleButton {
                    set_label: &fl!(I18N, "stats-display-bar"),
                    set_group: Some(&text_button),
                    set_active: self.entry.display == GpuStatDisplay::LevelBar,
                    connect_toggled[sender, stat = self.entry.stat] => move |button| {
                        if button.is_active() {
                            let _ = sender.output(StatsConfigPanelMsg::SetDisplay(stat, GpuStatDisplay::LevelBar));
                        }
                    },
                },
            },
        }
    }

    fn init_model(init: Self::Init, _index: &Self::Index, _sender: FactorySender<Self>) -> Self {
        init
    }
}

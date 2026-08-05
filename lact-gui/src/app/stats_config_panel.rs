mod stat_config_row;

use crate::{
    CONFIG, I18N,
    app::{
        APP_BROKER,
        components::gpu_stats_section::{GpuStat, GpuStatDisplay},
        msg::AppMsg,
    },
    config::{StatsLayout, StatsPage},
};
use adw::prelude::*;
use gtk::prelude::{ButtonExt, WidgetExt};
use i18n_embed_fl::fl;
use lact_schema::DeviceStats;
use relm4::{ComponentParts, ComponentSender, css, prelude::FactoryVecDeque};
use stat_config_row::StatConfigRow;
use std::sync::Arc;

pub struct StatsConfigPanel {
    page: StatsPage,
    layout: StatsLayout,
    stats: Option<Arc<DeviceStats>>,
    rows: FactoryVecDeque<StatConfigRow>,
}

#[derive(Debug)]
pub enum StatsConfigPanelMsg {
    Show {
        page: StatsPage,
        stats: Option<Arc<DeviceStats>>,
    },
    SetEnabled(GpuStat, bool),
    SetDisplay(GpuStat, GpuStatDisplay),
    ResetLayout,
}

#[relm4::component(pub)]
impl relm4::SimpleComponent for StatsConfigPanel {
    type Init = ();
    type Input = StatsConfigPanelMsg;
    type Output = ();

    view! {
        adw::ToolbarView {
            add_top_bar = &adw::HeaderBar {
                set_show_end_title_buttons: false,

                #[wrap(Some)]
                set_title_widget = &adw::WindowTitle {
                    #[watch]
                    set_title: &model.page_title(),
                    set_subtitle: &fl!(I18N, "configure-stats"),
                },

                pack_end = &gtk::Button {
                    set_icon_name: "window-close-symbolic",
                    set_tooltip_text: Some(&fl!(I18N, "close")),
                    add_css_class: "flat",
                    connect_clicked => move |_| APP_BROKER.send(AppMsg::HideStatsConfig),
                },
            },

            #[wrap(Some)]
            set_content = &adw::PreferencesPage {
                add = &adw::PreferencesGroup {
                    set_title: &fl!(I18N, "configure-stats"),
                    #[wrap(Some)]
                    set_header_suffix = &gtk::Button {
                        set_label: &fl!(I18N, "reset-button"),
                        set_tooltip_text: Some(&fl!(I18N, "reset-stats-layout")),
                        connect_clicked => StatsConfigPanelMsg::ResetLayout,
                    },

                    #[local_ref]
                    stat_rows -> gtk::ListBox {
                        set_selection_mode: gtk::SelectionMode::None,
                        add_css_class: css::BOXED_LIST,
                    },
                },
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            page: StatsPage::OcPage,
            layout: StatsPage::OcPage.default_layout(),
            stats: None,
            rows: FactoryVecDeque::builder()
                .launch_default()
                .forward(sender.input_sender(), |msg| msg),
        };
        let stat_rows = model.rows.widget();
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            StatsConfigPanelMsg::Show { page, stats } => {
                self.page = page;
                self.layout = CONFIG.read().stats_layout_for(page);
                self.stats = stats;
                self.rebuild_rows();
            }
            StatsConfigPanelMsg::SetEnabled(stat, enabled) => {
                if let Some(entry) = self.layout.0.iter_mut().find(|entry| entry.stat == stat) {
                    entry.enabled = enabled;
                    self.save();
                }
            }
            StatsConfigPanelMsg::SetDisplay(stat, display) => {
                if stat.supported_displays().contains(&display)
                    && let Some(entry) = self.layout.0.iter_mut().find(|entry| entry.stat == stat)
                {
                    entry.display = display;
                    self.save();
                }
            }
            StatsConfigPanelMsg::ResetLayout => {
                self.layout = self.page.default_layout();
                self.save();
                self.rebuild_rows();
            }
        }
    }
}

impl StatsConfigPanel {
    fn page_title(&self) -> String {
        match self.page {
            StatsPage::OcPage => fl!(I18N, "oc-page"),
            StatsPage::ThermalsPage => fl!(I18N, "thermals-page"),
        }
    }

    fn rebuild_rows(&mut self) {
        let entries = self.layout.0.clone();
        let stats = self.stats.clone();

        let mut rows = self.rows.guard();
        rows.clear();
        for entry in entries {
            rows.push_back(StatConfigRow {
                entry,
                available: stats
                    .as_deref()
                    .is_none_or(|stats| entry.stat.has_data_for(stats)),
            });
        }
    }

    fn save(&self) {
        let page = self.page;
        let layout = self.layout.clone();
        CONFIG.write().edit(|config| {
            config.stats_layout.insert(page, layout);
        });
        APP_BROKER.send(AppMsg::StatsLayoutChanged(page));
    }
}

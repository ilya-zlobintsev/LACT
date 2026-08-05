use crate::{
    CONFIG, I18N,
    app::{
        APP_BROKER,
        components::gpu_stats_section::{GpuStat, GpuStatDisplay},
        msg::AppMsg,
    },
    config::{StatsLayout, StatsPage},
};
use adw::prelude::{
    ActionRowExt, AdwDialogExt, PreferencesDialogExt, PreferencesGroupExt, PreferencesPageExt,
};
use gtk::prelude::{BoxExt, ButtonExt, ToggleButtonExt, WidgetExt};
use i18n_embed_fl::fl;
use lact_schema::DeviceStats;
use relm4::{ComponentParts, ComponentSender};
use std::sync::Arc;

pub struct StatsConfigDialog {
    parent: adw::ApplicationWindow,
    page: StatsPage,
    layout: StatsLayout,
    stats: Option<Arc<DeviceStats>>,
    rows: Vec<adw::ActionRow>,
}

#[derive(Debug)]
pub enum StatsConfigDialogMsg {
    Show {
        page: StatsPage,
        stats: Option<Arc<DeviceStats>>,
    },
    SetEnabled(GpuStat, bool),
    SetDisplay(GpuStat, GpuStatDisplay),
    Default,
}

#[relm4::component(pub)]
impl relm4::Component for StatsConfigDialog {
    type Init = adw::ApplicationWindow;
    type Input = StatsConfigDialogMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        adw::PreferencesDialog {
            #[watch]
            set_title: &model.page_title(),

            add = &adw::PreferencesPage {
                #[name = "stats_group"]
                add = &adw::PreferencesGroup {
                    set_title: &fl!(I18N, "configure-stats"),
                    #[wrap(Some)]
                    set_header_suffix = &gtk::Button {
                        set_label: &fl!(I18N, "default-button"),
                        set_tooltip_text: Some(&fl!(I18N, "reset-stats-layout")),
                        add_css_class: "destructive-action",
                        connect_clicked => StatsConfigDialogMsg::Default,
                    },
                },
            },
        }
    }

    fn init(
        parent: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            parent,
            page: StatsPage::OcPage,
            layout: StatsPage::OcPage.default_layout(),
            stats: None,
            rows: Vec::new(),
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
            StatsConfigDialogMsg::Show { page, stats } => {
                self.page = page;
                self.layout = page.layout();
                self.stats = stats;
                self.rebuild_rows(&widgets.stats_group, &sender);
                root.present(Some(&self.parent));
            }
            StatsConfigDialogMsg::SetEnabled(stat, enabled) => {
                if let Some(entry) = self.layout.0.iter_mut().find(|entry| entry.stat == stat) {
                    entry.enabled = enabled;
                    self.save();
                }
            }
            StatsConfigDialogMsg::SetDisplay(stat, display) => {
                if stat.supported_displays().contains(&display)
                    && let Some(entry) = self.layout.0.iter_mut().find(|entry| entry.stat == stat)
                {
                    entry.display = display;
                    self.save();
                }
            }
            StatsConfigDialogMsg::Default => {
                self.layout = self.page.default_layout();
                self.save();
                self.rebuild_rows(&widgets.stats_group, &sender);
            }
        }

        self.update_view(widgets, sender);
    }
}

impl StatsConfigDialog {
    fn page_title(&self) -> String {
        match self.page {
            StatsPage::OcPage => fl!(I18N, "oc-page"),
            StatsPage::ThermalsPage => fl!(I18N, "thermals-page"),
        }
    }

    fn rebuild_rows(&mut self, group: &adw::PreferencesGroup, sender: &ComponentSender<Self>) {
        for row in self.rows.drain(..) {
            group.remove(&row);
        }

        for entry in &self.layout.0 {
            let row = adw::ActionRow::builder().title(entry.stat.title()).build();
            if self
                .stats
                .as_deref()
                .is_some_and(|stats| !entry.stat.has_data_for(stats))
            {
                row.set_subtitle(&fl!(I18N, "stat-not-available"));
            }

            let enabled_switch = gtk::Switch::builder()
                .active(entry.enabled)
                .valign(gtk::Align::Center)
                .build();
            let stat = entry.stat;
            let input_sender = sender.input_sender().clone();
            enabled_switch.connect_active_notify(move |switch| {
                let _ =
                    input_sender.send(StatsConfigDialogMsg::SetEnabled(stat, switch.is_active()));
            });
            row.add_suffix(&enabled_switch);

            if entry.stat.supported_displays().len() > 1 {
                let display_box = gtk::Box::builder()
                    .orientation(gtk::Orientation::Horizontal)
                    .valign(gtk::Align::Center)
                    .css_classes(["linked"])
                    .build();

                let text_button = gtk::ToggleButton::builder()
                    .label(fl!(I18N, "stats-display-text"))
                    .active(entry.display == GpuStatDisplay::Text)
                    .build();
                let input_sender = sender.input_sender().clone();
                text_button.connect_toggled(move |button| {
                    if button.is_active() {
                        let _ = input_sender
                            .send(StatsConfigDialogMsg::SetDisplay(stat, GpuStatDisplay::Text));
                    }
                });
                display_box.append(&text_button);

                let bar_button = gtk::ToggleButton::builder()
                    .label(fl!(I18N, "stats-display-bar"))
                    .active(entry.display == GpuStatDisplay::LevelBar)
                    .group(&text_button)
                    .build();
                let input_sender = sender.input_sender().clone();
                bar_button.connect_toggled(move |button| {
                    if button.is_active() {
                        let _ = input_sender.send(StatsConfigDialogMsg::SetDisplay(
                            stat,
                            GpuStatDisplay::LevelBar,
                        ));
                    }
                });
                display_box.append(&bar_button);
                row.add_suffix(&display_box);
            }

            group.add(&row);
            self.rows.push(row);
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

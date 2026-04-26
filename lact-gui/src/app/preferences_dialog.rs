use crate::{
    CONFIG, I18N,
    app::{APP_BROKER, msg::AppMsg, styles, styles::AppTheme},
    config::{MAX_STATS_POLL_INTERVAL_MS, MIN_STATS_POLL_INTERVAL_MS},
};
use adw::prelude::{
    ActionRowExt, AdwDialogExt, PreferencesDialogExt, PreferencesGroupExt, PreferencesPageExt,
    PreferencesRowExt,
};
use gtk::prelude::{
    ButtonExt, EditableExt, EntryExt, ListBoxRowExt, OrientableExt, ToggleButtonExt, WidgetExt,
};
use i18n_embed_fl::fl;
use lact_schema::SystemInfo;
use relm4::{ComponentParts, ComponentSender};

pub struct PreferencesDialog {
    parent: adw::ApplicationWindow,
    system_info: SystemInfo,
}

#[derive(Debug)]
pub enum PreferencesDialogMsg {
    Show,
    ThemeSelected(AppTheme),
    AlarmThresholdChanged(f64),
    AlarmNotifyToggled(bool),
    AlarmCommandChanged(String),
    AlarmCommandToggled(bool),
}

#[relm4::component(pub)]
impl relm4::Component for PreferencesDialog {
    type Init = (SystemInfo, adw::ApplicationWindow);
    type Input = PreferencesDialogMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        adw::PreferencesDialog {
            set_title: &fl!(I18N, "preferences"),

            add = &adw::PreferencesPage {
                add = &adw::PreferencesGroup {
                    set_title: &fl!(I18N, "ui"),

                    adw::ActionRow {
                        set_title: &fl!(I18N, "theme"),

                        add_suffix = &gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            add_css_class: "linked",
                            set_valign: gtk::Align::Center,

                            #[name = "theme_auto_btn"]
                            gtk::ToggleButton {
                                set_label: &fl!(I18N, "theme-auto"),
                                #[watch]
                                set_active: CONFIG.read().theme == AppTheme::Automatic,
                                connect_toggled[sender] => move |btn| {
                                    if btn.is_active() {
                                        sender.input(PreferencesDialogMsg::ThemeSelected(AppTheme::Automatic));
                                    }
                                },
                            },

                            gtk::ToggleButton {
                                set_label: "Adwaita",
                                set_group: Some(&theme_auto_btn),
                                #[watch]
                                set_active: CONFIG.read().theme == AppTheme::Adwaita,
                                connect_toggled[sender] => move |btn| {
                                    if btn.is_active() {
                                        sender.input(PreferencesDialogMsg::ThemeSelected(AppTheme::Adwaita));
                                    }
                                },
                            },

                            gtk::ToggleButton {
                                set_label: "Breeze",
                                set_group: Some(&theme_auto_btn),
                                #[watch]
                                set_active: CONFIG.read().theme == AppTheme::Breeze,
                                connect_toggled[sender] => move |btn| {
                                    if btn.is_active() {
                                        sender.input(PreferencesDialogMsg::ThemeSelected(AppTheme::Breeze));
                                    }
                                },
                            },
                        },
                    },

                    adw::ActionRow {
                        set_title: &fl!(I18N, "stats-update-interval"),

                        add_suffix = &gtk::SpinButton {
                            set_range: (MIN_STATS_POLL_INTERVAL_MS as f64, MAX_STATS_POLL_INTERVAL_MS as f64),
                            set_increments: (250.0, 500.0),
                            set_digits: 0,
                            set_width_chars: 5,
                            set_valign: gtk::Align::Center,
                            set_value: CONFIG.read().stats_poll_interval_ms as f64,
                            connect_value_changed => move |btn| {
                                CONFIG.write().edit(|config| {
                                    config.stats_poll_interval_ms = btn.value() as i64;
                                })
                            },
                        },
                    },
                },

                add = &adw::PreferencesGroup {
                    set_title: &fl!(I18N, "daemon"),

                    adw::ActionRow {
                        set_title: &fl!(I18N, "disable-amd-oc"),
                        set_activatable: true,
                        add_suffix = &gtk::Image {
                            set_icon_name: Some("go-next-symbolic"),
                        },

                        #[watch]
                        set_visible: model.system_info.amdgpu_overdrive_enabled.is_some(),

                        connect_activated => move |_| {
                            APP_BROKER.send(AppMsg::ShowOverdriveDialog);
                        },
                    },

                    adw::ActionRow {
                        set_title: &fl!(I18N, "reset-all-config"),
                        set_activatable: true,
                        add_css_class: "error",
                        add_suffix = &gtk::Image {
                            set_icon_name: Some("go-next-symbolic"),
                        },
                        connect_activated => move |_| {
                            let msg = AppMsg::ask_confirmation(
                                AppMsg::ResetConfig,
                                fl!(I18N, "reset-config"),
                                fl!(I18N, "reset-config-description"),
                                gtk::ButtonsType::YesNo,
                            );
                            APP_BROKER.send(msg);
                        },
                    },
                },

                add = &adw::PreferencesGroup {
                    set_title: &fl!(I18N, "connector-alarm-group"),

                    adw::ActionRow {
                        set_title: &fl!(I18N, "connector-alarm-threshold"),
                        set_subtitle: &fl!(I18N, "connector-alarm-threshold-subtitle"),

                        add_suffix = &gtk::SpinButton {
                            set_range: (1.0, 10.0),
                            set_increments: (0.5, 1.0),
                            set_digits: 1,
                            set_width_chars: 4,
                            set_valign: gtk::Align::Center,
                            set_value: CONFIG.read().connector_alarm.as_ref().map(|a| a.pin_current_threshold_a).unwrap_or(8.0),
                            connect_value_changed[sender] => move |btn| {
                                sender.input(PreferencesDialogMsg::AlarmThresholdChanged(btn.value()));
                            },
                        },
                    },

                    adw::SwitchRow {
                        set_title: &fl!(I18N, "connector-alarm-notify"),
                        #[watch]
                        set_active: CONFIG.read().connector_alarm.as_ref().map(|a| a.notify).unwrap_or(true),
                        connect_active_notify[sender] => move |row| {
                            sender.input(PreferencesDialogMsg::AlarmNotifyToggled(row.is_active()));
                        },
                    },

                    adw::ActionRow {
                        set_title: &fl!(I18N, "connector-alarm-command"),
                        set_subtitle: &fl!(I18N, "connector-alarm-command-subtitle"),

                        add_suffix = &gtk::Entry {
                            set_placeholder_text: Some(&fl!(I18N, "connector-alarm-command-placeholder")),
                            set_width_chars: 30,
                            set_valign: gtk::Align::Center,
                            set_text: &CONFIG.read().connector_alarm.as_ref().and_then(|a| a.command.clone()).unwrap_or_default(),
                            connect_changed[sender] => move |entry| {
                                sender.input(PreferencesDialogMsg::AlarmCommandChanged(entry.text().to_string()));
                            },
                        },
                    },
                },
            },
        }
    }

    fn init(
        (system_info, parent): Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = PreferencesDialog {
            parent,
            system_info,
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
            PreferencesDialogMsg::Show => {
                root.present(Some(&self.parent));
            }
            PreferencesDialogMsg::ThemeSelected(theme) => {
                styles::apply_theme(theme).expect("Could not apply theme");

                CONFIG.write().edit(|config| {
                    config.theme = theme;
                });
            }
            PreferencesDialogMsg::AlarmThresholdChanged(value) => {
                CONFIG.write().edit(|config| {
                    config.connector_alarm.get_or_insert_with(Default::default).pin_current_threshold_a = value;
                });
            }
            PreferencesDialogMsg::AlarmNotifyToggled(active) => {
                CONFIG.write().edit(|config| {
                    config.connector_alarm.get_or_insert_with(Default::default).notify = active;
                });
            }
            PreferencesDialogMsg::AlarmCommandChanged(cmd) => {
                CONFIG.write().edit(|config| {
                    let alarm = config.connector_alarm.get_or_insert_with(Default::default);
                    alarm.command = if cmd.is_empty() { None } else { Some(cmd) };
                });
            }
            PreferencesDialogMsg::AlarmCommandToggled(_) => {}
        }
        self.update_view(widgets, sender);
    }
}

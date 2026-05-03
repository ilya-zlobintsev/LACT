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
    ButtonExt, EditableExt, ListBoxRowExt, OrientableExt, ToggleButtonExt, WidgetExt,
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
                            APP_BROKER.send(AppMsg::ResetConfig);
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
        }
        self.update_view(widgets, sender);
    }
}

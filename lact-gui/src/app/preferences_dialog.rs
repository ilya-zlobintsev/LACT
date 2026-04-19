use crate::{
    CONFIG, I18N,
    app::{APP_BROKER, msg::AppMsg, styles, styles::AppTheme},
};
use adw::prelude::{
    ActionRowExt, AdwDialogExt, PreferencesDialogExt, PreferencesGroupExt, PreferencesPageExt,
    PreferencesRowExt,
};
use gtk::prelude::{
    ButtonExt, ListBoxRowExt, OrientableExt, ToggleButtonExt, WidgetExt,
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
        #[root]
        adw::PreferencesDialog {
            set_title: &fl!(I18N, "preferences"),

            add = &adw::PreferencesPage {
                add = &adw::PreferencesGroup {
                    set_title: &fl!(I18N, "theme"),

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
                },

                add = &adw::PreferencesGroup {
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

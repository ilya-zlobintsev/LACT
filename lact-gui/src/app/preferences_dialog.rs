use crate::{
    CONFIG, I18N, Localizations,
    app::{
        APP_BROKER,
        msg::AppMsg,
        utils::{
            color_scheme::AppColorScheme,
            styles::{self, AppTheme},
        },
    },
    config::{MAX_STATS_POLL_INTERVAL_MS, MIN_STATS_POLL_INTERVAL_MS},
};
use adw::prelude::{
    ActionRowExt, AdwDialogExt, ComboRowExt, PreferencesDialogExt, PreferencesGroupExt,
    PreferencesPageExt, PreferencesRowExt,
};
use gtk::prelude::{
    ButtonExt, EditableExt, ListBoxRowExt, OrientableExt, ToggleButtonExt, WidgetExt,
};
use i18n_embed::LanguageLoader;
use i18n_embed_fl::fl;
use lact_schema::SystemInfo;
use relm4::{ComponentParts, ComponentSender};

pub struct PreferencesDialog {
    parent: adw::ApplicationWindow,
    system_info: SystemInfo,
    languages: Vec<String>,
}

#[derive(Debug)]
pub enum PreferencesDialogMsg {
    Show,
    ThemeSelected(AppTheme),
    ColorSchemeSelected(AppColorScheme),
    LanguageSelected(u32),
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

                    adw::ComboRow {
                        set_title: &fl!(I18N, "language"),
                        set_subtitle: &fl!(I18N, "language-restart-notice"),
                        set_model: Some(&language_names),
                        set_selected: selected_language,
                        connect_selected_notify[sender] => move |row| {
                            sender.input(PreferencesDialogMsg::LanguageSelected(row.selected()));
                        },
                    },

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
                        set_title: &fl!(I18N, "color-scheme"),

                        add_suffix = &gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            add_css_class: "linked",
                            set_valign: gtk::Align::Center,

                            #[name = "color_scheme_auto_btn"]
                            gtk::ToggleButton {
                                set_label: &fl!(I18N, "color-scheme-auto"),
                                #[watch]
                                set_active: CONFIG.read().color_scheme == AppColorScheme::Auto,
                                connect_toggled[sender] => move |btn| {
                                    if btn.is_active() {
                                        sender.input(PreferencesDialogMsg::ColorSchemeSelected(AppColorScheme::Auto));
                                    }
                                },
                            },

                            gtk::ToggleButton {
                                set_label: &fl!(I18N, "color-scheme-light"),
                                set_group: Some(&color_scheme_auto_btn),
                                #[watch]
                                set_active: CONFIG.read().color_scheme == AppColorScheme::Light,
                                connect_toggled[sender] => move |btn| {
                                    if btn.is_active() {
                                        sender.input(PreferencesDialogMsg::ColorSchemeSelected(AppColorScheme::Light));
                                    }
                                },
                            },

                            gtk::ToggleButton {
                                set_label: &fl!(I18N, "color-scheme-dark"),
                                set_group: Some(&color_scheme_auto_btn),
                                #[watch]
                                set_active: CONFIG.read().color_scheme == AppColorScheme::Dark,
                                connect_toggled[sender] => move |btn| {
                                    if btn.is_active() {
                                        sender.input(PreferencesDialogMsg::ColorSchemeSelected(AppColorScheme::Dark));
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
        let (languages, language_names, selected_language) = build_language_list();
        let model = PreferencesDialog {
            parent,
            system_info,
            languages,
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
            PreferencesDialogMsg::LanguageSelected(index) => {
                let language = if index == 0 {
                    None
                } else if let Some(language) = self.languages.get(index as usize - 1) {
                    Some(language.clone())
                } else {
                    return;
                };
                CONFIG.write().edit(|config| config.language = language);
            }
            PreferencesDialogMsg::Show => {
                root.present(Some(&self.parent));
            }
            PreferencesDialogMsg::ThemeSelected(theme) => {
                styles::apply_theme(theme).expect("Could not apply theme");

                CONFIG.write().edit(|config| {
                    config.theme = theme;
                });
            }
            PreferencesDialogMsg::ColorSchemeSelected(scheme) => {
                scheme.apply();
                CONFIG.write().edit(|config| {
                    config.color_scheme = scheme;
                });

                styles::apply_theme(CONFIG.read().theme).expect("Could not apply theme");
            }
        }
        self.update_view(widgets, sender);
    }
}

fn build_language_list() -> (Vec<String>, gtk::StringList, u32) {
    let languages: Vec<String> = I18N
        .available_languages(&Localizations)
        .expect("Could not list GUI languages")
        .into_iter()
        .map(|language| language.to_string())
        .collect();

    let selected_language = CONFIG
        .read()
        .language
        .as_ref()
        .and_then(|language| languages.iter().position(|id| id == language))
        .map_or(0, |index| index as u32 + 1);
    let language_names = gtk::StringList::new(&[&fl!(I18N, "language-system-default")]);
    for language in &languages {
        language_names.append(language_name(language));
    }

    (languages, language_names, selected_language)
}

// have to be manually updated if new languages are added
fn language_name(id: &str) -> &str {
    match id {
        "ar" => "العربية",
        "ca" => "Català",
        "cs" => "Čeština",
        "de" => "Deutsch",
        "en" => "English",
        "es" => "Español",
        "fi" => "Suomi",
        "fr" => "Français",
        "fur" => "Furlan",
        "he" => "עברית",
        "hu" => "Magyar",
        "id" => "Bahasa Indonesia",
        "it" => "Italiano",
        "ka" => "ქართული",
        "kab" => "Taqbaylit",
        "ko" => "한국어",
        "lo" => "ລາວ",
        "pl" => "Polski",
        "pt-BR" => "Português (Brasil)",
        "ru" => "Русский",
        "sr" => "Српски",
        "th" => "ไทย",
        "tr" => "Türkçe",
        "uk" => "Українська",
        "zh-Hans" => "简体中文",
        "zh-Hant" => "繁體中文",
        _ => id,
    }
}

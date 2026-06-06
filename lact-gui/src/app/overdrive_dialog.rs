use crate::{
    I18N,
    app::{APP_BROKER, msg::AppMsg},
};
use adw::prelude::*;
use gtk::pango;
use i18n_embed_fl::fl;
use lact_daemon::BASE_MODULE_CONF_PATH;
use lact_schema::{AmdgpuParamsConfigurator, BootArgConfigurator, SystemInfo};
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt};

pub struct OverdriveDialog {
    pub system_info: SystemInfo,
    pub parent: gtk::Widget,
    pub is_loading: bool,
    pub is_done: bool,
}

#[derive(Debug)]
pub enum OverdriveDialogMsg {
    Show,
    Loading,
    Loaded,
}

#[relm4::component(pub)]
impl relm4::Component for OverdriveDialog {
    type Init = (SystemInfo, gtk::Widget);
    type Input = OverdriveDialogMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        adw::Dialog {
            set_content_width: 500,
            set_follows_content_size: true,
            set_title: &fl!(I18N, "amd-oc"),

            #[wrap(Some)]
            set_child = &adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {},

                #[wrap(Some)]
                set_content = &adw::PreferencesPage {
                    add = &adw::PreferencesGroup {
                        gtk::Label {
                            set_xalign: 0.0,
                            set_markup: &fl!(
                                I18N,
                                "amd-oc-description",
                                config = format_config(&model.system_info)
                                path = BASE_MODULE_CONF_PATH,
                            ),
                            set_selectable: true,
                            set_can_focus: false,
                            set_wrap: true,
                            set_wrap_mode: pango::WrapMode::Word,
                        },
                    },

                    add = &adw::PreferencesGroup {
                        adw::ActionRow {
                            set_title_lines: 0,
                            set_use_markup: true,
                            set_title: &fl!(
                                I18N,
                                "amd-oc-status",
                                status = model
                                    .system_info
                                    .amdgpu_overdrive_enabled
                                    .map(|s| s.to_string())
                                    .unwrap_or_default(),
                            ),
                        },

                        adw::ActionRow {
                            set_title_lines: 0,
                            set_use_markup: true,
                            set_title: &fl!(I18N, "amd-oc-detected-system-config", config = format_config(&model.system_info)),
                        },

                        adw::ActionRow {
                            #[watch]
                            set_visible: model.is_loading || model.is_done,
                            #[watch]
                            set_title: &if model.is_done {
                                fl!(I18N, "amd-oc-updating-done")
                            } else {
                                fl!(I18N, "amd-oc-updating-configuration")
                            },

                            add_prefix = &gtk::Spinner {
                                #[watch]
                                set_visible: model.is_loading,
                                set_spinning: true,
                            },

                            add_prefix = &gtk::Image {
                                #[watch]
                                set_visible: model.is_done,
                                set_icon_name: Some("emblem-ok-symbolic"),
                                add_css_class: "success",
                            },
                        },
                    },
                },

                add_bottom_bar = &gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 10,
                    set_halign: gtk::Align::Fill,
                    set_margin_horizontal: 20,
                    set_margin_bottom: 20,

                    gtk::Button {
                        #[watch]
                        set_sensitive: model.system_info.amdgpu_overdrive_enabled == Some(true) && !model.is_loading && !model.is_done,
                        set_label: &fl!(I18N, "disable-amd-oc"),
                        add_css_class: "destructive-action",
                        set_hexpand: true,
                        connect_clicked => move |_| {
                            APP_BROKER.send(AppMsg::DisableOverdrive);
                        },
                    },

                    gtk::Button {
                        #[watch]
                        set_sensitive: model.system_info.amdgpu_overdrive_enabled == Some(false) && !model.is_loading && !model.is_done,
                        set_label: &fl!(I18N, "enable-amd-oc"),
                        add_css_class: "suggested-action",
                        set_hexpand: true,
                        connect_clicked => move |_| {
                            APP_BROKER.send(AppMsg::EnableOverdrive);
                        },
                    },
                },
            },
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let (system_info, parent) = init;
        let model = Self {
            system_info,
            parent,
            is_loading: false,
            is_done: false,
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
            OverdriveDialogMsg::Show => root.present(Some(&self.parent)),
            OverdriveDialogMsg::Loading => self.is_loading = true,
            OverdriveDialogMsg::Loaded => {
                self.is_loading = false;
                self.is_done = true;
            }
        }

        self.update_view(widgets, sender);
    }
}

fn format_config(system_info: &SystemInfo) -> String {
    system_info
        .amdgpu_params_configurator
        .and_then(|configurator| match configurator {
            AmdgpuParamsConfigurator::Modprobe(Some(initramfs)) => {
                Some(format!("Modprobe (Initramfs: {initramfs:?})"))
            }
            AmdgpuParamsConfigurator::Modprobe(None) => None,
            AmdgpuParamsConfigurator::BootArg(BootArgConfigurator::RpmOstree) => {
                Some("rpm-ostree".to_owned())
            }
        })
        .unwrap_or_else(|| "unsupported".to_owned())
}

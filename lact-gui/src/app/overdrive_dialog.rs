use crate::{
    app::{msg::AppMsg, APP_BROKER},
    I18N,
};
use gtk::{
    pango,
    prelude::{BoxExt, ButtonExt, GtkWindowExt, OrientableExt, WidgetExt},
};
use i18n_embed_fl::fl;
use lact_daemon::BASE_MODULE_CONF_PATH;
use lact_schema::{AmdgpuParamsConfigurator, BootArgConfigurator, SystemInfo};
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt};

pub struct OverdriveDialog {
    pub system_info: SystemInfo,
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
    type Init = Self;
    type Input = OverdriveDialogMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk::Window {
            set_default_size: (300, 150),
            set_title: Some(&fl!(I18N, "amd-oc")),
            set_hide_on_close: true,

            gtk::Box {
                set_spacing: 10,
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 10,

                gtk::Label {
                    set_xalign: 0.0,
                    set_markup: &fl!(
                        I18N,
                        "amd-oc-description",
                        config = format_config(&model.system_info)
                        path = BASE_MODULE_CONF_PATH,
                    ),
                    set_selectable: true,
                    set_hexpand: true,
                    set_wrap: true,
                    set_wrap_mode: pango::WrapMode::Word,
                },

                gtk::Label {
                    set_xalign: 0.0,
                    set_hexpand: true,
                    set_markup: &fl!(
                        I18N,
                        "amd-oc-status",
                        status = model
                            .system_info
                            .amdgpu_overdrive_enabled
                            .map(|s| s.to_string())
                            .unwrap_or_default(),
                    ),
                },

                gtk::Label {
                    set_xalign: 0.0,
                    set_hexpand: true,
                    set_markup: &fl!(I18N, "amd-oc-detected-system-config", config = format_config(&model.system_info)),
                },

                gtk::Box {
                    set_spacing: 10,
                    set_orientation: gtk::Orientation::Horizontal,

                    gtk::Button {
                        #[watch]
                        set_sensitive: model.system_info.amdgpu_overdrive_enabled == Some(false) && !model.is_loading && !model.is_done,
                        set_label: &fl!(I18N, "enable-amd-oc"),
                        connect_clicked => move |_| {
                            APP_BROKER.send(AppMsg::EnableOverdrive);
                        },
                    },

                    gtk::Button {
                        #[watch]
                        set_sensitive: model.system_info.amdgpu_overdrive_enabled == Some(true) && !model.is_loading && !model.is_done,
                        set_label: &fl!(I18N, "disable-amd-oc"),
                        connect_clicked => move |_| {
                            APP_BROKER.send(AppMsg::DisableOverdrive);
                        },
                    },
                },

                gtk::Label {
                    #[watch]
                    set_visible: model.is_loading,
                    set_label: &fl!(I18N, "amd-oc-updating-configuration"),
                },

                gtk::Label {
                    #[watch]
                    set_visible: model.is_done,
                    set_label: &fl!(I18N, "amd-oc-updating-done"),
                },
            },
        }
    }

    fn init(
        model: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
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
            OverdriveDialogMsg::Show => root.present(),
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

use crate::{APP_BROKER, app::msg::AppMsg};
use amdgpu_sysfs::gpu_handle::PowerLevelId;
use gtk::prelude::{BoxExt, ListBoxRowExt, OrientableExt, WidgetExt};
use lact_schema::PowerState;
use relm4::{RelmObjectExt, RelmWidgetExt, binding::BoolBinding, css};

pub struct PowerStateRow {
    active: BoolBinding,
    pub(super) enabled: BoolBinding,
    pub(super) power_state: PowerState,
    value_suffix: String,
    value_ratio: f64,
    configurable: BoolBinding,
    show_active_indicator: BoolBinding,
}

pub struct PowerStateRowOptions {
    pub power_state: PowerState,
    pub value_suffix: String,
    pub value_ratio: f64,
    pub active: bool,
    pub show_active_indicator: BoolBinding,
    pub configurable: BoolBinding,
}

#[derive(Clone, Debug)]
pub enum PowerStateRowMsg {
    Active(bool),
    ValueRatio(f64),
}

#[relm4::factory(pub)]
impl relm4::factory::FactoryComponent for PowerStateRow {
    type ParentWidget = gtk::ListBox;
    type CommandOutput = ();
    type Input = PowerStateRowMsg;
    type Output = ();
    type Init = PowerStateRowOptions;

    view! {
        gtk::ListBoxRow {
            set_selectable: false,
            set_activatable: false,
            set_focusable: false,

            #[wrap(Some)]
            set_child = &gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 5,
                set_margin_vertical: 2,
                set_margin_horizontal: 5,

                append: image = &gtk::Image {
                    set_icon_name: Some("pan-end-symbolic"),
                    add_binding: (&self.show_active_indicator, "visible"),
                    #[watch]
                    set_opacity: if self.active.value() { 1.0 } else { 0.0 },
                },

                append = &gtk::CheckButton {
                    add_binding: (&self.enabled, "active"),
                    add_binding: (&self.configurable, "visible"),
                    set_sensitive: matches!(self.power_state.id, Some(PowerLevelId::Index(_))),
                },

                append = &gtk::Label {
                    add_css_class: css::MONOSPACE,
                    #[watch]
                    set_class_active: (css::DIM_LABEL, !self.active.value()),
                    set_label: &match self.power_state.id {
                        Some(PowerLevelId::Index(index)) => format!("{index}:"),
                        Some(PowerLevelId::Sleep) => "S:".to_owned(),
                        None => String::new(),
                    },
                },

                append = &gtk::Label {
                    add_css_class: css::MONOSPACE,
                    set_hexpand: true,
                    set_xalign: 1.0,
                    #[watch]
                    set_class_active: (css::DIM_LABEL, !self.active.value()),
                    #[watch]
                    set_label: &{
                        let value = (self.power_state.value as f64 * self.value_ratio) as u64;
                        let value_text = match self.power_state.min_value {
                            Some(min) if min != self.power_state.value => {
                                let min = (min as f64 * self.value_ratio) as u64;
                                format!("{min}-{value}")
                            }
                            _ => value.to_string(),
                        };
                        format!("{value_text} {}", self.value_suffix)
                    },
                },
            },
        }
    }

    fn init_model(
        opts: Self::Init,
        _index: &Self::Index,
        _sender: relm4::FactorySender<Self>,
    ) -> Self {
        let enabled = BoolBinding::new(opts.power_state.enabled);
        enabled.connect_value_notify(|_| APP_BROKER.send(AppMsg::SettingsChanged));

        Self {
            enabled,
            active: BoolBinding::new(opts.active),
            power_state: opts.power_state,
            value_suffix: opts.value_suffix,
            value_ratio: opts.value_ratio,
            configurable: opts.configurable,
            show_active_indicator: opts.show_active_indicator,
        }
    }

    fn update(&mut self, msg: Self::Input, _: relm4::FactorySender<Self>) {
        match msg {
            PowerStateRowMsg::Active(active) => self.active.set_value(active),
            PowerStateRowMsg::ValueRatio(ratio) => self.value_ratio = ratio,
        }
    }
}

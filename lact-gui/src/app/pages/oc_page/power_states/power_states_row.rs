use crate::{APP_BROKER, app::msg::AppMsg};
use gtk::prelude::{BoxExt, CheckButtonExt, OrientableExt, WidgetExt};
use lact_schema::PowerState;
use relm4::{RelmObjectExt, binding::BoolBinding};

pub struct PowerStateRow {
    pub(super) active: BoolBinding,
    pub(super) enabled: BoolBinding,
    pub(super) power_state: PowerState,
    pub(super) value_suffix: String,
}

pub struct PowerStateRowOptions {
    pub power_state: PowerState,
    pub value_suffix: String,
    pub active: bool,
}

#[derive(Debug)]
pub enum PowerStateRowMsg {
    Active(bool),
}

#[relm4::factory(pub)]
impl relm4::factory::FactoryComponent for PowerStateRow {
    type ParentWidget = gtk::ListBox;
    type CommandOutput = ();
    type Input = PowerStateRowMsg;
    type Output = ();
    type Init = PowerStateRowOptions;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 5,

            append = &gtk::CheckButton {
                set_hexpand: true,
                add_binding: (&self.enabled, "active"),
                set_sensitive: false,
                set_label: {
                    let value_text = match self.power_state.min_value {
                        Some(min) if min != self.power_state.value => format!("{min}-{}", self.power_state.value),
                        _ => self.power_state.value.to_string(),
                    };
                    Some(format!("{}: {value_text} {}", index.current_index(), self.value_suffix))
                }.as_deref(),
            },

            append: image = &gtk::Image {
                set_icon_name: Some("pan-start-symbolic"),
                add_binding: (&self.active, "visible"),
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
        }
    }

    fn update(&mut self, msg: Self::Input, _: relm4::FactorySender<Self>) {
        match msg {
            PowerStateRowMsg::Active(active) => self.active.set_value(active),
        }
    }
}

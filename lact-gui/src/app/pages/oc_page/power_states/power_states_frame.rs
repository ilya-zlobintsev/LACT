use std::collections::HashMap;

use amdgpu_sysfs::gpu_handle::PowerLevelKind;
use gtk::{
    glib::{self, subclass::types::ObjectSubclassIsExt, Object},
    prelude::WidgetExt,
};
use lact_client::schema::{DeviceStats, PowerStates};

glib::wrapper! {
    pub struct PowerStatesFrame(ObjectSubclass<imp::PowerStatesFrame>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable;
}

impl PowerStatesFrame {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn set_power_states(&self, states: PowerStates) {
        let imp = self.imp();

        imp.expander.set_sensitive(!states.is_empty());
        if states.is_empty() {
            imp.expander.set_expanded(false);
        }

        if states
            .core
            .iter()
            .chain(states.vram.iter())
            .any(|state| !state.enabled)
        {
            self.set_configurable(true);
        }

        imp.core_states_list
            .set_power_states(states.core, "MHz", 1.0);
        imp.vram_states_list
            .set_power_states(states.vram, "MHz", self.vram_clock_ratio());
    }

    pub fn connect_values_changed<F: Fn() + 'static + Clone>(&self, f: F) {
        let imp = self.imp();
        imp.core_states_list.connect_values_changed(f.clone());
        imp.vram_states_list.connect_values_changed(f);
    }

    pub fn set_stats(&self, stats: &DeviceStats) {
        let imp = self.imp();

        imp.core_states_list
            .set_active_state(stats.core_power_state);
        imp.vram_states_list
            .set_active_state(stats.memory_power_state);
    }

    pub fn get_enabled_power_states(&self) -> HashMap<PowerLevelKind, Vec<u8>> {
        let core_states;
        let vram_states;

        if self.configurable() {
            let imp = self.imp();
            core_states = imp.core_states_list.get_enabled_power_states();
            vram_states = imp.vram_states_list.get_enabled_power_states();
        } else {
            core_states = vec![];
            vram_states = vec![];
        }

        [
            (PowerLevelKind::CoreClock, core_states),
            (PowerLevelKind::MemoryClock, vram_states),
        ]
        .into_iter()
        .collect()
    }
}

impl Default for PowerStatesFrame {
    fn default() -> Self {
        Self::new()
    }
}

mod imp {
    use crate::app::pages::oc_page::power_states::power_states_list::PowerStatesList;
    use gtk::{
        glib::{self, subclass::InitializingObject, types::StaticTypeExt, Properties},
        prelude::ObjectExt,
        subclass::{
            prelude::*,
            widget::{CompositeTemplateClass, WidgetImpl},
        },
        CompositeTemplate, Expander,
    };
    use std::{cell::Cell, sync::atomic::AtomicBool};

    #[derive(CompositeTemplate, Default, Properties)]
    #[properties(wrapper_type = super::PowerStatesFrame)]
    #[template(file = "ui/oc_page/power_states_frame.blp")]
    pub struct PowerStatesFrame {
        #[template_child]
        pub expander: TemplateChild<Expander>,
        #[template_child]
        pub core_states_list: TemplateChild<PowerStatesList>,
        #[template_child]
        pub vram_states_list: TemplateChild<PowerStatesList>,

        #[property(get, set)]
        configurable: AtomicBool,
        #[property(get, set)]
        toggleable: AtomicBool,
        #[property(get, set)]
        vram_clock_ratio: Cell<f64>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PowerStatesFrame {
        const NAME: &'static str = "PowerStatesFrame";
        type Type = super::PowerStatesFrame;
        type ParentType = gtk::Box;

        fn class_init(class: &mut Self::Class) {
            PowerStatesList::ensure_type();
            class.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for PowerStatesFrame {}

    impl WidgetImpl for PowerStatesFrame {}
    impl BoxImpl for PowerStatesFrame {}
}

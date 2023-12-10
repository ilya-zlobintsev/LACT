use std::collections::HashMap;

use crate::app::page_section::PageSection;
use gtk::glib::{self, subclass::types::ObjectSubclassIsExt, Object};
use lact_client::schema::{amdgpu_sysfs::gpu_handle::PowerLevelKind, DeviceStats, PowerStates};

glib::wrapper! {
    pub struct PowerStatesFrame(ObjectSubclass<imp::PowerStatesFrame>)
        @extends PageSection, gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable;
}

impl PowerStatesFrame {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn set_power_states(&self, states: PowerStates) {
        let imp = self.imp();

        imp.core_states_list.set_power_states(states.core, "MHz");
        imp.vram_states_list.set_power_states(states.vram, "MHz");
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
        let imp = self.imp();
        let core_states = imp.core_states_list.get_enabled_power_states();
        let vram_states = imp.vram_states_list.get_enabled_power_states();

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
    use crate::app::{
        page_section::PageSection,
        root_stack::oc_page::power_states::power_states_list::PowerStatesList,
    };
    use gtk::{
        glib::{self, subclass::InitializingObject, Properties, StaticTypeExt},
        prelude::ObjectExt,
        subclass::{
            prelude::*,
            widget::{CompositeTemplateClass, WidgetImpl},
        },
        CompositeTemplate,
    };
    use std::sync::atomic::AtomicBool;

    #[derive(CompositeTemplate, Default, Properties)]
    #[properties(wrapper_type = super::PowerStatesFrame)]
    #[template(file = "ui/oc_page/power_states_frame.blp")]
    pub struct PowerStatesFrame {
        #[template_child]
        pub core_states_list: TemplateChild<PowerStatesList>,
        #[template_child]
        pub vram_states_list: TemplateChild<PowerStatesList>,

        #[property(get, set)]
        configurable: AtomicBool,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PowerStatesFrame {
        const NAME: &'static str = "PowerStatesFrame";
        type Type = super::PowerStatesFrame;
        type ParentType = PageSection;

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

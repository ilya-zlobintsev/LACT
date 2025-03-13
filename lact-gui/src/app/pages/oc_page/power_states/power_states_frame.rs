use std::collections::HashMap;

use amdgpu_sysfs::gpu_handle::PowerLevelKind;
use gtk::{
    glib::{self, subclass::types::ObjectSubclassIsExt, Object},
    prelude::WidgetExt,
};
use lact_client::schema::{DeviceStats, PowerStates};

glib::wrapper! {
    pub struct PowerStatesFrame(ObjectSubclass<imp::PowerStatesFrame>)
        @extends gtk::Widget, gtk::Box,
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
        glib::{self, Properties},
        prelude::{BoxExt, CheckButtonExt, ObjectExt, OrientableExt, WidgetExt},
        subclass::{prelude::*, widget::WidgetImpl},
        Expander,
    };
    use relm4::{view, RelmWidgetExt};
    use std::{cell::Cell, sync::atomic::AtomicBool};

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::PowerStatesFrame)]
    pub struct PowerStatesFrame {
        pub expander: Expander,
        pub core_states_list: PowerStatesList,
        pub vram_states_list: PowerStatesList,

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
    }

    #[glib::derived_properties]
    impl ObjectImpl for PowerStatesFrame {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = &*self.obj();
            let expander = &self.expander;
            let core_states_list = &self.core_states_list;
            let vram_states_list = &self.vram_states_list;

            view! {
                #[local_ref]
                obj {
                    #[local_ref]
                    append = expander {
                        set_label: Some("Power states"),

                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_margin_all: 10,
                            set_spacing: 5,

                            gtk::Label {
                                set_label: "Note: performance level must be set to 'manual' to toggle power states",
                                set_margin_horizontal: 10,
                                set_halign: gtk::Align::Start,
                            },

                            #[name = "enable_checkbutton"]
                            gtk::CheckButton {
                                set_label: Some("Enable power state configuration"),
                            },

                            gtk::Box {
                                set_spacing: 10,
                                set_orientation: gtk::Orientation::Horizontal,

                                #[local_ref]
                                append = core_states_list {
                                    set_title: "GPU power states",
                                    // set_sensitive: bind template.configurable;
                                },

                                #[local_ref]
                                append = vram_states_list {
                                    set_title: "VRAM power states",
                                    // sensitive: bind template.configurable;
                                },
                            }
                        }
                    }
                }
            };

            obj.bind_property("toggleable", &enable_checkbutton, "sensitive")
                .sync_create()
                .build();
            obj.bind_property("configurable", &enable_checkbutton, "active")
                .bidirectional()
                .sync_create()
                .build();

            obj.bind_property("configurable", core_states_list, "sensitive")
                .sync_create()
                .build();
            obj.bind_property("configurable", vram_states_list, "sensitive")
                .sync_create()
                .build();
        }
    }

    impl WidgetImpl for PowerStatesFrame {}
    impl BoxImpl for PowerStatesFrame {}
}

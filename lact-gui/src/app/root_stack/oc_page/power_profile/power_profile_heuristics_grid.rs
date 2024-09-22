use amdgpu_sysfs::gpu_handle::power_profile_mode::{PowerProfileComponent, PowerProfileModesTable};
use gtk::{
    glib::{self, clone, Object},
    prelude::{AdjustmentExt, BoxExt, CheckButtonExt, GridExt, WidgetExt},
    subclass::prelude::ObjectSubclassIsExt,
    Adjustment, CheckButton, Label, SpinButton,
};

glib::wrapper! {
    pub struct PowerProfileHeuristicsGrid(ObjectSubclass<imp::PowerProfileHeuristicsGrid>)
        @extends gtk::Grid,
        @implements gtk::Accessible, gtk::Buildable, gtk::Widget;
}

impl PowerProfileHeuristicsGrid {
    pub fn new() -> Self {
        Object::builder()
            .property("margin-start", 5)
            .property("margin-end", 5)
            .property("margin-top", 5)
            .property("margin-bottom", 5)
            .build()
    }

    pub fn connect_component_values_changed<F: Fn() + 'static + Clone>(&self, f: F) {
        for (adj, toggle_button) in self.imp().adjustments.borrow().iter() {
            adj.connect_value_changed(clone!(
                #[strong]
                f,
                move |_| f()
            ));
            toggle_button.connect_toggled(clone!(
                #[strong]
                f,
                move |_| f()
            ));
        }
    }

    pub fn set_component(&self, component: &PowerProfileComponent, table: &PowerProfileModesTable) {
        self.imp().component.replace(component.clone());

        while let Some(child) = self.first_child() {
            self.remove(&child);
        }

        let mut adjustments = Vec::with_capacity(component.values.len());

        for (i, value) in component.values.iter().enumerate() {
            let name = &table.value_names[i];

            let name_label = Label::builder()
                .label(format!("{name}:"))
                .hexpand(true)
                .halign(gtk::Align::Start)
                .build();

            self.attach(&name_label, 0, i as i32, 1, 1);

            let value_box = gtk::Box::new(gtk::Orientation::Horizontal, 5);
            value_box.set_hexpand(true);

            let value_checkbutton = CheckButton::new();

            let adj = Adjustment::new(0.0, f64::MIN, f64::MAX, 1.0, 1.0, 1.0);
            let value_spinbutton = SpinButton::new(Some(&adj), 1.0, 0);

            value_checkbutton.connect_toggled(clone!(
                #[strong]
                value_spinbutton,
                #[strong(rename_to = this)]
                self,
                move |check| {
                    this.imp().update_values(&value_spinbutton, check, i);
                }
            ));
            value_spinbutton.connect_value_changed(clone!(
                #[strong]
                value_checkbutton,
                #[strong(rename_to = this)]
                self,
                move |spin_button| {
                    this.imp().update_values(spin_button, &value_checkbutton, i);
                }
            ));

            value_box.append(&value_checkbutton);
            value_box.append(&value_spinbutton);

            value_checkbutton.set_active(value.is_some());
            if let Some(value) = value {
                adj.set_value(*value as f64);
            }

            self.imp()
                .update_values(&value_spinbutton, &value_checkbutton, i);

            self.attach(&value_box, 1, i as i32, 1, 1);

            adjustments.push((adj, value_checkbutton));
        }

        *self.imp().adjustments.borrow_mut() = adjustments;
    }
}

impl Default for PowerProfileHeuristicsGrid {
    fn default() -> Self {
        Self::new()
    }
}

mod imp {
    use amdgpu_sysfs::gpu_handle::power_profile_mode::PowerProfileComponent;
    use gtk::{
        glib::{self, subclass::InitializingObject},
        prelude::{CheckButtonExt, WidgetExt},
        subclass::{
            prelude::*,
            widget::{CompositeTemplateClass, WidgetImpl},
        },
        Adjustment, CheckButton, CompositeTemplate, SpinButton,
    };
    use std::{cell::RefCell, rc::Rc};

    #[derive(CompositeTemplate, Default)]
    #[template(file = "ui/oc_page/power_profile/power_profile_heuristics_grid.blp")]
    pub struct PowerProfileHeuristicsGrid {
        pub component: Rc<RefCell<PowerProfileComponent>>,
        pub(super) adjustments: Rc<RefCell<Vec<(Adjustment, CheckButton)>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PowerProfileHeuristicsGrid {
        const NAME: &'static str = "PowerProfileHeuristicsGrid";
        type Type = super::PowerProfileHeuristicsGrid;
        type ParentType = gtk::Grid;

        fn class_init(class: &mut Self::Class) {
            class.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PowerProfileHeuristicsGrid {}

    impl WidgetImpl for PowerProfileHeuristicsGrid {}
    impl GridImpl for PowerProfileHeuristicsGrid {}

    impl PowerProfileHeuristicsGrid {
        pub fn update_values(&self, spin_button: &SpinButton, check: &CheckButton, i: usize) {
            let mut component = self.component.borrow_mut();

            spin_button.set_sensitive(check.is_active());

            if check.is_active() {
                component.values[i] = Some(spin_button.value() as i32);
            } else {
                component.values[i] = None;
            }
        }
    }
}

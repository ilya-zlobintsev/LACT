use amdgpu_sysfs::gpu_handle::power_profile_mode::{PowerProfileComponent, PowerProfileModesTable};
use gtk::{
    glib::{self, Object},
    prelude::GridExt,
    prelude::WidgetExt,
    Label,
};

glib::wrapper! {
    pub struct PowerProfileComponentGrid(ObjectSubclass<imp::PowerProfileComponentGrid>)
        @extends gtk::Grid,
        @implements gtk::Accessible, gtk::Buildable, gtk::Widget;
}

impl PowerProfileComponentGrid {
    pub fn new() -> Self {
        Object::builder()
            .property("margin-start", 5)
            .property("margin-end", 5)
            .property("margin-top", 5)
            .property("margin-bottom", 5)
            .build()
    }

    pub fn set_component(&self, component: &PowerProfileComponent, table: &PowerProfileModesTable) {
        while let Some(child) = self.first_child() {
            self.remove(&child);
        }

        for (i, value) in component.values.iter().enumerate() {
            let name = &table.value_names[i];

            let name_label = Label::builder()
                .label(&format!("{name}:"))
                .hexpand(true)
                .halign(gtk::Align::Start)
                .build();

            self.attach(&name_label, 0, i as i32, 1, 1);

            let mut value_label_builder = Label::builder().halign(gtk::Align::End);
            value_label_builder = match value {
                Some(value) => value_label_builder.label(value.to_string()),
                None => value_label_builder.label("Not set"),
            };
            self.attach(&value_label_builder.build(), 1, i as i32, 1, 1);
        }
    }
}

impl Default for PowerProfileComponentGrid {
    fn default() -> Self {
        Self::new()
    }
}

mod imp {
    use gtk::{
        glib::{self, subclass::InitializingObject},
        subclass::{
            prelude::*,
            widget::{CompositeTemplateClass, WidgetImpl},
        },
        CompositeTemplate,
    };

    #[derive(CompositeTemplate, Default)]
    #[template(file = "ui/oc_page/power_profile/power_profile_component_grid.blp")]
    pub struct PowerProfileComponentGrid {}

    #[glib::object_subclass]
    impl ObjectSubclass for PowerProfileComponentGrid {
        const NAME: &'static str = "PowerProfileComponentGrid";
        type Type = super::PowerProfileComponentGrid;
        type ParentType = gtk::Grid;

        fn class_init(class: &mut Self::Class) {
            class.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PowerProfileComponentGrid {}

    impl WidgetImpl for PowerProfileComponentGrid {}
    impl GridImpl for PowerProfileComponentGrid {}
}

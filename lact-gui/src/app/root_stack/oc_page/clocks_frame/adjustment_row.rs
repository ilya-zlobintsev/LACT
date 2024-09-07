use gtk::{
    glib::{self, Object},
    prelude::WidgetExt,
    prelude::{GridExt, ObjectExt},
    subclass::prelude::ObjectSubclassIsExt,
    Grid,
};
use std::sync::atomic::Ordering;

glib::wrapper! {
    pub struct AdjustmentRow(ObjectSubclass<imp::AdjustmentRow>)
        @extends gtk::Widget,
        @implements gtk::Orientable, gtk::Accessible, gtk::Buildable;
}

impl AdjustmentRow {
    pub fn new(title: &str) -> Self {
        Object::builder()
            .property("title", title)
            .property("visible", true)
            .property("value_ratio", 1.0)
            .build()
    }

    pub fn new_and_attach(title: &str, grid: &Grid, row: i32) -> Self {
        let adj_row = Self::new(title);
        adj_row.attach_to_grid(grid, row);
        adj_row
    }

    pub fn get_value(&self) -> Option<i32> {
        self.imp()
            .adjustment
            .get_changed_value(false)
            .map(|value| value as i32)
    }

    pub fn attach_to_grid(&self, grid: &Grid, row: i32) {
        let obj = self.imp();

        obj.label.unparent();
        obj.scale.unparent();
        obj.value_button.unparent();

        grid.attach(&obj.label.get(), 0, row, 1, 1);
        grid.attach(&obj.scale.get(), 1, row, 4, 1);
        grid.attach(&obj.value_button.get(), 6, row, 4, 1);
    }

    pub fn refresh(&self) {
        let obj = self.imp();
        obj.adjustment.emit_by_name::<()>("value-changed", &[]);
        self.notify("visible");
        obj.adjustment.imp().changed.store(false, Ordering::SeqCst);
    }
}

mod imp {
    use crate::app::root_stack::oc_adjustment::OcAdjustment;
    use glib::{clone, subclass::InitializingObject};
    use gtk::{
        glib::{self, Properties},
        prelude::*,
        subclass::{
            prelude::*,
            widget::{CompositeTemplateClass, WidgetImpl},
        },
        CompositeTemplate, Label, MenuButton, Scale, SpinButton, TemplateChild,
    };
    use std::{cell::RefCell, rc::Rc};

    #[derive(CompositeTemplate, Default, Properties)]
    #[properties(wrapper_type = super::AdjustmentRow)]
    #[template(file = "ui/oc_page/clocks_frame/adjustment_row.blp")]
    pub struct AdjustmentRow {
        #[property(get, set)]
        pub visible: Rc<RefCell<bool>>,
        #[property(get, set)]
        pub value_ratio: Rc<RefCell<f64>>,
        #[property(get, set)]
        pub title: Rc<RefCell<String>>,

        #[template_child]
        pub label: TemplateChild<Label>,
        #[template_child]
        pub scale: TemplateChild<Scale>,
        #[template_child]
        pub value_button: TemplateChild<MenuButton>,
        #[template_child]
        pub value_spinbutton: TemplateChild<SpinButton>,
        #[template_child]
        pub adjustment: TemplateChild<OcAdjustment>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AdjustmentRow {
        const NAME: &'static str = "AdjustmentRow";
        type Type = super::AdjustmentRow;
        type ParentType = gtk::Widget;

        fn class_init(class: &mut Self::Class) {
            OcAdjustment::ensure_type();
            class.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for AdjustmentRow {
        fn constructed(&self) {
            self.parent_constructed();

            self.adjustment.connect_value_changed(clone!(
                #[strong(rename_to = value_button)]
                self.value_button,
                #[strong(rename_to = value_ratio)]
                self.value_ratio,
                move |adj| {
                    let ratio = *value_ratio.borrow();
                    let text = (adj.value() * ratio).to_string();
                    value_button.set_label(&text);
                }
            ));

            self.value_spinbutton.connect_input(clone!(
                #[strong(rename_to = value_ratio)]
                self.value_ratio,
                move |spin| {
                    let text = spin.text();
                    let value: f64 = text.parse().ok()?;
                    Some(Ok(value / *value_ratio.borrow()))
                }
            ));

            self.value_spinbutton.connect_output(clone!(
                #[strong(rename_to = value_ratio)]
                self.value_ratio,
                move |spin| {
                    let display_value = spin.value() * *value_ratio.borrow();
                    spin.set_text(&display_value.to_string());
                    glib::Propagation::Stop
                }
            ));
        }
    }

    impl WidgetImpl for AdjustmentRow {}
}

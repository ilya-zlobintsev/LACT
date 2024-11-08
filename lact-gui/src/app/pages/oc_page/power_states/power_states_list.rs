use crate::app::pages::oc_page::power_states::power_state_row::PowerStateRow;
use gtk::{
    gio,
    glib::{
        self, clone,
        object::{Cast, CastNone},
        subclass::types::ObjectSubclassIsExt,
        Object,
    },
    prelude::{ListBoxRowExt, WidgetExt},
    ListBoxRow, Widget,
};
use lact_client::schema::PowerState;

glib::wrapper! {
    pub struct PowerStatesList(ObjectSubclass<imp::PowerStatesList>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable;
}

impl PowerStatesList {
    pub fn new(title: &str) -> Self {
        Object::builder().property("title", title).build()
    }

    pub fn get_enabled_power_states(&self) -> Vec<u8> {
        self.rows()
            .iter()
            .filter(|row| row.enabled())
            .map(|row| row.index())
            .collect()
    }

    pub fn set_power_states(&self, power_states: Vec<PowerState>, value_suffix: &str) {
        let store = gio::ListStore::new::<PowerStateRow>();
        for (i, state) in power_states.into_iter().enumerate() {
            let index = u8::try_from(i).expect("Power state index doesn't fit in u8?");
            let row = PowerStateRow::new(state, index, value_suffix);
            store.append(&row);
        }

        self.imp().states_listbox.bind_model(Some(&store), |obj| {
            obj.clone().downcast::<Widget>().unwrap()
        });
    }

    pub fn connect_values_changed<F: Fn() + 'static + Clone>(&self, f: F) {
        for row in self.rows() {
            row.connect_enabled_notify(clone!(
                #[strong]
                f,
                move |_| f()
            ));
        }
    }

    pub fn set_active_state(&self, i: Option<usize>) {
        let imp = self.imp();

        for object in imp.states_listbox.observe_children().into_iter().flatten() {
            let list_row: ListBoxRow = object.downcast().unwrap();
            if let Some(row) = list_row.child().and_downcast::<PowerStateRow>() {
                let active = Some(row.index() as usize) == i;
                row.set_active(active);
            }
        }
    }

    fn rows(&self) -> Vec<PowerStateRow> {
        let children = self.imp().states_listbox.observe_children();
        children
            .into_iter()
            .flatten()
            .filter_map(|object| {
                let item = object.downcast::<ListBoxRow>().unwrap();
                let child = item.child()?;
                let row = child
                    .downcast::<PowerStateRow>()
                    .expect("ListBoxRow child must be a PowerStateRow");
                Some(row)
            })
            .collect()
    }
}

mod imp {
    use gtk::{
        glib::{self, subclass::InitializingObject, Properties},
        prelude::ObjectExt,
        subclass::{
            prelude::*,
            widget::{CompositeTemplateClass, WidgetImpl},
        },
        CompositeTemplate, ListBox,
    };
    use std::cell::RefCell;

    #[derive(CompositeTemplate, Default, Properties)]
    #[properties(wrapper_type = super::PowerStatesList)]
    #[template(file = "ui/oc_page/power_states_list.blp")]
    pub struct PowerStatesList {
        #[property(get, set)]
        title: RefCell<String>,

        #[template_child]
        pub states_listbox: TemplateChild<ListBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PowerStatesList {
        const NAME: &'static str = "PowerStatesList";
        type Type = super::PowerStatesList;
        type ParentType = gtk::Frame;

        fn class_init(class: &mut Self::Class) {
            class.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for PowerStatesList {}

    impl WidgetImpl for PowerStatesList {}
    impl FrameImpl for PowerStatesList {}
}

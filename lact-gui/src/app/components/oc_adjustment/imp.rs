use gtk::glib;
use gtk::subclass::prelude::*;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Default)]
pub struct OcAdjustment {
    pub changed: Rc<AtomicBool>,
}

#[glib::object_subclass]
impl ObjectSubclass for OcAdjustment {
    const NAME: &'static str = "OcAdjustment";
    type Type = super::OcAdjustment;
    type ParentType = gtk::Adjustment;
}

impl ObjectImpl for OcAdjustment {}

impl AdjustmentImpl for OcAdjustment {
    fn value_changed(&self) {
        self.parent_value_changed();
        self.changed.store(true, Ordering::SeqCst);
    }
}

use gtk::glib;
use gtk::subclass::prelude::*;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Default)]
pub struct AdjustmentValue {
    pub changed: Rc<AtomicBool>,
}

#[glib::object_subclass]
impl ObjectSubclass for AdjustmentValue {
    const NAME: &'static str = "AdjustmentValue";
    type Type = super::AdjustmentValue;
    type ParentType = gtk::Adjustment;
}

impl ObjectImpl for AdjustmentValue {}

impl AdjustmentImpl for AdjustmentValue {
    fn value_changed(&self) {
        self.parent_value_changed();
        self.changed.store(true, Ordering::SeqCst);
    }
}

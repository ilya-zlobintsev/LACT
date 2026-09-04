mod imp;

use glib::Object;
use gtk::{
    glib::{self},
    prelude::*,
    subclass::prelude::*,
};
use std::sync::atomic::Ordering;
use tracing::debug;

glib::wrapper! {
    pub struct AdjustmentValue(ObjectSubclass<imp::AdjustmentValue>)
        @extends gtk::Adjustment,
        @implements gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for AdjustmentValue {
    fn default() -> Self {
        Object::builder().build()
    }
}

impl AdjustmentValue {
    pub fn new(
        value: f64,
        lower: f64,
        upper: f64,
        step_increment: f64,
        page_increment: f64,
    ) -> Self {
        let adjustment_value = Self::default();

        let adjustment = adjustment_value.imp().obj();
        adjustment.set_lower(lower);
        adjustment.set_upper(upper);
        adjustment.set_step_increment(step_increment);
        adjustment.set_page_increment(page_increment);
        adjustment.set_page_size(0.0);

        adjustment_value.set_initial_value(value);

        adjustment_value
    }

    pub fn get_changed_value(&self, filter_zero: bool) -> Option<f64> {
        let inner = self.imp();
        let changed = inner.changed.load(Ordering::SeqCst);

        if changed {
            let value = inner.obj().value();

            if filter_zero && value == 0.0 {
                None
            } else {
                debug!("Value was changed, returning {value}");
                Some(value)
            }
        } else {
            debug!("Value is unchanged, returning None");
            None
        }
    }

    pub fn get_nonzero_value(&self) -> Option<f64> {
        let value = self.value();
        if value == 0.0 { None } else { Some(value) }
    }

    pub fn set_initial_value(&self, value: f64) {
        let inner = self.imp();
        inner.obj().set_value(value);
        inner.obj().emit_by_name::<()>("value_changed", &[]);
        inner.changed.store(false, Ordering::SeqCst);
    }
}

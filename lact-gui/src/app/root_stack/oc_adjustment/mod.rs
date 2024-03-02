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
    pub struct OcAdjustment(ObjectSubclass<imp::OcAdjustment>)
        @extends gtk::Adjustment, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl OcAdjustment {
    pub fn new(
        value: f64,
        lower: f64,
        upper: f64,
        step_increment: f64,
        page_increment: f64,
        page_size: f64,
    ) -> Self {
        let oc_adjustment: Self = Object::builder().build();

        let adjustment = oc_adjustment.imp().obj();
        adjustment.set_lower(lower);
        adjustment.set_upper(upper);
        adjustment.set_step_increment(step_increment);
        adjustment.set_page_increment(page_increment);
        adjustment.set_page_size(page_size);

        oc_adjustment.set_initial_value(value);

        oc_adjustment
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
        if value == 0.0 {
            None
        } else {
            Some(value)
        }
    }

    pub fn set_initial_value(&self, value: f64) {
        let inner = self.imp();
        inner.obj().set_value(value);
        inner.obj().emit_by_name::<()>("value_changed", &[]);
        inner.changed.store(false, Ordering::SeqCst);
    }
}

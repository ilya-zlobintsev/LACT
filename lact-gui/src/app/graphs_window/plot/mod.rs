mod cubic_spline;
mod imp;
mod render_thread;
mod to_texture_ext;

use super::stat::{StatType, StatsData};
use gtk::glib::{self, subclass::types::ObjectSubclassIsExt, Object};
use std::sync::{Arc, RwLock};

glib::wrapper! {
    pub struct Plot(ObjectSubclass<imp::Plot>)
        @extends gtk::Widget;
}

impl Default for Plot {
    fn default() -> Self {
        Object::builder().build()
    }
}

impl Plot {
    pub fn set_data(&self, data: Arc<RwLock<StatsData>>) {
        *self.imp().data.borrow_mut() = data;
    }

    pub fn set_stats(&self, stats: Vec<StatType>) {
        *self.imp().stats.borrow_mut() = stats;
    }
}

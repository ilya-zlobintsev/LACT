pub mod config;
mod cubic_spline;
mod imp;
mod render_thread;
mod to_texture_ext;

use std::sync::{Arc, RwLock};

use config::PlotConfig;
use gtk::glib::{self, subclass::types::ObjectSubclassIsExt, Object};

use super::stat::StatsData;

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

    pub fn set_config(&self, config: PlotConfig) {
        *self.imp().config.borrow_mut() = config;
    }
}

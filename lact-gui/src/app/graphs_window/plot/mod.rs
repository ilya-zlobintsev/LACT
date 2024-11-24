mod cubic_spline;
mod imp;
mod render_thread;
mod to_texture_ext;

use std::cell::RefMut;

pub use imp::PlotData;

use gtk::glib::{self, subclass::types::ObjectSubclassIsExt};

glib::wrapper! {
    pub struct Plot(ObjectSubclass<imp::Plot>)
        @extends gtk::Widget;
}

impl Plot {
    pub fn data_mut(&self) -> RefMut<'_, PlotData> {
        self.imp().dirty.set(true);
        self.imp().data.borrow_mut()
    }
}

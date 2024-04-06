mod cubic_spline;
mod imp;

use std::cell::RefMut;

pub use imp::PlotData;

use gtk::glib::{self, subclass::types::ObjectSubclassIsExt};

glib::wrapper! {
    pub struct Plot(ObjectSubclass<imp::Plot>)
        @extends gtk::Widget;
}

impl Plot {
    pub fn data_mut(&self) -> RefMut<'_, PlotData> {
        self.imp().data.borrow_mut()
    }
}

mod cubic_spline;
mod imp;

pub use imp::PlotData;

use gtk::glib;

glib::wrapper! {
    pub struct Plot(ObjectSubclass<imp::Plot>)
        @extends gtk::Widget;
}

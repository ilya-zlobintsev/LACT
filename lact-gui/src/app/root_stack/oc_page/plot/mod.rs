mod imp;
use gtk::glib;

pub use imp::PlotData;

glib::wrapper! {
    pub struct Plot(ObjectSubclass<imp::Plot>)
        @extends gtk::Widget;
}

mod imp;
use gtk::glib;

pub use imp::GraphData;

glib::wrapper! {
    pub struct Graph(ObjectSubclass<imp::Graph>)
        @extends gtk::Widget;
}

use gtk::glib;
use relm4::factory::DynamicIndex;

#[derive(Clone, glib::Boxed)]
#[boxed_type(name = "DynamicIndexValue")]
pub struct DynamicIndexValue(pub DynamicIndex);

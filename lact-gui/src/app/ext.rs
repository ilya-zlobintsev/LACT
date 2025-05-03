use gtk::{
    glib::object::{Cast, IsA},
    prelude::WidgetExt,
    FlowBox, FlowBoxChild, Widget,
};

pub trait FlowBoxExt {
    fn append_child(&self, child: &impl IsA<Widget>) -> FlowBoxChild;
}

impl FlowBoxExt for FlowBox {
    fn append_child(&self, child: &impl IsA<Widget>) -> FlowBoxChild {
        self.append(child);
        self.last_child()
            .unwrap()
            .downcast::<FlowBoxChild>()
            .unwrap()
    }
}

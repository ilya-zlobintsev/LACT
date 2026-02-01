use gtk::{
    FlowBox, FlowBoxChild, Widget,
    glib::object::{Cast, IsA},
    prelude::WidgetExt,
};
use relm4::Component;

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
pub trait RelmDefaultLauchable: Component {
    fn detach_default() -> relm4::Controller<Self>;
}

impl<S: Default, T: Component<Init = S>> RelmDefaultLauchable for T {
    fn detach_default() -> relm4::Controller<Self> {
        Self::builder().launch(S::default()).detach()
    }
}

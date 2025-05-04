use gtk::{
    glib::object::{Cast, IsA},
    prelude::WidgetExt,
    FlowBox, FlowBoxChild, Widget,
};
use relm4::{
    factory::FactoryView,
    prelude::{DynamicIndex, FactoryComponent, FactoryVecDeque},
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

pub trait RelmDefaultLauchable {
    fn launch_default() -> Self;
}

impl<T, R> RelmDefaultLauchable for FactoryVecDeque<T>
where
    T: FactoryComponent<Index = DynamicIndex, ParentWidget = R>,
    R: Default + FactoryView,
{
    fn launch_default() -> Self {
        Self::builder().launch_default().detach()
    }
}

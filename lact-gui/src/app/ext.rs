use gtk::{
    FlowBox, FlowBoxChild, Widget,
    glib::{
        object::{Cast, IsA},
        types::StaticType,
    },
    prelude::{AdjustmentExt, EventControllerExt, WidgetExt},
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

pub trait RelmLaunchable: Component {
    fn launch(init: <Self as Component>::Init) -> relm4::component::Connector<Self>;

    fn detach(init: <Self as Component>::Init) -> relm4::Controller<Self> {
        Self::launch(init).detach()
    }
}

impl<T: Component> RelmLaunchable for T {
    fn launch(init: <Self as Component>::Init) -> relm4::component::Connector<Self> {
        Self::builder().launch(init)
    }
}

pub trait RelmDefaultLauchable: RelmLaunchable {
    fn launch_default() -> relm4::component::Connector<Self>;

    fn detach_default() -> relm4::Controller<Self>;
}

impl<S: Default, T: Component<Init = S>> RelmDefaultLauchable for T {
    fn detach_default() -> relm4::Controller<Self> {
        Self::detach(S::default())
    }

    fn launch_default() -> relm4::component::Connector<Self> {
        Self::builder().launch(S::default())
    }
}

pub fn make_event_controller_no_scroll() -> gtk::EventControllerScroll {
    let controller = gtk::EventControllerScroll::new(
        gtk::EventControllerScrollFlags::VERTICAL | gtk::EventControllerScrollFlags::HORIZONTAL,
    );
    controller.connect_scroll(|controller, dx, dy| {
        if let Some(parent) = controller
            .widget()
            .and_then(|widget| widget.ancestor(gtk::ScrolledWindow::static_type()))
        {
            let scrolled_window = parent.downcast::<gtk::ScrolledWindow>().unwrap();

            if dy != 0.0 {
                let current = scrolled_window.vadjustment().value();
                let step = scrolled_window.vadjustment().step_increment();

                // This is a bit of a hack, fractional values are generally touchpad inputs (in pixels),
                // while whole values are scroll wheel events (which should use the `step` value)
                // With newer GTK this should be changed to getting `unit()` from the scroll controller
                let delta = if dy.fract() == 0.0 { dy * step } else { dy };
                scrolled_window.vadjustment().set_value(current + delta);
            }

            if dx != 0.0 {
                let current = scrolled_window.hadjustment().value();
                let step = scrolled_window.hadjustment().step_increment();
                let delta = if dx.fract() == 0.0 { dy * step } else { dy };
                scrolled_window.hadjustment().set_value(current + delta);
            }
        }

        gtk::glib::Propagation::Stop
    });
    controller
}

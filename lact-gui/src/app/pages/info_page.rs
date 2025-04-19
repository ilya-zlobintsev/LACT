mod hardware_info;

use self::hardware_info::HardwareInfoSection;
use super::PageUpdate;
use gtk::prelude::*;
use relm4::{Component, ComponentController, ComponentParts, ComponentSender, RelmWidgetExt};

pub struct InformationPage {
    hardware_info: relm4::Controller<HardwareInfoSection>,
}

#[relm4::component(pub)]
impl Component for InformationPage {
    type Init = ();
    type Input = PageUpdate;
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk::ScrolledWindow {
            set_hscrollbar_policy: gtk::PolicyType::Never,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 15,
                set_margin_horizontal: 20,

                model.hardware_info.widget(),
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let hardware_info = HardwareInfoSection::builder().launch(()).detach();

        let model = Self { hardware_info };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        self.hardware_info.emit(msg.clone());

        self.update_view(widgets, sender);
    }
}

use super::OcPageMsg;
use crate::I18N;
use crate::app::APP_BROKER;
use crate::app::components::{adjustment_card::AdjustmentCard, page_section::PageSection};
use crate::app::msg::AppMsg;
use crate::app::utils::formatting;
use gtk::gio::prelude::ListModelExt as _;
use gtk::prelude::{BoxExt as _, ListBoxRowExt as _, OrientableExt as _, WidgetExt as _};
use i18n_embed_fl::fl;
use lact_schema::AmdUmaCarveout;
use relm4::{ComponentParts, ComponentSender, WidgetTemplate as _, css};

pub struct IGpuFrame {
    // options: Option<UmaCarveoutOptions>,
    // combo_row: relm4::Controller<SimpleComboRow<String>>,
    uma_options: gtk::StringList,
    uma_current: u32,
    uma_selected: u32,
}

#[derive(Debug)]
pub enum IGpuFrameMsg {
    UmaCarveout(Option<AmdUmaCarveout>),
    UmaSelected(u32),
}

#[relm4::component(pub)]
impl relm4::Component for IGpuFrame {
    type Init = ();
    type Input = IGpuFrameMsg;
    type Output = OcPageMsg;
    type CommandOutput = ();

    view! {
        PageSection {
            set_hide_visible_container: true,
            #[watch]
            set_name: fl!(I18N, "igpu-section"),
            #[watch]
            set_visible: model.uma_options.n_items() != 0,

            #[template]
            #[local]
            append_child = &card -> AdjustmentCard {
                #[template_child]
                content {
                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 10,

                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_hexpand: true,
                            set_valign: gtk::Align::Center,

                            gtk::Label {
                                set_label: &fl!(I18N, "uma-carveout"),
                                set_halign: gtk::Align::Start,
                            },

                            gtk::Label {
                                set_label: &fl!(I18N, "uma-carveout-caption"),
                                set_halign: gtk::Align::Start,
                                set_xalign: 0.0,
                                set_wrap: true,
                                add_css_class: css::DIM_LABEL,
                                add_css_class: css::CAPTION,
                            },
                        },

                        gtk::ListBoxRow {
                            set_activatable: false,
                            set_selectable: false,

                            #[name = "uma_dropdown"]
                            gtk::DropDown {
                                set_valign: gtk::Align::Center,
                                #[watch]
                                #[block_signal(uma_mode_select_handler)]
                                set_model: Some(&model.uma_options),
                                #[watch]
                                #[block_signal(uma_mode_select_handler)]
                                set_selected: model.uma_selected,

                                connect_selected_notify[sender] => move |dropdown| {
                                    sender.input(IGpuFrameMsg::UmaSelected(dropdown.selected()));
                                } @ uma_mode_select_handler,
                            },
                        },
                    },
                },
            },
        }
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let card = AdjustmentCard::init(());
        let model = Self {
            uma_options: gtk::StringList::default(),
            uma_selected: 0,
            uma_current: 0,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            IGpuFrameMsg::UmaCarveout(carveout) => {
                let uma_options = gtk::StringList::default();

                match carveout {
                    Some(carveout) => {
                        for option in carveout.options {
                            let size = formatting::fmt_human_bytes(option.size_bytes, None);
                            let text = if let Some(name) = option.name {
                                format!("{size} ({name})")
                            } else {
                                size
                            };
                            uma_options.append(&text);
                        }

                        self.uma_current = carveout.current as u32;
                        self.uma_selected = self.uma_current;
                    }
                    None => {
                        self.uma_current = 0;
                        self.uma_selected = 0;
                    }
                }

                self.uma_options = uma_options;
            }
            IGpuFrameMsg::UmaSelected(selected) => {
                self.uma_selected = selected;
                APP_BROKER.send(AppMsg::SettingsChanged);
            }
        }
    }
}

impl IGpuFrame {
    pub fn get_uma_carveout(&self) -> Option<u32> {
        if self.uma_current != self.uma_selected {
            Some(self.uma_selected)
        } else {
            None
        }
    }
}

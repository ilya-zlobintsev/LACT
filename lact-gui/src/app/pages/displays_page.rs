use crate::{
    I18N,
    app::{
        components::{info_row::InfoRow, page_section::PageSection},
        utils::ext::FlowBoxExt as _,
    },
};
use gtk::prelude::{BoxExt as _, OrientableExt as _, WidgetExt as _};
use i18n_embed_fl::fl;
use lact_schema::{DisplayConnector, DisplayInfo, DisplaysInfo};
use relm4::{
    ComponentParts, ComponentSender, FactorySender, RelmWidgetExt as _,
    prelude::{DynamicIndex, FactoryVecDeque},
};
use std::fmt::Write as _;

pub struct DisplaysPage {
    displays: FactoryVecDeque<DisplayComponent>,
}

#[relm4::component(pub)]
impl relm4::SimpleComponent for DisplaysPage {
    type Init = ();
    type Input = DisplaysInfo;
    type Output = ();

    view! {
        gtk::Box {
            set_expand: true,
            #[watch]
            set_align: if model.displays.is_empty() { gtk::Align::Center } else { gtk::Align::Fill },
            set_orientation: gtk::Orientation::Vertical,

            gtk::Image {
                set_icon_name: Some("action-unavailable-symbolic"),
                set_align: gtk::Align::Center,
                set_pixel_size: 64,
                #[watch]
                set_visible: model.displays.is_empty(),
            },

            gtk::Label {
                set_markup: &format!("<b><span size='large'>{}</span></b>", fl!(I18N, "displays-missing")),
                #[watch]
                set_visible: model.displays.is_empty(),
            },

            #[local_ref]
            displays_list -> gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 15,
                set_margin_all: 15,
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            displays: FactoryVecDeque::builder().launch_default().detach(),
        };
        let displays_list = model.displays.widget();

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, displays_info: Self::Input, _sender: ComponentSender<Self>) {
        let mut guard = self.displays.guard();

        guard.clear();
        for entry in displays_info.displays {
            guard.push_back(entry);
        }
    }
}

struct DisplayComponent {
    display_id: String,
    info: DisplayInfo,
}

#[relm4::factory]
impl relm4::factory::FactoryComponent for DisplayComponent {
    type Init = (String, DisplayInfo);
    type Input = ();
    type Output = ();
    type ParentWidget = gtk::Box;
    type CommandOutput = ();

    view! {
        PageSection {
            set_name: fl!(I18N, "display-title", identifier = self.display_id.clone()),

            append_child = &gtk::FlowBox {
                set_orientation: gtk::Orientation::Horizontal,
                set_column_spacing: 10,
                set_homogeneous: true,
                set_min_children_per_line: 2,
                set_max_children_per_line: 4,
                set_selection_mode: gtk::SelectionMode::None,

                append_child = &InfoRow {
                    set_name: fl!(I18N, "display-manufacturer"),
                    set_value: self.info.manufacturer.as_deref().unwrap_or("N/A"),
                    set_selectable: true,
                },

                append_child = &InfoRow {
                    set_name: fl!(I18N, "display-product-code"),
                    set_value: self.info.product_code.to_string(),
                    set_selectable: true,
                },

                append_child = &InfoRow {
                    set_name: fl!(I18N, "display-model"),
                    set_value: self.info.model.as_deref().unwrap_or("N/A"),
                    set_selectable: true,
                },

                append_child = &InfoRow {
                    set_name: fl!(I18N, "display-physical-size"),
                    set_value: self.info.size.map(|(width, height)| {
                        format!("{width}cm x {height}cm")
                    }).unwrap_or_default(),
                    set_selectable: true,
                    set_visible: self.info.size.is_some(),
                },

                append_child = &InfoRow {
                    set_name: fl!(I18N, "display-connection"),
                    set_value: self.format_connector(),
                    set_selectable: true,
                },

                append_child = &InfoRow {
                    set_name: fl!(I18N, "display-manufacture-date"),
                    set_value: self.info.manufacture_date.map(|date| {
                        if date.week == 0 {
                            date.year.to_string()
                        } else {
                            format!("{}, Week {}", date.year, date.week)
                        }
                    }).unwrap_or_default(),
                    set_selectable: true,
                    set_visible: self.info.manufacture_date.is_some(),
                },

            },
        }
    }

    fn init_model(
        (display_id, info): Self::Init,
        _index: &DynamicIndex,
        _sender: FactorySender<Self>,
    ) -> Self {
        Self { display_id, info }
    }
}

impl DisplayComponent {
    fn format_connector(&self) -> String {
        match self.info.connector_type {
            DisplayConnector::DisplayPort {
                lanes,
                bandwidth,
                embedded,
            } => {
                let mut text = "DisplayPort".to_owned();
                if embedded {
                    text.push_str(" (Internal)")
                }
                let lane_rate = bandwidth as f64 / 1000.0;
                write!(
                    text,
                    " @ {} Gbps ({lane_rate} Gbps x {lanes} lanes)",
                    lane_rate * lanes as f64
                )
                .unwrap();
                text
            }
            DisplayConnector::Hdmi => "HDMI".to_owned(),
            DisplayConnector::Dvi => "DVI".to_owned(),
            DisplayConnector::Vga => "VGA".to_owned(),
            DisplayConnector::Other => fl!(I18N, "unknown-throttling"),
        }
    }
}

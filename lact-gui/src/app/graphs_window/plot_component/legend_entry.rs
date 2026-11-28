use gtk::prelude::{
    BoxExt as _, DrawingAreaExt as _, DrawingAreaExtManual as _, OrientableExt as _, WidgetExt as _,
};
use plotters::style::RGBAColor;
use std::{cell::Cell, rc::Rc, sync::Arc};

use crate::app::graphs_window::plot::LegendLabel;

pub struct LegendEntry {
    text: Arc<str>,
    color: Rc<Cell<RGBAColor>>,
}

impl LegendEntry {
    pub fn matches(&self, label: &LegendLabel) -> bool {
        self.text == label.text && self.color.get() == label.color
    }
}

#[derive(Debug)]
pub enum LegendEntryMsg {
    SetLabel(LegendLabel),
}

#[relm4::factory(pub)]
impl relm4::factory::FactoryComponent for LegendEntry {
    type ParentWidget = gtk::FlowBox;
    type Init = LegendLabel;
    type Input = LegendEntryMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 6,

            #[name = "swatch"]
            gtk::DrawingArea {
                set_content_width: 10,
                set_content_height: 10,
                set_valign: gtk::Align::Center,

                set_draw_func[color = self.color.clone()] => move |_, cr, width, height| {
                    let color = color.get();
                    cr.set_source_rgba(
                        f64::from(color.0) / 255.0,
                        f64::from(color.1) / 255.0,
                        f64::from(color.2) / 255.0,
                        color.3,
                    );
                    cr.rectangle(0.0, 0.0, f64::from(width), f64::from(height));
                    cr.fill().ok();
                },
            },

            #[name = "text"]
            gtk::Label {
                set_text: &self.text,
                set_ellipsize: gtk::pango::EllipsizeMode::End,
                add_css_class: "caption",
            },
        }
    }

    fn init_model(
        label: Self::Init,
        _index: &Self::Index,
        _sender: relm4::FactorySender<Self>,
    ) -> Self {
        Self {
            text: label.text,
            color: Rc::new(Cell::new(label.color)),
        }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::Input,
        _sender: relm4::FactorySender<Self>,
    ) {
        match msg {
            LegendEntryMsg::SetLabel(label) => {
                if label.color != self.color.get() {
                    self.color.set(label.color);
                    widgets.swatch.queue_draw();
                }
                if label.text != self.text {
                    widgets.text.set_text(&label.text);
                    self.text = label.text;
                }
            }
        }
    }
}

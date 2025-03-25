use crate::app::graphs_window::DynamicIndexValue;

use super::{
    plot::Plot,
    stat::{StatType, StatsData},
    GraphsWindowMsg,
};
use gtk::{
    gdk,
    glib::{subclass::types::ObjectSubclassIsExt, types::StaticType, value::ToValue},
    prelude::{BoxExt, ButtonExt, OrientableExt, WidgetExt},
};
use relm4::{
    binding::{BoolBinding, F64Binding},
    factory::positions::GridPosition,
    prelude::DynamicIndex,
    RelmWidgetExt,
};
use std::sync::{Arc, RwLock};

pub struct PlotComponent {
    pub stats: Vec<StatType>,
    pub plots_per_row: F64Binding,
    pub data: Arc<RwLock<StatsData>>,
    pub edit_mode: BoolBinding,
}

#[derive(Debug, Clone)]
pub enum PlotComponentMsg {
    Redraw,
}

#[relm4::factory(pub)]
impl relm4::factory::FactoryComponent for PlotComponent {
    type ParentWidget = gtk::Grid;
    type Input = PlotComponentMsg;
    type Output = GraphsWindowMsg;
    type Init = Self;
    type Index = DynamicIndex;
    type CommandOutput = ();

    view! {
        gtk::Frame {
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,

                #[name = "plot"]
                append = &Plot {
                    set_data: self.data.clone(),
                    #[watch]
                    set_stats: self.stats.clone(),

                    #[watch]
                    set_cursor: self.get_cursor().as_ref(),

                    #[wrap(Clone::clone)]
                    add_controller = &gtk::DragSource {
                        #[watch]
                        set_actions: if self.edit_mode.value() { gdk::DragAction::MOVE } else { gdk::DragAction::empty() },

                        connect_prepare[plot, index, edit_mode = self.edit_mode.clone()] => move |drag_source, _, _| {
                            if edit_mode.value() {
                                if let Some(texture) = plot.imp().get_last_texture() {
                                    drag_source.set_icon(Some(&texture), 0, 0);
                                }
                                Some(gdk::ContentProvider::for_value(&DynamicIndexValue(index.clone()).to_value()))
                            } else {
                                None
                            }
                        }
                    },

                    add_controller = gtk::DropTarget {
                        set_actions: gdk::DragAction::MOVE,
                        set_types: &[DynamicIndexValue::static_type()],

                        connect_enter[root] => move |_, _, _| {
                            root.set_opacity(0.5);
                            gdk::DragAction::MOVE
                        },

                        connect_leave[root] => move |_| {
                            root.set_opacity(1.0);
                        },

                        connect_drop[root, index, sender] => move |_, value, _, _| {
                            root.set_opacity(1.0);

                            if let Ok(DynamicIndexValue(source_index)) = value.get::<DynamicIndexValue>() {
                                sender.output(GraphsWindowMsg::SwapPlots(index.clone(), source_index)).unwrap();
                            }

                            true
                        },
                    },
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 5,
                    #[watch]
                    set_visible: self.edit_mode.value(),
                    set_align: gtk::Align::End,

                    append = &gtk::Button  {
                        set_icon_name: "view-list-symbolic",
                        set_tooltip: "Edit graph sensors",
                    },

                    append = &gtk::Button  {
                        set_icon_name: "edit-delete-symbolic",
                        set_tooltip: "Delete graph",
                    },
                },
            },
        },
    }

    fn init_model(
        model: Self::Init,
        _index: &Self::Index,
        _sender: relm4::FactorySender<Self>,
    ) -> Self {
        model
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::Input,
        sender: relm4::FactorySender<Self>,
    ) {
        match msg {
            PlotComponentMsg::Redraw => {
                widgets.plot.set_dirty(true);
                widgets.plot.queue_draw();
            }
        }
        self.update_view(widgets, sender);
    }
}

impl relm4::factory::Position<GridPosition, DynamicIndex> for PlotComponent {
    fn position(&self, index: &DynamicIndex) -> GridPosition {
        let i = index.current_index() as i32;
        let plots_per_row = self.plots_per_row.value() as i32;
        GridPosition {
            column: i % plots_per_row,
            row: i / plots_per_row,
            width: 1,
            height: 1,
        }
    }
}

impl PlotComponent {
    fn get_cursor(&self) -> Option<gdk::Cursor> {
        if self.edit_mode.value() {
            gdk::Cursor::from_name("move", None)
        } else {
            None
        }
    }
}

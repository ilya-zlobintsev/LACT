use super::{
    GraphsWindowMsg,
    plot::Plot,
    stat::{StatType, StatsData},
};
use crate::I18N;
use crate::app::utils::dnd::DynamicIndexValue;
use crate::app::utils::ext::FlowBoxExt;
use gtk::{
    gdk, glib,
    glib::{subclass::types::ObjectSubclassIsExt, types::StaticType, value::ToValue},
    prelude::{
        AdjustmentExt, BoxExt, ButtonExt, CheckButtonExt, DrawingAreaExt, DrawingAreaExtManual,
        ObjectExt, OrientableExt, PopoverExt, WidgetExt,
    },
};
use i18n_embed_fl::fl;
use relm4::{
    RelmObjectExt, RelmWidgetExt,
    binding::{BoolBinding, ConnectBinding, F64Binding},
    css,
    factory::positions::GridPosition,
    prelude::{DynamicIndex, FactoryVecDeque},
};
use std::sync::{Arc, RwLock};

pub struct PlotComponent {
    stats: FactoryVecDeque<StatTypeRow>,
    plots_per_row: F64Binding,
    data: Arc<RwLock<StatsData>>,
    edit_mode: BoolBinding,
    print_extra_info: BoolBinding,
    time_period: gtk::Adjustment,
    /// Text of the legend entries of the last rendered frame.
    legend_texts: Vec<String>,
    legend_labels: Vec<gtk::Label>,
}

pub struct PlotComponentConfig {
    pub selected_stats: Vec<StatType>,
    pub data: Arc<RwLock<StatsData>>,
    pub edit_mode: BoolBinding,
    pub plots_per_row: F64Binding,
    pub time_period: gtk::Adjustment,
}

#[derive(Debug, Clone)]
pub enum PlotComponentMsg {
    Redraw,
    FrameRendered,
    UpdatedSelection,
}

#[relm4::factory(pub)]
impl relm4::factory::FactoryComponent for PlotComponent {
    type ParentWidget = gtk::Grid;
    type Input = PlotComponentMsg;
    type Output = GraphsWindowMsg;
    type Init = PlotComponentConfig;
    type Index = DynamicIndex;
    type CommandOutput = ();

    view! {
        gtk::Frame {
            gtk::Overlay {
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    #[name = "plot"]
                    Plot {
                    set_data: self.data.clone(),
                    set_margin_all: 5,

                    #[watch]
                    set_cursor: self.get_cursor().as_ref(),
                    #[watch]
                    set_time_period_seconds: self.time_period.value() as i64,
                    add_binding: (&self.print_extra_info, "print-extra-info"),

                    set_stats: self.selected_stats(),

                        connect_frame_rendered[sender] => move || {
                            sender.input(PlotComponentMsg::FrameRendered);
                        },
                    },

                    #[name = "legend"]
                    LegendContainer {},
                },

                add_overlay = &gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 5,
                    set_halign: gtk::Align::End,
                    set_valign: gtk::Align::Start,
                    set_margin_all: 20,
                    set_opacity: 0.9,

                    gtk::ToggleButton {
                        set_icon_name: "info-outline-symbolic",
                        set_tooltip: &fl!(I18N, "plot-show-detailed-info"),
                        bind: &self.print_extra_info,
                    },

                    gtk::MenuButton {
                        #[watch]
                        set_visible: self.edit_mode.value(),
                        set_icon_name: "view-list-symbolic",
                        set_tooltip: &fl!(I18N, "edit-graph-sensors"),

                        #[wrap(Some)]
                        set_popover = &gtk::Popover {
                            #[wrap(Some)]
                            set_child = &gtk::ScrolledWindow {
                                set_policy: (gtk::PolicyType::Never, gtk::PolicyType::Automatic),
                                set_propagate_natural_height: true,
                                set_max_content_height: 200,

                                self.stats.widget() {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_margin_all: 10,
                                }
                            },
                        },
                    },

                    gtk::Button {
                        #[watch]
                        set_visible: self.edit_mode.value(),
                        set_icon_name: "edit-delete-symbolic",
                        add_css_class: css::DESTRUCTIVE_ACTION,
                        set_tooltip: &fl!(I18N, "delete-graph"),

                        connect_clicked[sender, index] => move |_| {
                            sender.output(GraphsWindowMsg::RemovePlot(index.clone())).unwrap();
                        }
                    },
                },

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

                    connect_enter[root = root.downgrade()] => move |_, _, _| {
                        if let Some(root) = root.upgrade() {
                            root.set_opacity(0.5);
                        }
                        gdk::DragAction::MOVE
                    },

                    connect_leave[root = root.downgrade()] => move |_| {
                        if let Some(root) = root.upgrade() {
                            root.set_opacity(1.0);
                        }
                    },

                    connect_drop[root = root.downgrade(), index, sender] => move |_, value, _, _| {
                        if let Some(root) = root.upgrade() {
                            root.set_opacity(1.0);
                        }

                        if let Ok(DynamicIndexValue(source_index)) = value.get::<DynamicIndexValue>() {
                            sender.output(GraphsWindowMsg::SwapPlots(index.clone(), source_index)).unwrap();
                        }

                        true
                    },
                },
            },
        },
    }

    fn init_model(
        init: Self::Init,
        _index: &Self::Index,
        sender: relm4::FactorySender<Self>,
    ) -> Self {
        let mut stats = FactoryVecDeque::builder()
            .launch_default()
            .forward(sender.input_sender(), |msg| msg);

        {
            let mut stats_guard = stats.guard();
            let data_guard = init.data.read().unwrap();

            for stat in data_guard.list_stats() {
                let enabled = init.selected_stats.contains(stat);
                stats_guard.push_back((stat.clone(), enabled));
            }
        }

        let print_extra_info = BoolBinding::new(false);
        print_extra_info.connect_value_notify(move |_| {
            sender.input(PlotComponentMsg::Redraw);
        });

        Self {
            stats,
            plots_per_row: init.plots_per_row,
            data: init.data,
            edit_mode: init.edit_mode,
            print_extra_info,
            time_period: init.time_period,
            legend_texts: Vec::new(),
            legend_labels: Vec::new(),
        }
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
            PlotComponentMsg::FrameRendered => {
                widgets.plot.queue_draw();
                self.update_legend(widgets);
            }
            PlotComponentMsg::UpdatedSelection => {
                widgets.plot.set_stats(self.selected_stats());
                sender.input(PlotComponentMsg::Redraw);
                sender.output(GraphsWindowMsg::SaveConfig).unwrap();
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
    /// Rebuild the GTK legend below the plot when the labels change.
    fn update_legend(&mut self, widgets: &PlotComponentWidgets) {
        let labels = widgets.plot.labels();
        let texts = labels
            .iter()
            .map(|label| label.text.clone())
            .collect::<Vec<_>>();

        if texts == self.legend_texts {
            return;
        }

        self.legend_texts = texts;

        if labels.len() == self.legend_labels.len() {
            for (label_widget, label) in self.legend_labels.iter().zip(&labels) {
                if label_widget.text() != label.text {
                    label_widget.set_text(&label.text);
                }
            }

            return;
        }

        let legend = widgets.legend.flow_box();

        while let Some(child) = legend.first_child() {
            legend.remove(&child);
        }

        self.legend_labels.clear();

        for label in &labels {
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);

            let swatch = gtk::DrawingArea::new();
            swatch.set_content_width(10);
            swatch.set_content_height(10);
            swatch.set_valign(gtk::Align::Center);

            let color = label.color;
            swatch.set_draw_func(move |_, cr, width, height| {
                cr.set_source_rgba(
                    f64::from(color.0) / 255.0,
                    f64::from(color.1) / 255.0,
                    f64::from(color.2) / 255.0,
                    color.3,
                );
                cr.rectangle(0.0, 0.0, f64::from(width), f64::from(height));
                cr.fill().ok();
            });

            row.append(&swatch);
            let text = gtk::Label::new(Some(&label.text));
            text.add_css_class("caption");
            row.append(&text);
            legend.append_child(&row);
            self.legend_labels.push(text);
        }
    }

    fn get_cursor(&self) -> Option<gdk::Cursor> {
        if self.edit_mode.value() {
            gdk::Cursor::from_name("move", None)
        } else {
            None
        }
    }

    pub fn selected_stats(&self) -> Vec<StatType> {
        self.stats
            .iter()
            .filter(|row| row.enabled.value())
            .map(|row| row.stat.clone())
            .collect()
    }

    pub fn into_config(self) -> PlotComponentConfig {
        PlotComponentConfig {
            selected_stats: self.selected_stats(),
            data: self.data,
            edit_mode: self.edit_mode,
            plots_per_row: self.plots_per_row,
            time_period: self.time_period,
        }
    }
}

struct StatTypeRow {
    stat: StatType,
    enabled: BoolBinding,
}

#[relm4::factory]
impl relm4::factory::FactoryComponent for StatTypeRow {
    type ParentWidget = gtk::Box;
    type Init = (StatType, bool);
    type Input = ();
    type Output = PlotComponentMsg;
    type CommandOutput = ();

    view! {
        gtk::CheckButton {
            add_binding: (&self.enabled, "active"),
            set_label: Some(&self.stat.display()),
        }
    }

    fn init_model(
        (stat, enabled): Self::Init,
        _index: &Self::Index,
        sender: relm4::FactorySender<Self>,
    ) -> Self {
        let enabled = BoolBinding::new(enabled);

        enabled.connect_value_notify(move |_| {
            sender.output(PlotComponentMsg::UpdatedSelection).unwrap();
        });

        Self { stat, enabled }
    }
}

mod legend_container {
    use std::cell::RefCell;

    use gtk::glib;
    use gtk::gsk;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    /// Container of the plot legend.
    ///
    /// It reports no width to its parent, so the legend never changes the size
    /// of the chart. The chart defines the width of the cell, the legend just
    /// wraps inside it and grows vertically when needed.
    ///
    /// This is a plain widget instead of a `GtkBox`, because a box delegates
    /// measuring and allocating to its layout manager, which would ignore the
    /// custom `measure` implementation.
    #[derive(Default)]
    pub struct LegendContainer {
        pub flow_box: RefCell<Option<gtk::FlowBox>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LegendContainer {
        const NAME: &'static str = "LactPlotLegend";
        type Type = super::LegendContainer;
        type ParentType = gtk::Widget;
    }

    impl ObjectImpl for LegendContainer {
        fn constructed(&self) {
            self.parent_constructed();

            let flow_box = gtk::FlowBox::new();
            flow_box.set_selection_mode(gtk::SelectionMode::None);
            flow_box.set_homogeneous(false);
            flow_box.set_column_spacing(16);
            flow_box.set_row_spacing(4);
            flow_box.set_margin_start(6);
            flow_box.set_margin_end(6);
            flow_box.set_margin_top(6);
            flow_box.set_margin_bottom(8);
            flow_box.set_max_children_per_line(6);
            flow_box.set_halign(gtk::Align::Center);

            flow_box.set_parent(&*self.obj());
            *self.flow_box.borrow_mut() = Some(flow_box);
        }

        fn dispose(&self) {
            if let Some(flow_box) = self.flow_box.borrow().as_ref() {
                flow_box.unparent();
            }
        }
    }

    impl WidgetImpl for LegendContainer {
        fn request_mode(&self) -> gtk::SizeRequestMode {
            // The height of the legend depends on how many lines the flow box
            // needs, which in turn depends on the width given by the chart.
            gtk::SizeRequestMode::HeightForWidth
        }

        fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            if orientation == gtk::Orientation::Horizontal {
                // The labels change with the values, and a plot with more
                // entries would otherwise get wider than one with fewer.
                (0, 0, -1, -1)
            } else if let Some(flow_box) = self.flow_box.borrow().as_ref() {
                flow_box.measure(orientation, for_size)
            } else {
                (0, 0, -1, -1)
            }
        }

        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            let Some(flow_box) = self.flow_box.borrow().clone() else {
                return;
            };

            let (_, natural_width, _, _) = flow_box.measure(gtk::Orientation::Horizontal, height);
            let child_width = natural_width.min(width);
            let x = (width - child_width) / 2;

            let transform =
                gsk::Transform::new().translate(&gtk::graphene::Point::new(x as f32, 0.0));
            flow_box.allocate(child_width, height, baseline, Some(transform));
        }

        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            if let Some(flow_box) = self.flow_box.borrow().as_ref() {
                self.obj().snapshot_child(flow_box, snapshot);
            }
        }
    }
}

glib::wrapper! {
    /// Legend below the plot. Reports no width to its parent, so that the
    /// legend never changes the size of the chart.
    pub struct LegendContainer(ObjectSubclass<legend_container::LegendContainer>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for LegendContainer {
    fn default() -> Self {
        glib::Object::builder().build()
    }
}

impl LegendContainer {
    fn flow_box(&self) -> gtk::FlowBox {
        self.imp()
            .flow_box
            .borrow()
            .clone()
            .expect("legend flow box is created in constructed()")
    }
}

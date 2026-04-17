use crate::{
    I18N,
    app::{APP_BROKER, graphs_window::plot::PlotColorScheme, msg::AppMsg},
};
use gtk::{
    gdk,
    prelude::{
        AdjustmentExt, BoxExt as _, ButtonExt as _, CheckButtonExt as _, DrawingAreaExtManual as _,
        EventControllerExt as _, GestureSingleExt as _, GtkWindowExt as _, OrientableExt as _,
        PopoverExt, RangeExt as _, ScaleExt as _, WidgetExt as _,
    },
};
use i18n_embed_fl::fl;
use indexmap::IndexMap;
use lact_schema::{ClocksTable, DeviceStats, NvidiaVfPoint, config};
use plotters::{
    chart::{ChartBuilder, SeriesLabelPosition},
    prelude::{Circle, EmptyElement, IntoDrawingArea as _, Rectangle, Text},
    series::{DashedLineSeries, LineSeries, PointSeries},
    style::{Color as _, ShapeStyle, TextStyle, text_anchor::Pos},
};
use plotters_cairo::CairoBackend;
use relm4::{ComponentParts, RelmObjectExt as _, RelmWidgetExt as _, binding::BoolBinding, css};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::Arc,
};
use std::{cmp, fmt::Write as _};

// In percentage
const POINT_VOLTAGE_HOVER_MARGIN: f32 = 0.01;
const POINT_FREQ_HOVER_MARGIN: f32 = 0.03;

#[derive(Clone)]
pub struct VfCurveEditor {
    points: Rc<RefCell<Vec<NvidiaVfPoint>>>,
    stats: Rc<RefCell<Arc<DeviceStats>>>,
    allow_editing: BoolBinding,
    locked_clocks_range: Rc<Cell<Option<(u32, u32)>>>,

    visible_range_start: gtk::Adjustment,
    visible_range_end: gtk::Adjustment,

    // Passed from app
    global_settings_changed: BoolBinding,

    cursor_position: Rc<Cell<Option<(f64, f64)>>>,
    hovered_point: Rc<Cell<Option<usize>>>,
    dragging_point: Rc<Cell<Option<usize>>>,
    drag_modifiers: Rc<Cell<gdk::ModifierType>>,
}

#[derive(Debug)]
pub enum VfCurveEditorMsg {
    Show,
    Clocks(Option<Arc<ClocksTable>>),
    Stats(Arc<DeviceStats>),
    CursorUpdate {
        x: f64,
        y: f64,
        modifiers: gdk::ModifierType,
    },
    DragStart,
    DragEnd,
    FlattenCurve,
    ResetCurve,
}

#[relm4::component(pub)]
impl relm4::Component for VfCurveEditor {
    type Init = BoolBinding;
    type Input = VfCurveEditorMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        #[root]
        adw::Window {
            set_hide_on_close: true,
            set_default_size: (1100, 700),
            set_title: Some(&fl!(I18N, "vf-curve-editor")),

            adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {},

                #[wrap(Some)]
                set_content = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_margin_all: 5,
                    set_spacing: 10,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_halign: gtk::Align::Center,
                        set_spacing: 10,

                        gtk::Label {
                            set_markup: &fl!(I18N, "nvidia-vf-curve-warning"),
                            add_css_class: "error",
                            add_css_class: "heading",
                        },

                    },


                    #[name = "drawing_area"]
                    gtk::DrawingArea {
                        set_expand: true,
                        set_margin_all: 10,

                        set_draw_func[model] => move |area, ctx, width, height| {
                            let style_context = area.style_context();
                            let colors = PlotColorScheme::from_context(&style_context).unwrap_or_default();
                            model.draw_chart(ctx, width, height, colors);
                        },

                        add_controller = gtk::GestureClick {
                            connect_pressed[sender] => move |gesture, _, x, y| {
                                let modifiers = gesture.current_event_state();

                                sender.input(VfCurveEditorMsg::DragStart);
                                sender.input(VfCurveEditorMsg::CursorUpdate { x, y, modifiers });
                            },
                            connect_released[sender] => move |_, _, _x, _y| {
                                sender.input(VfCurveEditorMsg::DragEnd);
                            }
                        },

                        add_controller = gtk::GestureClick {
                            set_button: gdk::BUTTON_SECONDARY,
                            connect_pressed[drawing_area, point_menu, model] => move |_, _, x, y| {
                                if model.hovered_point.get().is_none() {
                                    return;
                                }

                                point_menu.set_parent(&drawing_area);
                                point_menu.set_pointing_to(Some(&gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
                                point_menu.popup();

                            },
                        },

                        add_controller = gtk::EventControllerMotion {
                            connect_motion[sender] => move |motion, x, y| {
                                let modifiers = motion.current_event_state();
                                sender.input(VfCurveEditorMsg::CursorUpdate { x, y, modifiers });
                            },
                        },

                        #[watch]
                        set_cursor: if model.dragging_point.get().is_some() {
                            gdk::Cursor::from_name("move", None)
                        } else {
                            None
                        }.as_ref(),
                    },

                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 5,
                        set_margin_horizontal: 5,

                        gtk::Label {
                            set_label: &fl!(I18N, "vf-curve-visible-range"),
                            add_css_class: "heading",
                        },

                        gtk::Scale {
                            set_adjustment: &model.visible_range_start,
                            set_draw_value: true,
                            set_width_request: 120,
                            set_value_pos: gtk::PositionType::Right,
                        },


                        gtk::Label {
                            set_label: &fl!(I18N, "vf-curve-visible-range-to"),
                            add_css_class: "heading",
                        },

                        gtk::Scale {
                            set_adjustment: &model.visible_range_end,
                            set_draw_value: true,
                            set_width_request: 120,
                            set_value_pos: gtk::PositionType::Right,
                        },

                        gtk::CheckButton {
                            set_label: Some(&fl!(I18N, "vf-curve-enable-editing")),
                            set_halign: gtk::Align::End,
                            set_hexpand: true,
                            add_css_class: "warning",
                            add_binding: (&model.allow_editing, "active"),

                            connect_toggled => move |_| {
                                APP_BROKER.send(AppMsg::SettingsChanged);
                            }
                        },

                        gtk::Button {
                            set_label: &fl!(I18N, "reset-button"),
                            set_halign: gtk::Align::End,
                            add_css_class: css::DESTRUCTIVE_ACTION,

                            #[watch]
                            set_sensitive: {
                                model.points.borrow()
                                    .iter()
                                    .any(|point| point.freq != point.base_freq)
                            },

                            connect_clicked => VfCurveEditorMsg::ResetCurve,
                            connect_clicked => move |_| {
                                APP_BROKER.send(AppMsg::SettingsChanged);
                            }
                        },


                        gtk::Button {
                            set_label: &fl!(I18N, "apply-button"),
                            set_halign: gtk::Align::End,
                            add_binding: (&model.global_settings_changed, "sensitive"),
                            add_css_class: css::SUGGESTED_ACTION,

                            connect_clicked => move |_| {
                                APP_BROKER.send(AppMsg::ApplyChanges);
                            },
                        },
                    },
                },
            },
        },

        #[name = "point_menu"]
        gtk::Popover {
            add_css_class: "menu",

            connect_closed => |popover| {
                popover.unparent();
            },

            gtk::Button {
                set_label: "Flatten curve to the right",
                connect_clicked => VfCurveEditorMsg::FlattenCurve,
                connect_clicked[point_menu] => move |_| {
                    point_menu.popdown();
                },
                add_css_class: "flat",
            },
        }
    }

    fn init(
        global_settings_changed: Self::Init,
        root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = Self {
            points: Rc::default(),
            stats: Rc::default(),
            global_settings_changed,
            locked_clocks_range: Rc::default(),
            allow_editing: BoolBinding::new(false),
            cursor_position: Rc::new(Cell::new(None)),
            visible_range_start: gtk::Adjustment::new(30.0, 0.0, 100.0, 1.0, 10.0, 0.0),
            visible_range_end: gtk::Adjustment::new(100.0, 0.0, 100.0, 1.0, 10.0, 0.0),
            hovered_point: Rc::new(Cell::new(None)),
            dragging_point: Rc::new(Cell::new(None)),
            drag_modifiers: Rc::new(Cell::new(gdk::ModifierType::empty())),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::Input,
        sender: relm4::ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match msg {
            VfCurveEditorMsg::Show => {
                root.present();
            }
            VfCurveEditorMsg::Clocks(clocks_table) => {
                let mut points = self.points.borrow_mut();
                points.clear();
                self.locked_clocks_range.take();

                if let Some(ClocksTable::Nvidia(nvidia_table)) = clocks_table.as_deref() {
                    points.extend_from_slice(&nvidia_table.gpu_vf_curve);
                    self.locked_clocks_range.set(nvidia_table.gpu_locked_clocks)
                }

                if points.is_empty() {
                    root.hide();
                }
            }
            VfCurveEditorMsg::Stats(device_stats) => {
                *self.stats.borrow_mut() = device_stats;
            }
            VfCurveEditorMsg::CursorUpdate { x, y, modifiers } => {
                self.cursor_position.set(Some((x, y)));
                self.drag_modifiers.set(modifiers);
            }
            VfCurveEditorMsg::DragStart => {
                if let Some(point) = self.hovered_point.get()
                    && self.allow_editing.value()
                {
                    self.dragging_point.set(Some(point));
                }
            }
            VfCurveEditorMsg::DragEnd => {
                if self.dragging_point.take().is_some() {
                    APP_BROKER.send(AppMsg::SettingsChanged);
                }
            }
            VfCurveEditorMsg::FlattenCurve => {
                if let Some(base_point_idx) = self.hovered_point.get() {
                    let (start, end) = self.visible_points_range();
                    let mut points = self.points.borrow_mut();
                    let points = &mut points[start..end];

                    let target_freq = points[base_point_idx].freq;

                    for point in points.iter_mut().skip(base_point_idx) {
                        point.freq = target_freq;
                    }

                    APP_BROKER.send(AppMsg::SettingsChanged);
                }
            }
            VfCurveEditorMsg::ResetCurve => {
                let mut points = self.points.borrow_mut();
                for point in points.iter_mut() {
                    point.freq = point.base_freq;
                }
            }
        }

        self.update_view(widgets, sender);
    }

    fn post_view() {
        drawing_area.queue_draw();
    }
}

impl VfCurveEditor {
    fn draw_chart(&self, ctx: &cairo::Context, width: i32, height: i32, colors: PlotColorScheme) {
        let (visible_start, visible_end) = self.visible_points_range();

        let mut points = self.points.borrow_mut();

        let points = &mut points[visible_start..visible_end];

        if points.is_empty() {
            return;
        }

        let backend = CairoBackend::new(ctx, (width as u32, height as u32)).unwrap();

        let root = backend.into_drawing_area();
        root.fill(&colors.background).unwrap();

        let min_point = points.first().unwrap();
        let max_point = points.last().unwrap();

        let x_spec = min_point.base_voltage..max_point.base_voltage;
        let y_spec = min_point.base_freq..(max_point.base_freq as f64 * 1.1) as u32;

        let mut chart = ChartBuilder::on(&root)
            .x_label_area_size(45)
            .y_label_area_size(110)
            .margin(50)
            .margin_bottom(20)
            .build_cartesian_2d(x_spec.clone(), y_spec.clone())
            .unwrap();

        chart
            .configure_mesh()
            .axis_style(colors.border_secondary)
            .bold_line_style(colors.border)
            .x_label_formatter(&|voltage| format!("{voltage} mV"))
            .y_label_formatter(&|clock| format!("{clock} MHz"))
            .x_label_style(("sans-serif", 14, &colors.text))
            .y_label_style(("sans-serif", 14, &colors.text))
            .x_desc(fl!(I18N, "voltage"))
            .y_desc(fl!(I18N, "frequency"))
            .draw()
            .unwrap();

        chart
            .draw_series(LineSeries::new(
                points.iter().map(vf_point_coords),
                &colors.success,
            ))
            .unwrap()
            .label(fl!(I18N, "vf-active-curve"))
            .legend(move |(x, y)| {
                Rectangle::new([(x - 15, y + 2), (x, y - 1)], colors.success.filled())
            });

        if points.iter().any(|point| point.freq != point.base_freq) {
            let base_line_style = colors.success.mix(0.3);
            chart
                .draw_series(LineSeries::new(
                    points.iter().map(vf_point_base_coords),
                    &base_line_style,
                ))
                .unwrap()
                .label(fl!(I18N, "vf-base-curve"))
                .legend(move |(x, y)| {
                    Rectangle::new([(x - 15, y + 2), (x, y - 1)], base_line_style.filled())
                });
        }

        if let Some((min_freq, max_freq)) = self.locked_clocks_range.get() {
            let mut curves = vec![(
                [(x_spec.start, max_freq), (x_spec.end, max_freq)],
                fl!(I18N, "max-gpu-clock"),
                colors.error,
            )];

            if min_freq >= y_spec.start {
                curves.push((
                    [(x_spec.start, min_freq), (x_spec.end, min_freq)],
                    fl!(I18N, "min-gpu-clock"),
                    colors.warning,
                ));
            }

            for (curve, label, color) in curves {
                chart
                    .draw_series(LineSeries::new(curve, &color))
                    .unwrap()
                    .label(label)
                    .legend(move |(x, y)| {
                        Rectangle::new([(x - 15, y + 2), (x, y - 1)], color.filled())
                    });
            }
        }

        let mut main_label = None;

        let stats = self.stats.borrow();
        if let Some(current_voltage) = stats.voltage.gpu
            && let Some(current_clock) = stats.clockspeed.gpu_clockspeed
        {
            let mut label = format!("Current: {current_clock} MHz @ {current_voltage} mV");

            if stats.core_power_state != Some(0) {
                label.push_str(" (Idle)");
            }

            main_label = Some(label);

            for line in [
                [
                    (current_voltage as u32, y_spec.start),
                    (current_voltage as u32, y_spec.end),
                ],
                [
                    (x_spec.start, current_clock as u32),
                    (x_spec.end, current_clock as u32),
                ],
            ] {
                chart
                    .draw_series(DashedLineSeries::new(
                        line,
                        8,
                        5,
                        ShapeStyle {
                            color: colors.text.mix(0.5),
                            filled: false,
                            stroke_width: 1,
                        },
                    ))
                    .unwrap();
            }
        }

        let active_style = colors.text;
        let hovered_style = colors.accent_bg;

        let hovered_point = self
            .dragging_point
            .get()
            .or_else(|| self.hovered_point.get());

        let main_series = chart
            .draw_series(PointSeries::of_element(
                points.iter().map(vf_point_coords).enumerate(),
                3,
                ShapeStyle::from(&colors.success).filled(),
                &|(i, coord), mut size, mut style| {
                    let is_active = self.stats.borrow().voltage.gpu == Some(coord.0 as u64);
                    if is_active {
                        style.color = active_style.to_rgba();
                        size = size * 3 / 2;
                    }

                    let mut negative_offset = false;
                    let mut text_width = 160;

                    let text = if hovered_point == Some(i) {
                        let point = points[i];

                        style.color = hovered_style.to_rgba();
                        size *= 2;

                        let mut text = format!("{} MHz", point.freq);

                        let offset = point.freq as i32 - point.base_freq as i32;
                        if offset != 0 {
                            let symbol = if offset > 0 {
                                "+"
                            } else {
                                negative_offset = true;
                                ""
                            };
                            write!(text, " ({symbol}{offset}) MHz").unwrap();
                            text_width += 50;
                        }

                        write!(text, " @ {} mV", point.voltage).unwrap();
                        text
                    } else {
                        String::new()
                    };

                    let text_style = TextStyle {
                        font: ("sans-serif", 15).into(),
                        color: colors.text.to_backend_color(),
                        pos: Pos::default(),
                    };

                    EmptyElement::at(coord)
                        + Circle::new((0, 0), size, style)
                        + Text::new(
                            text,
                            if negative_offset {
                                (0, 15)
                            } else {
                                (-text_width, -15)
                            },
                            text_style,
                        )
                },
            ))
            .unwrap();

        if let Some(label) = main_label {
            main_series
                .label(label)
                .legend(move |(x, y)| Circle::new((x - 7, y), 5, active_style.filled()));
        }

        chart
            .configure_series_labels()
            .position(SeriesLabelPosition::UpperLeft)
            .margin(20)
            .legend_area_size(5)
            .label_font(("sans-serif", 16, &colors.text))
            .position(SeriesLabelPosition::UpperLeft)
            .background_style(colors.background.mix(0.6))
            .draw()
            .unwrap();

        let freq_range = chart.as_coord_spec().get_y_range();
        let translate = chart.into_coord_trans();

        let voltage_hover_margin =
            ((max_point.voltage - min_point.voltage) as f32 * POINT_VOLTAGE_HOVER_MARGIN) as i32;
        let freq_hover_margin =
            ((max_point.freq - min_point.freq) as f32 * POINT_FREQ_HOVER_MARGIN) as i32;

        let hovered_coords = self
            .cursor_position
            .get()
            .and_then(|(x, y)| translate((x as i32, y as i32)));

        let hovered_point = hovered_coords.and_then(|(voltage, freq)| {
            points
                .iter()
                .enumerate()
                .map(|(i, point)| {
                    let voltage_distance = (voltage as i32 - point.voltage as i32).abs();
                    let freq_distance = (freq as i32 - point.freq as i32).abs();
                    (i, voltage_distance, freq_distance)
                })
                .filter(|(_, voltage_distance, freq_distance)| {
                    *voltage_distance < voltage_hover_margin && *freq_distance < freq_hover_margin
                })
                .min_by(|lhs, rhs| lhs.1.cmp(&rhs.1).then_with(|| lhs.2.cmp(&rhs.2)))
                .map(|(i, _, _)| i)
        });
        self.hovered_point.set(hovered_point);

        if let Some((_voltage, freq)) = hovered_coords
            && let Some(point_idx) = self.dragging_point.get()
        {
            let new_freq = freq.clamp(freq_range.start, freq_range.end);
            let drag_delta = new_freq as i32 - points[point_idx].freq as i32;

            if self
                .drag_modifiers
                .get()
                .contains(gdk::ModifierType::SHIFT_MASK)
            {
                for point in points.iter_mut() {
                    let new_freq = (point.freq as i32 + drag_delta) as u32;
                    if new_freq > 0 {
                        point.freq = new_freq;
                    }
                }
            } else {
                points[point_idx].freq = new_freq;
            }
        }

        root.present().unwrap();
    }

    fn visible_points_range(&self) -> (usize, usize) {
        let len = self.points.borrow().len();
        let start = (len as f64 * (self.visible_range_start.value() / 100.0)) as usize;
        let start = cmp::min(start, len);

        let end = (len as f64 * (self.visible_range_end.value() / 100.0)) as usize;
        let end = cmp::min(end, len);

        (cmp::min(start, end), cmp::max(end, start))
    }

    pub fn get_configured_curve(&self) -> IndexMap<u8, config::CurvePoint> {
        if !self.allow_editing.value() {
            return IndexMap::new();
        }

        self.points
            .borrow()
            .iter()
            .map(|point| {
                let vf_point = config::CurvePoint {
                    voltage: Some(point.voltage as i32),
                    clockspeed: Some(point.freq as i32),
                };
                (point.index, vf_point)
            })
            .collect()
    }
}

fn vf_point_coords(point: &NvidiaVfPoint) -> (u32, u32) {
    (point.voltage, point.freq)
}

fn vf_point_base_coords(point: &NvidiaVfPoint) -> (u32, u32) {
    (point.base_voltage, point.base_freq)
}

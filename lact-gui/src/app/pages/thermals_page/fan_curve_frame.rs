use crate::{
    app::{graphs_window::plot::PlotColorScheme, msg::AppMsg, pages::oc_adjustment::OcAdjustment},
    APP_BROKER,
};
use gtk::{
    gdk,
    prelude::{BoxExt, ButtonExt, DrawingAreaExtManual, OrientableExt, WidgetExt},
};
use lact_schema::{default_fan_curve, FanCurveMap};
use plotters::{
    chart::ChartBuilder,
    prelude::{Circle, EmptyElement, IntoDrawingArea, Text},
    series::{LineSeries, PointSeries},
    style::{full_palette::LIGHTBLUE, text_anchor::Pos, Color, ShapeStyle, TextStyle},
};
use plotters_cairo::CairoBackend;
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt};
use std::{
    cell::{Cell, RefCell},
    ops::RangeInclusive,
    rc::Rc,
    sync::atomic::{AtomicBool, Ordering},
};

pub const DEFAULT_TEMP_RANGE: RangeInclusive<f32> = 20.0..=115.0;
pub const DEFAULT_SPEED_RANGE: RangeInclusive<f32> = 0.0..=1.0;
const DEFAULT_CHANGE_THRESHOLD: u64 = 2;
const DEFAULT_SPINDOWN_DELAY_MS: u64 = 5000;

#[derive(Clone)]
pub struct FanCurveFrame {
    is_dragging: Rc<AtomicBool>,

    data: Rc<RefCell<Vec<(i32, f32)>>>,
    speed_range: Rc<RefCell<RangeInclusive<f32>>>,
    temperature_range: Rc<RefCell<RangeInclusive<f32>>>,
    spindown_delay_adj: OcAdjustment,
    change_threshold_adj: OcAdjustment,

    /// Index of the point currently being dragged
    drag_point: Rc<Cell<Option<usize>>>,
    /// Where the point was last moved to
    drag_coord: Rc<Cell<Option<(f64, f64)>>>,
}

#[derive(Debug)]
pub enum FanCurveFrameMsg {
    Curve {
        curve: FanCurveMap,
        speed_range: RangeInclusive<f32>,
        temperature_range: RangeInclusive<f32>,
    },
    DragStart,
    DragUpdate(f64, f64),
    DragEnd,
    AddPoint,
    RemovePoint,
    DefaultCurve,
}

#[relm4::component(pub)]
impl relm4::Component for FanCurveFrame {
    type Init = ();
    type Input = FanCurveFrameMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 5,

            #[name = "drawing_area"]
            gtk::DrawingArea {
                set_expand: true,
                set_draw_func[model = model.clone()] => move |area, ctx, width, height| {
                    let style_context = area.style_context();
                    let colors = PlotColorScheme::from_context(&style_context).unwrap_or_default();
                    model.draw_chart(ctx, width, height,colors);
                },

                add_controller = gtk::GestureClick {
                    connect_pressed[sender] => move |_, _, x, y| {
                        sender.input(FanCurveFrameMsg::DragStart);
                        sender.input(FanCurveFrameMsg::DragUpdate(x, y));
                    },
                    connect_released[sender] => move |_, _, _x, _y| {
                        sender.input(FanCurveFrameMsg::DragEnd);
                    }
                },

                add_controller = gtk::EventControllerMotion {
                    connect_motion[sender] => move |_, x, y| {
                        sender.input(FanCurveFrameMsg::DragUpdate(x, y));
                    },
                },

                #[watch]
                set_cursor: if model.is_dragging.load(Ordering::Relaxed) {
                    gdk::Cursor::from_name("move", None)
                } else {
                    None
                }.as_ref(),
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 5,
                set_halign: gtk::Align::End,

                gtk::Button {
                    set_icon_name: "list-add-symbolic",
                    connect_clicked => FanCurveFrameMsg::AddPoint,
                },

                gtk::Button {
                    set_icon_name: "list-remove-symbolic",
                    connect_clicked => FanCurveFrameMsg::RemovePoint,
                },

                gtk::Button {
                    set_label: "Default",
                    connect_clicked => FanCurveFrameMsg::DefaultCurve,
                },
            }
        }
    }

    fn post_view() {
        if self.is_dragging.load(Ordering::SeqCst) {
            drawing_area.queue_draw();
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            is_dragging: Rc::new(AtomicBool::new(false)),
            speed_range: Rc::new(RefCell::new(DEFAULT_SPEED_RANGE)),
            temperature_range: Rc::new(RefCell::new(DEFAULT_TEMP_RANGE)),
            spindown_delay_adj: OcAdjustment::new(
                DEFAULT_SPINDOWN_DELAY_MS as f64,
                0.0,
                30_000.0,
                10.0,
                10.0,
            ),
            change_threshold_adj: OcAdjustment::new(
                DEFAULT_CHANGE_THRESHOLD as f64,
                0.0,
                10.0,
                1.0,
                1.0,
            ),
            data: Rc::default(),
            drag_coord: Rc::default(),
            drag_point: Rc::default(),
        };

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
        match msg {
            FanCurveFrameMsg::Curve {
                curve,
                temperature_range,
                speed_range,
            } => {
                *self.data.borrow_mut() = curve.into_iter().collect();
                *self.speed_range.borrow_mut() = speed_range;
                *self.temperature_range.borrow_mut() = temperature_range;
                widgets.drawing_area.queue_draw();
            }
            FanCurveFrameMsg::DragStart => {
                self.is_dragging.store(true, Ordering::SeqCst);
            }
            FanCurveFrameMsg::DragUpdate(x, y) => {
                if self.is_dragging.load(Ordering::SeqCst) {
                    self.drag_coord.set(Some((x, y)));
                }
            }
            FanCurveFrameMsg::DragEnd => {
                self.drag_coord.take();
                self.drag_point.take();
                self.is_dragging.store(false, Ordering::SeqCst);
            }
            FanCurveFrameMsg::AddPoint => {
                let temp_range = self.temperature_range.borrow();
                let speed_range = self.speed_range.borrow();
                self.data
                    .borrow_mut()
                    .push((*temp_range.end() as i32, *speed_range.end()));
                widgets.drawing_area.queue_draw();
            }
            FanCurveFrameMsg::RemovePoint => {
                self.data.borrow_mut().pop();
                widgets.drawing_area.queue_draw();
            }
            FanCurveFrameMsg::DefaultCurve => {
                *self.data.borrow_mut() = default_fan_curve().into_iter().collect();
                widgets.drawing_area.queue_draw();
            }
        }
        self.update_view(widgets, sender);
    }
}

impl FanCurveFrame {
    pub fn get_curve(&self) -> FanCurveMap {
        self.data.borrow().iter().copied().collect()
    }

    fn draw_chart(&self, ctx: &cairo::Context, width: i32, height: i32, colors: PlotColorScheme) {
        let cairo_backend = CairoBackend::new(ctx, (width as u32, height as u32)).unwrap();

        let drag_coord = self.drag_coord.take();

        let new_value = draw_chart(
            cairo_backend,
            &self.data.borrow(),
            drag_coord,
            colors,
            &self.temperature_range.borrow(),
            &self.speed_range.borrow(),
        );
        if let Some(mut new_value) = new_value {
            let drag_point_idx = match self.drag_point.get() {
                Some(idx) => Some(idx),
                None => {
                    let point = self.data.borrow().iter().position(|(data_x, data_y)| {
                        (*data_x as f32 - new_value.0).abs() <= 3.0
                            && (*data_y - new_value.1).abs() <= 0.03
                    });
                    self.drag_point.set(point);
                    point
                }
            };
            if let Some(idx) = drag_point_idx {
                normalize_to_range(&mut new_value.0, &self.temperature_range.borrow());
                normalize_to_range(&mut new_value.1, &self.speed_range.borrow());
                self.data.borrow_mut()[idx] = (new_value.0 as i32, new_value.1);

                APP_BROKER.send(AppMsg::SettingsChanged);
            }
        }
    }
}

fn normalize_to_range(value: &mut f32, range: &RangeInclusive<f32>) {
    *value = f32::max(*value, *range.start());
    *value = f32::min(*value, *range.end());
}

fn draw_chart(
    backend: CairoBackend,
    data: &[(i32, f32)],
    translate_coord: Option<(f64, f64)>,
    colors: PlotColorScheme,
    temp_range: &RangeInclusive<f32>,
    speed_range: &RangeInclusive<f32>,
) -> Option<(f32, f32)> {
    let root = backend.into_drawing_area();
    root.fill(&colors.background).unwrap();

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(45)
        .y_label_area_size(60)
        .margin(10)
        .build_cartesian_2d(
            *temp_range.start()..*temp_range.end(),
            *speed_range.start()..*speed_range.end(),
        )
        .unwrap();

    chart
        .configure_mesh()
        .axis_style(colors.border_secondary)
        .bold_line_style(colors.border)
        .max_light_lines(0)
        .x_label_formatter(&|temp| format!("{temp:.}°C"))
        .y_label_formatter(&|speed| format!("{:.0}%", speed * 100.0))
        .x_label_style(("sans-serif", 14, &colors.text))
        .y_label_style(("sans-serif", 14, &colors.text))
        .x_desc("Temperature")
        .y_desc("Speed")
        .draw()
        .unwrap();

    chart
        .draw_series(LineSeries::new(
            data.first()
                .map(|(_, y)| (*temp_range.start(), { *y }))
                .into_iter()
                .chain(data.iter().map(|(x, y)| (*x as f32, *y)))
                .chain(data.last().map(|(_, y)| (*temp_range.end(), *y))),
            &LIGHTBLUE,
        ))
        .unwrap();

    chart
        .draw_series(PointSeries::of_element(
            data.iter().map(|(x, y)| (*x as f32, *y)),
            8,
            ShapeStyle::from(&LIGHTBLUE).filled(),
            &|coord, size, style| {
                EmptyElement::at(coord)
                    + Circle::new((0, 0), size, style)
                    + Text::new(
                        format!("{:.0}% at {}°C", coord.1 * 100.0, coord.0),
                        (
                            if coord.0 - temp_range.start() < 5.0 {
                                0
                            } else if temp_range.end() - coord.0 < 8.0 {
                                -85
                            } else {
                                -35
                            },
                            if coord.1 - speed_range.start() < 0.06 {
                                -25
                            } else {
                                15
                            },
                        ),
                        TextStyle {
                            font: ("sans-serif", 15).into(),
                            color: colors.text.to_backend_color(),
                            pos: Pos::default(),
                        },
                    )
            },
        ))
        .unwrap();

    let mapped_coord =
        translate_coord.and_then(|(x, y)| chart.into_coord_trans()((x as i32, y as i32)));

    root.present().unwrap();

    mapped_coord
}

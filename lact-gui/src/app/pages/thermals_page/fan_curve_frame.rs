use super::{adj_is_empty, FanSettingRow, PmfwOptions};
use crate::{
    app::{graphs_window::plot::PlotColorScheme, msg::AppMsg, pages::oc_adjustment::OcAdjustment},
    APP_BROKER,
};
use amdgpu_sysfs::hw_mon::Temperature;
use gtk::{
    gdk,
    gio::prelude::ListModelExt,
    glib::{
        self,
        object::{Cast, ObjectExt},
        SignalHandlerId,
    },
    prelude::{
        AdjustmentExt, BoxExt, ButtonExt, DrawingAreaExtManual, OrientableExt, RangeExt, WidgetExt,
    },
};
use lact_schema::{default_fan_curve, FanCurveMap};
use plotters::{
    chart::ChartBuilder,
    prelude::{Circle, EmptyElement, IntoDrawingArea, Text},
    series::{LineSeries, PointSeries},
    style::{full_palette::LIGHTBLUE, text_anchor::Pos, Color, ShapeStyle, TextStyle},
};
use plotters_cairo::CairoBackend;
use relm4::{
    binding::{ConnectBinding, U32Binding},
    ComponentParts, ComponentSender, RelmObjectExt, RelmWidgetExt,
};
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    ops::RangeInclusive,
    rc::Rc,
    sync::atomic::{AtomicBool, Ordering},
};

pub const DEFAULT_TEMP_RANGE: RangeInclusive<f32> = 20.0..=115.0;
pub const DEFAULT_SPEED_RANGE: RangeInclusive<f32> = 0.0..=1.0;
const DEFAULT_CHANGE_THRESHOLD: u64 = 2;
const DEFAULT_AUTO_THRESHOLD: u64 = 0;
const DEFAULT_SPINDOWN_DELAY_MS: u64 = 5000;

#[derive(Clone)]
pub(super) struct FanCurveFrame {
    pmfw_options: PmfwOptions,

    data: Rc<RefCell<Vec<(i32, f32)>>>,
    speed_range: Rc<RefCell<RangeInclusive<f32>>>,
    temperature_range: Rc<RefCell<RangeInclusive<f32>>>,
    temp_keys: gtk::StringList,
    current_temp_key: U32Binding,

    spindown_delay_adj: OcAdjustment,
    change_threshold_adj: OcAdjustment,
    auto_threshold_adj: OcAdjustment,
    change_signals: Rc<[(glib::Object, SignalHandlerId)]>,

    is_dragging: Rc<AtomicBool>,
    /// Index of the point currently being dragged
    drag_point: Rc<Cell<Option<usize>>>,
    /// Where the point was last moved to
    drag_coord: Rc<Cell<Option<(f64, f64)>>>,
}

#[derive(Debug)]
pub(super) enum FanCurveFrameMsg {
    Curve(CurveSetupMsg),
    DragStart,
    DragUpdate(f64, f64),
    DragEnd,
    AddPoint,
    RemovePoint,
    DefaultCurve,
}

#[derive(Debug)]
pub(super) struct CurveSetupMsg {
    pub curve: FanCurveMap,
    pub current_temperatures: HashMap<String, Temperature>,
    pub temperature_key: Option<String>,
    pub speed_range: RangeInclusive<f32>,
    pub temperature_range: RangeInclusive<f32>,
    // Non-PMFW only
    pub spindown_delay: Option<u64>,
    pub change_threshold: Option<u64>,
    /// Nvidia only
    pub auto_threshold_supported: bool,
    pub auto_threshold: Option<u64>,
}

#[relm4::component(pub)]
impl relm4::Component for FanCurveFrame {
    type Init = PmfwOptions;
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
                set_height_request: 350,
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
                    #[watch]
                    set_visible: model.pmfw_options.is_empty(),
                },

                gtk::Button {
                    set_icon_name: "list-remove-symbolic",
                    connect_clicked => FanCurveFrameMsg::RemovePoint,
                    #[watch]
                    set_visible: model.pmfw_options.is_empty(),
                },

                gtk::Button {
                    set_label: "Default",
                    connect_clicked => FanCurveFrameMsg::DefaultCurve,
                },
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 5,
                #[watch]
                set_visible: model.temp_keys_available(),

                gtk::Label {
                    set_label: "Temperature Sensor",
                    set_xalign: 0.0,
                    set_size_group: &label_size_group,
                },

                #[name = "temp_key_dropdown"]
                gtk::DropDown {
                    set_hexpand: true,
                    set_halign: gtk::Align::End,
                    add_binding: (&model.current_temp_key, "selected"),
                    set_model: Some(&model.temp_keys),
                },
            },

            #[template]
            FanSettingRow {
                #[watch]
                set_visible: model.pmfw_options.is_empty(),

                #[template_child]
                label {
                    set_label: "Spindown Delay (ms)",
                    set_tooltip: "How long the GPU needs to remain at a lower temperature value before ramping down the fan",
                    set_size_group: &label_size_group,
                },

                #[template_child]
                scale {
                    set_adjustment: &model.spindown_delay_adj,
                },

                #[template_child]
                spinbutton {
                    set_adjustment: &model.spindown_delay_adj,
                    set_size_group: &spin_size_group,
                },
            },

            #[template]
            FanSettingRow {
                #[watch]
                set_visible: model.pmfw_options.is_empty(),

                #[template_child]
                label {
                    set_label: "Speed change threshold (°C)",
                    set_size_group: &label_size_group,
                },

                #[template_child]
                scale {
                    set_adjustment: &model.change_threshold_adj,
                },

                #[template_child]
                spinbutton {
                    set_adjustment: &model.change_threshold_adj,
                    set_size_group: &spin_size_group,
                },
            },

            #[template]
            FanSettingRow {
                #[watch]
                set_visible: !adj_is_empty(&model.auto_threshold_adj),

                #[template_child]
                label {
                    set_label: "Automatic Mode Threshold (°C)",
                    set_tooltip: "Switch fan control to auto mode when the temperature is below this point.

Many Nvidia GPUs only support stopping the fan in the automatic fan control mode, while a custom curve has a limited speed range such as 30-100%.

This option allows to work around this limitation by only using the custom curve when above a specific temperature, \
    with the card's builtin auto mode that supports zero RPM being used below it.",
                    set_size_group: &label_size_group,
                },

                #[template_child]
                scale {
                    set_adjustment: &model.auto_threshold_adj,
                },

                #[template_child]
                spinbutton {
                    set_adjustment: &model.auto_threshold_adj,
                    set_size_group: &spin_size_group,
                },
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 5,
                #[watch]
                set_visible: model.pmfw_options.zero_rpm_available.get(),

                gtk::Label {
                    set_label: "Zero RPM",
                    set_xalign: 0.0,
                    set_size_group: &label_size_group,
                },

                gtk::Switch {
                    bind: &model.pmfw_options.zero_rpm,
                    set_hexpand: true,
                    set_halign: gtk::Align::End,
                },
            },

            #[template]
            FanSettingRow {
                #[watch]
                set_visible: !adj_is_empty(&model.pmfw_options.zero_rpm_temperature),

                #[template_child]
                label {
                    set_label: "Zero RPM stop temperature (°C)",
                    set_size_group: &label_size_group,
                },

                #[template_child]
                scale {
                    set_adjustment: &model.pmfw_options.zero_rpm_temperature,
                },

                #[template_child]
                spinbutton {
                    set_adjustment: &model.pmfw_options.zero_rpm_temperature,
                    set_size_group: &spin_size_group,
                },
            },
        }
    }

    fn post_view() {
        if self.is_dragging.load(Ordering::SeqCst) {
            drawing_area.queue_draw();
        }
    }

    fn init(
        pmfw_options: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let spindown_delay_adj =
            OcAdjustment::new(DEFAULT_SPINDOWN_DELAY_MS as f64, 0.0, 30_000.0, 10.0, 10.0);
        let change_threshold_adj =
            OcAdjustment::new(DEFAULT_CHANGE_THRESHOLD as f64, 0.0, 10.0, 1.0, 1.0);
        let auto_threshold_adj = OcAdjustment::new(0.0, 0.0, 0.0, 1.0, 5.0);
        let temp_keys = gtk::StringList::default();
        let current_temp_key = U32Binding::new(0u32);

        let change_signals = [
            &spindown_delay_adj,
            &change_threshold_adj,
            &auto_threshold_adj,
        ]
        .into_iter()
        .map(|adj| {
            let signal = adj.connect_value_changed(|_| {
                APP_BROKER.send(AppMsg::SettingsChanged);
            });
            (adj.clone().upcast(), signal)
        })
        .chain([(
            current_temp_key.clone().upcast(),
            current_temp_key.connect_value_notify(|_| {
                APP_BROKER.send(AppMsg::SettingsChanged);
            }),
        )])
        .collect();

        let model = Self {
            pmfw_options,
            is_dragging: Rc::new(AtomicBool::new(false)),
            speed_range: Rc::new(RefCell::new(DEFAULT_SPEED_RANGE)),
            temperature_range: Rc::new(RefCell::new(DEFAULT_TEMP_RANGE)),
            spindown_delay_adj,
            change_threshold_adj,
            auto_threshold_adj,
            temp_keys,
            current_temp_key,
            change_signals,
            data: Rc::default(),
            drag_coord: Rc::default(),
            drag_point: Rc::default(),
        };

        let label_size_group = gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal);
        let spin_size_group = gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal);
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
            FanCurveFrameMsg::Curve(msg) => {
                *self.data.borrow_mut() =
                    normalize_fan_curve(msg.curve, &msg.temperature_range, &msg.speed_range)
                        .collect();
                *self.speed_range.borrow_mut() = msg.speed_range;
                *self.temperature_range.borrow_mut() = msg.temperature_range.clone();

                for (adj, signal) in self.change_signals.iter() {
                    adj.block_signal(signal);
                }

                let mut temp_keys = msg.current_temperatures.into_keys().collect::<Vec<_>>();
                temp_keys.sort();

                let selected_idx = msg
                    .temperature_key
                    .and_then(|current_key| temp_keys.iter().position(|key| *key == current_key))
                    .or_else(|| temp_keys.iter().position(|key| key == "edge"))
                    .or(if temp_keys.is_empty() { None } else { Some(0) });

                while self.temp_keys.n_items() > 0 {
                    self.temp_keys.remove(0);
                }
                for key in temp_keys {
                    self.temp_keys.append(&key);
                }

                if let Some(idx) = selected_idx {
                    widgets.temp_key_dropdown.set_selected(idx as u32);
                }

                self.spindown_delay_adj.set_initial_value(
                    msg.spindown_delay.unwrap_or(DEFAULT_SPINDOWN_DELAY_MS) as f64,
                );
                self.change_threshold_adj.set_initial_value(
                    msg.change_threshold.unwrap_or(DEFAULT_CHANGE_THRESHOLD) as f64,
                );

                if msg.auto_threshold_supported {
                    self.auto_threshold_adj.set_lower(0.0);
                    self.auto_threshold_adj
                        .set_upper(*msg.temperature_range.end() as f64);
                    self.auto_threshold_adj.set_initial_value(
                        msg.auto_threshold.unwrap_or(DEFAULT_AUTO_THRESHOLD) as f64,
                    );
                } else {
                    self.auto_threshold_adj.set_lower(0.0);
                    self.auto_threshold_adj.set_upper(0.0);
                }

                for (adj, signal) in self.change_signals.iter() {
                    adj.unblock_signal(signal);
                }

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

                self.edit_curve(
                    |curve| {
                        curve.push((*temp_range.end() as i32, *speed_range.end()));
                    },
                    widgets,
                );
            }
            FanCurveFrameMsg::RemovePoint => {
                self.edit_curve(
                    |curve| {
                        curve.pop();
                    },
                    widgets,
                );
            }
            FanCurveFrameMsg::DefaultCurve => {
                self.edit_curve(
                    |curve| {
                        *curve = normalize_fan_curve(
                            default_fan_curve(),
                            &self.temperature_range.borrow(),
                            &self.speed_range.borrow(),
                        )
                        .collect();
                    },
                    widgets,
                );
                self.spindown_delay_adj
                    .set_value(DEFAULT_SPINDOWN_DELAY_MS as f64);
                self.change_threshold_adj
                    .set_value(DEFAULT_CHANGE_THRESHOLD as f64);
            }
        }
        self.update_view(widgets, sender);
    }
}

impl FanCurveFrame {
    pub fn get_curve(&self) -> FanCurveMap {
        self.data.borrow().iter().copied().collect()
    }

    pub fn spindown_delay(&self) -> u64 {
        self.spindown_delay_adj.value() as u64
    }

    pub fn change_threshold(&self) -> u64 {
        self.change_threshold_adj.value() as u64
    }

    pub fn temperature_key(&self) -> Option<String> {
        if self.temp_keys_available() {
            self.temp_keys
                .string(self.current_temp_key.value())
                .map(|obj| obj.to_string())
        } else {
            None
        }
    }

    pub fn auto_threshold(&self) -> Option<u64> {
        self.auto_threshold_adj
            .get_changed_value(false)
            .map(|val| val as u64)
    }

    fn temp_keys_available(&self) -> bool {
        self.pmfw_options.is_empty() && self.temp_keys.n_items() > 1
    }

    fn edit_curve(&self, f: impl FnOnce(&mut Vec<(i32, f32)>), widgets: &FanCurveFrameWidgets) {
        f(&mut self.data.borrow_mut());

        widgets.drawing_area.queue_draw();
        APP_BROKER.send(AppMsg::SettingsChanged);
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

fn normalize_fan_curve<'a>(
    curve: impl IntoIterator<Item = (i32, f32)> + 'a,
    temperature_range: &'a RangeInclusive<f32>,
    speed_range: &'a RangeInclusive<f32>,
) -> impl Iterator<Item = (i32, f32)> + 'a {
    curve.into_iter().map(|(temp, mut speed)| {
        let mut temp = temp as f32;
        normalize_to_range(&mut temp, temperature_range);
        normalize_to_range(&mut speed, speed_range);
        (temp as i32, speed)
    })
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

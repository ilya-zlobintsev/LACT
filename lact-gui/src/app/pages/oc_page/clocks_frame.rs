mod adjustment_row;

use crate::{
    APP_BROKER, I18N,
    app::{msg::AppMsg, page_section::PageSection},
};
use adjustment_row::{ClockAdjustmentRow, ClockAdjustmentRowMsg, ClocksData};
use amdgpu_sysfs::gpu_handle::overdrive::ClocksTableGen as AmdClocksTable;
use gtk::{
    glib::object::ObjectExt,
    pango,
    prelude::{BoxExt, ButtonExt, CheckButtonExt, OrientableExt, WidgetExt},
};
use i18n_embed_fl::fl;
use lact_schema::{
    ClocksTable, IntelClocksTable, NvidiaClockOffset, NvidiaClocksTable,
    request::{ClockspeedType, SetClocksCommand},
};
use relm4::{
    ComponentParts, ComponentSender, RelmObjectExt, RelmWidgetExt, binding::BoolBinding, css,
    factory::FactoryHashMap,
};

// This should not end up being used in practice
const DEFAULT_VOLTAGE_OFFSET_RANGE: i32 = 250;

pub struct ClocksFrame {
    core_clocks: FactoryHashMap<ClockspeedType, ClockAdjustmentRow>,
    core_voltages: FactoryHashMap<ClockspeedType, ClockAdjustmentRow>,
    vram_clocks: FactoryHashMap<ClockspeedType, ClockAdjustmentRow>,
    core_curve_clocks: FactoryHashMap<ClockspeedType, ClockAdjustmentRow>,
    vram_curve_clocks: FactoryHashMap<ClockspeedType, ClockAdjustmentRow>,
    core_curve_voltages: FactoryHashMap<ClockspeedType, ClockAdjustmentRow>,
    vram_curve_voltages: FactoryHashMap<ClockspeedType, ClockAdjustmentRow>,
    vram_clock_ratio: f64,
    show_nvidia_options: bool,
    show_all_pstates: BoolBinding,
    enable_gpu_locked_clocks: BoolBinding,
    enable_vram_locked_clocks: BoolBinding,
}

#[derive(Debug)]
pub enum ClocksFrameMsg {
    Clocks(Option<ClocksTable>),
    VramRatio(f64),
    TogglePStatesVisibility,
}

#[relm4::component(pub)]
impl relm4::Component for ClocksFrame {
    type Init = ();
    type Input = ClocksFrameMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        PageSection::new(&fl!(I18N, "overclock-section")) {
            add_css_class: "clocks-frame",

            append_header = &gtk::Box {
                set_spacing: 10,
                set_hexpand: true,
                set_halign: gtk::Align::End,

                append = &gtk::MenuButton {
                    #[watch]
                    set_visible: model.show_nvidia_options,
                    set_label: &fl!(I18N, "nvidia-oc-info"),

                    #[wrap(Some)]
                    set_popover = &gtk::Popover {
                        gtk::Label {
                            set_margin_all: 5,
                            set_markup: &fl!(I18N, "nvidia-oc-description"),
                            set_wrap: true,
                            set_max_width_chars: 75,
                        }
                    }
                },

                append = &gtk::Button {
                    set_label: &fl!(I18N, "reset-button"),
                    set_tooltip_text: Some(&fl!(I18N, "reset-oc-tooltip")),

                    add_css_class: css::DESTRUCTIVE_ACTION,

                    #[watch]
                    set_visible: !model.core_clocks.is_empty() || !model.core_voltages.is_empty() || !model.vram_clocks.is_empty() || !model.core_curve_clocks.is_empty() || !model.vram_curve_clocks.is_empty() || !model.core_curve_voltages.is_empty() || !model.vram_curve_voltages.is_empty(),

                    connect_clicked => move |_| {
                        APP_BROKER.send(AppMsg::ResetClocks);
                    }
                },
            },

            append_child = &gtk::Label {
                set_label: &fl!(I18N, "oc-warning"),
                set_wrap_mode: pango::WrapMode::Word,
                set_halign: gtk::Align::Start,
                add_css_class: css::WARNING,
                add_css_class: css::DIM_LABEL,
            },

            append_child = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,

                append = &gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_homogeneous: true,
                    set_hexpand: true,

                    append = &gtk::CheckButton {
                        #[watch]
                        set_visible: model.core_clocks.values().any(|row| row.is_secondary) || model.core_voltages.values().any(|row| row.is_secondary) || model.vram_clocks.values().any(|row| row.is_secondary) || model.core_curve_clocks.values().any(|row| row.is_secondary) || model.vram_curve_clocks.values().any(|row| row.is_secondary) || model.core_curve_voltages.values().any(|row| row.is_secondary) || model.vram_curve_voltages.values().any(|row| row.is_secondary),
                        set_label: Some(&fl!(I18N, "show-all-pstates")),
                        add_binding["active"]: &model.show_all_pstates,
                    },

                    append: gpu_locked_clocks_togglebutton = &gtk::CheckButton {
                        #[watch]
                        set_visible: model.show_nvidia_options,
                        set_label: Some(&fl!(I18N, "enable-gpu-locked-clocks")),
                        add_binding["active"]: &model.enable_gpu_locked_clocks,
                        connect_toggled => move |_| {
                            APP_BROKER.send(AppMsg::SettingsChanged);
                        } @ gpu_locked_clock_signal,
                    },

                    append: vram_locked_clocks_togglebutton = &gtk::CheckButton {
                        #[watch]
                        set_visible: model.show_nvidia_options,
                        set_label: Some(&fl!(I18N, "enable-vram-locked-clocks")),
                        add_binding["active"]: &model.enable_vram_locked_clocks,
                        connect_toggled => move |_| {
                            APP_BROKER.send(AppMsg::SettingsChanged);
                        } @ vram_locked_clock_signal,
                    },
                },

                append = &gtk::Label {
                    #[watch]
                    set_visible: model.show_all_pstates.value() && model.show_nvidia_options,
                    // add_binding["visible"]: &model.show_all_pstates,

                    set_margin_horizontal: 5,
                    set_markup: &fl!(I18N, "pstate-list-description"),
                    set_wrap_mode: pango::WrapMode::Word,
                    set_halign: gtk::Align::Start,
                },
            },

            append_child = &gtk::FlowBox {
                set_orientation: gtk::Orientation::Horizontal,
                set_selection_mode: gtk::SelectionMode::None,
                set_max_children_per_line: 2,
                set_min_children_per_line: 1,
                set_column_spacing: 15,
                set_row_spacing: 15,
                set_homogeneous: false,
                set_valign: gtk::Align::Start,
                set_hexpand: true,

                // Left Column: Core/GPU
                append = &gtk::FlowBoxChild {
                    add_css_class: "clocks-frame-group",
                    set_valign: gtk::Align::Start,
                    #[watch]
                    set_visible: !model.core_clocks.is_empty() || !model.core_voltages.is_empty() || !model.core_curve_clocks.is_empty() || !model.core_curve_voltages.is_empty(),

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 15,
                        set_hexpand: true,
                        set_valign: gtk::Align::Start,

                        append = &gtk::Box {
                            add_css_class: css::FRAME,
                            set_orientation: gtk::Orientation::Vertical,
                            set_valign: gtk::Align::Start,
                            set_spacing: 5,
                            #[watch]
                            set_visible: !model.core_clocks.is_empty(),
                            append = model.core_clocks.widget(),
                        },
                        append = &gtk::Box {
                            add_css_class: css::FRAME,
                            set_orientation: gtk::Orientation::Vertical,
                            set_valign: gtk::Align::Start,
                            set_spacing: 5,
                            #[watch]
                            set_visible: !model.core_voltages.is_empty(),
                            append = model.core_voltages.widget(),
                        },
                        append = &gtk::Box {
                            add_css_class: css::FRAME,
                            set_orientation: gtk::Orientation::Vertical,
                            set_valign: gtk::Align::Start,
                            set_spacing: 5,
                            #[watch]
                            set_visible: !model.core_curve_clocks.is_empty(),
                            append = model.core_curve_clocks.widget(),
                        },
                        append = &gtk::Box {
                            add_css_class: css::FRAME,
                            set_orientation: gtk::Orientation::Vertical,
                            set_valign: gtk::Align::Start,
                            set_spacing: 5,
                            #[watch]
                            set_visible: !model.core_curve_voltages.is_empty(),
                            append = model.core_curve_voltages.widget(),
                        },
                    },
                },

                // Right Column: VRAM
                append = &gtk::FlowBoxChild {
                    add_css_class: "clocks-frame-group",
                    set_valign: gtk::Align::Start,
                    #[watch]
                    set_visible: !model.vram_clocks.is_empty() || !model.vram_curve_clocks.is_empty() || !model.vram_curve_voltages.is_empty(),

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 15,
                        set_hexpand: true,
                        set_valign: gtk::Align::Start,

                        append = &gtk::Box {
                            add_css_class: css::FRAME,
                            set_orientation: gtk::Orientation::Vertical,
                            set_valign: gtk::Align::Start,
                            set_spacing: 5,
                            #[watch]
                            set_visible: !model.vram_clocks.is_empty(),
                            append = model.vram_clocks.widget(),
                        },
                        append = &gtk::Box {
                            add_css_class: css::FRAME,
                            set_orientation: gtk::Orientation::Vertical,
                            set_valign: gtk::Align::Start,
                            set_spacing: 5,
                            #[watch]
                            set_visible: !model.vram_curve_clocks.is_empty(),
                            append = model.vram_curve_clocks.widget(),
                        },
                        append = &gtk::Box {
                            add_css_class: css::FRAME,
                            set_orientation: gtk::Orientation::Vertical,
                            set_valign: gtk::Align::Start,
                            set_spacing: 5,
                            #[watch]
                            set_visible: !model.vram_curve_voltages.is_empty(),
                            append = model.vram_curve_voltages.widget(),
                        },
                    },
                },
            },

            append_child = &gtk::Label {
                set_label: &fl!(I18N, "no-clocks-data"),
                set_margin_horizontal: 10,
                set_halign: gtk::Align::Start,
                #[watch]
                set_visible: model.core_clocks.is_empty() && model.core_voltages.is_empty() && model.vram_clocks.is_empty() && model.core_curve_clocks.is_empty() && model.vram_curve_clocks.is_empty() && model.core_curve_voltages.is_empty() && model.vram_curve_voltages.is_empty(),
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let core_clocks = FactoryHashMap::builder().launch_default().detach();
        let core_voltages = FactoryHashMap::builder().launch_default().detach();
        let vram_clocks = FactoryHashMap::builder().launch_default().detach();
        let core_curve_clocks = FactoryHashMap::builder().launch_default().detach();
        let vram_curve_clocks = FactoryHashMap::builder().launch_default().detach();
        let core_curve_voltages = FactoryHashMap::builder().launch_default().detach();
        let vram_curve_voltages = FactoryHashMap::builder().launch_default().detach();

        let model = Self {
            core_clocks,
            core_voltages,
            vram_clocks,
            core_curve_clocks,
            vram_curve_clocks,
            core_curve_voltages,
            vram_curve_voltages,
            vram_clock_ratio: 1.0,
            show_nvidia_options: false,
            show_all_pstates: BoolBinding::new(false),
            enable_gpu_locked_clocks: BoolBinding::new(false),
            enable_vram_locked_clocks: BoolBinding::new(false),
        };

        for factory in [
            &model.core_clocks,
            &model.core_voltages,
            &model.vram_clocks,
            &model.core_curve_clocks,
            &model.vram_curve_clocks,
            &model.core_curve_voltages,
            &model.vram_curve_voltages,
        ] {
            let container = factory.widget();
            container.set_orientation(gtk::Orientation::Vertical);
            container.set_spacing(5);
            container.set_valign(gtk::Align::Start);
        }

        for binding in [
            &model.show_all_pstates,
            &model.enable_gpu_locked_clocks,
            &model.enable_vram_locked_clocks,
        ] {
            let sender = sender.clone();
            binding.connect_value_notify(move |_| {
                sender.input(ClocksFrameMsg::TogglePStatesVisibility)
            });
        }

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
            ClocksFrameMsg::Clocks(clocks_table) => {
                widgets
                    .gpu_locked_clocks_togglebutton
                    .block_signal(&widgets.gpu_locked_clock_signal);
                widgets
                    .vram_locked_clocks_togglebutton
                    .block_signal(&widgets.vram_locked_clock_signal);

                self.core_clocks.clear();
                self.core_voltages.clear();
                self.vram_clocks.clear();
                self.core_curve_clocks.clear();
                self.vram_curve_clocks.clear();
                self.core_curve_voltages.clear();
                self.vram_curve_voltages.clear();
                self.enable_gpu_locked_clocks.set_value(false);
                self.enable_vram_locked_clocks.set_value(false);
                self.show_nvidia_options = false;

                if let Some(table) = clocks_table {
                    match table {
                        ClocksTable::Amd(table) => self.set_amd_table(table),
                        ClocksTable::Nvidia(table) => self.set_nvidia_table(table),
                        ClocksTable::Intel(table) => self.set_intel_table(table),
                    }
                }

                // Make sure the width of all the labels is the same
                let label_size_group = gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal);
                let input_size_group = gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal);

                for factory in [
                    &self.core_clocks,
                    &self.core_voltages,
                    &self.vram_clocks,
                    &self.core_curve_clocks,
                    &self.vram_curve_clocks,
                    &self.core_curve_voltages,
                    &self.vram_curve_voltages,
                ] {
                    for clockspeed_type in factory.keys() {
                        factory.send(
                            clockspeed_type,
                            ClockAdjustmentRowMsg::AddSizeGroup {
                                label_group: label_size_group.clone(),
                                input_group: input_size_group.clone(),
                            },
                        );
                    }
                }

                widgets
                    .gpu_locked_clocks_togglebutton
                    .unblock_signal(&widgets.gpu_locked_clock_signal);
                widgets
                    .vram_locked_clocks_togglebutton
                    .unblock_signal(&widgets.vram_locked_clock_signal);

                sender.input(ClocksFrameMsg::TogglePStatesVisibility);
            }
            ClocksFrameMsg::VramRatio(vram_ratio) => {
                self.vram_clock_ratio = vram_ratio;
            }
            ClocksFrameMsg::TogglePStatesVisibility => {
                let show_secondary = self.show_all_pstates.value();
                for factory in [
                    &self.core_clocks,
                    &self.core_voltages,
                    &self.vram_clocks,
                    &self.core_curve_clocks,
                    &self.vram_curve_clocks,
                    &self.core_curve_voltages,
                    &self.vram_curve_voltages,
                ] {
                    for (key, row) in factory.iter() {
                        // Only show min/max core/vram clock when nvidia locked clocks are enabeld
                        let show_current = match key {
                            ClockspeedType::MaxCoreClock | ClockspeedType::MinCoreClock
                                if self.show_nvidia_options =>
                            {
                                self.enable_gpu_locked_clocks.value()
                            }
                            ClockspeedType::MaxMemoryClock | ClockspeedType::MinMemoryClock
                                if self.show_nvidia_options =>
                            {
                                self.enable_vram_locked_clocks.value()
                            }
                            _ => !row.is_secondary || show_secondary,
                        };

                        factory.send(key, ClockAdjustmentRowMsg::SetVisible(show_current));
                    }
                }
            }
        }
        self.update_vram_clock_ratio();

        self.update_view(widgets, sender);
    }
}

impl ClocksFrame {
    fn set_clock(&mut self, clock_type: ClockspeedType, data: ClocksData) {
        match clock_type {
            ClockspeedType::MaxCoreClock
            | ClockspeedType::MinCoreClock
            | ClockspeedType::GpuClockOffset(_) => {
                self.core_clocks.insert(clock_type, data);
            }
            ClockspeedType::MinVoltage
            | ClockspeedType::MaxVoltage
            | ClockspeedType::VoltageOffset => {
                self.core_voltages.insert(clock_type, data);
            }
            ClockspeedType::MaxMemoryClock
            | ClockspeedType::MinMemoryClock
            | ClockspeedType::MemClockOffset(_) => {
                self.vram_clocks.insert(clock_type, data);
            }
            ClockspeedType::GpuVfCurveClock(_) => {
                self.core_curve_clocks.insert(clock_type, data);
            }
            ClockspeedType::MemVfCurveClock(_) => {
                self.vram_curve_clocks.insert(clock_type, data);
            }
            ClockspeedType::GpuVfCurveVoltage(_) => {
                self.core_curve_voltages.insert(clock_type, data);
            }
            ClockspeedType::MemVfCurveVoltage(_) => {
                self.vram_curve_voltages.insert(clock_type, data);
            }
            ClockspeedType::Reset => {}
        }
    }

    fn update_vram_clock_ratio(&self) {
        for clock_type in [
            ClockspeedType::MaxMemoryClock,
            ClockspeedType::MinMemoryClock,
        ] {
            if self.vram_clocks.get(&clock_type).is_some() {
                self.vram_clocks.send(
                    &clock_type,
                    ClockAdjustmentRowMsg::ValueRatio(self.vram_clock_ratio),
                );
            }
        }
    }

    fn set_amd_table(&mut self, table: AmdClocksTable) {
        match table {
            AmdClocksTable::Gcn(table) => {
                let vddc_range = table.od_range.vddc.and_then(|range| range.into_full());

                if let Some((min_sclk, max_sclk)) = table.od_range.sclk.into_full() {
                    self.add_amd_list(
                        table.sclk_levels.iter().map(|level| level.clockspeed),
                        ClockspeedType::GpuVfCurveClock,
                        min_sclk,
                        max_sclk,
                        true,
                    );
                }

                if let Some((min_mclk, max_mclk)) =
                    table.od_range.mclk.and_then(|range| range.into_full())
                {
                    self.add_amd_list(
                        table.mclk_levels.iter().map(|level| level.clockspeed),
                        ClockspeedType::MemVfCurveClock,
                        min_mclk,
                        max_mclk,
                        false,
                    );
                }

                if let Some((min_vddc, max_vddc)) = vddc_range {
                    self.add_amd_list(
                        table.sclk_levels.iter().map(|level| level.voltage),
                        ClockspeedType::GpuVfCurveVoltage,
                        min_vddc,
                        max_vddc,
                        false,
                    );

                    self.add_amd_list(
                        table.mclk_levels.iter().map(|level| level.voltage),
                        ClockspeedType::MemVfCurveVoltage,
                        min_vddc,
                        max_vddc,
                        false,
                    );
                }
            }
            AmdClocksTable::Rdna(table) => {
                // RDNA4 clock offset
                if let Some((sclk_offset_min, sclk_offset_max)) = table
                    .od_range
                    .sclk_offset
                    .and_then(|range| range.into_full())
                    && let Some(sclk_offset) = table.sclk_offset
                {
                    self.set_clock(
                        ClockspeedType::GpuClockOffset(0),
                        ClocksData {
                            current: sclk_offset,
                            min: sclk_offset_min,
                            max: sclk_offset_max,
                            custom_title: Some(fl!(I18N, "gpu-clock-offset")),
                            is_secondary: false,
                        },
                    );
                }

                let mut clocks_types = Vec::with_capacity(4);

                if table.vddc_curve.is_empty() {
                    // RDNA2/3 min/max clock values
                    clocks_types.extend([
                        (
                            ClockspeedType::MaxCoreClock,
                            table.current_sclk_range.max,
                            table.od_range.sclk,
                        ),
                        (
                            ClockspeedType::MinCoreClock,
                            table.current_sclk_range.min,
                            table.od_range.sclk,
                        ),
                    ]);
                } else {
                    // RDNA1 VF curve
                    for (i, level) in table.vddc_curve.iter().enumerate().rev() {
                        if let Some((min_sclk, max_sclk)) = table
                            .od_range
                            .curve_sclk_points
                            .get(i)
                            .or(table.od_range.sclk.as_ref())
                            .and_then(|range| range.into_full())
                        {
                            self.set_clock(
                                ClockspeedType::GpuVfCurveClock(i as u8),
                                ClocksData {
                                    current: level.clockspeed,
                                    min: min_sclk,
                                    max: max_sclk,
                                    is_secondary: false,
                                    custom_title: None,
                                },
                            );
                        }
                    }

                    for (i, level) in table.vddc_curve.iter().enumerate().rev() {
                        if let Some((min_vddc, max_vddc)) = table
                            .od_range
                            .curve_voltage_points
                            .get(i)
                            .and_then(|range| range.into_full())
                        {
                            self.set_clock(
                                ClockspeedType::GpuVfCurveVoltage(i as u8),
                                ClocksData {
                                    current: level.voltage,
                                    min: min_vddc,
                                    max: max_vddc,
                                    is_secondary: false,
                                    custom_title: None,
                                },
                            );
                        }
                    }
                }

                clocks_types.extend([
                    (
                        ClockspeedType::MaxMemoryClock,
                        table.current_mclk_range.max,
                        table.od_range.mclk,
                    ),
                    (
                        ClockspeedType::MinMemoryClock,
                        table.current_mclk_range.min,
                        table.od_range.mclk,
                    ),
                ]);

                for (clockspeed_type, current_value, range) in clocks_types {
                    if let Some(current) = current_value
                        && let Some((min, max)) = range.and_then(|range| range.into_full())
                    {
                        self.set_clock(clockspeed_type, ClocksData::new(current, min, max));
                    }
                }

                if let Some(current) = table.voltage_offset {
                    let (min, max) = table
                        .od_range
                        .voltage_offset
                        .and_then(|range| range.into_full())
                        .unwrap_or((-DEFAULT_VOLTAGE_OFFSET_RANGE, DEFAULT_VOLTAGE_OFFSET_RANGE));

                    self.set_clock(
                        ClockspeedType::VoltageOffset,
                        ClocksData::new(current, min, max),
                    );
                }
            }
        }
    }

    fn add_amd_list(
        &mut self,
        values: impl ExactSizeIterator<Item = i32> + DoubleEndedIterator,
        clock_type: fn(u8) -> ClockspeedType,
        min: i32,
        max: i32,
        _disable_separator: bool,
    ) {
        let values_len = values.len();
        for (i, value) in values.enumerate().rev() {
            let is_secondary = i > 0 && i < values_len - 1;

            self.set_clock(
                clock_type(i as u8),
                ClocksData {
                    current: value,
                    min,
                    max,
                    is_secondary,
                    custom_title: None,
                },
            );
        }
    }

    fn set_nvidia_table(&mut self, table: NvidiaClocksTable) {
        self.show_nvidia_options = true;

        let locked_clocks = [
            (
                table.gpu_clock_range,
                table.gpu_locked_clocks,
                ClockspeedType::MinCoreClock,
                ClockspeedType::MaxCoreClock,
                &self.enable_gpu_locked_clocks.clone(),
            ),
            (
                table.vram_clock_range,
                table.vram_locked_clocks,
                ClockspeedType::MinMemoryClock,
                ClockspeedType::MaxMemoryClock,
                &self.enable_vram_locked_clocks.clone(),
            ),
        ];

        for (clock_range, locked_clocks, min_type, max_type, enable_binding) in locked_clocks {
            if let Some((gpu_min, gpu_max)) = clock_range {
                let (current_min, current_max) = match locked_clocks {
                    Some(locked_range) => {
                        enable_binding.set_value(true);
                        locked_range
                    }
                    None => (gpu_min, gpu_max),
                };

                self.set_clock(
                    min_type,
                    ClocksData::new(current_min as i32, gpu_min as i32, gpu_max as i32),
                );
                self.set_clock(
                    max_type,
                    ClocksData::new(current_max as i32, gpu_min as i32, gpu_max as i32),
                );
            }
        }

        for (pstate, offset) in table.gpu_offsets {
            self.set_clock(
                ClockspeedType::GpuClockOffset(pstate),
                nvidia_clock_offset_to_data(&offset, pstate > 0),
            );
        }
        for (pstate, offset) in table.mem_offsets {
            self.set_clock(
                ClockspeedType::MemClockOffset(pstate),
                nvidia_clock_offset_to_data(&offset, pstate > 0),
            );
        }
    }

    fn set_intel_table(&mut self, table: IntelClocksTable) {
        self.show_all_pstates.set_value(false);

        if let Some((current_gt_min, current_gt_max)) = table.gt_freq
            && let (Some(min_clock), Some(max_clock)) = (table.rpn_freq, table.rp0_freq)
        {
            self.set_clock(
                ClockspeedType::MaxCoreClock,
                ClocksData::new(current_gt_max as i32, min_clock as i32, max_clock as i32),
            );
            self.set_clock(
                ClockspeedType::MinCoreClock,
                ClocksData::new(current_gt_min as i32, min_clock as i32, max_clock as i32),
            );
        }
    }

    pub fn get_commands(&self) -> Vec<SetClocksCommand> {
        self.core_clocks
            .iter()
            .chain(self.core_voltages.iter())
            .chain(self.vram_clocks.iter())
            .chain(self.core_curve_clocks.iter())
            .chain(self.vram_curve_clocks.iter())
            .chain(self.core_curve_voltages.iter())
            .chain(self.vram_curve_voltages.iter())
            .filter_map(|(clock_type, row)| {
                // If nvidia options are enabled, we always set locked clocks to None or Some
                let value = if self.show_nvidia_options {
                    match clock_type {
                        ClockspeedType::MinCoreClock | ClockspeedType::MaxCoreClock => self
                            .enable_gpu_locked_clocks
                            .value()
                            .then(|| row.get_raw_value()),
                        ClockspeedType::MinMemoryClock | ClockspeedType::MaxMemoryClock => self
                            .enable_vram_locked_clocks
                            .value()
                            .then(|| row.get_raw_value()),
                        _ => Some(row.get_configured_value()?),
                    }
                } else {
                    Some(row.get_configured_value()?)
                };

                Some(SetClocksCommand {
                    r#type: *clock_type,
                    value,
                })
            })
            .collect()
    }
}

fn nvidia_clock_offset_to_data(clock_info: &NvidiaClockOffset, is_secondary: bool) -> ClocksData {
    ClocksData {
        current: clock_info.current,
        min: clock_info.min,
        max: clock_info.max,
        custom_title: None,
        is_secondary,
    }
}

mod adjustment_group;
mod adjustment_row;

use crate::{
    APP_BROKER, I18N,
    app::{
        msg::AppMsg, page_section::PageSection,
        pages::oc_page::clocks_frame::adjustment_group::AdjustmentGroup,
    },
};
use adjustment_group::ClockCategory;
use adjustment_row::ClocksData;
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

const DEFAULT_VOLTAGE_OFFSET_RANGE: i32 = 250;

pub struct ClocksFrame {
    core_groups: FactoryHashMap<ClockCategory, AdjustmentGroup>,
    vram_groups: FactoryHashMap<ClockCategory, AdjustmentGroup>,
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
                    set_visible: model.has_any_clocks(),

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
                        set_visible: model.any_is_secondary(),

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

                    set_margin_horizontal: 5,
                    set_markup: &fl!(I18N, "pstate-list-description"),
                    set_wrap_mode: pango::WrapMode::Word,
                    set_halign: gtk::Align::Start,
                },
            },

            append_child = &gtk::FlowBox {
                set_orientation: gtk::Orientation::Horizontal,
                set_selection_mode: gtk::SelectionMode::None,
                #[watch]
                set_max_children_per_line: if model.core_any_visible() && model.vram_any_visible() { 2 } else { 1 },
                set_column_spacing: 10,
                set_row_spacing: 10,
                set_homogeneous: false,
                set_valign: gtk::Align::Start,
                set_hexpand: true,

                append = &gtk::FlowBoxChild {
                    add_css_class: "clocks-frame-group",
                    set_valign: gtk::Align::Start,
                    #[watch]
                    set_visible: model.core_any_visible(),

                    #[local_ref]
                    core_groups_widget -> gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_valign: gtk::Align::Start,
                        set_spacing: 10,
                        set_hexpand: true,
                    },
                },

                append = &gtk::FlowBoxChild {
                    add_css_class: "clocks-frame-group",
                    set_valign: gtk::Align::Start,
                    #[watch]
                    set_visible: model.vram_any_visible(),

                    #[local_ref]
                    vram_groups_widget -> gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_valign: gtk::Align::Start,
                        set_spacing: 10,
                        set_hexpand: true,
                    },
                },
            },

            append_child = &gtk::Label {
                set_label: &fl!(I18N, "no-clocks-data"),
                set_margin_horizontal: 10,
                set_halign: gtk::Align::Start,
                #[watch]
                set_visible: !model.has_any_clocks(),
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            core_groups: FactoryHashMap::builder().launch_default().detach(),
            vram_groups: FactoryHashMap::builder().launch_default().detach(),
            vram_clock_ratio: 1.0,
            show_nvidia_options: false,
            show_all_pstates: BoolBinding::new(false),
            enable_gpu_locked_clocks: BoolBinding::new(false),
            enable_vram_locked_clocks: BoolBinding::new(false),
        };

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

        let core_groups_widget = model.core_groups.widget();
        let vram_groups_widget = model.vram_groups.widget();

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

                self.core_groups.clear();
                self.vram_groups.clear();

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

                let label_size_group = gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal);
                let input_size_group = gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal);

                for group in self.all_groups() {
                    group.add_size_group(label_size_group.clone(), input_size_group.clone());
                }

                widgets
                    .gpu_locked_clocks_togglebutton
                    .unblock_signal(&widgets.gpu_locked_clock_signal);
                widgets
                    .vram_locked_clocks_togglebutton
                    .unblock_signal(&widgets.vram_locked_clock_signal);

                self.update_vram_clock_ratio();
                sender.input(ClocksFrameMsg::TogglePStatesVisibility);
            }
            ClocksFrameMsg::VramRatio(vram_ratio) => {
                self.vram_clock_ratio = vram_ratio;
                self.update_vram_clock_ratio();
            }
            ClocksFrameMsg::TogglePStatesVisibility => {
                for group in self.all_groups() {
                    group.toggle_secondary_visibility(
                        self.show_all_pstates.value(),
                        self.show_nvidia_options,
                        self.enable_gpu_locked_clocks.value(),
                        self.enable_vram_locked_clocks.value(),
                    );
                }
            }
        }

        self.update_view(widgets, sender);
    }
}

impl ClocksFrame {
    fn set_clock(&mut self, clock_type: ClockspeedType, data: ClocksData) {
        let category = ClockCategory::from_type(clock_type);

        let groups = if category.is_core() {
            &mut self.core_groups
        } else if category.is_vram() {
            &mut self.vram_groups
        } else {
            unreachable!()
        };

        let mut group = if let Some(group) = groups.get_mut(&category) {
            group
        } else {
            groups.insert(category, ());
            groups.get_mut(&category).unwrap()
        };

        group.set_clock(clock_type, data);
    }

    fn all_groups(&self) -> impl Iterator<Item = &AdjustmentGroup> {
        self.core_groups.values().chain(self.vram_groups.values())
    }

    fn has_any_clocks(&self) -> bool {
        self.core_groups.values().any(|group| !group.is_empty())
    }

    fn any_is_secondary(&self) -> bool {
        self.all_groups().any(|group| group.has_secondary())
    }

    fn core_any_visible(&self) -> bool {
        self.core_groups.values().any(|group| !group.is_empty())
    }

    fn vram_any_visible(&self) -> bool {
        self.vram_groups.values().any(|group| !group.is_empty())
    }

    fn update_vram_clock_ratio(&self) {
        if let Some(vram_group) = self.vram_groups.get(&ClockCategory::VramClock) {
            vram_group.set_value_ratio(self.vram_clock_ratio);
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
                    );
                }

                if let Some((min_vddc, max_vddc)) = vddc_range {
                    self.add_amd_list(
                        table.sclk_levels.iter().map(|level| level.voltage),
                        ClockspeedType::GpuVfCurveVoltage,
                        min_vddc,
                        max_vddc,
                    );

                    self.add_amd_list(
                        table.mclk_levels.iter().map(|level| level.voltage),
                        ClockspeedType::MemVfCurveVoltage,
                        min_vddc,
                        max_vddc,
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
                self.enable_gpu_locked_clocks.clone(),
            ),
            (
                table.vram_clock_range,
                table.vram_locked_clocks,
                ClockspeedType::MinMemoryClock,
                ClockspeedType::MaxMemoryClock,
                self.enable_vram_locked_clocks.clone(),
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
        self.all_groups()
            .flat_map(|group| group.get_commands())
            .filter(|(clock_type, configured_value)| {
                if configured_value.is_some() {
                    true
                } else {
                    // Only allow None for Nvidia locked clocks
                    self.show_nvidia_options
                        && matches!(
                            clock_type,
                            ClockspeedType::MinCoreClock
                                | ClockspeedType::MaxCoreClock
                                | ClockspeedType::MinMemoryClock
                                | ClockspeedType::MaxMemoryClock
                        )
                }
            })
            .map(|(clock_type, configured_value)| {
                let value = if self.show_nvidia_options {
                    match clock_type {
                        ClockspeedType::MinCoreClock | ClockspeedType::MaxCoreClock => self
                            .enable_gpu_locked_clocks
                            .value()
                            .then(|| {
                                self.core_groups
                                    .get(&ClockCategory::from_type(clock_type))
                                    .map(|group| group.get_raw_value(clock_type))
                            })
                            .flatten(),
                        ClockspeedType::MinMemoryClock | ClockspeedType::MaxMemoryClock => self
                            .enable_vram_locked_clocks
                            .value()
                            .then(|| {
                                self.vram_groups
                                    .get(&ClockCategory::from_type(clock_type))
                                    .map(|group| group.get_raw_value(clock_type))
                            })
                            .flatten(),
                        _ => configured_value,
                    }
                } else {
                    configured_value
                };

                SetClocksCommand {
                    r#type: clock_type,
                    value,
                }
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

use crate::{
    APP_BROKER, I18N,
    app::{
        components::{
            adjustment_card::AdjustmentCard,
            adjustment_row::{AdjustmentRow, AdjustmentRowInit, AdjustmentRowMsg},
            page_section::PageSection,
        },
        msg::AppMsg,
        pages::oc_page::OcPageMsg,
    },
};
use adw::prelude::*;
use amdgpu_sysfs::gpu_handle::overdrive::ClocksTableGen as AmdClocksTable;
use i18n_embed_fl::fl;
use lact_schema::{
    ClocksTable, IntelClocksTable, NvidiaClockOffset, NvidiaClocksTable,
    request::{ClockspeedType, SetClocksCommand},
};
use relm4::{
    ComponentParts, ComponentSender, RelmObjectExt, RelmWidgetExt, binding::BoolBinding, css,
    factory::FactoryHashMap,
};
use std::{collections::HashSet, sync::Arc};

const DEFAULT_VOLTAGE_OFFSET_RANGE: i32 = 250;

/// Identifies a row in the frame.
///
/// Most rows map one to one onto a clockspeed the daemon can set. The MSVDD
/// master is the exception: it only exists in the GUI, where it drives the
/// per-domain offset rows, so it has no clockspeed type of its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RowId {
    Clock(ClockspeedType),
    MsvddMaster,
}

pub struct ClocksFrame {
    adjustments: FactoryHashMap<RowId, AdjustmentRow<RowId>>,
    secondary_p_state_clocks: HashSet<RowId>,
    domain: ClockDomain,
    vram_clock_ratio: f64,
    show_nvidia_options: bool,
    vf_curve_available: bool,
    show_all_pstates: BoolBinding,
    vf_curve_editing: BoolBinding,
    enable_locked_clocks: BoolBinding,
}

#[derive(Default)]
struct ClocksData {
    current: i32,
    min: i32,
    max: i32,
    custom_title: Option<String>,
    is_secondary_p_state: bool,
}

impl ClocksData {
    fn new(current: i32, min: i32, max: i32) -> Self {
        Self {
            current,
            min,
            max,
            ..Default::default()
        }
    }
}

pub struct ClocksFrameInit {
    pub domain: ClockDomain,
    pub vf_curve_editing: BoolBinding,
    pub show_all_pstates: BoolBinding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockDomain {
    Gpu,
    Vram,
    /// Clock domains the driver does not expose through its normal interface.
    Advanced,
}

impl ClockDomain {
    fn matches_row(self, id: RowId) -> bool {
        match id {
            RowId::Clock(clock_type) => self.matches(clock_type),
            RowId::MsvddMaster => self == Self::Advanced,
        }
    }

    fn matches(self, clock_type: ClockspeedType) -> bool {
        let domain = match clock_type {
            ClockspeedType::MaxCoreClock
            | ClockspeedType::MinCoreClock
            | ClockspeedType::GpuClockOffset(_)
            | ClockspeedType::MinVoltage
            | ClockspeedType::MaxVoltage
            | ClockspeedType::VoltageOffset
            | ClockspeedType::VoltageBoost
            | ClockspeedType::GpuVfCurveClock(_)
            | ClockspeedType::GpuVfCurveVoltage(_) => Self::Gpu,
            ClockspeedType::MaxMemoryClock
            | ClockspeedType::MinMemoryClock
            | ClockspeedType::MemClockOffset(_)
            | ClockspeedType::MemVfCurveClock(_)
            | ClockspeedType::MemVfCurveVoltage(_) => Self::Vram,
            ClockspeedType::ClockDomainOffset(_) | ClockspeedType::ClockDomainVoltageOffset(_) => {
                Self::Advanced
            }
            ClockspeedType::Reset => unreachable!(),
        };
        self == domain
    }
}

#[derive(Debug)]
pub enum ClocksFrameMsg {
    Clocks {
        table: Option<Arc<ClocksTable>>,
        vf_curve_is_configured: bool,
    },
    VramRatio(f64),
    TogglePStatesVisibility,
    ResetGpuClockOffsets,
    /// The user moved the MSVDD master row, which drives the per-domain rows.
    MsvddOffset,
}

#[relm4::component(pub)]
impl relm4::Component for ClocksFrame {
    type Init = ClocksFrameInit;
    type Input = ClocksFrameMsg;
    type Output = OcPageMsg;
    type CommandOutput = ();

    view! {
        PageSection::new("") {
            #[watch]
            set_name: match model.domain {
                ClockDomain::Gpu => fl!(I18N, "core-section"),
                ClockDomain::Vram => fl!(I18N, "vram-section"),
                ClockDomain::Advanced => fl!(I18N, "advanced-section"),
            },
            #[watch]
            set_visible: model.domain == ClockDomain::Gpu || model.has_any_clocks(),

            append_header = &gtk::Box {
                set_spacing: 10,
                set_hexpand: true,
                set_halign: gtk::Align::End,

                append = &gtk::MenuButton {
                    set_icon_name: "dialog-information-symbolic",
                    set_always_show_arrow: false,
                    add_css_class: "flat",
                    set_valign: gtk::Align::Center,
                    set_visible: model.domain == ClockDomain::Advanced,

                    #[wrap(Some)]
                    set_popover = &gtk::Popover {
                        gtk::Label {
                            set_margin_all: 5,
                            set_label: &fl!(I18N, "advanced-section-description"),
                            set_wrap: true,
                            set_max_width_chars: 55,
                        }
                    },
                },

                append = &gtk::Button {
                    set_label: &fl!(I18N, "vf-curve-editor"),

                    #[watch]
                    set_visible: model.domain == ClockDomain::Gpu
                        && model.show_nvidia_options
                        && model.vf_curve_available,

                    connect_clicked[sender] => move |_| {
                        sender.output(OcPageMsg::ShowVfCurveEditor).unwrap();
                    }
                },

                append = &gtk::Button {
                    set_label: &fl!(I18N, "reset-now-button"),
                    set_tooltip_text: Some(&fl!(I18N, "reset-oc-tooltip")),

                    add_css_class: css::DESTRUCTIVE_ACTION,

                    #[watch]
                    set_visible: model.domain == ClockDomain::Gpu && model.has_any_clocks(),

                    connect_clicked => move |_| {
                        APP_BROKER.send(AppMsg::ResetClocks);
                    }
                },
            },

            #[template]
            append_child = &AdjustmentCard {
                #[template_child]
                advanced_features {
                    #[watch]
                    set_visible: model.has_secondary_p_states() || model.show_nvidia_options,
                },

                #[template_child]
                controls {
                    append = &gtk::ToggleButton {
                        #[watch]
                        set_visible: model.has_secondary_p_states(),

                        add_css_class: "adjustment-card-option-toggle",
                        add_binding["active"]: &model.show_all_pstates,

                        #[watch]
                        set_sensitive: {
                            if model.domain == ClockDomain::Gpu
                                && model.show_nvidia_options
                                && model.vf_curve_available {
                                !model.vf_curve_editing.value()
                            } else {
                                true
                            }
                        },

                        #[wrap(Some)]
                        set_child = &gtk::Box {
                            append = &gtk::Label {
                                set_label: &fl!(I18N, "show-all-pstates"),
                            },
                        },
                    },

                    append: locked_clocks_togglebutton = &gtk::ToggleButton {
                        #[watch]
                        set_visible: model.show_nvidia_options,
                        add_css_class: "adjustment-card-option-toggle",
                        add_binding["active"]: &model.enable_locked_clocks,

                        #[wrap(Some)]
                        set_child = &gtk::Box {
                            append = &gtk::Label {
                                set_label: &fl!(I18N, "enable-locked-clocks"),
                            },
                        },
                        connect_toggled => move |_| {
                            APP_BROKER.send(AppMsg::SettingsChanged);
                        } @ locked_clock_signal,
                    },

                    append: vf_curve_editing_togglebutton = &gtk::ToggleButton {
                        #[watch]
                        set_visible: model.domain == ClockDomain::Gpu
                            && model.show_nvidia_options
                            && model.vf_curve_available,
                        add_css_class: "adjustment-card-option-toggle",
                        add_css_class: css::WARNING,
                        add_binding["active"]: &model.vf_curve_editing,

                        #[wrap(Some)]
                        set_child = &gtk::Box {
                            append = &gtk::Label {
                                set_label: &fl!(I18N, "enable-vf-curve"),
                            },
                        },

                        connect_toggled[sender] => move |button| {
                            sender.output(OcPageMsg::VfCurveEditingToggled(button.is_active())).unwrap();
                            APP_BROKER.send(AppMsg::SettingsChanged);
                        } @ vf_curve_editing_signal,
                    },
                },

                #[template_child]
                content {
                    #[local_ref]
                    adjustments_widget -> gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 5,
                    },

                    gtk::Label {
                        set_label: &fl!(I18N, "no-clocks-data"),
                        set_margin_horizontal: 10,
                        set_halign: gtk::Align::Start,
                        #[watch]
                        set_visible: !model.has_any_clocks(),
                    },
                },
            },
        }
    }

    fn init(
        ClocksFrameInit {
            domain,
            vf_curve_editing,
            show_all_pstates,
        }: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            adjustments: FactoryHashMap::builder()
                .launch_default()
                .forward(APP_BROKER.sender(), |()| AppMsg::SettingsChanged),
            secondary_p_state_clocks: HashSet::new(),
            domain,
            vram_clock_ratio: 1.0,
            show_nvidia_options: false,
            vf_curve_available: false,
            show_all_pstates,
            vf_curve_editing,
            enable_locked_clocks: BoolBinding::new(false),
        };

        for binding in [
            &model.show_all_pstates,
            &model.vf_curve_editing,
            &model.enable_locked_clocks,
        ] {
            let sender = sender.clone();
            binding.connect_value_notify(move |_| {
                sender.input(ClocksFrameMsg::TogglePStatesVisibility)
            });
        }

        let adjustments_widget = model.adjustments.widget();
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
            ClocksFrameMsg::Clocks {
                table: clocks_table,
                vf_curve_is_configured,
            } => {
                widgets
                    .vf_curve_editing_togglebutton
                    .block_signal(&widgets.vf_curve_editing_signal);
                widgets
                    .locked_clocks_togglebutton
                    .block_signal(&widgets.locked_clock_signal);

                self.adjustments.clear();
                self.secondary_p_state_clocks.clear();

                self.enable_locked_clocks.set_value(false);
                self.vf_curve_editing.set_value(vf_curve_is_configured);
                self.show_nvidia_options = false;

                if let Some(table) = clocks_table {
                    match table.as_ref() {
                        ClocksTable::Amd(table) => self.set_amd_table(table),
                        ClocksTable::Nvidia(table) => self.set_nvidia_table(table),
                        ClocksTable::Intel(table) => self.set_intel_table(table),
                    }
                }

                self.connect_msvdd_master(&sender);

                let label_size_group = gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal);
                let input_size_group = gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal);

                for clock_type in self.adjustments.keys() {
                    self.adjustments.send(
                        clock_type,
                        AdjustmentRowMsg::AddSizeGroup {
                            label_group: label_size_group.clone(),
                            input_group: input_size_group.clone(),
                        },
                    );
                }

                widgets
                    .vf_curve_editing_togglebutton
                    .unblock_signal(&widgets.vf_curve_editing_signal);
                widgets
                    .locked_clocks_togglebutton
                    .unblock_signal(&widgets.locked_clock_signal);

                self.update_vram_clock_ratio();
                sender.input(ClocksFrameMsg::TogglePStatesVisibility);
            }
            ClocksFrameMsg::ResetGpuClockOffsets => {
                for id in self.adjustments.keys() {
                    if matches!(id, RowId::Clock(ClockspeedType::GpuClockOffset(_))) {
                        self.adjustments.send(id, AdjustmentRowMsg::SetValue(0.0));
                    }
                }
            }
            ClocksFrameMsg::MsvddOffset => {
                self.set_domain_voltage_offsets();
            }
            ClocksFrameMsg::VramRatio(vram_ratio) => {
                self.vram_clock_ratio = vram_ratio;
                self.update_vram_clock_ratio();
            }
            ClocksFrameMsg::TogglePStatesVisibility => {
                for id in self.adjustments.keys() {
                    let visible = match id {
                        RowId::Clock(
                            ClockspeedType::MaxCoreClock
                            | ClockspeedType::MinCoreClock
                            | ClockspeedType::MaxMemoryClock
                            | ClockspeedType::MinMemoryClock,
                        ) if self.show_nvidia_options => self.enable_locked_clocks.value(),
                        RowId::Clock(ClockspeedType::GpuClockOffset(_))
                            if self.show_nvidia_options && self.vf_curve_editing.value() =>
                        {
                            false
                        }
                        _ => {
                            !self.secondary_p_state_clocks.contains(id)
                                || self.show_all_pstates.value()
                        }
                    };
                    self.adjustments
                        .send(id, AdjustmentRowMsg::SetVisible(visible));
                }
            }
        }

        self.update_view(widgets, sender);
    }
}

impl ClocksFrame {
    fn set_clock(&mut self, clock_type: ClockspeedType, data: ClocksData) {
        self.set_row(RowId::Clock(clock_type), data);
    }

    fn set_row(&mut self, id: RowId, data: ClocksData) {
        if !self.domain.matches_row(id) {
            return;
        }

        self.adjustments.insert(
            id,
            AdjustmentRowInit {
                title: data.custom_title.unwrap_or_else(|| row_title(id)),
                info_text: row_info_text(id),
                value: f64::from(data.current),
                lower: f64::from(data.min),
                upper: f64::from(data.max),
                step_increment: get_row_step(id),
                ..Default::default()
            },
        );
        if data.is_secondary_p_state {
            self.secondary_p_state_clocks.insert(id);
        } else {
            self.secondary_p_state_clocks.remove(&id);
        }
    }

    /// Pushes one offset into every per-domain MSVDD row.
    ///
    /// MSVDD is a single rail shared by all of these domains, so the master row
    /// sets them together; editing one afterwards overrides it for that domain.
    fn set_domain_voltage_offsets(&self) {
        let offset = self
            .adjustments
            .get(&RowId::MsvddMaster)
            .map_or(0.0, AdjustmentRow::get_value);

        for id in self.adjustments.keys() {
            if matches!(id, RowId::Clock(ClockspeedType::ClockDomainVoltageOffset(_))) {
                self.adjustments
                    .send(id, AdjustmentRowMsg::SetValue(offset));
            }
        }
    }

    /// Forwards user changes of the MSVDD master row to this component.
    ///
    /// The rows are rebuilt whenever the clocks table changes, so this runs again
    /// for each new row. The handler is attached after the row was initialized
    /// with its current value, so loading a table does not fire it.
    fn connect_msvdd_master(&self, sender: &ComponentSender<Self>) {
        let Some(adjustment) = self
            .adjustments
            .get(&RowId::MsvddMaster)
            .map(AdjustmentRow::adjustment)
        else {
            return;
        };

        let sender = sender.clone();
        adjustment.connect_value_changed(move |_| {
            sender.input(ClocksFrameMsg::MsvddOffset);
        });
    }

    fn has_any_clocks(&self) -> bool {
        !self.adjustments.is_empty()
    }

    fn has_secondary_p_states(&self) -> bool {
        !self.secondary_p_state_clocks.is_empty()
    }

    fn update_vram_clock_ratio(&self) {
        for id in self.adjustments.keys() {
            if matches!(
                id,
                RowId::Clock(
                    ClockspeedType::MaxMemoryClock
                        | ClockspeedType::MinMemoryClock
                        | ClockspeedType::MemClockOffset(_)
                )
            ) {
                self.adjustments
                    .send(id, AdjustmentRowMsg::ValueRatio(self.vram_clock_ratio));
            }
        }
    }

    fn set_amd_table(&mut self, table: &AmdClocksTable) {
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
                            ..Default::default()
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
                                    ..Default::default()
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
                                    ..Default::default()
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
            let is_secondary_p_state = i > 0 && i < values_len - 1;

            self.set_clock(
                clock_type(i as u8),
                ClocksData {
                    current: value,
                    min,
                    max,
                    is_secondary_p_state,
                    ..Default::default()
                },
            );
        }
    }

    fn set_nvidia_table(&mut self, table: &NvidiaClocksTable) {
        self.show_nvidia_options = true;
        self.vf_curve_available = !table.gpu_vf_curve.is_empty();

        let locked = match self.domain {
            ClockDomain::Gpu => Some((
                table.gpu_clock_range,
                table.gpu_locked_clocks,
                ClockspeedType::MinCoreClock,
                ClockspeedType::MaxCoreClock,
            )),
            ClockDomain::Vram => Some((
                table.vram_clock_range,
                table.vram_locked_clocks,
                ClockspeedType::MinMemoryClock,
                ClockspeedType::MaxMemoryClock,
            )),
            // None of these is a core or VRAM clock, so there is no range to lock
            ClockDomain::Advanced => None,
        };

        if let Some((clock_range, locked_clocks, min_type, max_type)) = locked
            && let Some((gpu_min, gpu_max)) = clock_range
        {
            let (current_min, current_max) = match locked_clocks {
                Some(locked_range) => {
                    self.enable_locked_clocks.set_value(true);
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

        for (pstate, offset) in &table.gpu_offsets {
            self.set_clock(
                ClockspeedType::GpuClockOffset(*pstate),
                nvidia_clock_offset_to_data(offset, *pstate > 0),
            );
        }
        for (pstate, offset) in &table.mem_offsets {
            self.set_clock(
                ClockspeedType::MemClockOffset(*pstate),
                nvidia_clock_offset_to_data(offset, *pstate > 0),
            );
        }

        // Every one of these domains is fed by the same MSVDD rail, so offsetting
        // all of them together is the common case. The master row covers that,
        // and the per-domain rows below it stay editable as overrides.
        let voltages: Vec<_> = table
            .clock_domain_offsets
            .iter()
            .filter_map(|domain_offset| domain_offset.voltage.as_ref())
            .collect();

        if voltages.len() > 1 {
            let highest = voltages.iter().map(|voltage| voltage.current).max().unwrap();
            // The domains carry independent offsets, so there is no single value to
            // show unless they happen to agree. When they do not, the master says so
            // rather than inventing one, and sits at the largest of them so that
            // moving it starts from the offset currently doing the most.
            let agreed = voltages
                .iter()
                .all(|voltage| voltage.current == highest)
                .then_some(highest);

            self.set_row(
                RowId::MsvddMaster,
                ClocksData {
                    current: agreed.unwrap_or(highest),
                    // Only values every domain accepts, as the master sets them all.
                    min: voltages.iter().map(|voltage| voltage.min).max().unwrap(),
                    max: voltages.iter().map(|voltage| voltage.max).min().unwrap(),
                    custom_title: agreed.is_none().then(|| fl!(I18N, "msvdd-offset-mixed")),
                    ..Default::default()
                },
            );
        }

        for domain_offset in &table.clock_domain_offsets {
            self.set_clock(
                ClockspeedType::ClockDomainOffset(domain_offset.domain),
                ClocksData {
                    current: domain_offset.freq.current,
                    min: domain_offset.freq.min,
                    max: domain_offset.freq.max,
                    custom_title: Some(fl!(
                        I18N,
                        "clock-domain-offset",
                        domain = domain_offset.name.clone()
                    )),
                    ..Default::default()
                },
            );

            if let Some(voltage) = &domain_offset.voltage {
                self.set_clock(
                    ClockspeedType::ClockDomainVoltageOffset(domain_offset.domain),
                    ClocksData {
                        current: voltage.current,
                        min: voltage.min,
                        max: voltage.max,
                        custom_title: Some(fl!(
                            I18N,
                            "clock-domain-voltage-offset",
                            domain = domain_offset.name.clone()
                        )),
                        ..Default::default()
                    },
                );
            }
        }

        if let Some(voltage_boost) = table.voltage_boost {
            self.set_clock(
                ClockspeedType::VoltageBoost,
                ClocksData {
                    current: voltage_boost.current,
                    min: voltage_boost.min,
                    max: voltage_boost.max,
                    ..Default::default()
                },
            );
        }
    }

    fn set_intel_table(&mut self, table: &IntelClocksTable) {
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
        self.adjustments
            .iter()
            .filter_map(|(id, row)| {
                // Rows that only exist in the GUI are skipped; they act through
                // the rows they drive rather than being applied themselves.
                let RowId::Clock(clock_type) = id else {
                    return None;
                };

                let configured_value = row.get_changed_value().map(|value| value as i32);
                // If nvidia options are enabled, we always set locked clocks to None or Some
                let value = if self.show_nvidia_options {
                    match clock_type {
                        ClockspeedType::MinCoreClock
                        | ClockspeedType::MaxCoreClock
                        | ClockspeedType::MinMemoryClock
                        | ClockspeedType::MaxMemoryClock => self
                            .enable_locked_clocks
                            .value()
                            .then(|| row.get_value() as i32),
                        _ => Some(configured_value?),
                    }
                } else {
                    Some(configured_value?)
                };

                Some(SetClocksCommand {
                    r#type: *clock_type,
                    value,
                })
            })
            .collect()
    }
}

fn nvidia_clock_offset_to_data(
    clock_info: &NvidiaClockOffset,
    is_secondary_p_state: bool,
) -> ClocksData {
    ClocksData {
        current: clock_info.current,
        min: clock_info.min,
        max: clock_info.max,
        is_secondary_p_state,
        ..Default::default()
    }
}

fn row_title(id: RowId) -> String {
    match id {
        RowId::Clock(clock_type) => clock_title(clock_type),
        RowId::MsvddMaster => fl!(I18N, "msvdd-offset"),
    }
}

fn row_info_text(id: RowId) -> String {
    match id {
        RowId::Clock(ClockspeedType::VoltageBoost) => fl!(I18N, "gpu-voltage-boost-tooltip"),
        RowId::MsvddMaster => fl!(I18N, "msvdd-offset-tooltip"),
        _ => String::new(),
    }
}

fn clock_title(clock_type: ClockspeedType) -> String {
    match clock_type {
        ClockspeedType::MaxCoreClock | ClockspeedType::MaxMemoryClock => fl!(I18N, "max-clock"),
        ClockspeedType::MaxVoltage => fl!(I18N, "max-gpu-voltage"),
        ClockspeedType::MinCoreClock | ClockspeedType::MinMemoryClock => fl!(I18N, "min-clock"),
        ClockspeedType::MinVoltage => fl!(I18N, "min-gpu-voltage"),
        ClockspeedType::VoltageOffset => fl!(I18N, "gpu-voltage-offset"),
        ClockspeedType::VoltageBoost => fl!(I18N, "gpu-voltage-boost"),
        ClockspeedType::GpuClockOffset(pstate) | ClockspeedType::MemClockOffset(pstate) => {
            fl!(I18N, "pstate-clock-offset", pstate = pstate)
        }
        ClockspeedType::GpuVfCurveClock(pstate) | ClockspeedType::MemVfCurveClock(pstate) => {
            fl!(I18N, "pstate-clock", pstate = pstate)
        }
        ClockspeedType::GpuVfCurveVoltage(pstate) | ClockspeedType::MemVfCurveVoltage(pstate) => {
            fl!(I18N, "pstate-clock-voltage", pstate = pstate)
        }
        // These always carry a custom title with the domain name
        ClockspeedType::ClockDomainOffset(domain) => {
            fl!(I18N, "clock-domain-offset", domain = domain)
        }
        ClockspeedType::ClockDomainVoltageOffset(domain) => {
            fl!(I18N, "clock-domain-voltage-offset", domain = domain)
        }
        ClockspeedType::Reset => unreachable!(),
    }
}

fn get_row_step(id: RowId) -> f64 {
    let RowId::Clock(clock_type) = id else {
        // The master is an MSVDD offset like the rows it drives
        return 1.0;
    };

    match clock_type {
        ClockspeedType::MaxCoreClock
        | ClockspeedType::MinCoreClock
        | ClockspeedType::GpuClockOffset(_)
        | ClockspeedType::MaxMemoryClock
        | ClockspeedType::MinMemoryClock
        | ClockspeedType::MemClockOffset(_)
        | ClockspeedType::GpuVfCurveClock(_)
        | ClockspeedType::MemVfCurveClock(_)
        | ClockspeedType::ClockDomainOffset(_) => 5.0,
        ClockspeedType::MinVoltage
        | ClockspeedType::MaxVoltage
        | ClockspeedType::VoltageOffset
        | ClockspeedType::VoltageBoost
        | ClockspeedType::GpuVfCurveVoltage(_)
        | ClockspeedType::MemVfCurveVoltage(_)
        | ClockspeedType::ClockDomainVoltageOffset(_) => 1.0,
        ClockspeedType::Reset => unreachable!(),
    }
}

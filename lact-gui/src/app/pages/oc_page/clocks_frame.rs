mod adjustment_row;

use crate::{
    app::{msg::AppMsg, page_section::PageSection},
    APP_BROKER,
};
use adjustment_row::{ClockAdjustmentRow, ClockAdjustmentRowMsg, ClocksData};
use amdgpu_sysfs::gpu_handle::overdrive::{ClocksTable as _, ClocksTableGen as AmdClocksTable};
use gtk::{
    pango,
    prelude::{BoxExt, ButtonExt, CheckButtonExt, OrientableExt, WidgetExt},
};
use lact_schema::{
    request::{ClockspeedType, SetClocksCommand},
    ClocksTable, IntelClocksTable, NvidiaClockOffset, NvidiaClocksTable,
};
use relm4::{
    binding::BoolBinding, factory::FactoryHashMap, ComponentParts, ComponentSender, RelmObjectExt,
    RelmWidgetExt,
};

// This is only used on RDNA1 in practice
const DEFAULT_VOLTAGE_OFFSET_RANGE: i32 = 250;
const WARNING_TEXT: &str = "Warning: changing these values may lead to system instability and potentially damage your hardware!";

pub struct ClocksFrame {
    clocks: FactoryHashMap<ClockspeedType, ClockAdjustmentRow>,
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
impl relm4::SimpleComponent for ClocksFrame {
    type Init = ();
    type Input = ClocksFrameMsg;
    type Output = ();

    view! {
        PageSection::new("Clockspeed and voltage") {
            append = &gtk::Label {
                set_label: WARNING_TEXT,
                set_wrap_mode: pango::WrapMode::Word,
                set_halign: gtk::Align::Start,
                set_margin_horizontal: 5,
            },

            append = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,

                #[watch]
                set_visible: model.show_nvidia_options,

                append = &gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_homogeneous: true,
                    set_hexpand: true,

                    append = &gtk::CheckButton {
                        set_label: Some("Show all P-States"),
                        add_binding["active"]: &model.show_all_pstates,
                    },


                    append = &gtk::CheckButton {
                        set_label: Some("Enable GPU Locked Clocks"),
                        add_binding["active"]: &model.enable_gpu_locked_clocks,
                        connect_toggled => move |_| {
                            APP_BROKER.send(AppMsg::SettingsChanged);
                        }
                    },

                    append = &gtk::CheckButton {
                        set_label: Some("Enable VRAM Locked Clocks"),
                        add_binding["active"]: &model.enable_vram_locked_clocks,
                        connect_toggled => move |_| {
                            APP_BROKER.send(AppMsg::SettingsChanged);
                        }
                    },
                },

                append = &gtk::Label {
                    add_binding["visible"]: &model.show_all_pstates,

                    set_margin_horizontal: 5,
                    set_markup: "<b>The following values are clock offsets for each P-State, going from highest to lowest.</b>",
                    set_wrap_mode: pango::WrapMode::Word,
                    set_halign: gtk::Align::Start,
                },
            },

            append = model.clocks.widget() {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,
                set_margin_horizontal: 5,
            },

            append = &gtk::Label {
                set_label: "No clocks data available",
                set_margin_horizontal: 10,
                set_halign: gtk::Align::Start,
                #[watch]
                set_visible: model.clocks.is_empty(),
            },

            append = &gtk::Button {
                set_label: "Reset",
                set_halign: gtk::Align::End,
                set_margin_horizontal: 5,
                set_tooltip_text: Some("Warning: this resets all clock settings to defaults!"),
                set_css_classes: &["destructive-action"],
                #[watch]
                set_visible: !model.clocks.is_empty(),

                connect_clicked => move |_| {
                    APP_BROKER.send(AppMsg::ResetClocks);
                }
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let clocks = FactoryHashMap::builder().launch_default().detach();

        let model = Self {
            clocks,
            vram_clock_ratio: 1.0,
            show_nvidia_options: false,
            show_all_pstates: BoolBinding::new(false),
            enable_gpu_locked_clocks: BoolBinding::new(false),
            enable_vram_locked_clocks: BoolBinding::new(false),
        };

        // TODO: block these signals when setting clocks
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

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            ClocksFrameMsg::Clocks(clocks_table) => {
                self.clocks.clear();
                self.show_all_pstates.set_value(false);
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

                for clockspeed_type in self.clocks.keys() {
                    self.clocks.send(
                        clockspeed_type,
                        ClockAdjustmentRowMsg::AddSizeGroup {
                            label_group: label_size_group.clone(),
                            input_group: input_size_group.clone(),
                        },
                    );
                }
            }
            ClocksFrameMsg::VramRatio(vram_ratio) => {
                self.vram_clock_ratio = vram_ratio;
            }
            ClocksFrameMsg::TogglePStatesVisibility => {
                let show_secondary = self.show_all_pstates.value();
                for (key, row) in self.clocks.iter() {
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

                    self.clocks
                        .send(key, ClockAdjustmentRowMsg::SetVisible(show_current));
                }
            }
        }
        self.update_vram_clock_ratio();
    }
}

impl ClocksFrame {
    fn update_vram_clock_ratio(&self) {
        for clock_type in [
            ClockspeedType::MaxMemoryClock,
            ClockspeedType::MinMemoryClock,
        ] {
            if self.clocks.get(&clock_type).is_some() {
                self.clocks.send(
                    &clock_type,
                    ClockAdjustmentRowMsg::ValueRatio(self.vram_clock_ratio),
                );
            }
        }
    }

    fn set_amd_table(&mut self, table: AmdClocksTable) {
        let clocks_types = [
            (
                ClockspeedType::MaxCoreClock,
                table.get_max_sclk(),
                table.get_max_sclk_range(),
            ),
            (
                ClockspeedType::MaxMemoryClock,
                table.get_max_mclk(),
                table.get_max_mclk_range(),
            ),
            (
                ClockspeedType::MaxVoltage,
                table.get_max_sclk_voltage(),
                table.get_max_voltage_range(),
            ),
            (
                ClockspeedType::MinCoreClock,
                table.get_current_sclk_range().min,
                table.get_min_sclk_range(),
            ),
            (
                ClockspeedType::MinMemoryClock,
                table.get_current_mclk_range().min,
                table.get_min_mclk_range(),
            ),
            (
                ClockspeedType::MinVoltage,
                table
                    .get_current_voltage_range()
                    .and_then(|range| range.min),
                table.get_min_voltage_range(),
            ),
        ];

        for (clockspeed_type, current_value, range) in clocks_types {
            if let Some(current) = current_value {
                if let Some((min, max)) = range.and_then(|range| range.into_full()) {
                    self.clocks
                        .insert(clockspeed_type, ClocksData::new(current, min, max));
                }
            }
        }

        if let AmdClocksTable::Vega20(table) = table {
            if let Some(current) = table.voltage_offset {
                let (min, max) = table
                    .od_range
                    .voltage_offset
                    .and_then(|range| range.into_full())
                    .unwrap_or((-DEFAULT_VOLTAGE_OFFSET_RANGE, DEFAULT_VOLTAGE_OFFSET_RANGE));

                self.clocks.insert(
                    ClockspeedType::VoltageOffset,
                    ClocksData::new(current, min, max),
                );
            }
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
                &self.enable_gpu_locked_clocks,
            ),
            (
                table.vram_clock_range,
                table.vram_locked_clocks,
                ClockspeedType::MinMemoryClock,
                ClockspeedType::MaxMemoryClock,
                &self.enable_vram_locked_clocks,
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

                self.clocks.insert(
                    min_type,
                    ClocksData::new(current_min as i32, gpu_min as i32, gpu_max as i32),
                );
                self.clocks.insert(
                    max_type,
                    ClocksData::new(current_max as i32, gpu_min as i32, gpu_max as i32),
                );
            }
        }

        for (pstate, offset) in table.gpu_offsets {
            self.clocks.insert(
                ClockspeedType::GpuClockOffset(pstate),
                nvidia_clock_offset_to_data(&offset, pstate > 0),
            );
        }
        for (pstate, offset) in table.mem_offsets {
            self.clocks.insert(
                ClockspeedType::MemClockOffset(pstate),
                nvidia_clock_offset_to_data(&offset, pstate > 0),
            );
        }
    }

    fn set_intel_table(&mut self, table: IntelClocksTable) {
        if let Some((current_gt_min, current_gt_max)) = table.gt_freq {
            if let (Some(min_clock), Some(max_clock)) = (table.rpn_freq, table.rp0_freq) {
                self.clocks.insert(
                    ClockspeedType::MaxCoreClock,
                    ClocksData::new(current_gt_max as i32, min_clock as i32, max_clock as i32),
                );
                self.clocks.insert(
                    ClockspeedType::MinCoreClock,
                    ClocksData::new(current_gt_min as i32, min_clock as i32, max_clock as i32),
                );
            }
        }
    }

    pub fn get_commands(&self) -> Vec<SetClocksCommand> {
        self.clocks
            .iter()
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
        is_secondary,
    }
}

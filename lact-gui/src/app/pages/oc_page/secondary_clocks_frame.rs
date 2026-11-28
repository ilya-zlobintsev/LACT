use crate::{
    APP_BROKER, I18N,
    app::{
        components::{
            adjustment_card::AdjustmentCard,
            page_section::PageSection,
            setting_row::{
                AdjustmentInit, SettingRow, SettingRowInit, SettingRowMsg, SettingRowValue,
            },
        },
        msg::AppMsg,
    },
};
use adw::prelude::*;
use i18n_embed_fl::fl;
use lact_schema::ClocksTable;
use relm4::{ComponentParts, ComponentSender, WidgetTemplate, factory::FactoryHashMap};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SecondaryClockSetting {
    XbarClock,
    SysClock,
    XbarVoltage,
    SysVoltage,
}

pub struct SecondaryClocksFrameSerialized {
    pub xbar_clock_offset: Option<i32>,
    pub sys_clock_offset: Option<i32>,
    pub xbar_voltage_offset: Option<i32>,
    pub sys_voltage_offset: Option<i32>,
}

struct SizeGroups {
    label: gtk::SizeGroup,
    input: gtk::SizeGroup,
    lower_label: gtk::SizeGroup,
    upper_label: gtk::SizeGroup,
}

impl Default for SizeGroups {
    fn default() -> Self {
        let group = || gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal);
        Self {
            label: group(),
            input: group(),
            lower_label: group(),
            upper_label: group(),
        }
    }
}

pub struct SecondaryClocksFrame {
    rows: FactoryHashMap<SecondaryClockSetting, SettingRow<SecondaryClockSetting>>,
    size_groups: SizeGroups,
}

#[derive(Debug)]
pub enum SecondaryClocksFrameMsg {
    Clocks(Option<Arc<ClocksTable>>),
    Reset,
}

#[relm4::component(pub)]
impl relm4::SimpleComponent for SecondaryClocksFrame {
    type Init = ();
    type Input = SecondaryClocksFrameMsg;
    type Output = ();

    view! {
        PageSection {
            set_name: fl!(I18N, "secondary-clocks-section"),
            set_hide_visible_container: true,
            #[watch]
            set_visible: !model.rows.is_empty(),

            append_header = &gtk::Button {
                set_label: &fl!(I18N, "default-button"),
                connect_clicked => SecondaryClocksFrameMsg::Reset,
                set_halign: gtk::Align::End,
                set_hexpand: true,
            },
            append_child = model.rows.widget(),
        },
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            rows: FactoryHashMap::builder()
                .launch(AdjustmentCard::init(()).content.clone())
                .forward(APP_BROKER.sender(), |_| AppMsg::SettingsChanged),
            size_groups: SizeGroups::default(),
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            SecondaryClocksFrameMsg::Clocks(table) => {
                self.rows.clear();
                self.size_groups = SizeGroups::default();

                if let Some(ClocksTable::Nvidia(table)) = table.as_deref() {
                    for (setting, offset) in [
                        (SecondaryClockSetting::XbarClock, table.xbar_clock_offset),
                        (SecondaryClockSetting::SysClock, table.sys_clock_offset),
                    ] {
                        if let Some(offset) = offset {
                            self.insert_row(
                                setting,
                                AdjustmentInit {
                                    unit: fl!(I18N, "mhz"),
                                    value: f64::from(offset.current),
                                    default_value: Some(f64::from(offset.default)),
                                    lower: f64::from(offset.min),
                                    upper: f64::from(offset.max),
                                    step_increment: 5.0,
                                    ..Default::default()
                                },
                            );
                        }
                    }

                    for (setting, offset) in [
                        (
                            SecondaryClockSetting::XbarVoltage,
                            table.xbar_voltage_offset,
                        ),
                        (SecondaryClockSetting::SysVoltage, table.sys_voltage_offset),
                    ] {
                        if let Some(offset) = offset {
                            self.insert_row(
                                setting,
                                AdjustmentInit {
                                    unit: fl!(I18N, "mv"),
                                    value: f64::from(offset.current),
                                    default_value: Some(f64::from(offset.default)),
                                    lower: f64::from(offset.min),
                                    upper: f64::from(offset.max),
                                    step_increment: 1.0,
                                    ..Default::default()
                                },
                            );
                        }
                    }
                }
            }
            SecondaryClocksFrameMsg::Reset => {
                for setting in self.rows.keys() {
                    self.rows.send(setting, SettingRowMsg::Reset);
                }
            }
        }
    }
}

impl SecondaryClocksFrame {
    pub fn serialize(&self) -> SecondaryClocksFrameSerialized {
        let value = |setting| {
            self.rows
                .get(&setting)
                .and_then(SettingRow::serialize)
                .and_then(SettingRowValue::as_numeric)
                .map(|value| value as i32)
        };
        SecondaryClocksFrameSerialized {
            xbar_clock_offset: value(SecondaryClockSetting::XbarClock),
            sys_clock_offset: value(SecondaryClockSetting::SysClock),
            xbar_voltage_offset: value(SecondaryClockSetting::XbarVoltage),
            sys_voltage_offset: value(SecondaryClockSetting::SysVoltage),
        }
    }

    fn insert_row(&mut self, setting: SecondaryClockSetting, mut control: AdjustmentInit) {
        control.lower_label_group = Some(self.size_groups.lower_label.clone());
        control.upper_label_group = Some(self.size_groups.upper_label.clone());
        self.rows.insert(
            setting,
            SettingRowInit {
                title: match setting {
                    SecondaryClockSetting::XbarClock => fl!(I18N, "xbar-clock-offset"),
                    SecondaryClockSetting::SysClock => fl!(I18N, "sys-clock-offset"),
                    SecondaryClockSetting::XbarVoltage => fl!(I18N, "xbar-voltage-offset"),
                    SecondaryClockSetting::SysVoltage => fl!(I18N, "sys-voltage-offset"),
                },
                label_group: Some(self.size_groups.label.clone()),
                input_group: Some(self.size_groups.input.clone()),
                control: control.into(),
                ..Default::default()
            },
        );
    }
}

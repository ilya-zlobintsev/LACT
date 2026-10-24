mod adjustment_control;
mod adjustment_value;
mod boolean_control;
mod boolean_value;
mod select_control;
mod select_value;
mod setting_value;

use self::{
    adjustment_control::AdjustmentControl, boolean_control::BooleanControl,
    select_control::SelectControl, setting_value::SettingValue,
};
#[expect(unused_imports, reason = "setting-row-migration")]
pub use self::{
    adjustment_control::{AdjustmentControlMsg, AdjustmentInit},
    boolean_control::BooleanInit,
    select_control::{SelectInit, SelectOption},
};
use adw::prelude::*;
use relm4::{
    Component, ComponentController, Controller, FactorySender, RelmWidgetExt, WidgetTemplate, css,
    factory::FactoryComponent,
};
use std::marker::PhantomData;

pub struct SettingRow<Key> {
    pub control: SettingControl,
    _key: PhantomData<Key>,
}

#[derive(Default)]
pub struct SettingRowInit {
    pub title: String,
    pub title_tooltip: String,
    pub info_text: String,
    pub label_group: Option<gtk::SizeGroup>,
    pub input_group: Option<gtk::SizeGroup>,
    pub control: SettingControlInit,
}

pub enum SettingControlInit {
    Adjustment(AdjustmentInit),
    Boolean(BooleanInit),
    Select(SelectInit),
}

impl Default for SettingControlInit {
    fn default() -> Self {
        Self::Adjustment(AdjustmentInit::default())
    }
}

impl From<AdjustmentInit> for SettingControlInit {
    fn from(init: AdjustmentInit) -> Self {
        Self::Adjustment(init)
    }
}

impl From<BooleanInit> for SettingControlInit {
    fn from(init: BooleanInit) -> Self {
        Self::Boolean(init)
    }
}

impl From<SelectInit> for SettingControlInit {
    fn from(init: SelectInit) -> Self {
        Self::Select(init)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SettingRowValue {
    Numeric(f64),
    Boolean(bool),
    Selected(u32),
}

impl SettingRowValue {
    pub fn as_numeric(self) -> Option<f64> {
        match self {
            Self::Numeric(value) => Some(value),
            _ => None,
        }
    }

    #[expect(dead_code, reason = "setting-row-migration")]
    pub fn as_boolean(self) -> Option<bool> {
        match self {
            Self::Boolean(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_selected(self) -> Option<u32> {
        match self {
            Self::Selected(value) => Some(value),
            _ => None,
        }
    }
}

impl<Key> SettingRow<Key> {
    pub fn widget(&self) -> &gtk::Box {
        self.control.widget()
    }

    /// Serialize the current value, omitting unset values and defaults.
    pub fn serialize(&self) -> Option<SettingRowValue> {
        match &self.control {
            SettingControl::Adjustment(control) => control
                .model()
                .value
                .serialize()
                .map(SettingRowValue::Numeric),
            SettingControl::Boolean(control) => control
                .model()
                .value
                .serialize()
                .map(SettingRowValue::Boolean),
            SettingControl::Select(control) => control
                .model()
                .value
                .serialize()
                .map(SettingRowValue::Selected),
        }
    }

    fn set_value(&self, value: SettingRowValue) {
        match (&self.control, value) {
            (SettingControl::Adjustment(control), SettingRowValue::Numeric(value)) => {
                control.model().value.set_value(value)
            }
            (SettingControl::Boolean(control), SettingRowValue::Boolean(value)) => {
                control.model().value.set_value(value)
            }
            (SettingControl::Select(control), SettingRowValue::Selected(value)) => {
                control.model().value.set_value(value)
            }
            _ => panic!("setting value does not match the row's control"),
        }
    }

    pub fn get_value(&self) -> SettingRowValue {
        match &self.control {
            SettingControl::Adjustment(control) => {
                SettingRowValue::Numeric(control.model().value.get_value())
            }
            SettingControl::Boolean(control) => {
                SettingRowValue::Boolean(control.model().value.get_value())
            }
            SettingControl::Select(control) => {
                SettingRowValue::Selected(control.model().value.get_value())
            }
        }
    }
}

pub enum SettingControl {
    Adjustment(Controller<AdjustmentControl>),
    Boolean(Controller<BooleanControl>),
    Select(Controller<SelectControl>),
}

impl SettingControl {
    fn new(init: SettingRowInit, output: &relm4::Sender<()>) -> Self {
        let title = SettingTitle::init(SettingTitleInit {
            title: init.title,
            tooltip: init.title_tooltip,
            info_text: init.info_text,
        })
        .root;
        let label_group = init.label_group;
        let input_group = init.input_group;
        match init.control {
            SettingControlInit::Adjustment(init) => Self::Adjustment(
                AdjustmentControl::builder()
                    .launch((init, title, label_group, input_group))
                    .forward(output, |()| ()),
            ),
            SettingControlInit::Boolean(init) => Self::Boolean(
                BooleanControl::builder()
                    .launch((init, title, label_group, input_group))
                    .forward(output, |()| ()),
            ),
            SettingControlInit::Select(init) => Self::Select(
                SelectControl::builder()
                    .launch((init, title, label_group, input_group))
                    .forward(output, |()| ()),
            ),
        }
    }

    fn widget(&self) -> &gtk::Box {
        match self {
            Self::Adjustment(control) => control.widget(),
            Self::Boolean(control) => control.widget(),
            Self::Select(control) => control.widget(),
        }
    }

    /// Size groups also measure widgets inside hidden rows, so hidden rows leave their groups.
    fn set_size_groups_enabled(&self, enabled: bool) {
        let toggle = |size_group_widgets: &[(gtk::SizeGroup, gtk::Widget)]| {
            for (group, widget) in size_group_widgets {
                if enabled {
                    group.add_widget(widget);
                } else {
                    group.remove_widget(widget);
                }
            }
        };
        match self {
            Self::Adjustment(control) => toggle(&control.model().size_group_widgets),
            Self::Boolean(control) => toggle(&control.model().size_group_widgets),
            Self::Select(control) => toggle(&control.model().size_group_widgets),
        }
    }

    fn reset(&self) {
        match self {
            Self::Adjustment(control) => control.model().value.reset(),
            Self::Boolean(control) => control.model().value.reset(),
            Self::Select(control) => control.model().value.reset(),
        }
    }
}

#[derive(Debug)]
pub enum SettingRowMsg {
    Reset,
    SetValue(SettingRowValue),
    #[expect(dead_code, reason = "setting-row-migration")]
    SetVisible(bool),
}

#[relm4::factory(pub)]
impl<Key: 'static> FactoryComponent for SettingRow<Key> {
    type ParentWidget = gtk::ListBox;
    type Index = Key;
    type Init = SettingRowInit;
    type Input = SettingRowMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        #[name = "root_row"]
        gtk::ListBoxRow {
            set_activatable: false,
            set_selectable: false,
            add_css_class: "setting-row",
            add_css_class: match &self.control {
                SettingControl::Adjustment(_) => "adjustment-row",
                SettingControl::Boolean(_) => "boolean-row",
                SettingControl::Select(_) => "select-row",
            },
            set_child: Some(self.control.widget()),
        },
    }

    fn init_model(init: Self::Init, _index: &Self::Index, sender: FactorySender<Self>) -> Self {
        Self {
            control: SettingControl::new(init, sender.output_sender()),
            _key: PhantomData,
        }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::Input,
        _sender: FactorySender<Self>,
    ) {
        match msg {
            SettingRowMsg::Reset => self.control.reset(),
            SettingRowMsg::SetValue(value) => self.set_value(value),
            SettingRowMsg::SetVisible(visible) => {
                if widgets.root_row.get_visible() != visible {
                    self.control.set_size_groups_enabled(visible);
                    widgets.root_row.set_visible(visible);
                }
            }
        }
    }
}

struct SettingTitleInit {
    title: String,
    tooltip: String,
    info_text: String,
}

#[relm4::widget_template]
impl WidgetTemplate for SettingTitle {
    type Init = SettingTitleInit;

    view! {
        #[name = "root"]
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_valign: gtk::Align::Center,
            set_hexpand: true,
            set_spacing: 5,

            gtk::Label {
                set_valign: gtk::Align::Center,
                set_xalign: 0.0,
                set_markup: &init.title,
                set_tooltip_text: (!init.tooltip.is_empty()).then_some(init.tooltip.as_str()),
            },
            gtk::Box {
                add_css_class: css::CAPTION,
                set_valign: gtk::Align::End,

                gtk::MenuButton {
                    set_visible: !init.info_text.is_empty(),
                    add_css_class: css::FLAT,
                    #[wrap(Some)]
                    set_popover = &gtk::Popover {
                        gtk::Label {
                            set_label: &init.info_text,
                            set_margin_all: 5,
                            set_wrap: true,
                            set_wrap_mode: gtk::pango::WrapMode::Word,
                            set_max_width_chars: 55,
                        },
                    },
                    #[wrap(Some)]
                    set_child = &gtk::Image {
                        set_icon_name: Some("info-outline-symbolic"),
                        set_pixel_size: 12,
                    },
                },
            },
        },
    }
}

mod heuristics_list;

use crate::{
    APP_BROKER, I18N,
    app::{msg::AppMsg, pages::oc_page::OcPageMsg},
};
use adw::prelude::*;
use amdgpu_sysfs::gpu_handle::{PerformanceLevel, power_profile_mode::PowerProfileModesTable};
use gtk::StringObject;
use heuristics_list::PowerProfileHeuristicsList;
use i18n_embed_fl::fl;
use relm4::{Component, ComponentController, ComponentParts, ComponentSender, RelmWidgetExt, css};

pub struct PerformanceFrame {
    performance_level: Option<PerformanceLevel>,
    power_profile_modes_table: Option<PowerProfileModesTable>,
    power_profile_modes: gtk::StringList,
    heuristics_components: Vec<relm4::Controller<PowerProfileHeuristicsList>>,
}

#[derive(Debug, Clone, Default)]
pub struct PerformanceRows {
    pub power_profile: gtk::ListBoxRow,
}

#[derive(Debug)]
pub enum PerformanceFrameMsg {
    PerformanceLevel(Option<PerformanceLevel>),
    PowerProfileModes(Option<PowerProfileModesTable>),
    PowerProfileSelected(u16),
}

#[relm4::component(pub)]
impl relm4::Component for PerformanceFrame {
    type Init = ();
    type Input = PerformanceFrameMsg;
    type Output = OcPageMsg;
    type CommandOutput = ();

    view! {
        #[root]
        PerformanceRows::default() {},

        #[local]
        power_profile_row -> gtk::ListBoxRow {
            set_activatable: false,
            set_selectable: false,
            #[watch]
            set_visible: model.power_profile_modes_table.is_some(),

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 10,

                gtk::Label {
                    set_label: &fl!(I18N, "power-profile-mode"),
                    set_halign: gtk::Align::Start,
                },

                gtk::MenuButton {
                    #[watch]
                    set_visible: model.performance_level != Some(PerformanceLevel::Manual),

                    #[wrap(Some)]
                    set_popover =  &gtk::Popover {
                        gtk::Label {
                            set_label: &fl!(I18N, "manual-level-needed"),
                        }
                    },

                    add_css_class: css::FLAT,
                    #[wrap(Some)]
                    set_child = &gtk::Image {
                        set_icon_name: Some("info-outline-symbolic"),
                        set_pixel_size: 12,
                    },
                },

                gtk::MenuButton {
                    set_hexpand: true,
                    set_halign: gtk::Align::End,
                    set_always_show_arrow: false,
                    #[watch]
                    set_label: model.power_profile_modes_table.as_ref().and_then(|table| table.modes.get(&table.active)).map(|profile| profile.name.as_str()).unwrap_or_default(),

                    #[wrap(Some)]
                    set_popover =  &gtk::Popover {
                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_margin_all: 5,

                            #[name = "modes_listbox"]
                            gtk::ListBox {
                                set_selection_mode: gtk::SelectionMode::Single,
                                #[watch]
                                set_sensitive: model.performance_level.is_some_and(|level| level == PerformanceLevel::Manual),

                                bind_model: (Some(&model.power_profile_modes), |obj| {
                                    let string = obj.downcast_ref::<StringObject>().unwrap();
                                    gtk::Label::builder().label(string.string()).build().into()
                                }),

                                connect_row_selected[sender] => move |_, row| {
                                    if let Some(row) = row
                                        && let Ok(idx) = u16::try_from(row.index()) {
                                            sender.input(PerformanceFrameMsg::PowerProfileSelected(idx));
                                        }
                                } @ power_profile_selected_handler,

                                #[watch]
                                #[block_signal(power_profile_selected_handler)]
                                select_row: model.power_profile_modes_table.as_ref().and_then(|table| modes_listbox.row_at_index(table.active.into())).as_ref(),
                            },

                            #[name = "heuristics_notebook"]
                            gtk::Notebook {
                                #[watch]
                                set_show_tabs: model.heuristics_components.len() > 1,
                            },
                        }
                    },
                },
            },
        },
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let power_profile_row = root.power_profile.clone();

        let model = Self {
            performance_level: None,
            power_profile_modes_table: None,
            power_profile_modes: gtk::StringList::new(&[]),
            heuristics_components: vec![],
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
            PerformanceFrameMsg::PerformanceLevel(level) => {
                self.performance_level = level;
                sender.output(OcPageMsg::PerformanceLevelChanged).unwrap();
            }
            PerformanceFrameMsg::PowerProfileModes(table) => {
                while self.power_profile_modes.n_items() != 0 {
                    self.power_profile_modes.remove(0);
                }

                if let Some(table) = &table {
                    for mode in table.modes.values() {
                        self.power_profile_modes.append(&mode.name);
                    }
                }

                self.power_profile_modes_table = table;
                self.update_heuristic_components(widgets);
            }
            PerformanceFrameMsg::PowerProfileSelected(idx) => {
                if let Some(table) = &mut self.power_profile_modes_table
                    && table.active != idx
                {
                    table.active = idx;
                    APP_BROKER.send(AppMsg::SettingsChanged);
                    self.update_heuristic_components(widgets);
                }
            }
        }

        if let Some(table) = &self.power_profile_modes_table
            && let Some(active) = table.modes.get(&table.active)
        {
            for heuristics_component in &self.heuristics_components {
                heuristics_component
                    .widget()
                    .set_sensitive(active.is_custom());
            }
        }

        self.update_view(widgets, sender);
    }
}

impl PerformanceFrame {
    fn update_heuristic_components(&mut self, widgets: &PerformanceFrameWidgets) {
        while let Some(component) = self.heuristics_components.pop() {
            let page = widgets.heuristics_notebook.page(component.widget());
            widgets
                .heuristics_notebook
                .remove_page(Some(page.position() as u32));
        }

        if let Some(table) = &self.power_profile_modes_table
            && let Some(active_profile) = table.modes.get(&table.active)
        {
            for component in &active_profile.components {
                let title = component.clock_type.as_deref().unwrap_or("All");

                let heuristics_component = PowerProfileHeuristicsList::builder()
                    .launch((component.values.clone(), table.value_names.clone()))
                    .detach();

                widgets.heuristics_notebook.append_page(
                    heuristics_component.widget(),
                    Some(&gtk::Label::new(Some(title))),
                );

                self.heuristics_components.push(heuristics_component);
            }
        }
    }

    pub fn power_profile_mode(&self) -> Option<u16> {
        if self.performance_level == Some(PerformanceLevel::Manual) {
            self.power_profile_modes_table
                .as_ref()
                .map(|table| table.active)
        } else {
            None
        }
    }

    pub fn power_profile_mode_custom_heuristics(&self) -> Vec<Vec<Option<i32>>> {
        if let Some(table) = &self.power_profile_modes_table
            && let Some(mode) = table.modes.get(&table.active)
            && mode.is_custom()
        {
            return self
                .heuristics_components
                .iter()
                .map(|list| list.model().get_values())
                .collect();
        }

        vec![]
    }
}

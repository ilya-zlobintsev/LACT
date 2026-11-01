mod heuristics_list;

use crate::I18N;
use adw::prelude::*;
use amdgpu_sysfs::gpu_handle::{PerformanceLevel, power_profile_mode::PowerProfileModesTable};
use gtk::StringObject;
use heuristics_list::PowerProfileHeuristicsList;
use i18n_embed_fl::fl;
use relm4::{Component, ComponentController, ComponentParts, ComponentSender, RelmWidgetExt, css};

pub struct AmdPowerProfileMode {
    performance_level: Option<PerformanceLevel>,
    table: PowerProfileModesTable,
    heuristics_components: Vec<relm4::Controller<PowerProfileHeuristicsList>>,
}

pub struct AmdPowerProfileModeSerialized {
    pub index: u16,
    pub custom_heuristics: Vec<Vec<Option<i32>>>,
}

#[derive(Debug)]
pub enum AmdPowerProfileModeMsg {
    PerformanceLevel(Option<PerformanceLevel>),
    Selected(u16),
}

#[relm4::component(pub)]
impl Component for AmdPowerProfileMode {
    type Init = (PowerProfileModesTable, Option<PerformanceLevel>);
    type Input = AmdPowerProfileModeMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk::ListBoxRow {
            set_activatable: false,
            set_selectable: false,
            add_css_class: "power-profile-mode",

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
                    set_popover = &gtk::Popover {
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
                    set_label: model.table.modes.get(&model.table.active).map(|profile| profile.name.as_str()).unwrap_or_default(),

                    #[wrap(Some)]
                    set_popover = &gtk::Popover {
                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_margin_all: 5,

                            #[name = "modes_listbox"]
                            gtk::ListBox {
                                set_selection_mode: gtk::SelectionMode::Single,
                                #[watch]
                                set_sensitive: model.performance_level == Some(PerformanceLevel::Manual),

                                bind_model: (
                                    Some(&model.table.modes.values().map(|mode| mode.name.clone()).collect::<gtk::StringList>()),
                                    |obj| {
                                        let string = obj.downcast_ref::<StringObject>().unwrap();
                                        gtk::Label::builder().label(string.string()).build().into()
                                    },
                                ),

                                connect_row_selected[sender] => move |_, row| {
                                    if let Some(row) = row
                                        && let Ok(idx) = u16::try_from(row.index()) {
                                            sender.input(AmdPowerProfileModeMsg::Selected(idx));
                                        }
                                } @ selected_handler,

                                #[watch]
                                #[block_signal(selected_handler)]
                                select_row: modes_listbox.row_at_index(model.table.active.into()).as_ref(),
                            },

                            #[name = "heuristics_notebook"]
                            gtk::Notebook {
                                #[watch]
                                set_show_tabs: model.table.modes.get(&model.table.active).is_some_and(|mode| mode.components.len() > 1),
                            },
                        }
                    },
                },
            },
        },
    }

    fn init(
        (table, performance_level): Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut model = Self {
            performance_level,
            table,
            heuristics_components: vec![],
        };
        let widgets = view_output!();
        model.update_heuristic_components(&widgets);
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
            AmdPowerProfileModeMsg::PerformanceLevel(level) => {
                self.performance_level = level;
            }
            AmdPowerProfileModeMsg::Selected(idx) => {
                if self.table.active != idx {
                    self.table.active = idx;
                    let _ = sender.output(());
                    self.update_heuristic_components(widgets);
                }
            }
        }
        self.update_view(widgets, sender);
    }
}

impl AmdPowerProfileMode {
    fn update_heuristic_components(&mut self, widgets: &AmdPowerProfileModeWidgets) {
        while let Some(component) = self.heuristics_components.pop() {
            let page = widgets.heuristics_notebook.page(component.widget());
            widgets
                .heuristics_notebook
                .remove_page(Some(page.position() as u32));
        }

        if let Some(active_profile) = self.table.modes.get(&self.table.active) {
            for component in &active_profile.components {
                let title = component.clock_type.as_deref().unwrap_or("All");

                let heuristics_component = PowerProfileHeuristicsList::builder()
                    .launch((component.values.clone(), self.table.value_names.clone()))
                    .detach();
                heuristics_component
                    .widget()
                    .set_sensitive(active_profile.is_custom());

                widgets.heuristics_notebook.append_page(
                    heuristics_component.widget(),
                    Some(&gtk::Label::new(Some(title))),
                );

                self.heuristics_components.push(heuristics_component);
            }
        }
    }

    /// Serialize the active mode with its custom heuristics; modes only apply at the manual level.
    pub fn serialize(&self) -> Option<AmdPowerProfileModeSerialized> {
        if self.performance_level != Some(PerformanceLevel::Manual) {
            return None;
        }
        let custom_heuristics = match self.table.modes.get(&self.table.active) {
            Some(mode) if mode.is_custom() => self
                .heuristics_components
                .iter()
                .map(|list| list.model().get_values())
                .collect(),
            _ => Vec::new(),
        };
        Some(AmdPowerProfileModeSerialized {
            index: self.table.active,
            custom_heuristics,
        })
    }
}

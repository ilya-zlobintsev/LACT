use crate::app::ext::RelmDefaultLauchable;
use gtk::prelude::{FrameExt, GtkWindowExt, WidgetExt};
use lact_schema::IntelTopology;
use relm4::{
    factory::{positions::GridPosition, CloneableFactoryComponent, Position},
    prelude::{DynamicIndex, FactoryComponent, FactoryVecDeque},
    ComponentParts, ComponentSender, RelmWidgetExt,
};

pub struct TopologyWindow {
    root: FactoryVecDeque<TopologyItem>,
}

#[derive(Debug, Clone)]
pub enum TopologyType {
    Intel(IntelTopology),
}

#[relm4::component(pub)]
impl relm4::SimpleComponent for TopologyWindow {
    type Init = (String, TopologyType);
    type Input = ();
    type Output = ();

    view! {
        gtk::Window {
            set_hide_on_close: true,
            set_title: Some("GPU Topology"),

            gtk::ScrolledWindow {
                set_policy: (gtk::PolicyType::Automatic, gtk::PolicyType::Automatic),
                set_max_content_height: 700,
                set_max_content_width: 1300,
                set_propagate_natural_height: true,
                set_propagate_natural_width: true,

                model.root.widget(),
            },
        }
    }

    fn init(
        (root_name, topology): Self::Init,
        root_window: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut root_item = TopologyItem {
            name: root_name,
            subitems: FactoryVecDeque::builder().launch_default().detach(),
            items_per_row: 1,
        };

        match topology {
            TopologyType::Intel(intel_topology) => {
                process_intel_topology(intel_topology, &mut root_item.subitems)
            }
        }

        let mut root = FactoryVecDeque::detach_default();
        root.guard().push_back(root_item);

        let model = Self { root };

        let widgets = view_output!();

        root_window.present();

        ComponentParts { model, widgets }
    }
}

fn process_intel_topology(topology: IntelTopology, items: &mut FactoryVecDeque<TopologyItem>) {
    let mut guard = items.guard();

    let mut slice_num = 1;
    let mut subslice_num = 1;
    let mut eu_num = 1;

    let mut engines_item = TopologyItem {
        name: "Engines".to_owned(),
        subitems: FactoryVecDeque::detach_default(),
        items_per_row: 1,
    };

    {
        let mut engines = engines_item.subitems.guard();

        for engine in topology.engines {
            engines.push_back(TopologyItem {
                name: format!("{:?} engine {}", engine.class, engine.name),
                subitems: FactoryVecDeque::detach_default(),
                items_per_row: 4,
            });
        }
    }
    guard.push_back(engines_item);

    let mut slices_item = TopologyItem {
        name: "Slices".to_owned(),
        subitems: FactoryVecDeque::detach_default(),
        items_per_row: 1,
    };

    for slice in topology.slices {
        let mut slice_item = TopologyItem {
            name: format!("Slice {slice_num}"),
            subitems: FactoryVecDeque::detach_default(),
            items_per_row: 4,
        };

        {
            let mut slice_subitems = slice_item.subitems.guard();

            for subslice in slice.subslices {
                let mut subslice_item = TopologyItem {
                    name: format!("Subslice {subslice_num}"),
                    subitems: FactoryVecDeque::detach_default(),
                    items_per_row: 4,
                };

                {
                    let mut subslice_subitems = subslice_item.subitems.guard();
                    for _eu in subslice.eu {
                        let eu_item = TopologyItem {
                            name: format!("EU {eu_num}"),
                            subitems: FactoryVecDeque::detach_default(),
                            items_per_row: 4,
                        };
                        subslice_subitems.push_back(eu_item);
                        eu_num += 1;
                    }
                }

                slice_subitems.push_back(subslice_item);
                subslice_num += 1;
            }
        }

        slices_item.subitems.guard().push_back(slice_item);
        slice_num += 1;
    }

    guard.push_back(slices_item);

    if topology.vram_size != 0 {
        let vram_item = TopologyItem {
            name: format!("{} MiB VRAM", topology.vram_size / 1024 / 1024),
            subitems: FactoryVecDeque::detach_default(),
            items_per_row: 1,
        };
        guard.push_back(vram_item);
    }
}

#[derive(Clone)]
struct TopologyItem {
    name: String,
    subitems: FactoryVecDeque<TopologyItem>,
    items_per_row: usize,
}

#[relm4::factory]
impl FactoryComponent for TopologyItem {
    type Init = Self;
    type Input = ();
    type Output = ();
    type ParentWidget = gtk::Grid;
    type CommandOutput = ();

    view! {
        gtk::Frame {
            set_label: Some(&self.name),
            set_margin_all: 10,
            set_hexpand: true,
            set_label_align: if self.subitems.is_empty() { 0.5 } else { 0.0 },

            self.subitems.widget(),
        }
    }

    fn init_model(
        init: Self::Init,
        _index: &Self::Index,
        _sender: relm4::FactorySender<Self>,
    ) -> Self {
        init
    }
}

impl Position<GridPosition, DynamicIndex> for TopologyItem {
    fn position(&self, index: &DynamicIndex) -> GridPosition {
        let index = index.current_index();
        let x = index / self.items_per_row;
        let y = index % self.items_per_row;
        GridPosition {
            column: y as i32,
            row: x as i32,
            width: 1,
            height: 1,
        }
    }
}

impl CloneableFactoryComponent for TopologyItem {
    fn get_init(&self) -> Self::Init {
        self.clone()
    }
}

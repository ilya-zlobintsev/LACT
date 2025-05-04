use gtk::prelude::{FrameExt, GtkWindowExt};
use lact_schema::IntelTopology;
use relm4::{
    factory::{positions::GridPosition, CloneableFactoryComponent, Position},
    prelude::{DynamicIndex, FactoryComponent, FactoryVecDeque},
    ComponentParts, ComponentSender, RelmWidgetExt,
};

use crate::app::ext::RelmDefaultLauchable;

const ITEMS_PER_ROW: usize = 4;

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

            model.root.widget(),
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
        };

        match topology {
            TopologyType::Intel(intel_topology) => {
                process_intel_topology(intel_topology, &mut root_item.subitems)
            }
        }

        let mut root = FactoryVecDeque::launch_default();
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

    for slice in topology.slices {
        let mut slice_item = TopologyItem {
            name: format!("Slice {slice_num}"),
            subitems: FactoryVecDeque::launch_default(),
        };

        {
            let mut slice_subitems = slice_item.subitems.guard();

            for subslice in slice.subslices {
                let mut subslice_item = TopologyItem {
                    name: format!("Subslice {subslice_num}"),
                    subitems: FactoryVecDeque::launch_default(),
                };

                {
                    let mut subslice_subitems = subslice_item.subitems.guard();
                    for _eu in subslice.eu {
                        let eu_item = TopologyItem {
                            name: format!("EU {eu_num}"),
                            subitems: FactoryVecDeque::launch_default(),
                        };
                        subslice_subitems.push_back(eu_item);
                        eu_num += 1;
                    }
                }

                slice_subitems.push_back(subslice_item);
                subslice_num += 1;
            }
        }

        guard.push_back(slice_item);
        slice_num += 1;
    }
}

#[derive(Clone)]
struct TopologyItem {
    name: String,
    subitems: FactoryVecDeque<TopologyItem>,
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
            set_expand: true,

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
        let x = index / ITEMS_PER_ROW;
        let y = index % ITEMS_PER_ROW;
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

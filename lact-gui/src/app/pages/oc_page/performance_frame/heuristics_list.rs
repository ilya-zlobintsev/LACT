use gtk::prelude::{BoxExt, OrientableExt, WidgetExt};
use relm4::{
    binding::{BoolBinding, ConnectBinding, F64Binding},
    prelude::{DynamicIndex, FactoryVecDeque},
    ComponentParts, ComponentSender, RelmObjectExt, RelmWidgetExt,
};

pub struct PowerProfileHeuristicsList {
    values: FactoryVecDeque<HeuristicRow>,
}

#[relm4::component(pub)]
impl relm4::SimpleComponent for PowerProfileHeuristicsList {
    type Init = (Vec<Option<i32>>, Vec<String>);
    type Input = ();
    type Output = ();

    view! {
        gtk::Box {
            set_margin_all: 10,

            model.values.widget() {
                set_spacing: 5,
                set_orientation: gtk::Orientation::Vertical,
            },
        }
    }

    fn init(
        (values, value_names): Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut model = Self {
            values: FactoryVecDeque::builder().launch_default().detach(),
        };

        {
            let mut rows = model.values.guard();
            for (i, value) in values.into_iter().enumerate() {
                let name = value_names[i].clone();
                rows.push_back((name, value));
            }
        }

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}

struct HeuristicRow {
    name: String,
    enabled: BoolBinding,
    value: F64Binding,
}

#[relm4::factory]
impl relm4::factory::FactoryComponent for HeuristicRow {
    type Init = (String, Option<i32>);
    type Input = ();
    type Output = ();
    type Index = DynamicIndex;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 5,

            gtk::Label {
                set_label: &format!("{}:", self.name),
                set_halign: gtk::Align::Start,
                set_hexpand: true,
            },

            gtk::CheckButton {
                add_binding["active"]: &self.enabled,
            },

            gtk::SpinButton {
                set_digits: 0,
                set_climb_rate: 1.0,
                set_numeric: true,
                set_increments: (0.1, 1.0),
                set_range: (f64::MIN, f64::MAX),

                bind: &self.value,
                add_binding["sensitive"]: &self.enabled,
            },
        },
    }

    fn init_model(
        (name, value): Self::Init,
        _index: &Self::Index,
        _sender: relm4::FactorySender<Self>,
    ) -> Self {
        Self {
            name,
            enabled: BoolBinding::new(value.is_some()),
            value: F64Binding::new(value.unwrap_or(0) as f64),
        }
    }
}

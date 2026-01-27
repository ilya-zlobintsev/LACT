use std::collections::HashMap;

use crate::app::{formatting, msg::AppMsg, APP_BROKER};
use gtk::{
    glib::{
        object::{Cast, ObjectExt},
        GString,
    },
    pango,
    prelude::{EditableExt, GtkWindowExt, OrientableExt, SorterExt, WidgetExt},
};
use lact_schema::{ProcessInfo, ProcessList, ProcessType, ProcessUtilizationType};
use relm4::{
    binding::{Binding, StringBinding, U32Binding, U64Binding},
    typed_view::{
        column::{LabelColumn, RelmColumn, TypedColumnView},
        OrdFn,
    },
    ComponentParts, ComponentSender, RelmObjectExt,
};

pub struct ProcessMonitorWindow {
    processes: TypedColumnView<ProcessRow, gtk::NoSelection>,
}

#[derive(Debug)]
pub enum ProcessMonitorWindowMsg {
    Show,
    Data(ProcessList),
    FilterChanged(GString),
}

#[relm4::component(pub)]
impl relm4::Component for ProcessMonitorWindow {
    type Init = ();
    type Input = ProcessMonitorWindowMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk::Window {
            set_title: Some("Process Monitor"),
            set_default_height: 600,
            set_default_width: 900,
            set_hide_on_close: true,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                #[name = "search_entry"]
                gtk::SearchEntry {
                    connect_search_changed[sender] => move |entry| {
                        sender.input(ProcessMonitorWindowMsg::FilterChanged(entry.text()));
                    },

                    connect_stop_search[root] => move |_| {
                        root.close();
                    },
                },

                gtk::ScrolledWindow {
                    set_hscrollbar_policy: gtk::PolicyType::Automatic,
                    set_vscrollbar_policy: gtk::PolicyType::Automatic,
                    set_vexpand: true,

                    model.processes.view.clone() {
                        set_show_column_separators: true,
                        set_show_row_separators: true,
                        set_reorderable: false,
                    }
                },
            },
        }
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut processes = TypedColumnView::new();
        processes.append_column::<PidColumn>();
        processes.append_column::<NameColumn>();
        processes.append_column::<TypeColumn>();
        processes.append_column::<VramColumn>();
        processes.append_column::<GraphicsUtilColumn>();
        processes.append_column::<ComputeUtilColumn>();
        processes.append_column::<MemoryUtilColumn>();
        processes.append_column::<EncodeUtilColumn>();
        processes.append_column::<DecodeUtilColumn>();

        processes.view.sort_by_column(
            processes.get_columns().get("Graphics"),
            gtk::SortType::Descending,
        );

        let mut model = Self { processes };

        let widgets = view_output!();

        model.processes.add_filter({
            let search_entry = widgets.search_entry.clone();
            move |process| {
                process
                    .name
                    .value()
                    .to_lowercase()
                    .contains(&search_entry.text().to_lowercase())
            }
        });

        ComponentParts { widgets, model }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, root: &Self::Root) {
        match msg {
            ProcessMonitorWindowMsg::Show => {
                root.present();
                APP_BROKER.send(AppMsg::FetchProcessList);
            }
            ProcessMonitorWindowMsg::Data(mut process_list) => {
                for column in self.processes.view.columns().into_iter() {
                    let column: gtk::ColumnViewColumn = column.unwrap().downcast().unwrap();
                    if let Some(title) = column.title() {
                        let util_type = match title.as_str() {
                            GraphicsUtilColumn::COLUMN_NAME => ProcessUtilizationType::Graphics,
                            ComputeUtilColumn::COLUMN_NAME => ProcessUtilizationType::Compute,
                            MemoryUtilColumn::COLUMN_NAME => ProcessUtilizationType::Memory,
                            EncodeUtilColumn::COLUMN_NAME => ProcessUtilizationType::Encode,
                            DecodeUtilColumn::COLUMN_NAME => ProcessUtilizationType::Decode,
                            _ => continue,
                        };
                        column.set_visible(process_list.supported_util_types.contains(&util_type));
                    }
                }

                let mut i = 0;
                while i < self.processes.len() {
                    let existing_process = self.processes.get(i).unwrap();
                    let existing_process = existing_process.borrow();

                    if let Some(new_process_info) =
                        process_list.processes.remove(&existing_process.pid)
                    {
                        existing_process.update(new_process_info);
                        i += 1;
                    } else {
                        drop(existing_process);

                        self.processes.remove(i);
                    }
                }

                // Remaining items are new processes not previously present in the list, add them
                self.processes.extend_from_iter(
                    process_list
                        .processes
                        .into_iter()
                        .rev()
                        .map(|(pid, process)| ProcessRow::new(process, pid)),
                );

                if let Some(sorter) = self.processes.view.sorter() {
                    sorter.changed(gtk::SorterChange::Different);
                }
            }
            ProcessMonitorWindowMsg::FilterChanged(filter) => {
                self.processes.set_filter_status(0, false);
                if !filter.is_empty() {
                    self.processes.set_filter_status(0, true);
                }
            }
        }
    }
}

struct ProcessRow {
    pid: u32,
    name: StringBinding,
    types: StringBinding,
    memory_usage: U64Binding,
    utils: HashMap<ProcessUtilizationType, U32Binding>,
}

impl ProcessRow {
    fn new(process: ProcessInfo, pid: u32) -> Self {
        let utils = ProcessUtilizationType::ALL
            .iter()
            .map(|util| (*util, U32Binding::new(0u32)))
            .collect::<HashMap<ProcessUtilizationType, U32Binding>>();

        for (process_util, value) in process.util {
            utils.get(&process_util).unwrap().set_value(value);
        }

        Self {
            pid,
            name: StringBinding::new(format_name(&process.name, &process.args)),
            types: StringBinding::new(fmt_process_types(&process.types)),
            memory_usage: U64Binding::new(process.memory_used),
            utils,
        }
    }

    fn update(&self, new_info: ProcessInfo) {
        self.name.set(format_name(&new_info.name, &new_info.args));
        self.types
            .set(fmt_process_types(&new_info.types).to_owned());
        self.memory_usage.set(new_info.memory_used);

        for (process_util, value) in new_info.util {
            self.utils.get(&process_util).unwrap().set_value(value);
        }
    }
}

fn format_name(name: &str, args: &str) -> String {
    let mut output = format!("<b>{name}</b>");
    if !args.is_empty() {
        output.push(' ');
        output.push_str(args);
    }
    output
}

struct PidColumn;

impl LabelColumn for PidColumn {
    type Item = ProcessRow;
    type Value = u32;

    const COLUMN_NAME: &'static str = "PID";
    const ENABLE_SORT: bool = true;

    fn get_cell_value(item: &Self::Item) -> Self::Value {
        item.pid
    }
}

struct NameColumn;

impl RelmColumn for NameColumn {
    type Item = ProcessRow;
    type Root = gtk::Label;
    type Widgets = ();

    const COLUMN_NAME: &'static str = "Name";
    const ENABLE_RESIZE: bool = true;
    const ENABLE_EXPAND: bool = true;

    fn setup(_: &gtk::ListItem) -> (Self::Root, Self::Widgets) {
        let label = gtk::Label::builder()
            .halign(gtk::Align::Start)
            .use_markup(true)
            .single_line_mode(true)
            .ellipsize(pango::EllipsizeMode::End)
            .build();

        (label, ())
    }

    fn bind(item: &mut Self::Item, _: &mut Self::Widgets, label: &mut Self::Root) {
        label.add_write_only_binding(&item.name, "label");
    }

    fn sort_fn() -> OrdFn<Self::Item> {
        Some(Box::new(|a, b| a.name.value().cmp(&b.name.value())))
    }
}

struct TypeColumn;

impl RelmColumn for TypeColumn {
    type Item = ProcessRow;
    type Root = gtk::Label;
    type Widgets = ();

    const COLUMN_NAME: &'static str = "Type";

    fn setup(_: &gtk::ListItem) -> (Self::Root, Self::Widgets) {
        let label = gtk::Label::new(None);
        (label, ())
    }

    fn bind(item: &mut Self::Item, _: &mut Self::Widgets, label: &mut Self::Root) {
        label.add_write_only_binding(&item.types, "label");
    }

    fn sort_fn() -> OrdFn<Self::Item> {
        Some(Box::new(|a, b| a.types.value().cmp(&b.types.value())))
    }
}

struct VramColumn;

impl RelmColumn for VramColumn {
    type Item = ProcessRow;
    type Root = gtk::Label;
    type Widgets = ();

    const COLUMN_NAME: &'static str = "VRAM";

    fn setup(_list_item: &gtk::ListItem) -> (Self::Root, Self::Widgets) {
        let label = gtk::Label::new(None);
        (label, ())
    }

    fn bind(item: &mut Self::Item, _widgets: &mut Self::Widgets, label: &mut Self::Root) {
        item.memory_usage
            .bind_property(U64Binding::property_name(), label, "label")
            .transform_to(|_binding, value: u64| Some(formatting::fmt_human_bytes(value, None)))
            .sync_create()
            .build();
    }

    fn sort_fn() -> OrdFn<Self::Item> {
        Some(Box::new(|a, b| {
            a.memory_usage.value().cmp(&b.memory_usage.value())
        }))
    }
}

macro_rules! util_columns {
    ($(($struct:ident, $name:literal, $property:expr),)*) => {
        $(
            struct $struct;

            impl RelmColumn for $struct {
                type Item = ProcessRow;
                type Root = gtk::Label;
                type Widgets = ();

                const COLUMN_NAME: &'static str = $name;

                fn setup(_list_item: &gtk::ListItem) -> (Self::Root, Self::Widgets) {
                    let label = gtk::Label::new(None);
                    (label, ())
                }

                fn bind(item: &mut Self::Item, _widgets: &mut Self::Widgets, label: &mut Self::Root) {
                    item.utils
                        .get(&$property)
                        .unwrap()
                        .bind_property(U32Binding::property_name(), label, "label")
                        .transform_to(|_binding, value: u32| Some(format!("{value}%")))
                        .sync_create()
                        .build();
                }

                fn sort_fn() -> OrdFn<Self::Item> {
                    Some(Box::new(|a, b| {
                        a.utils.get(&$property).unwrap().value().cmp(&b.utils.get(&$property).unwrap().value())
                    }))
                }
            }
        )*
    };
}

// This must be kept in sync with adding the columns in `init` and visibility logic in `update`
util_columns!(
    (
        GraphicsUtilColumn,
        "Graphics",
        ProcessUtilizationType::Graphics
    ),
    (
        ComputeUtilColumn,
        "Compute",
        ProcessUtilizationType::Compute
    ),
    (MemoryUtilColumn, "Memory", ProcessUtilizationType::Memory),
    (EncodeUtilColumn, "Encode", ProcessUtilizationType::Encode),
    (DecodeUtilColumn, "Decode", ProcessUtilizationType::Decode),
);

fn fmt_process_types(types: &[ProcessType]) -> &'static str {
    let gfx = types.contains(&ProcessType::Graphics);
    let compute = types.contains(&ProcessType::Compute);

    if gfx && compute {
        "Graph+Comp"
    } else if gfx {
        "Graphics"
    } else if compute {
        "Compute"
    } else {
        "Unknown"
    }
}

mod info_page;
mod oc_page;
mod software_page;
mod thermals_page;

use self::software_page::software_page;
use gtk::{prelude::IsA, traits::GridExt, *};
use info_page::InformationPage;
use lact_client::schema::SystemInfo;
use libadwaita::prelude::ActionRowExt;
use oc_page::OcPage;
use thermals_page::ThermalsPage;

#[derive(Clone)]
pub struct RootStack {
    pub container: Stack,
    pub info_page: InformationPage,
    pub thermals_page: ThermalsPage,
    pub oc_page: OcPage,
}

impl RootStack {
    pub fn new(
        root_win: libadwaita::ApplicationWindow,
        system_info: SystemInfo,
        embedded_daemon: bool,
    ) -> Self {
        let container = Stack::builder().vexpand(true).hexpand(true).build();

        let info_page = InformationPage::new();

        container.add_titled(&info_page.container, Some("info_page"), "Information");

        let oc_page = OcPage::new(&system_info);

        container.add_titled(&oc_page.container, Some("oc_page"), "OC");

        let thermals_page = ThermalsPage::new(root_win);

        container.add_titled(&thermals_page.container, Some("thermals_page"), "Thermals");

        let software_page = software_page(system_info, embedded_daemon);
        container.add_titled(&software_page, Some("software_page"), "Software");

        Self {
            container,
            info_page,
            thermals_page,
            oc_page,
        }
    }
}

#[deprecated]
fn values_row<W: IsA<Widget>>(
    title: &str,
    parent: &Grid,
    value_child: &W,
    row: i32,
    column_offset: i32,
) {
    let title_label = Label::builder().label(title).halign(Align::Start).build();

    parent.attach(&title_label, column_offset, row, 1, 1);
    parent.attach(value_child, column_offset + 1, row, 1, 1);
}

#[deprecated]
fn label_row(title: &str, parent: &Grid, row: i32, column_offset: i32, selectable: bool) -> Label {
    let value_label = Label::builder()
        .halign(Align::End)
        .hexpand(true)
        .selectable(selectable)
        .build();
    values_row(title, parent, &value_label, row, column_offset);

    value_label
}

#[deprecated]
fn values_grid() -> Grid {
    Grid::builder()
        .margin_start(10)
        .margin_end(5)
        .row_spacing(10)
        .column_spacing(10)
        .build()
}

#[derive(Clone)]
pub struct LabelRow {
    pub container: libadwaita::ActionRow,
    content_label: Label,
}

impl LabelRow {
    pub fn new(title: &str) -> Self {
        let container = libadwaita::ActionRow::builder().title(title).build();
        let label = Label::builder()
            .css_classes(["dim-label"])
            .ellipsize(pango::EllipsizeMode::End)
            .selectable(true)
            .build();
        container.add_suffix(&label);

        Self {
            container,
            content_label: label,
        }
    }

    pub fn new_with_content(title: &str, content: &str) -> Self {
        let row = Self::new(title);
        row.set_content(content);
        row
    }

    pub fn set_content(&self, content: &str) {
        self.content_label.set_label(content);
    }
}

pub fn list_clamp(child: &impl IsA<Widget>) -> libadwaita::Clamp {
    libadwaita::Clamp::builder()
        .maximum_size(600)
        .margin_top(24)
        .margin_bottom(24)
        .child(child)
        .valign(Align::Start)
        .build()
}

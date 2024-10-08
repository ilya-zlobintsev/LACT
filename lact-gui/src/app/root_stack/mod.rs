mod info_page;
mod oc_adjustment;
mod oc_page;
mod software_page;
mod thermals_page;

use gtk::{prelude::*, *};
use relm4::{Component, ComponentController};

use self::software_page::SoftwarePage;
use info_page::InformationPage;
use lact_client::schema::SystemInfo;
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
    pub fn new(system_info: SystemInfo, embedded_daemon: bool) -> Self {
        let container = Stack::builder()
            .vexpand(true)
            .margin_top(15)
            .margin_start(30)
            .margin_end(30)
            .build();

        let info_page = InformationPage::new();

        container.add_titled(&info_page.container, Some("info_page"), "Information");

        let oc_page = OcPage::new(&system_info);

        container.add_titled(&oc_page.container, Some("oc_page"), "OC");

        let thermals_page = ThermalsPage::new(&system_info);

        container.add_titled(&thermals_page.container, Some("thermals_page"), "Thermals");

        let mut software_page = SoftwarePage::builder()
            .launch((system_info, embedded_daemon))
            .detach();
        container.add_titled(software_page.widget(), Some("software_page"), "Software");
        software_page.detach_runtime();

        Self {
            container,
            info_page,
            thermals_page,
            oc_page,
        }
    }
}

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

fn label_row(title: &str, parent: &Grid, row: i32, column_offset: i32, selectable: bool) -> Label {
    let value_label = Label::builder()
        .halign(Align::End)
        .hexpand(true)
        .selectable(selectable)
        .build();
    values_row(title, parent, &value_label, row, column_offset);

    value_label
}

fn values_grid() -> Grid {
    Grid::builder()
        .margin_start(10)
        .margin_end(5)
        .row_spacing(10)
        .column_spacing(10)
        .build()
}

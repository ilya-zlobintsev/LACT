pub mod info_page;
pub mod oc_adjustment;
pub mod oc_page;
pub mod software_page;
pub mod thermals_page;

use gtk::{prelude::*, *};
use lact_schema::{DeviceInfo, DeviceStats};
use std::sync::Arc;

#[derive(Debug)]
pub enum PageUpdate {
    Info(Arc<DeviceInfo>),
    Stats(Arc<DeviceStats>),
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

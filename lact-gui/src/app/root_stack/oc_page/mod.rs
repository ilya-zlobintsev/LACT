mod clocks_frame;
mod performance_level_frame;
mod power_cap_frame;
mod stats_grid;
mod warning_frame;

use std::{cell::Cell, rc::Rc};

use clocks_frame::ClocksFrame;
use gtk::*;
use gtk::{glib::clone, prelude::*};
use lact_client::schema::{ClocksTableGen, DeviceStats, PerformanceLevel};
use performance_level_frame::PerformanceLevelFrame;
use power_cap_frame::PowerCapFrame;
use stats_grid::StatsGrid;
use tracing::{error, warn};
use warning_frame::WarningFrame;

#[derive(Clone)]
pub struct OcPage {
    pub container: Box,
    stats_grid: StatsGrid,
    performance_level_frame: PerformanceLevelFrame,
    power_cap_frame: PowerCapFrame,
    clocks_frame: ClocksFrame,
    pub warning_frame: WarningFrame,
}

impl OcPage {
    pub fn new() -> Self {
        let container = Box::new(Orientation::Vertical, 5);

        let warning_frame = WarningFrame::new();

        container.append(&warning_frame.container);

        let stats_grid = StatsGrid::new();

        container.append(&stats_grid.container);

        let power_cap_frame = PowerCapFrame::new();
        let performance_level_frame = PerformanceLevelFrame::new();
        let clocks_frame = ClocksFrame::new();

        container.append(&power_cap_frame.container);
        container.append(&performance_level_frame.container);
        container.append(&clocks_frame.container);

        Self {
            container,
            stats_grid,
            performance_level_frame,
            clocks_frame,
            warning_frame,
            power_cap_frame,
        }
    }

    pub fn set_stats(&self, stats: &DeviceStats, initial: bool) {
        self.stats_grid.set_stats(stats);
        if initial {
            self.power_cap_frame.set_data(
                stats.power.cap_current,
                stats.power.cap_max,
                stats.power.cap_default,
            );
            self.set_performance_level(stats.performance_level);
        }
    }

    pub fn set_clocks_table(&self, table: Option<ClocksTableGen>) {
        match table {
            Some(table) => match self.clocks_frame.set_table(table) {
                Ok(()) => {
                    self.clocks_frame.container.show();
                }
                Err(err) => {
                    warn!("Got invalid clocks table: {err}");
                    self.clocks_frame.container.hide();
                }
            },
            None => {
                self.clocks_frame.container.hide();
            }
        }
    }

    pub fn connect_clocks_reset<F: Fn() + 'static + Clone>(&self, f: F) {
        /*self.clocks_frame.connect_clocks_reset(move || {
            f();
        });*/
        todo!()
    }

    pub fn connect_settings_changed<F: Fn() + 'static + Clone>(&self, f: F) {
        self.performance_level_frame
            .connect_power_profile_changed(f.clone());
        self.power_cap_frame.connect_cap_changed(f);

        /*let f = f.clone();
        self.clocks_frame.connect_clocks_changed(move || {
            f();
        })*/
    }

    pub fn set_performance_level(&self, profile: Option<PerformanceLevel>) {
        match profile {
            Some(profile) => {
                self.performance_level_frame.show();
                self.performance_level_frame.set_active_profile(profile);
            }
            None => self.performance_level_frame.hide(),
        }
    }

    pub fn get_performance_level(&self) -> Option<PerformanceLevel> {
        if self.performance_level_frame.get_visibility() {
            let level = self
                .performance_level_frame
                .get_selected_performance_level();
            Some(level)
        } else {
            None
        }
    }

    /*pub fn get_clocks(&self) -> Option<ClocksSettings> {
        match self.clocks_frame.get_visibility() {
            true => Some(self.clocks_frame.get_settings()),
            false => None,
        }
    }*/

    pub fn get_power_cap(&self) -> Option<f64> {
        self.power_cap_frame.get_cap()
    }
}

fn section_box(title: &str, spacing: i32, margin: i32) -> Box {
    let container = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(spacing)
        .margin_start(margin)
        .margin_end(margin)
        .build();

    let label = Label::builder()
        .use_markup(true)
        .label(&format!("<span font_desc='11'><b>{title}</b></span>"))
        .xalign(0.1)
        .build();

    container.append(&label);
    container
}

fn oc_adjustment(
    default_value: Option<Rc<Cell<Option<f64>>>>,
    value_suffix: &'static str,
) -> (Box, Adjustment) {
    let root_box = Box::new(Orientation::Horizontal, 0);

    let label = Label::new(None);
    root_box.append(&label);

    let adjustment = Adjustment::new(0.0, 0.0, 0.0, 1.0, 10.0, 0.0);

    adjustment.connect_value_changed(clone!(@strong label => move |adj| {
        let text = format!("{}/{} {}", adj.value().round(), adj.upper(), value_suffix);
        label.set_label(&text);
    }));

    let scale = Scale::builder()
        .orientation(Orientation::Horizontal)
        .adjustment(&adjustment)
        .hexpand(true)
        .round_digits(0)
        .build();

    scale.set_draw_value(false);

    root_box.append(&scale);

    if let Some(default_value) = default_value {
        let reset_button = Button::with_label("Default");
        reset_button.connect_clicked(clone!(@strong adjustment => move |_| {
            if let Some(cap) = default_value.get() {
                adjustment.set_value(cap);
            } else {
                error!("Could not set default cap, value not provided");
            }
        }));
        root_box.append(&reset_button);
    }

    (root_box, adjustment)
}

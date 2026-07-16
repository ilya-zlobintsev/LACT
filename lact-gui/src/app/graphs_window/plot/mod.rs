mod cubic_spline;
mod imp;
mod render_thread;
mod to_texture_ext;

use super::stat::{StatType, StatsData};
use crate::app::utils::css_colors;
use gtk::glib::{self, Object, subclass::types::ObjectSubclassIsExt};
use plotters::style::{
    BLACK, BLUE, Color, RED, RGBAColor, WHITE, YELLOW,
    full_palette::{DEEPORANGE_100, GREEN_500},
};
use std::sync::{Arc, RwLock};

glib::wrapper! {
    pub struct Plot(ObjectSubclass<imp::Plot>)
        @extends gtk::Widget,
        @implements gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for Plot {
    fn default() -> Self {
        Object::builder().build()
    }
}

impl Plot {
    pub fn set_data(&self, data: Arc<RwLock<StatsData>>) {
        *self.imp().data.borrow_mut() = data;
    }

    pub fn set_stats(&self, stats: Vec<StatType>) {
        *self.imp().stats.borrow_mut() = stats;
    }

    pub fn connect_frame_rendered<F: Fn() + 'static>(&self, f: F) {
        let mut rx = self.imp().render_thread.render_notifier();
        relm4::spawn_local(async move {
            while let Ok(()) = rx.recv().await {
                f();
            }
        });
    }
}

#[derive(Debug)]
pub struct PlotColorScheme {
    pub background: RGBAColor,
    pub text: RGBAColor,
    pub border: RGBAColor,
    pub border_secondary: RGBAColor,
    pub throttling: RGBAColor,
    pub success: RGBAColor,
    pub accent_bg: RGBAColor,
    pub error: RGBAColor,
    pub warning: RGBAColor,
}

impl Default for PlotColorScheme {
    fn default() -> Self {
        Self {
            background: WHITE.into(),
            text: BLACK.into(),
            border: BLACK.mix(0.8),
            border_secondary: BLACK.mix(0.5),
            throttling: DEEPORANGE_100.into(),
            success: GREEN_500.into(),
            accent_bg: BLUE.mix(0.5),
            error: RED.into(),
            warning: YELLOW.into(),
        }
    }
}

impl PlotColorScheme {
    pub fn current() -> Self {
        let Some(colors) = css_colors::current() else {
            return Self::default();
        };

        let mut throttling = gtk_to_plotters_color(colors.theme_unfocused_fg_color);
        throttling.3 = 0.5;

        Self {
            background: gtk_to_plotters_color(colors.theme_base_color),
            text: gtk_to_plotters_color(colors.theme_text_color),
            border: gtk_to_plotters_color(colors.borders),
            border_secondary: gtk_to_plotters_color(colors.unfocused_borders),
            throttling,
            success: gtk_to_plotters_color(colors.success_color),
            accent_bg: gtk_to_plotters_color(colors.accent_bg_color),
            error: gtk_to_plotters_color(colors.error_color),
            warning: gtk_to_plotters_color(colors.warning_color),
        }
    }
}

fn gtk_to_plotters_color(color: gtk::gdk::RGBA) -> RGBAColor {
    RGBAColor(
        (color.red() * u8::MAX as f32) as u8,
        (color.green() * u8::MAX as f32) as u8,
        (color.blue() * u8::MAX as f32) as u8,
        color.alpha() as f64,
    )
}

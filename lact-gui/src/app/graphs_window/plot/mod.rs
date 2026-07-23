mod cubic_spline;
mod imp;
mod render_thread;
mod to_texture_ext;

use super::stat::{StatType, StatsData};
use gtk::glib::{self, Object, subclass::types::ObjectSubclassIsExt};
use plotters::style::RGBAColor;
use std::sync::{Arc, RwLock};

glib::wrapper! {
    pub struct Plot(ObjectSubclass<imp::Plot>)
        @extends gtk::Widget,
        @implements gtk::Buildable, gtk::ConstraintTarget, gtk::Accessible;
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
        Self::LIGHT
    }
}

impl PlotColorScheme {
    const LIGHT: Self = Self {
        background: RGBAColor(255, 255, 255, 1.0),
        text: RGBAColor(0, 0, 0, 0.8),
        border: RGBAColor(0, 0, 0, 0.15),
        border_secondary: RGBAColor(0, 0, 0, 0.15),
        throttling: RGBAColor(0, 0, 0, 0.5),
        success: RGBAColor(27, 133, 83, 1.0),
        accent_bg: RGBAColor(53, 132, 228, 1.0),
        error: RGBAColor(192, 28, 40, 1.0),
        warning: RGBAColor(156, 110, 3, 1.0),
    };

    const DARK: Self = Self {
        background: RGBAColor(29, 29, 32, 1.0),
        text: RGBAColor(255, 255, 255, 1.0),
        border: RGBAColor(255, 255, 255, 0.15),
        border_secondary: RGBAColor(255, 255, 255, 0.15),
        throttling: RGBAColor(255, 255, 255, 0.5),
        success: RGBAColor(143, 240, 164, 1.0),
        accent_bg: RGBAColor(53, 132, 228, 1.0),
        error: RGBAColor(255, 123, 99, 1.0),
        warning: RGBAColor(248, 228, 92, 1.0),
    };

    pub fn current() -> Self {
        if adw::StyleManager::default().is_dark() {
            Self::DARK
        } else {
            Self::LIGHT
        }
    }
}

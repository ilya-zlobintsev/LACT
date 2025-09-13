mod cubic_spline;
mod imp;
mod render_thread;
mod to_texture_ext;

use super::stat::{StatType, StatsData};
use gtk::{
    glib::{self, subclass::types::ObjectSubclassIsExt, Object},
    prelude::StyleContextExt,
};
use plotters::style::{full_palette::DEEPORANGE_100, Color, RGBAColor, BLACK, WHITE};
use std::sync::{Arc, RwLock};

glib::wrapper! {
    pub struct Plot(ObjectSubclass<imp::Plot>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
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
}

impl Default for PlotColorScheme {
    fn default() -> Self {
        Self {
            background: WHITE.into(),
            text: BLACK.into(),
            border: BLACK.mix(0.8),
            border_secondary: BLACK.mix(0.5),
            throttling: DEEPORANGE_100.into(),
        }
    }
}

impl PlotColorScheme {
    pub fn from_context(ctx: &gtk::StyleContext) -> Option<Self> {
        let background = lookup_color(
            ctx,
            &["theme_base_color", "theme_bg_color", "view_bg_color"],
        )?;
        let text = lookup_color(ctx, &["theme_text_color"])?;
        let border = lookup_color(ctx, &["borders"])?;
        let border_secondary = lookup_color(ctx, &["unfocused_borders"])?;
        let mut throttling = lookup_color(ctx, &["theme_unfocused_fg_color"])?;
        throttling.3 = 0.5;

        Some(PlotColorScheme {
            background,
            text,
            border,
            border_secondary,
            throttling,
        })
    }
}

fn lookup_color(ctx: &gtk::StyleContext, names: &[&str]) -> Option<RGBAColor> {
    for name in names {
        if let Some(color) = ctx.lookup_color(name) {
            return Some(gtk_to_plotters_color(color));
        }
    }
    None
}

fn gtk_to_plotters_color(color: gtk::gdk::RGBA) -> RGBAColor {
    RGBAColor(
        (color.blue() * u8::MAX as f32) as u8,
        (color.green() * u8::MAX as f32) as u8,
        (color.red() * u8::MAX as f32) as u8,
        color.alpha() as f64,
    )
}

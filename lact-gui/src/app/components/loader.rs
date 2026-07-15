use gtk::{
    glib::{self, ControlFlow},
    prelude::*,
};
use std::{cell::Cell, rc::Rc, time::Duration};

const APP_SVG: &[u8] = include_bytes!("../../../../res/io.github.ilya_zlobintsev.LACT.svg");
const FRAME_COUNT: usize = 12;
const FRAME_INTERVAL: Duration = Duration::from_millis(65);
const SIZE: i32 = 192;

pub(crate) fn new() -> gtk::Picture {
    let textures = Rc::new(build_textures());
    let picture = gtk::Picture::new();

    picture.set_size_request(SIZE, SIZE);
    picture.set_can_shrink(false);
    picture.set_keep_aspect_ratio(true);
    picture.set_paintable(Some(&textures[0]));

    let frame = Rc::new(Cell::new(0));
    let picture_weak = picture.downgrade();
    glib::timeout_add_local(FRAME_INTERVAL, move || {
        let Some(picture) = picture_weak.upgrade() else {
            return ControlFlow::Break;
        };

        let next_frame = (frame.get() + 1) % textures.len();
        frame.set(next_frame);
        picture.set_paintable(Some(&textures[next_frame]));

        ControlFlow::Continue
    });

    picture
}

fn build_textures() -> Vec<gtk::gdk::Texture> {
    let svg = std::str::from_utf8(APP_SVG)
        .expect("app SVG should be valid UTF-8")
        .replace(
            "<svg height=\"128px\" viewBox=\"0 0 128 128\" width=\"128px\"",
            &format!("<svg height=\"{SIZE}px\" viewBox=\"0 0 128 128\" width=\"{SIZE}px\""),
        );

    (0..FRAME_COUNT)
        .map(|frame| {
            let angle = -(frame as i32 * 360 / FRAME_COUNT as i32);
            let svg = svg
                .replace(
                    "<g id=\"left-fan-blades\">",
                    &format!(
                        "<g id=\"left-fan-blades\" transform=\"rotate({angle} 41.4375 69.824219)\">"
                    ),
                )
                .replace(
                    "<g id=\"right-fan-blades\">",
                    &format!(
                        "<g id=\"right-fan-blades\" transform=\"rotate({angle} 86.5625 69.824219)\">"
                    ),
                );

            gtk::gdk::Texture::from_bytes(&glib::Bytes::from_owned(svg.into_bytes()))
                .expect("loader SVG frame should be loadable")
        })
        .collect()
}
